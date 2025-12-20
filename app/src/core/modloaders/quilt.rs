//! Quilt version API client and installer.
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
use tracing::{info, debug};

use crate::core::error::{OxideError, Result};
use crate::core::instance::ModLoaderType;
use super::profile::{ModloaderProfile, ModloaderLibrary};
use super::installer::{ModloaderInstaller, InstallProgress, ProgressCallback, download_modloader_libraries};

const QUILT_META_URL: &str = "https://meta.quiltmc.org/v3";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuiltVersion {
    pub version: String,
}

// API response types
#[derive(Debug, Deserialize)]
struct QuiltLoaderResponse {
    loader: QuiltLoaderInfo,
    hashed: Option<QuiltHashedInfo>,
    intermediary: Option<QuiltIntermediaryInfo>,
    #[serde(rename = "launcherMeta")]
    launcher_meta: QuiltLauncherMeta,
}

#[derive(Debug, Deserialize)]
struct QuiltLoaderInfo {
    version: String,
}

#[derive(Debug, Deserialize)]
struct QuiltHashedInfo {
    maven: String,
}

#[derive(Debug, Deserialize)]
struct QuiltIntermediaryInfo {
    maven: String,
}

#[derive(Debug, Deserialize)]
struct QuiltLauncherMeta {
    libraries: QuiltLibraries,
    #[serde(rename = "mainClass")]
    main_class: QuiltMainClass,
}

#[derive(Debug, Deserialize)]
struct QuiltLibraries {
    client: Vec<QuiltLibrary>,
    common: Vec<QuiltLibrary>,
    #[allow(dead_code)] // Reserved for server-side installation
    server: Vec<QuiltLibrary>,
}

#[derive(Debug, Deserialize)]
struct QuiltLibrary {
    name: String,
    url: Option<String>,
    sha1: Option<String>,
    size: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum QuiltMainClass {
    Simple(String),
    Complex { client: String, #[allow(dead_code)] server: String },
}

impl QuiltMainClass {
    fn client(&self) -> &str {
        match self {
            QuiltMainClass::Simple(s) => s,
            QuiltMainClass::Complex { client, .. } => client,
        }
    }
}

/// Quilt modloader installer
pub struct QuiltInstaller {
    client: reqwest::Client,
}

impl QuiltInstaller {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Fetch the loader profile from Quilt meta
    async fn fetch_profile(&self, minecraft_version: &str, loader_version: &str) -> Result<QuiltLoaderResponse> {
        let url = format!(
            "{}/versions/loader/{}/{}",
            QUILT_META_URL, minecraft_version, loader_version
        );
        
        debug!("Fetching Quilt profile from: {}", url);
        
        let response = self.client
            .get(&url)
            .header("User-Agent", format!("OxideLauncher/{}", env!("CARGO_PKG_VERSION")))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OxideError::Modloader(format!(
                "Failed to fetch Quilt profile: HTTP {}",
                response.status()
            )));
        }

        let profile: QuiltLoaderResponse = response.json().await?;
        Ok(profile)
    }
}

impl Default for QuiltInstaller {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ModloaderInstaller for QuiltInstaller {
    fn loader_type(&self) -> ModLoaderType {
        ModLoaderType::Quilt
    }

    async fn get_versions(&self, minecraft_version: &str) -> Result<Vec<String>> {
        let versions = get_quilt_versions(minecraft_version).await?;
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

        // Fetch the Quilt profile
        let quilt_profile = self.fetch_profile(minecraft_version, loader_version).await?;
        
        info!(
            "Installing Quilt {} for Minecraft {}",
            loader_version, minecraft_version
        );
        debug!("Quilt main class: {}", quilt_profile.launcher_meta.main_class.client());
        debug!("Common libraries count: {}", quilt_profile.launcher_meta.libraries.common.len());
        debug!("Client libraries count: {}", quilt_profile.launcher_meta.libraries.client.len());

        // Build the modloader profile
        let mut profile = ModloaderProfile::new(
            "org.quiltmc.quilt-loader".to_string(),
            loader_version.to_string(),
            minecraft_version.to_string(),
        );

        // Set main class
        profile.main_class = quilt_profile.launcher_meta.main_class.client().to_string();
        debug!("Profile main class set to: {}", profile.main_class);

        // Add the quilt-loader itself (this is critical and not included in the libraries list!)
        let loader_maven = format!("org.quiltmc:quilt-loader:{}", loader_version);
        let loader_path = super::profile::maven_to_path(&loader_maven);
        debug!("Adding quilt-loader library: {} -> {}", loader_maven, loader_path);
        
        let quilt_loader = ModloaderLibrary {
            name: loader_maven,
            url: Some(format!("https://maven.quiltmc.org/repository/release/{}", loader_path)),
            sha1: None,
            size: None,
            path: None,
            natives: None,
            rules: Vec::new(),
        };
        profile.libraries.push(quilt_loader);

        // Add hashed library (Quilt's mappings)
        if let Some(hashed) = &quilt_profile.hashed {
            let hashed_path = super::profile::maven_to_path(&hashed.maven);
            debug!("Adding hashed library: {} -> {}", hashed.maven, hashed_path);
            profile.libraries.push(ModloaderLibrary {
                name: hashed.maven.clone(),
                url: Some(format!("https://maven.quiltmc.org/repository/release/{}", hashed_path)),
                sha1: None,
                size: None,
                path: None,
                natives: None,
                rules: Vec::new(),
            });
        }

        // Add intermediary library if present (fallback for older versions)
        if let Some(intermediary) = &quilt_profile.intermediary {
            let intermediary_path = super::profile::maven_to_path(&intermediary.maven);
            debug!("Adding intermediary library: {} -> {}", intermediary.maven, intermediary_path);
            profile.libraries.push(ModloaderLibrary {
                name: intermediary.maven.clone(),
                url: Some(format!("https://maven.fabricmc.net/{}", intermediary_path)),
                sha1: None,
                size: None,
                path: None,
                natives: None,
                rules: Vec::new(),
            });
        }

        // Add common libraries
        for lib in &quilt_profile.launcher_meta.libraries.common {
            debug!("Adding common library: {}", lib.name);
            profile.libraries.push(ModloaderLibrary {
                name: lib.name.clone(),
                url: lib.url.clone(),
                sha1: lib.sha1.clone(),
                size: lib.size,
                path: None,
                natives: None,
                rules: Vec::new(),
            });
        }

        // Add client libraries
        for lib in &quilt_profile.launcher_meta.libraries.client {
            debug!("Adding client library: {}", lib.name);
            profile.libraries.push(ModloaderLibrary {
                name: lib.name.clone(),
                url: lib.url.clone(),
                sha1: lib.sha1.clone(),
                size: lib.size,
                path: None,
                natives: None,
                rules: Vec::new(),
            });
        }

        debug!("Total libraries in profile: {}", profile.libraries.len());

        // Download all libraries
        download_modloader_libraries(&profile, libraries_dir, progress.as_ref()).await?;

        if let Some(ref callback) = progress {
            callback(InstallProgress::Complete);
        }

        info!("Quilt installation complete");
        Ok(profile)
    }

    fn is_installed(&self, _minecraft_version: &str, loader_version: &str, libraries_dir: &PathBuf) -> bool {
        // Check if the main Quilt loader library exists
        let loader_path = format!(
            "org/quiltmc/quilt-loader/{}/quilt-loader-{}.jar",
            loader_version, loader_version
        );
        libraries_dir.join(loader_path).exists()
    }
}

/// Fetch available Quilt versions for a Minecraft version
pub async fn get_quilt_versions(minecraft_version: &str) -> Result<Vec<QuiltVersion>> {
    let client = reqwest::Client::new();
    let url = format!("{}/versions/loader/{}", QUILT_META_URL, minecraft_version);
    
    let response = client
        .get(&url)
        .header("User-Agent", format!("OxideLauncher/{}", env!("CARGO_PKG_VERSION")))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(OxideError::Modloader(format!(
            "Failed to fetch Quilt versions: HTTP {}",
            response.status()
        )));
    }

    let versions: Vec<QuiltLoaderResponse> = response.json().await?;
    
    let quilt_versions: Vec<QuiltVersion> = versions.into_iter()
        .map(|v| QuiltVersion {
            version: v.loader.version,
        })
        .collect();
    
    Ok(quilt_versions)
}

/// Get the latest Quilt version for a Minecraft version
#[allow(dead_code)] // Utility function for future auto-select feature
pub async fn get_recommended_quilt(minecraft_version: &str) -> Result<Option<String>> {
    let versions = get_quilt_versions(minecraft_version).await?;
    Ok(versions.into_iter().next().map(|v| v.version))
}
