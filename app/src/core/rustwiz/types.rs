//! RustWiz - Packwiz-compatible type definitions
//!
//! Native Rust implementation of the packwiz format specification:
//! - pack.toml: Main pack manifest
//! - index.toml: File index with hashes
//! - *.pw.toml: Individual mod/resource metadata files
//!
//! Compatible with packwiz tools: https://packwiz.infra.link/reference/pack-format/

use serde::{Deserialize, Serialize};

// =============================================================================
// pack.toml - Main Pack Manifest
// =============================================================================

/// Main pack manifest (pack.toml)
/// 
/// This is the entry point for a packwiz modpack, containing metadata
/// about the pack and references to the index file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PackToml {
    /// Pack name (displayed in UIs)
    pub name: String,
    
    /// Pack format version (e.g., "packwiz:1.1.0")
    #[serde(default = "default_pack_format")]
    pub pack_format: String,
    
    /// Pack version string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    
    /// Pack author(s)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    
    /// Pack description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Component versions (Minecraft, mod loaders)
    pub versions: PackVersions,
    
    /// Index file reference
    pub index: PackIndex,
}

fn default_pack_format() -> String {
    "packwiz:1.1.0".to_string()
}

/// Component versions in the pack
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PackVersions {
    /// Minecraft version
    pub minecraft: String,
    
    /// Fabric loader version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fabric: Option<String>,
    
    /// Forge version (without MC prefix)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forge: Option<String>,
    
    /// NeoForge version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neoforge: Option<String>,
    
    /// Quilt loader version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quilt: Option<String>,
    
    /// LiteLoader version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liteloader: Option<String>,
}

/// Index file reference in pack.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct PackIndex {
    /// Path to the index file (usually "index.toml")
    pub file: String,
    
    /// Hash format used for the index file
    pub hash_format: HashFormat,
    
    /// Hash of the index file
    pub hash: String,
}

// =============================================================================
// index.toml - File Index
// =============================================================================

/// File index (index.toml)
/// 
/// Contains references to all files in the pack, including both
/// regular files and metafiles (*.pw.toml).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct IndexToml {
    /// Default hash format for all files
    pub hash_format: HashFormat,
    
    /// List of files in the pack
    #[serde(default)]
    pub files: Vec<IndexFile>,
}

impl Default for IndexToml {
    fn default() -> Self {
        Self {
            hash_format: HashFormat::Sha256,
            files: Vec::new(),
        }
    }
}

/// A file entry in the index
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct IndexFile {
    /// Relative path to the file
    pub file: String,
    
    /// Hash of the file
    pub hash: String,
    
    /// Hash format (if different from index default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash_format: Option<HashFormat>,
    
    /// Whether this is a metafile (*.pw.toml)
    #[serde(default, skip_serializing_if = "is_false")]
    pub metafile: bool,
    
    /// Whether to preserve existing file (don't overwrite user changes)
    #[serde(default, skip_serializing_if = "is_false")]
    pub preserve: bool,
    
    /// Alternative filename for download
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

// =============================================================================
// *.pw.toml - Mod/Resource Metadata Files
// =============================================================================

/// Mod/resource metadata file (*.pw.toml)
/// 
/// Contains information about a downloadable file, including
/// download URL, hash, and update sources.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ModToml {
    /// Display name of the mod
    pub name: String,
    
    /// Destination filename relative to game directory
    pub filename: String,
    
    /// Side requirement (client, server, or both)
    #[serde(default)]
    pub side: Side,
    
    /// Download information
    pub download: ModDownload,
    
    /// Optional mod settings
    #[serde(skip_serializing_if = "Option::is_none")]
    pub option: Option<ModOption>,
    
    /// Update sources for automatic updates
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update: Option<ModUpdate>,
}

/// Download information for a mod
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ModDownload {
    /// Download URL
    pub url: String,
    
    /// Hash format
    pub hash_format: HashFormat,
    
    /// File hash
    pub hash: String,
}

/// Optional mod configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModOption {
    /// Whether this mod is optional
    pub optional: bool,
    
    /// Whether enabled by default (for optional mods)
    #[serde(default = "default_true")]
    pub default: bool,
    
    /// Description shown when selecting optional mods
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Update sources for a mod
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModUpdate {
    /// Modrinth update source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modrinth: Option<ModrinthUpdate>,
    
    /// CurseForge update source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub curseforge: Option<CurseForgeUpdate>,
}

/// Modrinth update source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ModrinthUpdate {
    /// Modrinth project ID
    pub mod_id: String,
    
    /// Current version ID
    pub version: String,
}

/// CurseForge update source
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct CurseForgeUpdate {
    /// CurseForge project ID
    pub project_id: u32,
    
    /// Current file ID
    pub file_id: u32,
}

// =============================================================================
// Shared Types
// =============================================================================

/// Supported hash formats
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum HashFormat {
    #[default]
    Sha256,
    Sha512,
    Sha1,
    Md5,
    Murmur2,
}

impl HashFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            HashFormat::Sha256 => "sha256",
            HashFormat::Sha512 => "sha512",
            HashFormat::Sha1 => "sha1",
            HashFormat::Md5 => "md5",
            HashFormat::Murmur2 => "murmur2",
        }
    }
}

impl std::fmt::Display for HashFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Side requirement for mods
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    /// Required on both client and server
    #[default]
    Both,
    /// Client-only mod
    Client,
    /// Server-only mod
    Server,
}

impl Side {
    pub fn as_str(&self) -> &'static str {
        match self {
            Side::Both => "both",
            Side::Client => "client",
            Side::Server => "server",
        }
    }
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// =============================================================================
// Extended Metadata (OxideLauncher-specific)
// =============================================================================

/// Extended metadata stored alongside packwiz files
/// 
/// This contains additional information that OxideLauncher tracks
/// but isn't part of the packwiz specification.
/// 
/// Some fields follow Prism Launcher's naming convention (x-prismlauncher-*)
/// for potential interoperability.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OxideMetadata {
    /// Icon URL from the platform
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon_url: Option<String>,
    
    /// Project description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    /// Project author(s)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authors: Option<Vec<String>>,
    
    /// Homepage URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage_url: Option<String>,
    
    /// Issues/bug tracker URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issues_url: Option<String>,
    
    /// Source code URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    
    /// License identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,
    
    /// Categories/tags
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub categories: Vec<String>,
    
    /// Download count from platform
    #[serde(skip_serializing_if = "Option::is_none")]
    pub downloads: Option<u64>,
    
    /// When this metadata was last updated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata_updated: Option<String>,
    
    /// Compatible Minecraft versions (follows Prism's x-prismlauncher-mc-versions)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mc_versions: Vec<String>,
    
    /// Compatible mod loaders (follows Prism's x-prismlauncher-loaders)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub loaders: Vec<String>,
    
    /// Release type (release, beta, alpha)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub release_type: Option<String>,
    
    /// Human-readable version number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_number: Option<String>,
}

/// Combined mod.pw.toml with OxideLauncher extensions
/// 
/// We store the standard packwiz format plus our extensions
/// in a way that maintains compatibility with other tools.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModTomlExtended {
    /// Standard packwiz mod metadata
    #[serde(flatten)]
    pub packwiz: ModToml,
    
    /// OxideLauncher-specific extensions (stored under [oxide] table)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oxide: Option<OxideMetadata>,
}

// =============================================================================
// Update Check Results
// =============================================================================

/// Result of checking for mod updates
#[derive(Debug, Clone, Serialize)]
pub struct UpdateCheckResult {
    /// Mod filename
    pub filename: String,
    
    /// Current version
    pub current_version: String,
    
    /// Latest available version (if update available)
    pub latest_version: Option<String>,
    
    /// Latest version ID (for downloading)
    pub latest_version_id: Option<String>,
    
    /// Whether an update is available
    pub update_available: bool,
    
    /// Platform this mod is from
    pub platform: String,
    
    /// Changelog or release notes (if available)
    pub changelog: Option<String>,
}

/// Batch update check results
#[derive(Debug, Clone, Serialize)]
pub struct BatchUpdateResult {
    /// Mods with available updates
    pub updates_available: Vec<UpdateCheckResult>,
    
    /// Mods that are up-to-date
    pub up_to_date: Vec<String>,
    
    /// Mods that couldn't be checked (no update info)
    pub unchecked: Vec<String>,
    
    /// Errors encountered during checking
    pub errors: Vec<String>,
}

// =============================================================================
// Builder Helpers
// =============================================================================

impl PackToml {
    /// Create a new pack.toml for an instance
    pub fn new(
        name: String,
        minecraft_version: String,
        mod_loader: Option<(&str, &str)>, // (loader_type, version)
    ) -> Self {
        let mut versions = PackVersions {
            minecraft: minecraft_version,
            ..Default::default()
        };
        
        if let Some((loader_type, version)) = mod_loader {
            match loader_type.to_lowercase().as_str() {
                "fabric" => versions.fabric = Some(version.to_string()),
                "forge" => versions.forge = Some(version.to_string()),
                "neoforge" => versions.neoforge = Some(version.to_string()),
                "quilt" => versions.quilt = Some(version.to_string()),
                "liteloader" => versions.liteloader = Some(version.to_string()),
                _ => {}
            }
        }
        
        Self {
            name,
            pack_format: default_pack_format(),
            version: Some("1.0.0".to_string()),
            author: None,
            description: None,
            versions,
            index: PackIndex {
                file: "index.toml".to_string(),
                hash_format: HashFormat::Sha256,
                hash: String::new(), // Will be computed when saving
            },
        }
    }
    
    /// Get the mod loader type and version
    pub fn get_mod_loader(&self) -> Option<(&str, &str)> {
        if let Some(ref v) = self.versions.fabric {
            return Some(("fabric", v));
        }
        if let Some(ref v) = self.versions.forge {
            return Some(("forge", v));
        }
        if let Some(ref v) = self.versions.neoforge {
            return Some(("neoforge", v));
        }
        if let Some(ref v) = self.versions.quilt {
            return Some(("quilt", v));
        }
        if let Some(ref v) = self.versions.liteloader {
            return Some(("liteloader", v));
        }
        None
    }
}

impl ModToml {
    /// Create a new mod.pw.toml from download info
    pub fn new(
        name: String,
        filename: String,
        url: String,
        hash: String,
        hash_format: HashFormat,
    ) -> Self {
        Self {
            name,
            filename,
            side: Side::Both,
            download: ModDownload {
                url,
                hash_format,
                hash,
            },
            option: None,
            update: None,
        }
    }
    
    /// Add Modrinth update source
    pub fn with_modrinth_update(mut self, mod_id: String, version_id: String) -> Self {
        let update = self.update.get_or_insert(ModUpdate::default());
        update.modrinth = Some(ModrinthUpdate {
            mod_id,
            version: version_id,
        });
        self
    }
    
    /// Add CurseForge update source
    pub fn with_curseforge_update(mut self, project_id: u32, file_id: u32) -> Self {
        let update = self.update.get_or_insert(ModUpdate::default());
        update.curseforge = Some(CurseForgeUpdate {
            project_id,
            file_id,
        });
        self
    }
    
    /// Set side requirement
    pub fn with_side(mut self, side: Side) -> Self {
        self.side = side;
        self
    }
    
    /// Make optional with description
    #[allow(dead_code)] // Reserved for future optional mod support
    pub fn as_optional(mut self, description: Option<String>, default_enabled: bool) -> Self {
        self.option = Some(ModOption {
            optional: true,
            default: default_enabled,
            description,
        });
        self
    }
}

impl ModTomlExtended {
    /// Create from basic ModToml
    pub fn from_packwiz(packwiz: ModToml) -> Self {
        Self {
            packwiz,
            oxide: None,
        }
    }
    
    /// Add OxideLauncher metadata
    #[allow(dead_code)] // Reserved for future metadata enrichment
    pub fn with_oxide_metadata(mut self, metadata: OxideMetadata) -> Self {
        self.oxide = Some(metadata);
        self
    }
}
