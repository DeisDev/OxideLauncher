//! Minecraft version manifest and version data structures.
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

#![allow(dead_code)] // Helpers will be used as features are completed

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::core::error::Result;

/// URL for the Minecraft version manifest
const VERSION_MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

/// Minecraft version manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionInfo>,
}

impl VersionManifest {
    /// Get version by ID
    pub fn get_version(&self, id: &str) -> Option<&VersionInfo> {
        self.versions.iter().find(|v| v.id == id)
    }

    /// Get all release versions
    pub fn releases(&self) -> Vec<&VersionInfo> {
        self.versions.iter().filter(|v| v.version_type == VersionType::Release).collect()
    }

    /// Get all snapshot versions
    pub fn snapshots(&self) -> Vec<&VersionInfo> {
        self.versions.iter().filter(|v| v.version_type == VersionType::Snapshot).collect()
    }

    /// Get all versions newer than a specific version
    pub fn newer_than(&self, version_id: &str) -> Vec<&VersionInfo> {
        if let Some(pos) = self.versions.iter().position(|v| v.id == version_id) {
            self.versions[..pos].iter().collect()
        } else {
            Vec::new()
        }
    }
}

/// Latest version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestVersions {
    pub release: String,
    pub snapshot: String,
}

/// Information about a specific version
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: VersionType,
    pub url: String,
    pub time: DateTime<Utc>,
    #[serde(rename = "releaseTime")]
    pub release_time: DateTime<Utc>,
    pub sha1: String,
    #[serde(rename = "complianceLevel")]
    pub compliance_level: Option<u32>,
}

/// Version type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum VersionType {
    Release,
    Snapshot,
    OldBeta,
    OldAlpha,
}

impl VersionType {
    pub fn display_name(&self) -> &'static str {
        match self {
            VersionType::Release => "Release",
            VersionType::Snapshot => "Snapshot",
            VersionType::OldBeta => "Old Beta",
            VersionType::OldAlpha => "Old Alpha",
        }
    }
}

/// Detailed version data (from version-specific JSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionData {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: VersionType,
    pub main_class: String,
    pub minimum_launcher_version: Option<u32>,
    pub release_time: DateTime<Utc>,
    pub time: DateTime<Utc>,
    pub assets: String,
    pub asset_index: AssetIndex,
    pub downloads: Downloads,
    pub libraries: Vec<Library>,
    #[serde(default)]
    pub arguments: Option<Arguments>,
    #[serde(rename = "minecraftArguments")]
    pub minecraft_arguments: Option<String>,
    pub java_version: Option<JavaVersion>,
    pub logging: Option<Logging>,
    #[serde(rename = "inheritsFrom")]
    pub inherits_from: Option<String>,
}

/// Asset index information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIndex {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub total_size: Option<u64>,
    pub url: String,
}

/// Download information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Downloads {
    pub client: Option<DownloadInfo>,
    pub server: Option<DownloadInfo>,
    pub client_mappings: Option<DownloadInfo>,
    pub server_mappings: Option<DownloadInfo>,
}

/// Information about a downloadable file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadInfo {
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

/// Library dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub name: String,
    pub downloads: Option<LibraryDownloads>,
    pub natives: Option<std::collections::HashMap<String, String>>,
    pub rules: Option<Vec<Rule>>,
    pub extract: Option<ExtractRules>,
    pub url: Option<String>,
}

impl Library {
    /// Get the artifact path for this library
    pub fn artifact_path(&self) -> String {
        let parts: Vec<&str> = self.name.split(':').collect();
        if parts.len() >= 3 {
            let group = parts[0].replace('.', "/");
            let artifact = parts[1];
            let version = parts[2];
            
            // Check for classifier
            if parts.len() >= 4 {
                let classifier = parts[3];
                format!("{}/{}/{}/{}-{}-{}.jar", group, artifact, version, artifact, version, classifier)
            } else {
                format!("{}/{}/{}/{}-{}.jar", group, artifact, version, artifact, version)
            }
        } else {
            self.name.clone()
        }
    }

    /// Check if this library should be used on the current OS
    pub fn applies_to_current_os(&self) -> bool {
        if let Some(rules) = &self.rules {
            evaluate_rules(rules)
        } else {
            true
        }
    }

    /// Get native classifier for current OS
    pub fn native_classifier(&self) -> Option<String> {
        let os = current_os_name();
        self.natives.as_ref()?.get(os).cloned()
    }
}

/// Library download information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<LibraryArtifact>,
    pub classifiers: Option<std::collections::HashMap<String, LibraryArtifact>>,
}

/// Library artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryArtifact {
    pub path: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

/// Rule for conditional application
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub action: RuleAction,
    pub os: Option<OsRule>,
    pub features: Option<std::collections::HashMap<String, bool>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuleAction {
    Allow,
    Disallow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsRule {
    pub name: Option<String>,
    pub version: Option<String>,
    pub arch: Option<String>,
}

/// Extraction rules for natives
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractRules {
    pub exclude: Option<Vec<String>>,
}

/// Game arguments (modern format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arguments {
    pub game: Vec<ArgumentValue>,
    pub jvm: Vec<ArgumentValue>,
}

/// Argument value (can be string or conditional)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    Simple(String),
    Conditional {
        rules: Vec<Rule>,
        value: ArgumentValueInner,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValueInner {
    Single(String),
    Multiple(Vec<String>),
}

/// Java version requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaVersion {
    pub component: String,
    #[serde(rename = "majorVersion")]
    pub major_version: u32,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logging {
    pub client: Option<LoggingConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub argument: String,
    pub file: LoggingFile,
    #[serde(rename = "type")]
    pub log_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingFile {
    pub id: String,
    pub sha1: String,
    pub size: u64,
    pub url: String,
}

// Functions

/// Fetch the version manifest from Mojang
pub async fn fetch_version_manifest() -> Result<VersionManifest> {
    let client = reqwest::Client::new();
    let response = client
        .get(VERSION_MANIFEST_URL)
        .send()
        .await?
        .json::<VersionManifest>()
        .await?;
    
    Ok(response)
}

/// Fetch detailed version data
pub async fn fetch_version_data(version: &VersionInfo) -> Result<VersionData> {
    let client = reqwest::Client::new();
    let response = client
        .get(&version.url)
        .send()
        .await?
        .json::<VersionData>()
        .await?;
    
    Ok(response)
}

/// Get current OS name in Minecraft format
pub fn current_os_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "osx"
    } else {
        "linux"
    }
}

/// Get current architecture in Minecraft format
pub fn current_arch() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else {
        "unknown"
    }
}

/// Launch features used for conditional argument evaluation
/// These control which conditional arguments are included in the launch command
#[derive(Debug, Clone, Default)]
pub struct LaunchFeatures {
    /// Whether the user is in demo mode (no valid license)
    pub is_demo_user: bool,
    /// Whether custom resolution is set
    pub has_custom_resolution: bool,
    /// Whether quick play is supported (MC 1.20+)
    pub has_quick_plays_support: bool,
    /// Whether to quick play into a singleplayer world
    pub is_quick_play_singleplayer: bool,
    /// Whether to quick play into a multiplayer server
    pub is_quick_play_multiplayer: bool,
    /// Whether to quick play into a realm
    pub is_quick_play_realms: bool,
}

impl LaunchFeatures {
    /// Create features for normal launch
    pub fn normal() -> Self {
        Self::default()
    }
    
    /// Create features for demo mode
    pub fn demo() -> Self {
        Self {
            is_demo_user: true,
            ..Default::default()
        }
    }
    
    /// Create features with custom resolution
    pub fn with_custom_resolution(mut self) -> Self {
        self.has_custom_resolution = true;
        self
    }
}

/// Evaluate rules to determine if something applies (OS-only, no features)
pub fn evaluate_rules(rules: &[Rule]) -> bool {
    evaluate_rules_with_features(rules, &LaunchFeatures::default())
}

/// Evaluate rules with feature context to determine if something applies
pub fn evaluate_rules_with_features(rules: &[Rule], features: &LaunchFeatures) -> bool {
    let current_os = current_os_name();
    let current_arch = current_arch();
    
    let mut result = false;
    
    for rule in rules {
        // Check OS conditions
        let os_matches = if let Some(os) = &rule.os {
            let os_matches = os.name.as_ref().map(|n| n == current_os).unwrap_or(true);
            let arch_matches = os.arch.as_ref().map(|a| a == current_arch).unwrap_or(true);
            os_matches && arch_matches
        } else {
            true
        };
        
        // Check feature conditions
        let features_match = if let Some(rule_features) = &rule.features {
            let mut all_match = true;
            
            // Check each feature that's specified in the rule
            if let Some(&required) = rule_features.get("is_demo_user") {
                all_match = all_match && (features.is_demo_user == required);
            }
            if let Some(&required) = rule_features.get("has_custom_resolution") {
                all_match = all_match && (features.has_custom_resolution == required);
            }
            if let Some(&required) = rule_features.get("has_quick_plays_support") {
                all_match = all_match && (features.has_quick_plays_support == required);
            }
            if let Some(&required) = rule_features.get("is_quick_play_singleplayer") {
                all_match = all_match && (features.is_quick_play_singleplayer == required);
            }
            if let Some(&required) = rule_features.get("is_quick_play_multiplayer") {
                all_match = all_match && (features.is_quick_play_multiplayer == required);
            }
            if let Some(&required) = rule_features.get("is_quick_play_realms") {
                all_match = all_match && (features.is_quick_play_realms == required);
            }
            
            all_match
        } else {
            true
        };
        
        let matches = os_matches && features_match;
        
        if matches {
            result = rule.action == RuleAction::Allow;
        }
    }
    
    result
}
