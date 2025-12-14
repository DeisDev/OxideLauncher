//! Modloader profile types
//! 
//! This module defines the data structures for modloader version profiles,
//! which describe the libraries, main class, and arguments needed to launch
//! with a specific modloader version.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A complete modloader profile that can be merged with vanilla Minecraft
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModloaderProfile {
    /// Unique identifier (e.g., "net.fabricmc.fabric-loader", "net.minecraftforge")
    pub uid: String,
    
    /// Version of the modloader
    pub version: String,
    
    /// Minecraft version this profile is for
    pub minecraft_version: String,
    
    /// Main class to use (overrides vanilla main class)
    pub main_class: String,
    
    /// Libraries required by this modloader
    pub libraries: Vec<ModloaderLibrary>,
    
    /// Additional JVM arguments
    #[serde(default)]
    pub jvm_arguments: Vec<String>,
    
    /// Additional game arguments
    #[serde(default)]
    pub game_arguments: Vec<String>,
    
    /// Tweaker classes (for legacy Forge/LiteLoader)
    #[serde(default)]
    pub tweakers: Vec<String>,
    
    /// Minimum Java version required
    pub min_java_version: Option<u32>,
    
    /// Recommended Java version
    pub recommended_java_version: Option<u32>,
}

impl ModloaderProfile {
    /// Create a new empty profile
    pub fn new(uid: String, version: String, minecraft_version: String) -> Self {
        Self {
            uid,
            version,
            minecraft_version,
            main_class: String::new(),
            libraries: Vec::new(),
            jvm_arguments: Vec::new(),
            game_arguments: Vec::new(),
            tweakers: Vec::new(),
            min_java_version: None,
            recommended_java_version: None,
        }
    }

    /// Save the profile to a file
    pub fn save(&self, path: &PathBuf) -> crate::core::error::Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Load a profile from a file
    pub fn load(path: &PathBuf) -> crate::core::error::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let profile = serde_json::from_str(&content)?;
        Ok(profile)
    }
}

/// A library required by a modloader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModloaderLibrary {
    /// Maven coordinate (e.g., "net.fabricmc:fabric-loader:0.14.21")
    pub name: String,
    
    /// Download URL (if not from default Maven repos)
    pub url: Option<String>,
    
    /// SHA1 hash for verification
    pub sha1: Option<String>,
    
    /// File size in bytes
    pub size: Option<u64>,
    
    /// Relative path in libraries directory
    pub path: Option<String>,
    
    /// Native classifier (e.g., "natives-windows")
    pub natives: Option<LibraryNatives>,
    
    /// Rules for when this library applies
    #[serde(default)]
    pub rules: Vec<LibraryRule>,
}

impl ModloaderLibrary {
    /// Create a new library from a Maven coordinate
    pub fn from_maven(name: &str) -> Self {
        let path = maven_to_path(name);
        Self {
            name: name.to_string(),
            url: None,
            sha1: None,
            size: None,
            path: Some(path),
            natives: None,
            rules: Vec::new(),
        }
    }

    /// Create with a custom URL
    pub fn with_url(mut self, url: &str) -> Self {
        self.url = Some(url.to_string());
        self
    }

    /// Get the local path for this library
    pub fn get_path(&self) -> String {
        self.path.clone().unwrap_or_else(|| maven_to_path(&self.name))
    }

    /// Get the download URL for this library
    pub fn get_url(&self, base_url: &str) -> String {
        if let Some(url) = &self.url {
            url.clone()
        } else {
            format!("{}{}", base_url, self.get_path())
        }
    }

    /// Check if this library applies to the current OS
    pub fn applies_to_current_os(&self) -> bool {
        if self.rules.is_empty() {
            return true;
        }

        let mut dominated = false;
        let mut dominated_by = false;
        let os = std::env::consts::OS;

        for rule in &self.rules {
            if rule.matches_os(os) {
                dominated = true;
                dominated_by = rule.action == "allow";
            }
        }

        if dominated {
            dominated_by
        } else {
            // No matching rule, check for any "allow" rule
            self.rules.iter().any(|r| r.action == "allow" && r.os.is_none())
        }
    }
}

/// Native library classifiers for different platforms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryNatives {
    pub linux: Option<String>,
    pub osx: Option<String>,
    pub windows: Option<String>,
}

/// A rule for when a library applies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryRule {
    pub action: String,
    pub os: Option<OsRule>,
}

impl LibraryRule {
    /// Check if this rule matches the given OS
    pub fn matches_os(&self, os: &str) -> bool {
        match &self.os {
            Some(os_rule) => {
                let os_name = match os {
                    "windows" => "windows",
                    "macos" => "osx",
                    "linux" => "linux",
                    _ => os,
                };
                os_rule.name.as_ref().map_or(true, |n| n == os_name)
            }
            None => true,
        }
    }
}

/// OS-specific rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsRule {
    pub name: Option<String>,
    pub version: Option<String>,
    pub arch: Option<String>,
}

/// Convert a Maven coordinate to a file path
/// e.g., "net.fabricmc:fabric-loader:0.14.21" -> "net/fabricmc/fabric-loader/0.14.21/fabric-loader-0.14.21.jar"
/// Handles formats:
/// - group:artifact:version
/// - group:artifact:version:classifier
/// - group:artifact:version@extension
/// - group:artifact:version:classifier@extension
pub fn maven_to_path(coordinate: &str) -> String {
    let parts: Vec<&str> = coordinate.split(':').collect();
    if parts.len() < 3 {
        return coordinate.to_string();
    }

    let group = parts[0].replace('.', "/");
    let artifact = parts[1];
    
    // Handle @extension in version or classifier
    let (version, extension_from_version) = if parts[2].contains('@') {
        let v_parts: Vec<&str> = parts[2].split('@').collect();
        (v_parts[0], Some(v_parts[1]))
    } else {
        (parts[2], None)
    };
    
    // Check for classifier (4th part)
    let (classifier, extension) = if parts.len() > 3 {
        // Could be classifier or classifier@extension
        let part = parts[3];
        if part.contains('@') {
            let ext_parts: Vec<&str> = part.split('@').collect();
            (Some(ext_parts[0]), ext_parts.get(1).copied().unwrap_or("jar"))
        } else {
            (Some(part), extension_from_version.unwrap_or("jar"))
        }
    } else {
        (None, extension_from_version.unwrap_or("jar"))
    };

    let filename = match classifier {
        Some(c) if !c.is_empty() => format!("{}-{}-{}.{}", artifact, version, c, extension),
        _ => format!("{}-{}.{}", artifact, version, extension),
    };

    format!("{}/{}/{}/{}", group, artifact, version, filename)
}

/// Convert a file path back to a Maven coordinate (approximate)
pub fn path_to_maven(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() < 4 {
        return None;
    }

    let version = parts[parts.len() - 2];
    let artifact = parts[parts.len() - 3];
    let group = parts[..parts.len() - 3].join(".");

    Some(format!("{}:{}:{}", group, artifact, version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_maven_to_path() {
        assert_eq!(
            maven_to_path("net.fabricmc:fabric-loader:0.14.21"),
            "net/fabricmc/fabric-loader/0.14.21/fabric-loader-0.14.21.jar"
        );
        assert_eq!(
            maven_to_path("org.lwjgl:lwjgl:3.3.1:natives-windows"),
            "org/lwjgl/lwjgl/3.3.1/lwjgl-3.3.1-natives-windows.jar"
        );
    }
}
