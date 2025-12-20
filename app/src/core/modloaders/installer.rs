//! Modloader installation system.
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
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use async_trait::async_trait;
use tracing::{info, debug, warn};

use crate::core::config::Config;
use crate::core::error::{Result};
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
    #[allow(dead_code)] // Placeholder for future error handling UI
    Failed(String),
}

/// Trait for modloader installers
#[async_trait]
pub trait ModloaderInstaller: Send + Sync {
    /// Get the modloader type
    #[allow(dead_code)] // Part of trait interface for future use
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
    #[allow(dead_code)] // Part of trait interface for future use
    fn is_installed(&self, minecraft_version: &str, loader_version: &str, libraries_dir: &PathBuf) -> bool;
}

/// Check if a library is a natives-only library (has no main JAR)
/// These are libraries like lwjgl-platform, jinput-platform that only contain native files
fn is_natives_only_library(name: &str) -> bool {
    // Parse artifact ID from name (format: group:artifact:version)
    let parts: Vec<&str> = name.split(':').collect();
    if parts.len() < 2 {
        return false;
    }
    let artifact = parts[1];
    
    // Known natives-only libraries
    artifact.ends_with("-platform") 
        || artifact == "twitch-platform" 
        || artifact == "twitch-external-platform"
}

/// Resolve the download URL for a library
fn resolve_library_url(lib: &ModloaderLibrary) -> String {
    let path = lib.get_path();
    match &lib.url {
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
            } else if lib.name.starts_with("net.minecraft:") 
                || lib.name.starts_with("com.mojang:") 
                || lib.name.starts_with("lzma:")
            {
                // Minecraft/Mojang libraries - these are hosted on Mojang's server, not Maven Central
                format!("https://libraries.minecraft.net/{}", path)
            } else {
                // Default to Maven Central
                format!("https://repo1.maven.org/maven2/{}", path)
            }
        }
    }
}

/// Download all libraries for a modloader profile (parallel)
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
    
    // Collect libraries that need downloading (not already present)
    let mut to_download: Vec<(&ModloaderLibrary, PathBuf, String)> = Vec::new();
    let mut skipped = 0;
    
    for lib in &libraries {
        // Skip natives-only libraries (they have no main JAR)
        // These libraries only exist as native classifiers like -natives-windows
        if is_natives_only_library(&lib.name) {
            debug!("Skipping natives-only library: {}", lib.name);
            skipped += 1;
            continue;
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

        let url = resolve_library_url(lib);
        to_download.push((lib, lib_path, url));
    }
    
    if to_download.is_empty() {
        info!("All {} libraries already present, skipping downloads", skipped);
        return Ok(());
    }
    
    // Use config setting for max concurrent downloads
    let config = Config::load().unwrap_or_default();
    let max_concurrent = config.network.max_concurrent_downloads;
    let semaphore = Arc::new(tokio::sync::Semaphore::new(max_concurrent));
    
    // Progress tracking
    let downloaded_count = Arc::new(AtomicUsize::new(0));
    let failed_count = Arc::new(AtomicUsize::new(0));
    let _download_total = to_download.len();
    
    // Initial progress
    if let Some(ref callback) = progress {
        callback(InstallProgress::DownloadingLibraries { current: skipped, total });
    }
    
    // Create download futures
    let download_futures: Vec<_> = to_download.into_iter().map(|(lib, lib_path, url)| {
        let sem = semaphore.clone();
        let downloaded_count = downloaded_count.clone();
        let failed_count = failed_count.clone();
        let lib_name = lib.name.clone();
        let lib_sha1 = lib.sha1.clone();
        
        async move {
            let _permit = sem.acquire().await.unwrap();
            
            debug!("Downloading library: {} from {}", lib_name, url);
            
            // Download with verification if hash is available
            let result = if let Some(sha1) = &lib_sha1 {
                download_file_verified(&url, &lib_path, sha1, None).await
            } else {
                download_file(&url, &lib_path, None).await
            };
            
            match result {
                Ok(_) => {
                    downloaded_count.fetch_add(1, Ordering::SeqCst);
                    Ok(lib_name)
                }
                Err(e) => {
                    warn!("Failed to download library {}: {}", lib_name, e);
                    failed_count.fetch_add(1, Ordering::SeqCst);
                    Err(lib_name)
                }
            }
        }
    }).collect();
    
    // Execute downloads in parallel
    let _results: Vec<_> = futures::future::join_all(download_futures).await;
    
    // Update progress after completion
    let downloaded = downloaded_count.load(Ordering::SeqCst);
    let failed = failed_count.load(Ordering::SeqCst);
    
    if let Some(ref callback) = progress {
        callback(InstallProgress::DownloadingLibraries { current: skipped + downloaded + failed, total });
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
#[allow(dead_code)] // Utility for future pre-launch validation
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
#[allow(dead_code)] // Utility for future incremental download feature
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
