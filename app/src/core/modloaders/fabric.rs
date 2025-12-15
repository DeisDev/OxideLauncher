//! Fabric version API client and installer
//!
//! Fabric uses a simple JSON-based installation where the launcher profile
//! contains all necessary information (main class, libraries, etc.)

use std::path::PathBuf;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{info, debug};

use crate::core::error::{OxideError, Result};
use crate::core::instance::ModLoaderType;
use super::profile::{ModloaderProfile, ModloaderLibrary};
use super::installer::{ModloaderInstaller, InstallProgress, ProgressCallback, download_modloader_libraries};

const FABRIC_META_URL: &str = "https://meta.fabricmc.net/v2";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricVersion {
    pub version: String,
    pub stable: bool,
}

// API response types
#[derive(Debug, Deserialize)]
struct FabricLoaderResponse {
    loader: FabricLoaderInfo,
    intermediary: FabricIntermediaryInfo,
    #[serde(rename = "launcherMeta")]
    launcher_meta: FabricLauncherMeta,
}

#[derive(Debug, Deserialize)]
struct FabricLoaderInfo {
    version: String,
    stable: bool,
}

#[derive(Debug, Deserialize)]
struct FabricIntermediaryInfo {
    maven: String,
    version: String,
    stable: bool,
}

#[derive(Debug, Deserialize)]
struct FabricLauncherMeta {
    version: i32,
    libraries: FabricLibraries,
    #[serde(rename = "mainClass")]
    main_class: FabricMainClass,
}

#[derive(Debug, Deserialize)]
struct FabricLibraries {
    client: Vec<FabricLibrary>,
    common: Vec<FabricLibrary>,
    server: Vec<FabricLibrary>,
}

#[derive(Debug, Deserialize)]
struct FabricLibrary {
    name: String,
    url: Option<String>,
    sha1: Option<String>,
    size: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum FabricMainClass {
    Simple(String),
    Complex { client: String, server: String },
}

impl FabricMainClass {
    fn client(&self) -> &str {
        match self {
            FabricMainClass::Simple(s) => s,
            FabricMainClass::Complex { client, .. } => client,
        }
    }
}

/// Fabric modloader installer
pub struct FabricInstaller {
    client: reqwest::Client,
}

impl FabricInstaller {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Fetch the loader profile from Fabric meta
    async fn fetch_profile(&self, minecraft_version: &str, loader_version: &str) -> Result<FabricLoaderResponse> {
        let url = format!(
            "{}/versions/loader/{}/{}",
            FABRIC_META_URL, minecraft_version, loader_version
        );
        
        debug!("Fetching Fabric profile from: {}", url);
        
        let response = self.client
            .get(&url)
            .header("User-Agent", format!("OxideLauncher/{}", env!("CARGO_PKG_VERSION")))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(OxideError::Modloader(format!(
                "Failed to fetch Fabric profile: HTTP {}",
                response.status()
            )));
        }

        let profile: FabricLoaderResponse = response.json().await?;
        Ok(profile)
    }
}

impl Default for FabricInstaller {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ModloaderInstaller for FabricInstaller {
    fn loader_type(&self) -> ModLoaderType {
        ModLoaderType::Fabric
    }

    async fn get_versions(&self, minecraft_version: &str) -> Result<Vec<String>> {
        let versions = get_fabric_versions(minecraft_version).await?;
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

        // Fetch the Fabric profile
        let fabric_profile = self.fetch_profile(minecraft_version, loader_version).await?;
        
        info!(
            "Installing Fabric {} for Minecraft {}",
            loader_version, minecraft_version
        );
        debug!("Fabric main class: {}", fabric_profile.launcher_meta.main_class.client());
        debug!("Intermediary maven: {}", fabric_profile.intermediary.maven);
        debug!("Common libraries count: {}", fabric_profile.launcher_meta.libraries.common.len());
        debug!("Client libraries count: {}", fabric_profile.launcher_meta.libraries.client.len());

        // Build the modloader profile
        let mut profile = ModloaderProfile::new(
            "net.fabricmc.fabric-loader".to_string(),
            loader_version.to_string(),
            minecraft_version.to_string(),
        );

        // Set main class
        profile.main_class = fabric_profile.launcher_meta.main_class.client().to_string();
        debug!("Profile main class set to: {}", profile.main_class);

        // Add the fabric-loader itself (this is critical and not included in the libraries list!)
        let loader_maven = format!("net.fabricmc:fabric-loader:{}", loader_version);
        let loader_path = super::profile::maven_to_path(&loader_maven);
        debug!("Adding fabric-loader library: {} -> {}", loader_maven, loader_path);
        
        let fabric_loader = ModloaderLibrary {
            name: loader_maven,
            url: Some(format!("https://maven.fabricmc.net/{}", loader_path)),
            sha1: None,
            size: None,
            path: None,
            natives: None,
            rules: Vec::new(),
        };
        profile.libraries.push(fabric_loader);

        // Add intermediary library (mappings)
        let intermediary_path = super::profile::maven_to_path(&fabric_profile.intermediary.maven);
        debug!("Adding intermediary library: {} -> {}", fabric_profile.intermediary.maven, intermediary_path);
        
        let intermediary = ModloaderLibrary {
            name: fabric_profile.intermediary.maven.clone(),
            url: Some(format!("https://maven.fabricmc.net/{}", intermediary_path)),
            sha1: None,
            size: None,
            path: None,
            natives: None,
            rules: Vec::new(),
        };
        profile.libraries.push(intermediary);

        // Add common libraries
        for lib in &fabric_profile.launcher_meta.libraries.common {
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
        for lib in &fabric_profile.launcher_meta.libraries.client {
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

        // Download all libraries
        download_modloader_libraries(&profile, libraries_dir, progress.as_ref()).await?;

        if let Some(ref callback) = progress {
            callback(InstallProgress::Complete);
        }

        info!("Fabric installation complete");
        Ok(profile)
    }

    fn is_installed(&self, minecraft_version: &str, loader_version: &str, libraries_dir: &PathBuf) -> bool {
        // Check if the main Fabric loader library exists
        let loader_path = format!(
            "net/fabricmc/fabric-loader/{}/fabric-loader-{}.jar",
            loader_version, loader_version
        );
        libraries_dir.join(loader_path).exists()
    }
}

/// Fetch available Fabric versions for a Minecraft version
pub async fn get_fabric_versions(minecraft_version: &str) -> Result<Vec<FabricVersion>> {
    let client = reqwest::Client::new();
    let url = format!("{}/versions/loader/{}", FABRIC_META_URL, minecraft_version);
    
    let response = client
        .get(&url)
        .header("User-Agent", format!("OxideLauncher/{}", env!("CARGO_PKG_VERSION")))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(OxideError::Modloader(format!(
            "Failed to fetch Fabric versions: HTTP {}",
            response.status()
        )));
    }

    let versions: Vec<FabricLoaderResponse> = response.json().await?;
    
    let fabric_versions: Vec<FabricVersion> = versions.into_iter()
        .map(|v| FabricVersion {
            version: v.loader.version,
            stable: v.loader.stable,
        })
        .collect();
    
    Ok(fabric_versions)
}

/// Get the latest stable Fabric version for a Minecraft version
pub async fn get_recommended_fabric(minecraft_version: &str) -> Result<Option<String>> {
    let versions = get_fabric_versions(minecraft_version).await?;
    Ok(versions.into_iter().find(|v| v.stable).map(|v| v.version))
}
