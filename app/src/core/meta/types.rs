//! PrismLauncher meta format type definitions.
//!
//! Oxide Launcher — A Rust-based Minecraft launcher
//! Copyright (C) 2025 Oxide Launcher contributors
//!
//! This file is part of Oxide Launcher.
//!
//! Oxide Launcher is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License, or
//! (at your option) any later version.
//!
//! Oxide Launcher is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License
//! along with this program. If not, see <https://www.gnu.org/licenses/>.

use serde::{Deserialize, Serialize};

/// Main index response from meta server root.
/// Contains list of all available packages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaIndex {
    #[serde(rename = "formatVersion")]
    pub format_version: u32,
    pub packages: Vec<MetaPackage>,
}

/// Package entry in the main index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetaPackage {
    pub name: String,
    pub sha256: String,
    pub uid: String,
}

/// Package-specific index containing all versions.
/// Fetched from `/{uid}/index.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageIndex {
    #[serde(rename = "formatVersion")]
    pub format_version: u32,
    pub name: String,
    pub uid: String,
    pub versions: Vec<VersionEntry>,
}

/// Version entry within a package index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEntry {
    /// Whether this version is recommended for use.
    #[serde(default)]
    pub recommended: bool,
    
    /// Release timestamp in ISO 8601 format.
    #[serde(rename = "releaseTime")]
    pub release_time: String,
    
    /// Dependencies/requirements for this version.
    #[serde(default)]
    pub requires: Vec<Requirement>,
    
    /// SHA256 hash of the version JSON file.
    pub sha256: String,
    
    /// Version type (release, snapshot, old_alpha, old_beta, experiment).
    /// Only present for some packages like net.minecraft.
    #[serde(rename = "type")]
    pub version_type: Option<String>,
    
    /// Version string (e.g., "1.21.8", "0.18.3", "61.0.3").
    pub version: String,
}

/// Dependency requirement for a version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Requirement {
    /// Package UID this requirement refers to (e.g., "net.minecraft").
    pub uid: String,
    
    /// Exact version required (e.g., "1.21.11" for Forge requiring specific MC version).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equals: Option<String>,
    
    /// Suggested version (e.g., LWJGL version suggested by Minecraft).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggests: Option<String>,
}

/// Known package UIDs in the PrismLauncher meta format.
pub mod uids {
    /// Minecraft versions
    pub const MINECRAFT: &str = "net.minecraft";
    
    /// Fabric Loader
    pub const FABRIC_LOADER: &str = "net.fabricmc.fabric-loader";
    
    /// Fabric Intermediary (dependency of Fabric Loader)
    pub const FABRIC_INTERMEDIARY: &str = "net.fabricmc.intermediary";
    
    /// Forge
    pub const FORGE: &str = "net.minecraftforge";
    
    /// NeoForge
    pub const NEOFORGE: &str = "net.neoforged";
    
    /// Quilt Loader
    pub const QUILT_LOADER: &str = "org.quiltmc.quilt-loader";
    
    /// LiteLoader
    pub const LITELOADER: &str = "com.mumfrey.liteloader";
    
    /// LWJGL3 (suggested dependency for modern Minecraft)
    pub const LWJGL3: &str = "org.lwjgl3";
}

impl VersionEntry {
    /// Check if this version is compatible with a specific Minecraft version.
    /// Returns true if there's a requirement with equals matching the MC version.
    pub fn is_compatible_with(&self, minecraft_version: &str) -> bool {
        self.requires.iter().any(|req| {
            req.uid == uids::MINECRAFT && req.equals.as_deref() == Some(minecraft_version)
        })
    }
    
    /// Get the required Minecraft version, if specified.
    pub fn required_minecraft_version(&self) -> Option<&str> {
        self.requires
            .iter()
            .find(|req| req.uid == uids::MINECRAFT)
            .and_then(|req| req.equals.as_deref())
    }
}

impl PackageIndex {
    /// Filter versions compatible with a specific Minecraft version.
    pub fn versions_for_minecraft(&self, minecraft_version: &str) -> Vec<&VersionEntry> {
        self.versions
            .iter()
            .filter(|v| v.is_compatible_with(minecraft_version))
            .collect()
    }
    
    /// Get all recommended versions.
    pub fn recommended_versions(&self) -> Vec<&VersionEntry> {
        self.versions.iter().filter(|v| v.recommended).collect()
    }
    
    /// Get the latest recommended version for a Minecraft version.
    pub fn latest_recommended_for(&self, minecraft_version: &str) -> Option<&VersionEntry> {
        self.versions_for_minecraft(minecraft_version)
            .into_iter()
            .find(|v| v.recommended)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_compatibility() {
        let entry = VersionEntry {
            recommended: true,
            release_time: "2025-01-01T00:00:00Z".to_string(),
            requires: vec![Requirement {
                uid: uids::MINECRAFT.to_string(),
                equals: Some("1.21.1".to_string()),
                suggests: None,
            }],
            sha256: "test".to_string(),
            version_type: None,
            version: "0.18.3".to_string(),
        };
        
        assert!(entry.is_compatible_with("1.21.1"));
        assert!(!entry.is_compatible_with("1.20.4"));
        assert_eq!(entry.required_minecraft_version(), Some("1.21.1"));
    }
}
