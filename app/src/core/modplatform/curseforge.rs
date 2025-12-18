//! CurseForge API client

#![allow(dead_code)] // API client will be used as features are completed

use serde::Deserialize;
use crate::core::error::{OxideError, Result};
use crate::core::config::Config;
use super::types::*;

const CURSEFORGE_API_URL: &str = "https://api.curseforge.com/v1";
const MINECRAFT_GAME_ID: u32 = 432;

/// CurseForge mod class IDs
mod class_ids {
    pub const MODS: u32 = 6;
    pub const MODPACKS: u32 = 4471;
    pub const RESOURCE_PACKS: u32 = 12;
    pub const WORLDS: u32 = 17;
    pub const SHADERS: u32 = 6552;
}

/// CurseForge API client
pub struct CurseForgeClient {
    client: reqwest::Client,
    api_key: Option<String>,
}

impl CurseForgeClient {
    /// Create a new CurseForge client
    pub fn new() -> Self {
        let config = Config::load().unwrap_or_default();
        Self {
            client: reqwest::Client::new(),
            api_key: config.api_keys.curseforge_api_key,
        }
    }

    /// Check if the client has an API key configured
    pub fn has_api_key(&self) -> bool {
        self.api_key.is_some()
    }

    /// Create a request builder with common headers
    fn request(&self, method: reqwest::Method, endpoint: &str) -> Result<reqwest::RequestBuilder> {
        let api_key = self.api_key.as_ref()
            .ok_or_else(|| OxideError::ModPlatform("CurseForge API key not configured".into()))?;
        
        Ok(self.client
            .request(method, format!("{}{}", CURSEFORGE_API_URL, endpoint))
            .header("x-api-key", api_key)
            .header("Accept", "application/json"))
    }

    /// Search for mods
    pub async fn search(&self, query: &SearchQuery) -> Result<SearchResults> {
        let class_id = match query.resource_type {
            Some(ResourceType::Mod) => class_ids::MODS,
            Some(ResourceType::Modpack) => class_ids::MODPACKS,
            Some(ResourceType::ResourcePack) => class_ids::RESOURCE_PACKS,
            Some(ResourceType::ShaderPack) => class_ids::SHADERS,
            _ => class_ids::MODS,
        };
        
        // CurseForge API max pageSize is 50
        let page_size = query.limit.min(50);
        
        let mut params = vec![
            ("gameId", MINECRAFT_GAME_ID.to_string()),
            ("classId", class_id.to_string()),
            ("searchFilter", query.query.clone()),
            ("pageSize", page_size.to_string()),
            ("index", query.offset.to_string()),
            ("sortField", query.sort.curseforge_id().to_string()),
            ("sortOrder", "desc".to_string()),
        ];
        
        // Add game version filter
        if !query.game_versions.is_empty() {
            params.push(("gameVersion", query.game_versions[0].clone()));
        }
        
        // Add mod loader filter
        if !query.loaders.is_empty() {
            if let Some(loader_type) = get_loader_type(&query.loaders[0]) {
                params.push(("modLoaderType", loader_type.to_string()));
            }
        }
        
        // Add category filter - convert category names to CurseForge category IDs
        if !query.categories.is_empty() {
            let category_ids: Vec<u32> = query.categories.iter()
                .filter_map(|cat| get_mod_category_id(cat))
                .collect();
            if !category_ids.is_empty() {
                // CurseForge API supports categoryIds parameter for multiple categories
                let ids_str = format!("[{}]", category_ids.iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<_>>()
                    .join(","));
                params.push(("categoryIds", ids_str));
            }
        }
        
        let response: CurseForgeSearchResponse = self.request(reqwest::Method::GET, "/mods/search")?
            .query(&params)
            .send()
            .await?
            .json()
            .await?;
        
        let resource_type = query.resource_type.unwrap_or(ResourceType::Mod);
        
        Ok(SearchResults {
            hits: response.data.into_iter().map(|m| m.into_search_hit(resource_type)).collect(),
            total_hits: response.pagination.total_count,
            offset: response.pagination.index,
            limit: response.pagination.page_size,
        })
    }

    /// Get mod details
    pub async fn get_mod(&self, mod_id: u32) -> Result<Project> {
        let response: CurseForgeModResponse = self.request(reqwest::Method::GET, &format!("/mods/{}", mod_id))?
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response.data.into())
    }

    /// Get mod files
    pub async fn get_files(
        &self,
        mod_id: u32,
        game_version: Option<&str>,
        mod_loader: Option<&str>,
    ) -> Result<Vec<ProjectVersion>> {
        let mut params = Vec::new();
        
        if let Some(version) = game_version {
            params.push(("gameVersion", version.to_string()));
        }
        
        if let Some(loader) = mod_loader {
            if let Some(loader_type) = get_loader_type(loader) {
                params.push(("modLoaderType", loader_type.to_string()));
            }
        }
        
        let response: CurseForgeFilesResponse = self.request(reqwest::Method::GET, &format!("/mods/{}/files", mod_id))?
            .query(&params)
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response.data.into_iter().map(|f| f.into_version(mod_id)).collect())
    }

    /// Get a specific file
    pub async fn get_file(&self, mod_id: u32, file_id: u32) -> Result<ProjectVersion> {
        let response: CurseForgeFileResponse = self.request(reqwest::Method::GET, &format!("/mods/{}/files/{}", mod_id, file_id))?
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response.data.into_version(mod_id))
    }

    /// Get download URL for a file
    pub async fn get_download_url(&self, mod_id: u32, file_id: u32) -> Result<String> {
        let response: CurseForgeDownloadUrlResponse = self.request(reqwest::Method::GET, &format!("/mods/{}/files/{}/download-url", mod_id, file_id))?
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response.data)
    }

    /// Get mod description (full HTML body)
    pub async fn get_mod_description(&self, mod_id: u32) -> Result<String> {
        let response: CurseForgeDescriptionResponse = self.request(reqwest::Method::GET, &format!("/mods/{}/description", mod_id))?
            .send()
            .await?
            .json()
            .await?;
        
        Ok(response.data)
    }

    /// Get multiple mods by their IDs (batch request)
    /// Returns a map of mod_id -> class_id for routing files to correct folders
    pub async fn get_mods_class_ids(&self, mod_ids: &[u32]) -> Result<std::collections::HashMap<u32, u32>> {
        if mod_ids.is_empty() {
            return Ok(std::collections::HashMap::new());
        }

        #[derive(serde::Serialize)]
        struct GetModsRequest {
            #[serde(rename = "modIds")]
            mod_ids: Vec<u32>,
        }

        let response: CurseForgeModsResponse = self.request(reqwest::Method::POST, "/mods")?
            .json(&GetModsRequest { mod_ids: mod_ids.to_vec() })
            .send()
            .await?
            .json()
            .await?;

        let mut result = std::collections::HashMap::new();
        for m in response.data {
            result.insert(m.id, m.class_id);
        }
        Ok(result)
    }

    /// Get the appropriate folder name for a CurseForge class ID
    pub fn get_resource_folder(class_id: u32) -> &'static str {
        match class_id {
            class_ids::MODS => "mods",
            class_ids::RESOURCE_PACKS => "resourcepacks",
            class_ids::SHADERS => "shaderpacks",
            class_ids::WORLDS => "saves",
            _ => "mods", // Default to mods for unknown types
        }
    }
}

impl Default for CurseForgeClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Get CurseForge mod loader type ID
fn get_loader_type(loader: &str) -> Option<u32> {
    match loader.to_lowercase().as_str() {
        "forge" => Some(1),
        "fabric" => Some(4),
        "quilt" => Some(5),
        "neoforge" => Some(6),
        _ => None,
    }
}

/// Get CurseForge category ID from category name
/// Note: These are mod categories (classId = 6). Modpack categories have different IDs.
fn get_mod_category_id(category: &str) -> Option<u32> {
    match category.to_lowercase().as_str() {
        "adventure" | "adventure and rpg" => Some(422),
        "magic" => Some(419),
        "tech" | "technology" => Some(412),
        "storage" => Some(420),
        "library" | "api and library" => Some(421),
        "utility" | "utility & qol" => Some(5191),
        "world gen" | "worldgen" => Some(406),
        "cosmetic" => Some(424),
        "food" => Some(436),
        "armor" | "armor, tools, and weapons" => Some(434),
        "mobs" => Some(411),
        "miscellaneous" => Some(425),
        "performance" => Some(6814),
        "server utility" => Some(435),
        "map and information" => Some(423),
        "redstone" => Some(4558),
        "twitch integration" => Some(4671),
        "bug fixes" => Some(6821),
        "education" => Some(5299),
        "mcreator" => Some(4906),
        _ => None,
    }
}

// CurseForge API response types

#[derive(Debug, Deserialize)]
struct CurseForgeSearchResponse {
    data: Vec<CurseForgeMod>,
    pagination: CurseForgePagination,
}

#[derive(Debug, Deserialize)]
struct CurseForgePagination {
    index: u32,
    #[serde(rename = "pageSize")]
    page_size: u32,
    #[serde(rename = "resultCount")]
    result_count: u32,
    #[serde(rename = "totalCount")]
    total_count: u32,
}

#[derive(Debug, Deserialize)]
struct CurseForgeModResponse {
    data: CurseForgeMod,
}

#[derive(Debug, Deserialize)]
struct CurseForgeModsResponse {
    data: Vec<CurseForgeMod>,
}

#[derive(Debug, Deserialize)]
struct CurseForgeMod {
    id: u32,
    slug: String,
    name: String,
    summary: String,
    #[serde(rename = "downloadCount")]
    download_count: u64,
    #[serde(rename = "thumbsUpCount")]
    thumbs_up_count: u32,
    logo: Option<CurseForgeAsset>,
    screenshots: Vec<CurseForgeAsset>,
    categories: Vec<CurseForgeCategory>,
    #[serde(rename = "latestFilesIndexes")]
    latest_files_indexes: Vec<CurseForgeFileIndex>,
    authors: Vec<CurseForgeAuthor>,
    links: CurseForgeLinks,
    #[serde(rename = "dateCreated")]
    date_created: String,
    #[serde(rename = "dateModified")]
    date_modified: String,
    #[serde(rename = "classId")]
    class_id: u32,
}

#[derive(Debug, Deserialize)]
struct CurseForgeAsset {
    url: String,
    title: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct CurseForgeCategory {
    id: u32,
    name: String,
    slug: String,
}

#[derive(Debug, Deserialize)]
struct CurseForgeFileIndex {
    #[serde(rename = "gameVersion")]
    game_version: String,
    #[serde(rename = "modLoader")]
    mod_loader: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct CurseForgeAuthor {
    name: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct CurseForgeLinks {
    #[serde(rename = "websiteUrl")]
    website_url: Option<String>,
    #[serde(rename = "wikiUrl")]
    wiki_url: Option<String>,
    #[serde(rename = "issuesUrl")]
    issues_url: Option<String>,
    #[serde(rename = "sourceUrl")]
    source_url: Option<String>,
}

impl CurseForgeMod {
    fn into_search_hit(self, resource_type: ResourceType) -> SearchHit {
        let mut loaders = Vec::new();
        for index in &self.latest_files_indexes {
            if let Some(loader) = index.mod_loader {
                let loader_name = match loader {
                    1 => "forge",
                    4 => "fabric",
                    5 => "quilt",
                    6 => "neoforge",
                    _ => continue,
                };
                if !loaders.contains(&loader_name.to_string()) {
                    loaders.push(loader_name.to_string());
                }
            }
        }
        
        SearchHit {
            id: self.id.to_string(),
            slug: self.slug,
            title: self.name,
            description: self.summary,
            author: self.authors.first().map(|a| a.name.clone()).unwrap_or_default(),
            icon_url: self.logo.map(|l| l.url),
            downloads: self.download_count,
            follows: self.thumbs_up_count,
            categories: self.categories.iter().map(|c| c.name.clone()).collect(),
            versions: self.latest_files_indexes.iter().map(|i| i.game_version.clone()).collect(),
            loaders,
            date_created: self.date_created.parse().unwrap_or_default(),
            date_modified: self.date_modified.parse().unwrap_or_default(),
            platform: Platform::CurseForge,
            resource_type,
        }
    }
}

impl From<CurseForgeMod> for Project {
    fn from(m: CurseForgeMod) -> Self {
        let mut loaders = Vec::new();
        let mut versions = Vec::new();
        
        for index in &m.latest_files_indexes {
            if !versions.contains(&index.game_version) {
                versions.push(index.game_version.clone());
            }
            if let Some(loader) = index.mod_loader {
                let loader_name = match loader {
                    1 => "forge",
                    4 => "fabric",
                    5 => "quilt",
                    6 => "neoforge",
                    _ => continue,
                };
                if !loaders.contains(&loader_name.to_string()) {
                    loaders.push(loader_name.to_string());
                }
            }
        }
        
        let resource_type = match m.class_id {
            class_ids::MODS => ResourceType::Mod,
            class_ids::MODPACKS => ResourceType::Modpack,
            class_ids::RESOURCE_PACKS => ResourceType::ResourcePack,
            class_ids::SHADERS => ResourceType::ShaderPack,
            _ => ResourceType::Mod,
        };
        
        Self {
            id: m.id.to_string(),
            slug: m.slug,
            title: m.name,
            description: m.summary,
            body: String::new(), // Need separate API call for full description
            author: m.authors.first().map(|a| a.name.clone()).unwrap_or_default(),
            icon_url: m.logo.as_ref().map(|l| l.url.clone()),
            banner_url: None,
            downloads: m.download_count,
            followers: m.thumbs_up_count,
            categories: m.categories.iter().map(|c| c.name.clone()).collect(),
            versions,
            loaders,
            links: ProjectLinks {
                website: m.links.website_url,
                wiki: m.links.wiki_url,
                issues: m.links.issues_url,
                source: m.links.source_url,
                discord: None,
            },
            gallery: m.screenshots.into_iter().map(|s| GalleryImage {
                url: s.url,
                title: s.title,
                description: s.description,
            }).collect(),
            license: None,
            date_created: m.date_created.parse().unwrap_or_default(),
            date_modified: m.date_modified.parse().unwrap_or_default(),
            platform: Platform::CurseForge,
            resource_type,
        }
    }
}

#[derive(Debug, Deserialize)]
struct CurseForgeFilesResponse {
    data: Vec<CurseForgeFile>,
}

#[derive(Debug, Deserialize)]
struct CurseForgeFileResponse {
    data: CurseForgeFile,
}

#[derive(Debug, Deserialize)]
struct CurseForgeFile {
    id: u32,
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(rename = "fileName")]
    file_name: String,
    #[serde(rename = "fileLength")]
    file_length: u64,
    #[serde(rename = "downloadUrl")]
    download_url: Option<String>,
    #[serde(rename = "gameVersions")]
    game_versions: Vec<String>,
    #[serde(rename = "releaseType")]
    release_type: u32,
    #[serde(rename = "fileDate")]
    file_date: String,
    #[serde(rename = "downloadCount")]
    download_count: u64,
    hashes: Vec<CurseForgeHash>,
    #[serde(default)]
    dependencies: Vec<CurseForgeDependency>,
}

#[derive(Debug, Deserialize)]
struct CurseForgeDependency {
    #[serde(rename = "modId")]
    mod_id: u32,
    #[serde(rename = "relationType")]
    relation_type: u32,
}

#[derive(Debug, Deserialize)]
struct CurseForgeHash {
    value: String,
    algo: u32,
}

impl CurseForgeFile {
    fn into_version(self, mod_id: u32) -> ProjectVersion {
        let sha1 = self.hashes.iter()
            .find(|h| h.algo == 1)
            .map(|h| h.value.clone());
        
        // Separate game versions from loader names
        let game_versions: Vec<String> = self.game_versions.iter()
            .filter(|v| v.starts_with("1.") || v.starts_with("20") || v.starts_with("21") || v.starts_with("22") || v.starts_with("23") || v.starts_with("24"))
            .cloned()
            .collect();
        
        let loaders: Vec<String> = self.game_versions.iter()
            .filter(|v| ["forge", "fabric", "quilt", "neoforge"].contains(&v.to_lowercase().as_str()))
            .map(|v| v.to_lowercase())
            .collect();
        
        ProjectVersion {
            id: self.id.to_string(),
            project_id: mod_id.to_string(),
            name: self.display_name,
            version_number: self.file_name.clone(),
            changelog: None,
            game_versions,
            loaders,
            files: vec![VersionFile {
                url: self.download_url.unwrap_or_default(),
                filename: self.file_name,
                size: self.file_length,
                sha1,
                sha512: None,
                primary: true,
            }],
            downloads: self.download_count,
            date_published: self.file_date.parse().unwrap_or_default(),
            version_type: match self.release_type {
                1 => VersionType::Release,
                2 => VersionType::Beta,
                3 => VersionType::Alpha,
                _ => VersionType::Release,
            },
            platform: Platform::CurseForge,
            dependencies: self.dependencies.into_iter().map(|d| Dependency {
                project_id: Some(d.mod_id.to_string()),
                version_id: None,
                dependency_type: match d.relation_type {
                    1 => DependencyType::Embedded,   // EmbeddedLibrary
                    2 => DependencyType::Optional,   // OptionalDependency
                    3 => DependencyType::Required,   // RequiredDependency
                    4 => DependencyType::Unknown,    // Tool
                    5 => DependencyType::Incompatible, // Incompatible
                    6 => DependencyType::Optional,   // Include (treat as optional)
                    _ => DependencyType::Unknown,
                },
            }).collect(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct CurseForgeDownloadUrlResponse {
    data: String,
}

#[derive(Debug, Deserialize)]
struct CurseForgeDescriptionResponse {
    data: String,
}
