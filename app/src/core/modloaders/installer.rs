//! Modloader installation system
//! 
//! This module provides the infrastructure for installing different modloaders
//! (Fabric, Forge, NeoForge, Quilt, LiteLoader) to Minecraft instances.

use std::path::PathBuf;
use async_trait::async_trait;
use tracing::{info, debug, warn};

use crate::core::error::{OxideError, Result};
use crate::core::download::{download_file, download_file_verified};
use crate::core::instance::ModLoaderType;
use super::profile::{ModloaderProfile, ModloaderLibrary};

/// Progress callback for installation
pub type ProgressCallback = Box<dyn Fn(InstallProgress) + Send + Sync>;

/// Installation progress events
#[derive(Debug, Clone)]
pub enum InstallProgress {
    /// Fetching metadata from the API
    FetchingMetadata,
    /// Downloading libraries
    DownloadingLibraries { current: usize, total: usize },
    /// Processing/extracting files
    Processing(String),
    /// Installation complete
    Complete,
    /// Installation failed
    Failed(String),
}

/// Trait for modloader installers
#[async_trait]
pub trait ModloaderInstaller: Send + Sync {
    /// Get the modloader type
    fn loader_type(&self) -> ModLoaderType;

    /// Get available versions for a Minecraft version
    async fn get_versions(&self, minecraft_version: &str) -> Result<Vec<String>>;

    /// Install the modloader and return the profile
    async fn install(
        &self,
        minecraft_version: &str,
        loader_version: &str,
        libraries_dir: &PathBuf,
        progress: Option<ProgressCallback>,
    ) -> Result<ModloaderProfile>;

    /// Check if a version is installed
    fn is_installed(&self, minecraft_version: &str, loader_version: &str, libraries_dir: &PathBuf) -> bool;
}

/// Download all libraries for a modloader profile
pub async fn download_modloader_libraries(
    profile: &ModloaderProfile,
    libraries_dir: &PathBuf,
    progress: Option<&ProgressCallback>,
) -> Result<()> {
    let libraries: Vec<_> = profile.libraries
        .iter()
        .filter(|lib| lib.applies_to_current_os())
        .collect();

    let total = libraries.len();
    info!("Downloading {} modloader libraries (filtered from {} total)", total, profile.libraries.len());
    
    let mut downloaded = 0;
    let mut skipped = 0;
    let mut failed = 0;
    
    for (i, lib) in libraries.iter().enumerate() {
        if let Some(ref callback) = progress {
            callback(InstallProgress::DownloadingLibraries { current: i + 1, total });
        }

        let lib_path = libraries_dir.join(lib.get_path());
        
        // Skip if already exists
        if lib_path.exists() {
            debug!("Library already exists: {}", lib.name);
            skipped += 1;
            continue;
        }

        // Create parent directories
        if let Some(parent) = lib_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Determine the download URL
        let path = lib.get_path();
        let url = match &lib.url {
            // If URL is provided and looks like a complete URL (ends with .jar), use it directly
            Some(u) if u.ends_with(".jar") || u.ends_with(".lzma") || u.ends_with(".tsrg") => u.clone(),
            // If URL is a base Maven URL, append the path
            Some(u) if u.ends_with('/') => format!("{}{}", u, path),
            // If URL is a base Maven URL without trailing slash, append the path
            Some(u) if !u.contains(&path) => format!("{}/{}", u.trim_end_matches('/'), path),
            // Use URL as-is if it seems complete
            Some(u) => u.clone(),
            // No URL provided, determine based on package name
            None => {
                // Default Maven repositories based on library name
                if lib.name.starts_with("net.fabricmc") || lib.name.starts_with("net.fabricmc.") {
                    format!("https://maven.fabricmc.net/{}", path)
                } else if lib.name.starts_with("org.quiltmc") {
                    format!("https://maven.quiltmc.org/repository/release/{}", path)
                } else if lib.name.starts_with("net.minecraftforge") || lib.name.starts_with("cpw.mods") {
                    format!("https://maven.minecraftforge.net/{}", path)
                } else if lib.name.starts_with("net.neoforged") {
                    format!("https://maven.neoforged.net/releases/{}", path)
                } else if lib.name.starts_with("org.ow2.asm") {
                    // ASM libraries - try Maven Central first
                    format!("https://repo1.maven.org/maven2/{}", path)
                } else {
                    // Default to Maven Central
                    format!("https://repo1.maven.org/maven2/{}", path)
                }
            }
        };

        debug!("Downloading library: {} from {}", lib.name, url);

        // Download with verification if hash is available
        let result = if let Some(sha1) = &lib.sha1 {
            download_file_verified(&url, &lib_path, sha1, None).await
        } else {
            download_file(&url, &lib_path, None).await
        };
        
        match result {
            Ok(_) => {
                downloaded += 1;
            }
            Err(e) => {
                warn!("Failed to download library {}: {}", lib.name, e);
                failed += 1;
                // Continue with other libraries instead of failing completely
            }
        }
    }
    
    info!(
        "Library download complete: {} downloaded, {} skipped (existing), {} failed",
        downloaded, skipped, failed
    );
    
    if failed > 0 {
        warn!("{} libraries failed to download - game may not launch correctly", failed);
    }

    Ok(())
}

/// Get the appropriate installer for a modloader type
pub fn get_installer(loader_type: ModLoaderType) -> Box<dyn ModloaderInstaller> {
    match loader_type {
        ModLoaderType::Fabric => Box::new(super::fabric::FabricInstaller::new()),
        ModLoaderType::Quilt => Box::new(super::quilt::QuiltInstaller::new()),
        ModLoaderType::Forge => Box::new(super::forge::ForgeInstaller::new()),
        ModLoaderType::NeoForge => Box::new(super::neoforge::NeoForgeInstaller::new()),
        ModLoaderType::LiteLoader => Box::new(super::liteloader::LiteLoaderInstaller::new()),
    }
}

/// Install a modloader for an instance
pub async fn install_modloader(
    loader_type: ModLoaderType,
    minecraft_version: &str,
    loader_version: &str,
    libraries_dir: &PathBuf,
    progress: Option<ProgressCallback>,
) -> Result<ModloaderProfile> {
    let installer = get_installer(loader_type);
    
    info!(
        "Installing {} {} for Minecraft {}",
        loader_type.name(),
        loader_version,
        minecraft_version
    );

    let profile = installer
        .install(minecraft_version, loader_version, libraries_dir, progress)
        .await?;

    Ok(profile)
}

/// Check if all libraries for a profile are downloaded
pub fn check_libraries_installed(profile: &ModloaderProfile, libraries_dir: &PathBuf) -> bool {
    for lib in &profile.libraries {
        if !lib.applies_to_current_os() {
            continue;
        }
        
        let lib_path = libraries_dir.join(lib.get_path());
        if !lib_path.exists() {
            return false;
        }
    }
    true
}

/// Get missing libraries for a profile
pub fn get_missing_libraries<'a>(profile: &'a ModloaderProfile, libraries_dir: &PathBuf) -> Vec<&'a ModloaderLibrary> {
    profile.libraries
        .iter()
        .filter(|lib| {
            if !lib.applies_to_current_os() {
                return false;
            }
            let lib_path = libraries_dir.join(lib.get_path());
            !lib_path.exists()
        })
        .collect()
}
