//! Modrinth API client

use serde::{Deserialize, Serialize};
use crate::core::error::{OxideError, Result};
use crate::core::config::Config;
use super::types::*;

const MODRINTH_API_URL: &str = "https://api.modrinth.com/v2";

/// Modrinth API client
pub struct ModrinthClient {
    client: reqwest::Client,
    api_token: Option<String>,
}

impl ModrinthClient {
    /// Create a new Modrinth client
    pub fn new() -> Self {
        let config = Config::load().unwrap_or_default();
        Self {
            client: reqwest::Client::new(),
            api_token: config.api_keys.modrinth_api_token,
        }
    }

    /// Create a request builder with common headers
    fn request(&self, method: reqwest::Method, endpoint: &str) -> reqwest::RequestBuilder {
        let mut builder = self.client
            .request(method, format!("{}{}", MODRINTH_API_URL, endpoint))
            .header("User-Agent", format!("OxideLauncher/{}", env!("CARGO_PKG_VERSION")));
        
        if let Some(token) = &self.api_token {
            builder = builder.header("Authorization", token);
        }
        
        builder
    }

    /// Search for projects
    pub async fn search(&self, query: &SearchQuery) -> Result<SearchResults> {
        let facets = build_facets(query);
        
        let mut params = vec![
            ("query", query.query.clone()),
            ("limit", query.limit.to_string()),
            ("offset", query.offset.to_string()),
            ("index", query.sort.modrinth_name().to_string()),
        ];
        
        if !facets.is_empty() {
            params.push(("facets", format!("[{}]", facets.join(","))));
        }
        
        let response: ModrinthSearchResponse = self.request(reqwest::Method::GET, "/search")
            .query(&params)
            .send()
            .await?
            .json()
            .await?;
        
        Ok(SearchResults {
            hits: response.hits.into_iter().map(|h| h.into()).collect(),
            total_hits: response.total_hits,
            offset: response.offset,
            limit: response.limit,
        })
    }

    /// Get project details
    pub async fn get_project(&self, id_or_slug: &str) -> Result<Project> {
        let response: ModrinthProject = self.request(reqwest::Method::GET, &format!("/project/{}", id_or_slug))
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response.into())
    }

    /// Get project versions
    pub async fn get_versions(
        &self,
        project_id: &str,
        game_versions: Option<&[String]>,
        loaders: Option<&[String]>,
    ) -> Result<Vec<ProjectVersion>> {
        let mut params = Vec::new();
        
        if let Some(versions) = game_versions {
            params.push(("game_versions", serde_json::to_string(versions).unwrap()));
        }
        
        if let Some(loaders) = loaders {
            params.push(("loaders", serde_json::to_string(loaders).unwrap()));
        }
        
        let response: Vec<ModrinthVersion> = self.request(reqwest::Method::GET, &format!("/project/{}/version", project_id))
            .query(&params)
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response.into_iter().map(|v| v.into()).collect())
    }

    /// Get a specific version
    pub async fn get_version(&self, version_id: &str) -> Result<ProjectVersion> {
        let response: ModrinthVersion = self.request(reqwest::Method::GET, &format!("/version/{}", version_id))
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response.into())
    }

    /// Get categories
    pub async fn get_categories(&self) -> Result<Vec<Category>> {
        let response: Vec<ModrinthCategory> = self.request(reqwest::Method::GET, "/tag/category")
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response.into_iter().map(|c| Category {
            name: c.name,
            icon: c.icon,
            project_type: c.project_type,
        }).collect())
    }
}

impl Default for ModrinthClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Build facets string for search
fn build_facets(query: &SearchQuery) -> Vec<String> {
    let mut facets = Vec::new();
    
    // Project type
    if let Some(resource_type) = &query.resource_type {
        let project_type = match resource_type {
            ResourceType::Mod => "mod",
            ResourceType::Modpack => "modpack",
            ResourceType::ResourcePack => "resourcepack",
            ResourceType::ShaderPack => "shader",
            ResourceType::DataPack => "datapack",
        };
        facets.push(format!("[\"project_type:{}\"]", project_type));
    }
    
    // Categories
    if !query.categories.is_empty() {
        let cats: Vec<String> = query.categories.iter()
            .map(|c| format!("\"categories:{}\"", c))
            .collect();
        facets.push(format!("[{}]", cats.join(",")));
    }
    
    // Game versions
    if !query.game_versions.is_empty() {
        let versions: Vec<String> = query.game_versions.iter()
            .map(|v| format!("\"versions:{}\"", v))
            .collect();
        facets.push(format!("[{}]", versions.join(",")));
    }
    
    // Loaders
    if !query.loaders.is_empty() {
        let loaders: Vec<String> = query.loaders.iter()
            .map(|l| format!("\"categories:{}\"", l))
            .collect();
        facets.push(format!("[{}]", loaders.join(",")));
    }
    
    facets
}

// Modrinth API response types

#[derive(Debug, Deserialize)]
struct ModrinthSearchResponse {
    hits: Vec<ModrinthSearchHit>,
    total_hits: u32,
    offset: u32,
    limit: u32,
}

#[derive(Debug, Deserialize)]
struct ModrinthSearchHit {
    project_id: String,
    slug: String,
    title: String,
    description: String,
    author: String,
    icon_url: Option<String>,
    downloads: u64,
    follows: u32,
    categories: Vec<String>,
    versions: Vec<String>,
    date_created: String,
    date_modified: String,
    project_type: String,
}

impl From<ModrinthSearchHit> for SearchHit {
    fn from(hit: ModrinthSearchHit) -> Self {
        Self {
            id: hit.project_id,
            slug: hit.slug,
            title: hit.title,
            description: hit.description,
            author: hit.author,
            icon_url: hit.icon_url,
            downloads: hit.downloads,
            follows: hit.follows,
            categories: hit.categories.clone(),
            versions: hit.versions,
            loaders: hit.categories.into_iter()
                .filter(|c| ["fabric", "forge", "quilt", "neoforge"].contains(&c.as_str()))
                .collect(),
            date_created: hit.date_created.parse().unwrap_or_default(),
            date_modified: hit.date_modified.parse().unwrap_or_default(),
            platform: Platform::Modrinth,
            resource_type: match hit.project_type.as_str() {
                "mod" => ResourceType::Mod,
                "modpack" => ResourceType::Modpack,
                "resourcepack" => ResourceType::ResourcePack,
                "shader" => ResourceType::ShaderPack,
                _ => ResourceType::Mod,
            },
        }
    }
}

#[derive(Debug, Deserialize)]
struct ModrinthProject {
    id: String,
    slug: String,
    title: String,
    description: String,
    body: String,
    icon_url: Option<String>,
    downloads: u64,
    followers: u32,
    categories: Vec<String>,
    game_versions: Vec<String>,
    loaders: Vec<String>,
    source_url: Option<String>,
    issues_url: Option<String>,
    wiki_url: Option<String>,
    discord_url: Option<String>,
    published: String,
    updated: String,
    project_type: String,
    license: Option<ModrinthLicense>,
    gallery: Vec<ModrinthGalleryImage>,
}

#[derive(Debug, Deserialize)]
struct ModrinthLicense {
    id: String,
    name: String,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ModrinthGalleryImage {
    url: String,
    title: Option<String>,
    description: Option<String>,
}

impl From<ModrinthProject> for Project {
    fn from(p: ModrinthProject) -> Self {
        Self {
            id: p.id,
            slug: p.slug,
            title: p.title,
            description: p.description,
            body: p.body,
            author: String::new(), // Not in project response
            icon_url: p.icon_url,
            banner_url: None,
            downloads: p.downloads,
            followers: p.followers,
            categories: p.categories.clone(),
            versions: p.game_versions,
            loaders: p.loaders,
            links: ProjectLinks {
                source: p.source_url,
                issues: p.issues_url,
                wiki: p.wiki_url,
                discord: p.discord_url,
                website: None,
            },
            gallery: p.gallery.into_iter().map(|g| GalleryImage {
                url: g.url,
                title: g.title,
                description: g.description,
            }).collect(),
            license: p.license.map(|l| License {
                id: l.id,
                name: l.name,
                url: l.url,
            }),
            date_created: p.published.parse().unwrap_or_default(),
            date_modified: p.updated.parse().unwrap_or_default(),
            platform: Platform::Modrinth,
            resource_type: match p.project_type.as_str() {
                "mod" => ResourceType::Mod,
                "modpack" => ResourceType::Modpack,
                "resourcepack" => ResourceType::ResourcePack,
                "shader" => ResourceType::ShaderPack,
                _ => ResourceType::Mod,
            },
        }
    }
}

#[derive(Debug, Deserialize)]
struct ModrinthVersion {
    id: String,
    project_id: String,
    name: String,
    version_number: String,
    changelog: Option<String>,
    game_versions: Vec<String>,
    loaders: Vec<String>,
    files: Vec<ModrinthFile>,
    downloads: u64,
    date_published: String,
    version_type: String,
}

#[derive(Debug, Deserialize)]
struct ModrinthFile {
    url: String,
    filename: String,
    size: u64,
    hashes: ModrinthHashes,
    primary: bool,
}

#[derive(Debug, Deserialize)]
struct ModrinthHashes {
    sha1: String,
    sha512: String,
}

impl From<ModrinthVersion> for ProjectVersion {
    fn from(v: ModrinthVersion) -> Self {
        Self {
            id: v.id,
            project_id: v.project_id,
            name: v.name,
            version_number: v.version_number,
            changelog: v.changelog,
            game_versions: v.game_versions,
            loaders: v.loaders,
            files: v.files.into_iter().map(|f| VersionFile {
                url: f.url,
                filename: f.filename,
                size: f.size,
                sha1: Some(f.hashes.sha1),
                sha512: Some(f.hashes.sha512),
                primary: f.primary,
            }).collect(),
            downloads: v.downloads,
            date_published: v.date_published.parse().unwrap_or_default(),
            version_type: match v.version_type.as_str() {
                "release" => VersionType::Release,
                "beta" => VersionType::Beta,
                "alpha" => VersionType::Alpha,
                _ => VersionType::Release,
            },
            platform: Platform::Modrinth,
        }
    }
}

#[derive(Debug, Deserialize)]
struct ModrinthCategory {
    name: String,
    icon: String,
    project_type: String,
}

/// Category information
#[derive(Debug, Clone)]
pub struct Category {
    pub name: String,
    pub icon: String,
    pub project_type: String,
}
