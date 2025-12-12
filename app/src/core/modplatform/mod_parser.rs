//! Mod JAR file metadata parser
//! 
//! Parses mod information from various mod loader formats:
//! - Fabric: fabric.mod.json
//! - Forge (modern): META-INF/mods.toml
//! - Forge (legacy): mcmod.info
//! - Quilt: quilt.mod.json
//! - LiteLoader: litemod.json

use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::Path;
use zip::ZipArchive;

/// Parsed mod details from a JAR file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModDetails {
    /// Mod ID as defined in the mod loader metadata
    pub mod_id: String,
    /// Human-readable name
    pub name: String,
    /// Version string
    pub version: String,
    /// Description
    pub description: String,
    /// List of authors
    pub authors: Vec<String>,
    /// Homepage URL
    pub homepage: Option<String>,
    /// Issues/bug tracker URL
    pub issues_url: Option<String>,
    /// Source code URL
    pub source_url: Option<String>,
    /// License
    pub license: Option<String>,
    /// Icon file path within the JAR
    pub icon_path: Option<String>,
    /// Detected mod loader type
    pub loader_type: Option<String>,
}

/// Parse mod details from a JAR file
pub fn parse_mod_jar(path: &Path) -> Option<ModDetails> {
    let file = std::fs::File::open(path).ok()?;
    let mut archive = ZipArchive::new(file).ok()?;
    
    // Try each format in order of preference
    if let Some(details) = try_parse_fabric(&mut archive) {
        return Some(details);
    }
    if let Some(details) = try_parse_quilt(&mut archive) {
        return Some(details);
    }
    if let Some(details) = try_parse_forge_toml(&mut archive) {
        return Some(details);
    }
    if let Some(details) = try_parse_mcmod_info(&mut archive) {
        return Some(details);
    }
    if let Some(details) = try_parse_litemod(&mut archive) {
        return Some(details);
    }
    
    None
}

/// Try to parse fabric.mod.json
fn try_parse_fabric<R: Read + std::io::Seek>(archive: &mut ZipArchive<R>) -> Option<ModDetails> {
    let mut file = archive.by_name("fabric.mod.json").ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    
    let json: FabricModJson = serde_json::from_str(&contents).ok()?;
    
    let authors = json.authors.unwrap_or_default()
        .into_iter()
        .map(|a| match a {
            AuthorEntry::String(s) => s,
            AuthorEntry::Object { name, .. } => name,
        })
        .collect();
    
    let contact = json.contact.unwrap_or_default();
    
    Some(ModDetails {
        mod_id: json.id,
        name: json.name.unwrap_or_default(),
        version: json.version,
        description: json.description.unwrap_or_default(),
        authors,
        homepage: contact.homepage,
        issues_url: contact.issues,
        source_url: contact.sources,
        license: json.license.map(|l| match l {
            LicenseEntry::String(s) => s,
            LicenseEntry::Object { id, .. } => id,
        }),
        icon_path: json.icon,
        loader_type: Some("Fabric".to_string()),
    })
}

/// Try to parse quilt.mod.json
fn try_parse_quilt<R: Read + std::io::Seek>(archive: &mut ZipArchive<R>) -> Option<ModDetails> {
    let mut file = archive.by_name("quilt.mod.json").ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    
    let json: QuiltModJson = serde_json::from_str(&contents).ok()?;
    let loader = json.quilt_loader;
    let metadata = loader.metadata.unwrap_or_default();
    
    let authors = metadata.contributors.unwrap_or_default()
        .into_iter()
        .map(|(name, _role)| name)
        .collect();
    
    let contact = metadata.contact.unwrap_or_default();
    
    Some(ModDetails {
        mod_id: loader.id,
        name: metadata.name.unwrap_or_default(),
        version: loader.version,
        description: metadata.description.unwrap_or_default(),
        authors,
        homepage: contact.homepage,
        issues_url: contact.issues,
        source_url: contact.sources,
        license: metadata.license.map(|l| match l {
            LicenseEntry::String(s) => s,
            LicenseEntry::Object { id, .. } => id,
        }),
        icon_path: metadata.icon,
        loader_type: Some("Quilt".to_string()),
    })
}

/// Try to parse META-INF/mods.toml (modern Forge)
fn try_parse_forge_toml<R: Read + std::io::Seek>(archive: &mut ZipArchive<R>) -> Option<ModDetails> {
    let mut file = archive.by_name("META-INF/mods.toml").ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    
    let toml: toml::Value = toml::from_str(&contents).ok()?;
    
    // Get the first mod entry from [[mods]] array
    let mods = toml.get("mods")?.as_array()?;
    let first_mod = mods.first()?;
    
    let mod_id = first_mod.get("modId")?.as_str()?.to_string();
    let version = first_mod.get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let name = first_mod.get("displayName")
        .and_then(|v| v.as_str())
        .unwrap_or(&mod_id)
        .to_string();
    let description = first_mod.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    
    // Authors can be in root or mod entry
    let authors_str = first_mod.get("authors")
        .or_else(|| toml.get("authors"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    let authors: Vec<String> = authors_str
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    
    let homepage = toml.get("displayURL")
        .or_else(|| first_mod.get("displayURL"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let issues_url = toml.get("issueTrackerURL")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let license = toml.get("license")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let icon_path = first_mod.get("logoFile")
        .or_else(|| toml.get("logoFile"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    Some(ModDetails {
        mod_id,
        name,
        version,
        description,
        authors,
        homepage,
        issues_url,
        source_url: None,
        license,
        icon_path,
        loader_type: Some("Forge".to_string()),
    })
}

/// Try to parse mcmod.info (legacy Forge)
fn try_parse_mcmod_info<R: Read + std::io::Seek>(archive: &mut ZipArchive<R>) -> Option<ModDetails> {
    let mut file = archive.by_name("mcmod.info").ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    
    // mcmod.info can be either an array or an object with "modList"
    let json: serde_json::Value = serde_json::from_str(&contents).ok()?;
    
    let mod_info = if json.is_array() {
        json.as_array()?.first()?.clone()
    } else {
        json.get("modList")?.as_array()?.first()?.clone()
    };
    
    let mod_id = mod_info.get("modid")?.as_str()?.to_string();
    let name = mod_info.get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&mod_id)
        .to_string();
    let version = mod_info.get("version")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let description = mod_info.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    
    let authors: Vec<String> = mod_info.get("authorList")
        .or_else(|| mod_info.get("authors"))
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect())
        .unwrap_or_default();
    
    let homepage = mod_info.get("url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let icon_path = mod_info.get("logoFile")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    Some(ModDetails {
        mod_id,
        name,
        version,
        description,
        authors,
        homepage,
        issues_url: None,
        source_url: None,
        license: None,
        icon_path,
        loader_type: Some("Forge".to_string()),
    })
}

/// Try to parse litemod.json
fn try_parse_litemod<R: Read + std::io::Seek>(archive: &mut ZipArchive<R>) -> Option<ModDetails> {
    let mut file = archive.by_name("litemod.json").ok()?;
    let mut contents = String::new();
    file.read_to_string(&mut contents).ok()?;
    
    let json: serde_json::Value = serde_json::from_str(&contents).ok()?;
    
    let name = json.get("name")?.as_str()?.to_string();
    let version = json.get("version")
        .or_else(|| json.get("revision"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let description = json.get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    
    let author = json.get("author")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    let homepage = json.get("url")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    
    Some(ModDetails {
        mod_id: name.clone(),
        name,
        version,
        description,
        authors: author.into_iter().collect(),
        homepage,
        issues_url: None,
        source_url: None,
        license: None,
        icon_path: None,
        loader_type: Some("LiteLoader".to_string()),
    })
}

// JSON structures for parsing

#[derive(Debug, Deserialize)]
struct FabricModJson {
    id: String,
    version: String,
    name: Option<String>,
    description: Option<String>,
    authors: Option<Vec<AuthorEntry>>,
    contact: Option<ContactInfo>,
    license: Option<LicenseEntry>,
    icon: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum AuthorEntry {
    String(String),
    Object { name: String, contact: Option<ContactInfo> },
}

#[derive(Debug, Default, Deserialize)]
struct ContactInfo {
    homepage: Option<String>,
    issues: Option<String>,
    sources: Option<String>,
    email: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum LicenseEntry {
    String(String),
    Object { id: String, name: Option<String>, url: Option<String> },
}

#[derive(Debug, Deserialize)]
struct QuiltModJson {
    quilt_loader: QuiltLoader,
}

#[derive(Debug, Deserialize)]
struct QuiltLoader {
    id: String,
    version: String,
    metadata: Option<QuiltMetadata>,
}

#[derive(Debug, Default, Deserialize)]
struct QuiltMetadata {
    name: Option<String>,
    description: Option<String>,
    contributors: Option<std::collections::HashMap<String, String>>,
    contact: Option<ContactInfo>,
    license: Option<LicenseEntry>,
    icon: Option<String>,
}
