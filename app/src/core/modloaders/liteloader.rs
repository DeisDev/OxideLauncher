//! LiteLoader version API client and installer.
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

use std::path::PathBuf;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::core::error::{OxideError, Result};
use crate::core::instance::ModLoaderType;
use super::profile::{ModloaderProfile, ModloaderLibrary};
use super::installer::{ModloaderInstaller, InstallProgress, ProgressCallback, download_modloader_libraries};

const LITELOADER_VERSIONS_URL: &str = "http://dl.liteloader.com/versions/versions.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteLoaderVersion {
    pub version: String,
    pub minecraft_version: String,
}

#[derive(Debug, Deserialize)]
struct LiteLoaderVersionsResponse {
    versions: std::collections::HashMap<String, LiteLoaderMcVersions>,
}

#[derive(Debug, Deserialize)]
struct LiteLoaderMcVersions {
    snapshots: Option<std::collections::HashMap<String, LiteLoaderVersionInfo>>,
    releases: Option<std::collections::HashMap<String, LiteLoaderVersionInfo>>,
    #[serde(rename = "artefacts")]
    artefacts: Option<LiteLoaderArtefacts>,
}

#[derive(Debug, Deserialize)]
struct LiteLoaderArtefacts {
    #[serde(rename = "com.mumfrey:liteloader")]
    liteloader: Option<std::collections::HashMap<String, LiteLoaderArtefact>>,
}

#[derive(Debug, Deserialize)]
struct LiteLoaderArtefact {
    #[serde(rename = "tweakClass")]
    tweak_class: Option<String>,
    libraries: Option<Vec<LiteLoaderLibraryInfo>>,
    #[allow(dead_code)] // Placeholder for future Technic import support
    file: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LiteLoaderLibraryInfo {
    name: String,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LiteLoaderVersionInfo {
    version: String,
}

/// LiteLoader modloader installer
pub struct LiteLoaderInstaller {
    client: reqwest::Client,
}

impl LiteLoaderInstaller {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

impl Default for LiteLoaderInstaller {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ModloaderInstaller for LiteLoaderInstaller {
    fn loader_type(&self) -> ModLoaderType {
        ModLoaderType::LiteLoader
    }

    async fn get_versions(&self, minecraft_version: &str) -> Result<Vec<String>> {
        let versions = get_liteloader_versions(minecraft_version).await?;
        Ok(versions.into_iter().map(|v| v.version).collect())
    }

    async fn install(
        &self,
        minecraft_version: &str,
        loader_version: &str,
        libraries_dir: &PathBuf,
        progress: Option<ProgressCallback>,
    ) -> Result<ModloaderProfile> {
        if let Some(ref callback) = progress {
            callback(InstallProgress::FetchingMetadata);
        }

        warn!("LiteLoader is a legacy modloader and may not work correctly");

        // Fetch LiteLoader metadata
        let response = self.client
            .get(LITELOADER_VERSIONS_URL)
            .header("User-Agent", format!("OxideLauncher/{}", env!("CARGO_PKG_VERSION")))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OxideError::Modloader(format!(
                "Failed to fetch LiteLoader versions: HTTP {}",
                response.status()
            )));
        }

        let versions_response: LiteLoaderVersionsResponse = response.json().await?;
        
        let mc_versions = versions_response.versions.get(minecraft_version)
            .ok_or_else(|| OxideError::Modloader(format!(
                "No LiteLoader available for Minecraft {}",
                minecraft_version
            )))?;

        // Build profile
        let mut profile = ModloaderProfile::new(
            "com.mumfrey.liteloader".to_string(),
            loader_version.to_string(),
            minecraft_version.to_string(),
        );

        // LiteLoader uses the vanilla main class with a tweaker
        profile.main_class = "net.minecraft.client.main.Main".to_string();
        profile.tweakers.push("com.mumfrey.liteloader.launch.LiteLoaderTweaker".to_string());

        // Add LiteLoader library
        profile.libraries.push(ModloaderLibrary {
            name: format!("com.mumfrey:liteloader:{}", loader_version),
            url: Some(format!(
                "http://dl.liteloader.com/versions/com/mumfrey/liteloader/{}/liteloader-{}.jar",
                minecraft_version, loader_version
            )),
            sha1: None,
            size: None,
            path: None,
            natives: None,
            rules: Vec::new(),
        });

        // Add additional libraries from artefacts if available
        if let Some(artefacts) = &mc_versions.artefacts {
            if let Some(ll_artefacts) = &artefacts.liteloader {
                if let Some(artefact) = ll_artefacts.get(loader_version) {
                    if let Some(libs) = &artefact.libraries {
                        for lib in libs {
                            profile.libraries.push(ModloaderLibrary {
                                name: lib.name.clone(),
                                url: lib.url.clone(),
                                sha1: None,
                                size: None,
                                path: None,
                                natives: None,
                                rules: Vec::new(),
                            });
                        }
                    }
                    if let Some(tweak_class) = &artefact.tweak_class {
                        profile.tweakers.clear();
                        profile.tweakers.push(tweak_class.clone());
                    }
                }
            }
        }

        // Download libraries
        download_modloader_libraries(&profile, libraries_dir, progress.as_ref()).await?;

        if let Some(ref callback) = progress {
            callback(InstallProgress::Complete);
        }

        info!("LiteLoader installation complete");
        Ok(profile)
    }

    fn is_installed(&self, minecraft_version: &str, loader_version: &str, libraries_dir: &PathBuf) -> bool {
        let loader_path = format!(
            "com/mumfrey/liteloader/{}/liteloader-{}.jar",
            minecraft_version, loader_version
        );
        libraries_dir.join(loader_path).exists()
    }
}

/// Fetch available LiteLoader versions for a Minecraft version
pub async fn get_liteloader_versions(minecraft_version: &str) -> Result<Vec<LiteLoaderVersion>> {
    let client = reqwest::Client::new();
    let response = client
        .get(LITELOADER_VERSIONS_URL)
        .header("User-Agent", format!("OxideLauncher/{}", env!("CARGO_PKG_VERSION")))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(OxideError::Modloader(format!(
            "Failed to fetch LiteLoader versions: HTTP {}",
            response.status()
        )));
    }

    let versions_response: LiteLoaderVersionsResponse = response.json().await?;
    
    let mut versions = Vec::new();
    
    if let Some(mc_versions) = versions_response.versions.get(minecraft_version) {
        // Add releases
        if let Some(releases) = &mc_versions.releases {
            for info in releases.values() {
                versions.push(LiteLoaderVersion {
                    version: info.version.clone(),
                    minecraft_version: minecraft_version.to_string(),
                });
            }
        }
        
        // Add snapshots
        if let Some(snapshots) = &mc_versions.snapshots {
            for info in snapshots.values() {
                versions.push(LiteLoaderVersion {
                    version: info.version.clone(),
                    minecraft_version: minecraft_version.to_string(),
                });
            }
        }
    }
    
    Ok(versions)
}

/// Get the latest LiteLoader version for a Minecraft version
#[allow(dead_code)] // Utility function for future auto-select feature
pub async fn get_recommended_liteloader(minecraft_version: &str) -> Result<Option<String>> {
    let versions = get_liteloader_versions(minecraft_version).await?;
    Ok(versions.into_iter().next().map(|v| v.version))
}
