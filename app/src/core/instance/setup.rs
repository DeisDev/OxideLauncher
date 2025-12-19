//! Instance setup - downloading and preparing instance files

#![allow(dead_code)] // Setup fields will be used as features are completed

use std::path::PathBuf;
use tokio::sync::mpsc;
use crate::core::{
    download::{download_file, download_file_verified, download_files, DownloadTask, DownloadProgress},
    error::{OxideError, Result},
    minecraft::{
        version::{fetch_version_manifest, fetch_version_data},
        libraries::{get_missing_libraries, get_native_libraries, get_missing_native_libraries},
        assets::get_missing_assets,
    },
};
use super::Instance;

/// Setup progress events
#[derive(Debug, Clone)]
pub enum SetupProgress {
    DownloadingVersionManifest,
    DownloadingVersionData,
    DownloadingClientJar { progress: u64, total: Option<u64> },
    DownloadingLibraries { current: usize, total: usize },
    ExtractingNatives,
    DownloadingAssetIndex,
    DownloadingAssets { current: usize, total: usize },
    InstallingModloader(String),
    Complete,
    Error(String),
}

/// Setup a newly created instance - download all required files
pub async fn setup_instance(
    instance: &Instance,
    data_dir: &PathBuf,
    progress_tx: Option<mpsc::Sender<SetupProgress>>,
) -> Result<()> {
    // Send progress update helper
    let send_progress = |progress: SetupProgress| {
        if let Some(tx) = &progress_tx {
            let _ = tx.try_send(progress);
        }
    };

    send_progress(SetupProgress::DownloadingVersionManifest);
    
    // 1. Fetch version manifest
    let manifest = fetch_version_manifest().await?;
    let version_info = manifest.get_version(&instance.minecraft_version)
        .ok_or_else(|| OxideError::Instance(format!(
            "Version {} not found in manifest", instance.minecraft_version
        )))?;

    send_progress(SetupProgress::DownloadingVersionData);
    
    // 2. Fetch version data (JSON with all the details)
    let version_data = fetch_version_data(version_info).await?;
    
    // Setup directory structure
    let meta_dir = data_dir.join("meta");
    let versions_dir = meta_dir.join("versions").join(&instance.minecraft_version);
    let libraries_dir = data_dir.join("libraries");
    let assets_dir = data_dir.join("assets");
    let assets_objects_dir = assets_dir.join("objects");
    let assets_indexes_dir = assets_dir.join("indexes");
    
    std::fs::create_dir_all(&versions_dir)?;
    std::fs::create_dir_all(&libraries_dir)?;
    std::fs::create_dir_all(&assets_objects_dir)?;
    std::fs::create_dir_all(&assets_indexes_dir)?;
    
    // 3. Download client JAR
    let client_jar_path = versions_dir.join(format!("{}.jar", &instance.minecraft_version));
    if !client_jar_path.exists() {
        if let Some(client) = &version_data.downloads.client {
            let (download_tx, mut download_rx) = mpsc::channel(100);
            
            // Spawn download task
            let url = client.url.clone();
            let dest = client_jar_path.clone();
            let sha1 = client.sha1.clone();
            
            tokio::spawn(async move {
                let _ = download_file_verified(&url, &dest, &sha1, Some(download_tx)).await;
            });
            
            // Forward progress
            while let Some(progress) = download_rx.recv().await {
                if let DownloadProgress::Progress { downloaded, total, .. } = progress {
                    send_progress(SetupProgress::DownloadingClientJar { progress: downloaded, total });
                }
            }
        }
    }
    
    // 4. Download libraries
    let missing_libs = get_missing_libraries(&version_data, &libraries_dir);
    if !missing_libs.is_empty() {
        send_progress(SetupProgress::DownloadingLibraries { current: 0, total: missing_libs.len() });
        
        let download_tasks: Vec<DownloadTask> = missing_libs.iter().map(|lib| {
            DownloadTask {
                url: lib.url.clone(),
                dest: libraries_dir.join(&lib.path),
                sha1: Some(lib.sha1.clone()),
                size: Some(lib.size),
            }
        }).collect();
        
        let (download_tx, mut download_rx) = mpsc::channel(100);
        let tasks_clone = download_tasks.clone();
        
        // Download in background
        tokio::spawn(async move {
            let _ = download_files(tasks_clone, 5, Some(download_tx)).await;
        });
        
        // Track progress
        let mut completed = 0;
        while let Some(progress) = download_rx.recv().await {
            if let DownloadProgress::Completed { .. } = progress {
                completed += 1;
                send_progress(SetupProgress::DownloadingLibraries { 
                    current: completed, 
                    total: missing_libs.len() 
                });
            }
        }
    }
    
    // 5. Download and extract native libraries
    send_progress(SetupProgress::ExtractingNatives);
    
    // First, download any missing native JARs
    let missing_natives = get_missing_native_libraries(&version_data, &libraries_dir);
    if !missing_natives.is_empty() {
        tracing::info!("Downloading {} native library JARs...", missing_natives.len());
        
        let download_tasks: Vec<DownloadTask> = missing_natives.iter().map(|native| {
            DownloadTask {
                url: native.url.clone(),
                dest: libraries_dir.join(&native.path),
                sha1: Some(native.sha1.clone()),
                size: Some(native.size),
            }
        }).collect();
        
        let (download_tx, mut download_rx) = mpsc::channel(100);
        let tasks_clone = download_tasks.clone();
        
        // Download in background
        tokio::spawn(async move {
            let _ = download_files(tasks_clone, 5, Some(download_tx)).await;
        });
        
        // Wait for all downloads to complete
        while download_rx.recv().await.is_some() {}
        
        tracing::info!("Native library JARs downloaded");
    }
    
    // Now extract natives
    let natives = get_native_libraries(&version_data, &libraries_dir);
    let natives_dir = instance.game_dir().join("natives");
    std::fs::create_dir_all(&natives_dir)?;
    
    for native in natives {
        let native_path = libraries_dir.join(&native.path);
        if native_path.exists() {
            extract_native_library(&native_path, &natives_dir, Some(&native.extract_exclude))?;
        } else {
            tracing::warn!("Native JAR not found: {:?}", native_path);
        }
    }
    
    // 6. Download asset index
    send_progress(SetupProgress::DownloadingAssetIndex);
    let asset_index_path = assets_indexes_dir.join(format!("{}.json", &version_data.assets));
    
    if !asset_index_path.exists() {
        download_file(&version_data.asset_index.url, &asset_index_path, None).await?;
    }
    
    // 7. Load asset index and download missing assets
    if asset_index_path.exists() {
        let asset_index_content = tokio::fs::read_to_string(&asset_index_path).await?;
        let asset_index: crate::core::minecraft::assets::AssetIndexData = 
            serde_json::from_str(&asset_index_content)?;
        
        let missing_assets = get_missing_assets(&asset_index, &assets_dir);
        
        if !missing_assets.is_empty() {
            send_progress(SetupProgress::DownloadingAssets { current: 0, total: missing_assets.len() });
            
            let download_tasks: Vec<DownloadTask> = missing_assets.iter().map(|(_, asset)| {
                DownloadTask {
                    url: asset.get_url(),
                    dest: assets_objects_dir.join(asset.get_path()),
                    sha1: Some(asset.hash.clone()),
                    size: Some(asset.size),
                }
            }).collect();
            
            let (download_tx, mut download_rx) = mpsc::channel(100);
            let tasks_clone = download_tasks.clone();
            
            // Download assets in background
            tokio::spawn(async move {
                let _ = download_files(tasks_clone, 10, Some(download_tx)).await;
            });
            
            // Track progress
            let mut completed = 0;
            while let Some(progress) = download_rx.recv().await {
                if let DownloadProgress::Completed { .. } = progress {
                    completed += 1;
                    send_progress(SetupProgress::DownloadingAssets { 
                        current: completed, 
                        total: missing_assets.len() 
                    });
                }
            }
        }
    }
    
    // 8. Install modloader if present
    if let Some(ref mod_loader) = instance.mod_loader {
        send_progress(SetupProgress::InstallingModloader(mod_loader.loader_type.name().to_string()));
        
        install_modloader_for_instance(instance, &libraries_dir).await?;
    }
    
    send_progress(SetupProgress::Complete);
    
    Ok(())
}

/// Install modloader and save profile for an instance
pub async fn install_modloader_for_instance(
    instance: &Instance,
    libraries_dir: &std::path::Path,
) -> Result<()> {
    use crate::core::modloaders::{install_modloader, InstallProgress, get_installer};
    
    let mod_loader = match &instance.mod_loader {
        Some(ml) => ml,
        None => return Ok(()),
    };
    
    // Resolve "latest" version to actual version
    let loader_version = if mod_loader.version == "latest" || mod_loader.version.is_empty() {
        tracing::info!("Resolving 'latest' version for {} (MC {})", 
            mod_loader.loader_type.name(), 
            instance.minecraft_version
        );
        
        let installer = get_installer(mod_loader.loader_type);
        let versions = installer.get_versions(&instance.minecraft_version).await?;
        
        if versions.is_empty() {
            return Err(crate::core::error::OxideError::Modloader(format!(
                "No {} versions available for Minecraft {}",
                mod_loader.loader_type.name(),
                instance.minecraft_version
            )));
        }
        
        let resolved = versions.first().unwrap().clone();
        tracing::info!("Resolved to version: {}", resolved);
        resolved
    } else {
        mod_loader.version.clone()
    };
    
    tracing::info!(
        "Installing {} {} for instance '{}' (MC {})",
        mod_loader.loader_type.name(),
        loader_version,
        instance.name,
        instance.minecraft_version
    );
    
    tracing::debug!("Libraries directory: {:?}", libraries_dir);
    tracing::debug!("Instance path: {:?}", instance.path);
    
    // Install the modloader
    let profile = install_modloader(
        mod_loader.loader_type,
        &instance.minecraft_version,
        &loader_version,
        &libraries_dir.to_path_buf(),
        Some(Box::new(|progress| {
            match progress {
                InstallProgress::FetchingMetadata => {
                    tracing::debug!("Fetching modloader metadata...");
                }
                InstallProgress::DownloadingLibraries { current, total } => {
                    tracing::debug!("Downloading library {}/{}", current, total);
                }
                InstallProgress::Processing(msg) => {
                    tracing::debug!("Processing: {}", msg);
                }
                InstallProgress::Complete => {
                    tracing::info!("Modloader installation complete");
                }
                InstallProgress::Failed(err) => {
                    tracing::error!("Modloader installation failed: {}", err);
                }
            }
        })),
    ).await?;
    
    tracing::info!(
        "Modloader profile created: uid='{}', version='{}', main_class='{}', libraries={}",
        profile.uid,
        profile.version,
        profile.main_class,
        profile.libraries.len()
    );
    
    // Log library details
    for lib in &profile.libraries {
        tracing::debug!("  Library: {} (url: {:?})", lib.name, lib.url);
    }
    
    // Save the profile to the instance directory
    let profile_path = instance.path.join("modloader_profile.json");
    
    // Ensure the instance directory exists
    if !instance.path.exists() {
        tracing::debug!("Creating instance directory: {:?}", instance.path);
        std::fs::create_dir_all(&instance.path)?;
    }
    
    profile.save(&profile_path)?;
    
    // Verify the file was saved
    if profile_path.exists() {
        let saved_size = std::fs::metadata(&profile_path).map(|m| m.len()).unwrap_or(0);
        tracing::info!("Modloader profile saved to {:?} ({} bytes)", profile_path, saved_size);
    } else {
        tracing::error!("Failed to save modloader profile - file does not exist after save!");
        return Err(crate::core::error::OxideError::Instance(
            "Failed to save modloader profile".to_string()
        ));
    }
    
    Ok(())
}

/// Extract a native library (JAR file) to the natives directory
fn extract_native_library(
    jar_path: &PathBuf,
    natives_dir: &PathBuf,
    exclude_patterns: Option<&[String]>,
) -> Result<()> {

    
    if !jar_path.exists() {
        return Ok(()); // Skip if library doesn't exist
    }
    
    let file = std::fs::File::open(jar_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();
        
        // Check if file should be excluded
        if let Some(patterns) = exclude_patterns {
            let should_exclude = patterns.iter().any(|pattern| {
                name.contains(pattern)
            });
            
            if should_exclude {
                continue;
            }
        }
        
        // Only extract DLL/SO/DYLIB files
        if !name.ends_with(".dll") && !name.ends_with(".so") && !name.ends_with(".dylib") {
            continue;
        }
        
        let target_path = natives_dir.join(&name);
        
        if let Some(parent) = target_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let mut output = std::fs::File::create(&target_path)?;
        std::io::copy(&mut file, &mut output)?;
    }
    
    Ok(())
}
