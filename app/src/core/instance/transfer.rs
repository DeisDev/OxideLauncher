//! Instance import/export functionality
//! 
//! Supports:
//! - OxideLauncher native format (.oxide)
//! - Modrinth modpack format (.mrpack)
//! - CurseForge modpack format
//! - Prism Launcher instances

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;
use chrono::{DateTime, Utc};

// =============================================================================
// Export Format (OxideLauncher Native)
// =============================================================================

/// OxideLauncher export manifest format
/// Stored as `oxide.manifest.json` in the root of the .oxide archive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OxideManifest {
    /// Format version (currently 1)
    pub format_version: u32,
    
    /// Instance metadata
    pub instance: OxideInstanceMetadata,
    
    /// Files included in the export
    pub files: Vec<OxideFileEntry>,
    
    /// Timestamp when export was created
    pub exported_at: DateTime<Utc>,
    
    /// OxideLauncher version that created this export
    pub launcher_version: String,
}

/// Instance metadata for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OxideInstanceMetadata {
    /// Original instance ID (for reference only)
    pub original_id: String,
    
    /// Instance name
    pub name: String,
    
    /// Instance icon (base64 encoded if custom, or name if default)
    pub icon: OxideIcon,
    
    /// Minecraft version
    pub minecraft_version: String,
    
    /// Mod loader info
    pub mod_loader: Option<OxideModLoader>,
    
    /// Notes
    pub notes: String,
    
    /// Total playtime in seconds
    pub total_played_seconds: u64,
    
    /// Original creation date
    pub created_at: DateTime<Utc>,
    
    /// Instance-specific settings
    pub settings: OxideInstanceSettings,
    
    /// Managed pack info (if applicable)
    pub managed_pack: Option<OxideManagedPack>,
}

/// Icon representation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum OxideIcon {
    /// Default/builtin icon by name
    Default { name: String },
    /// Custom icon as base64 PNG
    Custom { data: String, filename: String },
}

/// Mod loader info for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OxideModLoader {
    pub loader_type: String, // "forge", "neoforge", "fabric", "quilt"
    pub version: String,
}

/// Instance settings for export
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OxideInstanceSettings {
    pub jvm_args: Option<String>,
    pub game_args: Option<String>,
    pub min_memory: Option<u32>,
    pub max_memory: Option<u32>,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub fullscreen: bool,
}

/// Managed pack info for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OxideManagedPack {
    pub platform: String, // "modrinth", "curseforge", etc
    pub pack_id: String,
    pub pack_name: String,
    pub version_id: String,
    pub version_name: String,
}

/// File entry in export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OxideFileEntry {
    /// Relative path from instance root
    pub path: String,
    
    /// SHA256 hash of file
    pub hash: String,
    
    /// File size in bytes
    pub size: u64,
    
    /// Whether this is a downloadable file (mod, resource pack, etc)
    pub downloadable: Option<OxideDownloadableFile>,
}

/// Information about a downloadable file (for license compliance)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OxideDownloadableFile {
    /// Platform the file is from
    pub platform: String, // "modrinth", "curseforge"
    
    /// Project ID
    pub project_id: String,
    
    /// Version/file ID
    pub version_id: String,
    
    /// Download URL
    pub url: String,
}

// =============================================================================
// Modrinth Modpack Format (.mrpack)
// =============================================================================

/// Modrinth modpack index (modrinth.index.json)
/// https://docs.modrinth.com/docs/modpacks/format_definition/
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModrinthIndex {
    /// Format version (should be 1)
    pub format_version: u32,
    
    /// Game (should be "minecraft")
    pub game: String,
    
    /// Version ID of the modpack
    pub version_id: String,
    
    /// Name of the modpack
    pub name: String,
    
    /// Summary/description
    #[serde(default)]
    pub summary: Option<String>,
    
    /// Files to download
    pub files: Vec<ModrinthFile>,
    
    /// Loader dependencies (minecraft version, loader versions)
    pub dependencies: HashMap<String, String>,
}

/// A file in the Modrinth modpack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthFile {
    /// Relative path where the file should be placed
    pub path: String,
    
    /// File hashes
    pub hashes: ModrinthHashes,
    
    /// Environment requirements
    #[serde(default)]
    pub env: Option<ModrinthEnv>,
    
    /// Download URLs (try in order)
    pub downloads: Vec<String>,
    
    /// File size in bytes
    #[serde(rename = "fileSize")]
    pub file_size: u64,
}

/// File hashes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthHashes {
    pub sha1: String,
    pub sha512: String,
}

/// Environment requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthEnv {
    /// Client requirement: "required", "optional", "unsupported"
    pub client: String,
    /// Server requirement: "required", "optional", "unsupported"
    pub server: String,
}

// =============================================================================
// CurseForge Modpack Format (manifest.json)
// =============================================================================

/// CurseForge modpack manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeManifest {
    /// Manifest type (should be "minecraftModpack")
    pub manifest_type: String,
    
    /// Manifest version (should be 1)
    pub manifest_version: u32,
    
    /// Pack name
    pub name: String,
    
    /// Pack version
    pub version: String,
    
    /// Pack author
    #[serde(default)]
    pub author: String,
    
    /// Minecraft configuration
    pub minecraft: CurseForgeMinecraft,
    
    /// Files to download
    pub files: Vec<CurseForgeFile>,
    
    /// Overrides folder name
    #[serde(default = "default_overrides")]
    pub overrides: String,
}

fn default_overrides() -> String {
    "overrides".to_string()
}

/// Minecraft configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeMinecraft {
    /// Minecraft version
    pub version: String,
    
    /// Mod loaders
    #[serde(default)]
    pub mod_loaders: Vec<CurseForgeModLoader>,
}

/// Mod loader in CurseForge manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurseForgeModLoader {
    /// Loader ID (e.g., "forge-47.1.0")
    pub id: String,
    
    /// Whether this is the primary loader
    #[serde(default)]
    pub primary: bool,
}

/// File reference in CurseForge manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurseForgeFile {
    /// CurseForge project ID
    pub project_id: u32,
    
    /// CurseForge file ID
    pub file_id: u32,
    
    /// Whether the file is required
    #[serde(default = "default_true")]
    pub required: bool,
}

fn default_true() -> bool {
    true
}

// =============================================================================
// Prism Launcher Instance Format
// =============================================================================

/// Prism/MultiMC instance.cfg (INI format)
/// We only need to parse key fields
#[derive(Debug, Clone, Default)]
pub struct PrismInstanceConfig {
    /// Instance name
    pub name: String,
    
    /// Icon key
    pub icon_key: String,
    
    /// Notes
    pub notes: String,
    
    /// Total time played (milliseconds)
    pub total_time_played: u64,
    
    /// Last time played (milliseconds)
    pub last_time_played: u64,
    
    /// Managed pack type ("modrinth", "flame", etc)
    pub managed_pack_type: Option<String>,
    
    /// Managed pack ID
    pub managed_pack_id: Option<String>,
    
    /// Managed pack name
    pub managed_pack_name: Option<String>,
    
    /// Managed pack version ID
    pub managed_pack_version_id: Option<String>,
    
    /// Managed pack version name
    pub managed_pack_version_name: Option<String>,
}

impl PrismInstanceConfig {
    /// Parse from INI content
    pub fn parse(content: &str) -> Self {
        let mut config = Self::default();
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
                continue;
            }
            
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                
                match key {
                    "name" => config.name = value.to_string(),
                    "iconKey" => config.icon_key = value.to_string(),
                    "notes" => config.notes = value.to_string(),
                    "totalTimePlayed" => {
                        config.total_time_played = value.parse().unwrap_or(0);
                    }
                    "lastTimePlayed" => {
                        config.last_time_played = value.parse().unwrap_or(0);
                    }
                    "ManagedPackType" => config.managed_pack_type = Some(value.to_string()),
                    "ManagedPackID" => config.managed_pack_id = Some(value.to_string()),
                    "ManagedPackName" => config.managed_pack_name = Some(value.to_string()),
                    "ManagedPackVersionID" => config.managed_pack_version_id = Some(value.to_string()),
                    "ManagedPackVersionName" => config.managed_pack_version_name = Some(value.to_string()),
                    _ => {}
                }
            }
        }
        
        config
    }
}

/// Prism mmc-pack.json (component configuration)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrismPackJson {
    /// Components (Minecraft, loaders, etc)
    pub components: Vec<PrismComponent>,
    
    /// Format version
    #[serde(rename = "formatVersion")]
    pub format_version: u32,
}

/// A component in Prism's pack profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrismComponent {
    /// Cached name
    #[serde(rename = "cachedName")]
    pub cached_name: Option<String>,
    
    /// Cached version
    #[serde(rename = "cachedVersion")]
    pub cached_version: Option<String>,
    
    /// Whether cached requires java
    #[serde(rename = "cachedRequires")]
    pub cached_requires: Option<Vec<PrismRequirement>>,
    
    /// Dependency only (not user-selected)
    #[serde(default, rename = "dependencyOnly")]
    pub dependency_only: bool,
    
    /// Important (can't be removed)
    #[serde(default)]
    pub important: bool,
    
    /// Component UID (e.g., "net.minecraft", "net.fabricmc.fabric-loader")
    pub uid: String,
    
    /// Component version
    pub version: String,
}

/// A requirement in Prism components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrismRequirement {
    /// Required component UID
    pub uid: String,
    
    /// Suggested version
    pub suggests: Option<String>,
    
    /// Equals version
    pub equals: Option<String>,
}

// =============================================================================
// Import Result
// =============================================================================

/// Result of importing a modpack
#[derive(Debug)]
pub struct ImportResult {
    /// Instance name
    pub name: String,
    
    /// Minecraft version
    pub minecraft_version: String,
    
    /// Mod loader (if any)
    pub mod_loader: Option<(String, String)>, // (type, version)
    
    /// Files that need to be downloaded
    pub files_to_download: Vec<FileToDownload>,
    
    /// Overrides extracted
    pub overrides_path: Option<PathBuf>,
    
    /// Icon (if found)
    pub icon: Option<OxideIcon>,
    
    /// Total playtime to restore (seconds)
    pub playtime: u64,
    
    /// Notes to restore
    pub notes: String,
    
    /// Managed pack info
    pub managed_pack: Option<OxideManagedPack>,
    
    /// Original settings
    pub settings: OxideInstanceSettings,
}

/// A file that needs to be downloaded
#[derive(Debug, Clone)]
pub struct FileToDownload {
    /// Relative path where file should go
    pub path: String,
    
    /// Download URLs (try in order)
    pub urls: Vec<String>,
    
    /// Expected file size
    pub size: u64,
    
    /// Expected hash (SHA1)
    pub hash_sha1: Option<String>,
    
    /// Expected hash (SHA512)
    pub hash_sha512: Option<String>,
    
    /// Platform info for API lookups
    pub platform_info: Option<PlatformFileInfo>,
}

/// Platform-specific file info for downloading
#[derive(Debug, Clone)]
pub struct PlatformFileInfo {
    pub platform: String, // "modrinth" or "curseforge"
    pub project_id: String,
    pub file_id: String,
}

// =============================================================================
// Detected Import Type
// =============================================================================

/// Type of import detected from archive
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportType {
    /// OxideLauncher native export
    OxideLauncher,
    /// Modrinth .mrpack
    Modrinth,
    /// CurseForge modpack
    CurseForge,
    /// Prism/MultiMC instance
    Prism,
    /// Unknown format
    Unknown,
}

impl ImportType {
    /// Detect import type from archive file list
    pub fn detect(file_list: &[String]) -> Self {
        // Check for OxideLauncher format
        if file_list.iter().any(|f| f == "oxide.manifest.json" || f.ends_with("/oxide.manifest.json")) {
            return ImportType::OxideLauncher;
        }
        
        // Check for Modrinth format (has modrinth.index.json at root)
        if file_list.iter().any(|f| f == "modrinth.index.json" || f == "/modrinth.index.json") {
            return ImportType::Modrinth;
        }
        
        // Check for CurseForge format
        if file_list.iter().any(|f| f == "manifest.json" || f.ends_with("/manifest.json")) {
            // Need to verify it's a CurseForge manifest, not just any manifest.json
            return ImportType::CurseForge;
        }
        
        // Check for Prism/MultiMC format
        if file_list.iter().any(|f| f == "instance.cfg" || f.ends_with("/instance.cfg")) {
            return ImportType::Prism;
        }
        
        ImportType::Unknown
    }
}
