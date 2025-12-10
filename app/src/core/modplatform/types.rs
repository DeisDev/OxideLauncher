//! Common types for mod platforms

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Resource types that can be searched
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    Mod,
    Modpack,
    ResourcePack,
    ShaderPack,
    DataPack,
}

impl ResourceType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ResourceType::Mod => "Mods",
            ResourceType::Modpack => "Modpacks",
            ResourceType::ResourcePack => "Resource Packs",
            ResourceType::ShaderPack => "Shader Packs",
            ResourceType::DataPack => "Data Packs",
        }
    }
}

/// Search results from a mod platform
#[derive(Debug, Clone)]
pub struct SearchResults {
    pub hits: Vec<SearchHit>,
    pub total_hits: u32,
    pub offset: u32,
    pub limit: u32,
}

/// A search result hit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub author: String,
    pub icon_url: Option<String>,
    pub downloads: u64,
    pub follows: u32,
    pub categories: Vec<String>,
    pub versions: Vec<String>,
    pub loaders: Vec<String>,
    pub date_created: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
    pub platform: Platform,
    pub resource_type: ResourceType,
}

/// Mod platform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Platform {
    Modrinth,
    CurseForge,
}

impl Platform {
    pub fn name(&self) -> &'static str {
        match self {
            Platform::Modrinth => "Modrinth",
            Platform::CurseForge => "CurseForge",
        }
    }
}

/// Detailed project information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub author: String,
    pub icon_url: Option<String>,
    pub banner_url: Option<String>,
    pub downloads: u64,
    pub followers: u32,
    pub categories: Vec<String>,
    pub versions: Vec<String>,
    pub loaders: Vec<String>,
    pub links: ProjectLinks,
    pub gallery: Vec<GalleryImage>,
    pub license: Option<License>,
    pub date_created: DateTime<Utc>,
    pub date_modified: DateTime<Utc>,
    pub platform: Platform,
    pub resource_type: ResourceType,
}

/// Project links
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectLinks {
    pub source: Option<String>,
    pub issues: Option<String>,
    pub wiki: Option<String>,
    pub discord: Option<String>,
    pub website: Option<String>,
}

/// Gallery image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GalleryImage {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
}

/// License information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub id: String,
    pub name: String,
    pub url: Option<String>,
}

/// Project version/file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectVersion {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub version_number: String,
    pub changelog: Option<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub files: Vec<VersionFile>,
    pub downloads: u64,
    pub date_published: DateTime<Utc>,
    pub version_type: VersionType,
    pub platform: Platform,
}

/// Version release type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionType {
    Release,
    Beta,
    Alpha,
}

impl VersionType {
    pub fn display_name(&self) -> &'static str {
        match self {
            VersionType::Release => "Release",
            VersionType::Beta => "Beta",
            VersionType::Alpha => "Alpha",
        }
    }
}

/// Version file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionFile {
    pub url: String,
    pub filename: String,
    pub size: u64,
    pub sha1: Option<String>,
    pub sha512: Option<String>,
    pub primary: bool,
}

/// Search query parameters
#[derive(Debug, Clone, Default)]
pub struct SearchQuery {
    pub query: String,
    pub resource_type: Option<ResourceType>,
    pub categories: Vec<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub sort: SortOrder,
    pub offset: u32,
    pub limit: u32,
}

/// Sort order for search results
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SortOrder {
    #[default]
    Relevance,
    Downloads,
    Follows,
    Newest,
    Updated,
}

impl SortOrder {
    pub fn modrinth_name(&self) -> &'static str {
        match self {
            SortOrder::Relevance => "relevance",
            SortOrder::Downloads => "downloads",
            SortOrder::Follows => "follows",
            SortOrder::Newest => "newest",
            SortOrder::Updated => "updated",
        }
    }

    pub fn curseforge_id(&self) -> u32 {
        match self {
            SortOrder::Relevance => 0,
            SortOrder::Downloads => 6,
            SortOrder::Follows => 3,
            SortOrder::Newest => 1,
            SortOrder::Updated => 2,
        }
    }
}
