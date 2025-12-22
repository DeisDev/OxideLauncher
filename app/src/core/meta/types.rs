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
    
    /// Version type (release, snapshot, old_alpha, old_beta, old_snapshot).
    /// Only present for some packages like net.minecraft.
    #[serde(rename = "type")]
    pub version_type: Option<String>,
    
    /// Version string (e.g., "1.21.8", "0.18.3", "61.0.3").
    pub version: String,
    
    /// Whether this version is volatile (may change).
    /// Present on LWJGL packages.
    #[serde(default)]
    pub volatile: bool,
    
    /// Package UIDs that this version conflicts with.
    /// Present on LWJGL3 (conflicts with org.lwjgl).
    #[serde(default)]
    pub conflicts: Vec<Conflict>,
}

/// A conflict declaration for a version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conflict {
    /// Package UID that conflicts with this version.
    pub uid: String,
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
    // ===== Minecraft =====
    /// Minecraft versions
    pub const MINECRAFT: &str = "net.minecraft";
    
    // ===== Modloaders =====
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
    
    // ===== Libraries =====
    /// LWJGL 2 (legacy Minecraft)
    pub const LWJGL: &str = "org.lwjgl";
    
    /// LWJGL 3 (modern Minecraft)
    pub const LWJGL3: &str = "org.lwjgl3";
    
    // ===== Java Runtimes =====
    /// Azul Zulu Java runtimes
    pub const JAVA_AZUL: &str = "com.azul.java";
    
    /// Eclipse Adoptium Java runtimes
    pub const JAVA_ADOPTIUM: &str = "net.adoptium.java";
    
    /// Mojang's bundled Java runtimes
    pub const JAVA_MOJANG: &str = "net.minecraft.java";
}

impl VersionEntry {
    /// Check if this version is compatible with a specific Minecraft version.
    /// Returns true if there's a requirement with equals matching the MC version.
    /// 
    /// Note: Some loaders (Fabric, Quilt) don't have direct MC version requirements,
    /// they require intermediary instead. Use `has_minecraft_requirement()` to check.
    pub fn is_compatible_with(&self, minecraft_version: &str) -> bool {
        self.requires.iter().any(|req| {
            req.uid == uids::MINECRAFT && req.equals.as_deref() == Some(minecraft_version)
        })
    }
    
    /// Check if this version has a direct Minecraft version requirement.
    /// Forge/NeoForge/LiteLoader have this, Fabric/Quilt do not.
    pub fn has_minecraft_requirement(&self) -> bool {
        self.requires.iter().any(|req| req.uid == uids::MINECRAFT)
    }
    
    /// Check if this version requires intermediary (Fabric/Quilt pattern).
    pub fn requires_intermediary(&self) -> bool {
        self.requires.iter().any(|req| req.uid == uids::FABRIC_INTERMEDIARY)
    }
    
    /// Get the required Minecraft version, if specified.
    pub fn required_minecraft_version(&self) -> Option<&str> {
        self.requires
            .iter()
            .find(|req| req.uid == uids::MINECRAFT)
            .and_then(|req| req.equals.as_deref())
    }
    
    /// Check if this is a release type version.
    pub fn is_release(&self) -> bool {
        self.version_type.as_deref() == Some("release")
    }
    
    /// Check if this is a snapshot type version.
    pub fn is_snapshot(&self) -> bool {
        self.version_type.as_deref() == Some("snapshot")
    }
    
    /// Check if this is an old version (alpha, beta, old_snapshot).
    pub fn is_old(&self) -> bool {
        matches!(
            self.version_type.as_deref(),
            Some("old_alpha") | Some("old_beta") | Some("old_snapshot")
        )
    }
}

impl PackageIndex {
    /// Filter versions compatible with a specific Minecraft version.
    /// For Forge/NeoForge/LiteLoader that have direct MC requirements.
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
    /// Works for loaders with direct MC requirements (Forge, NeoForge, LiteLoader).
    pub fn latest_recommended_for(&self, minecraft_version: &str) -> Option<&VersionEntry> {
        self.versions_for_minecraft(minecraft_version)
            .into_iter()
            .find(|v| v.recommended)
    }
    
    /// Get the latest recommended version (for loaders without MC requirements).
    /// Falls back to first version if none are marked recommended (e.g., Quilt beta).
    pub fn latest_recommended(&self) -> Option<&VersionEntry> {
        self.versions
            .iter()
            .find(|v| v.recommended)
            .or_else(|| self.versions.first())
    }
    
    /// Check if this package uses intermediary-based compatibility (Fabric/Quilt).
    pub fn uses_intermediary(&self) -> bool {
        self.uid == uids::FABRIC_LOADER || self.uid == uids::QUILT_LOADER
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entry(requires: Vec<Requirement>, version_type: Option<&str>) -> VersionEntry {
        VersionEntry {
            recommended: true,
            release_time: "2025-01-01T00:00:00Z".to_string(),
            requires,
            sha256: "test".to_string(),
            version_type: version_type.map(String::from),
            version: "0.18.3".to_string(),
            volatile: false,
            conflicts: vec![],
        }
    }

    #[test]
    fn test_version_compatibility() {
        let entry = test_entry(
            vec![Requirement {
                uid: uids::MINECRAFT.to_string(),
                equals: Some("1.21.1".to_string()),
                suggests: None,
            }],
            None,
        );
        
        assert!(entry.is_compatible_with("1.21.1"));
        assert!(!entry.is_compatible_with("1.20.4"));
        assert_eq!(entry.required_minecraft_version(), Some("1.21.1"));
        assert!(entry.has_minecraft_requirement());
    }
    
    #[test]
    fn test_intermediary_based_loader() {
        let entry = test_entry(
            vec![Requirement {
                uid: uids::FABRIC_INTERMEDIARY.to_string(),
                equals: None,
                suggests: None,
            }],
            Some("release"),
        );
        
        assert!(!entry.has_minecraft_requirement());
        assert!(entry.requires_intermediary());
        assert!(entry.is_release());
    }
    
    #[test]
    fn test_version_types() {
        assert!(test_entry(vec![], Some("release")).is_release());
        assert!(test_entry(vec![], Some("snapshot")).is_snapshot());
        assert!(test_entry(vec![], Some("old_alpha")).is_old());
        assert!(test_entry(vec![], Some("old_beta")).is_old());
        assert!(test_entry(vec![], Some("old_snapshot")).is_old());
    }
}
