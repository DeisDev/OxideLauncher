//! Mod download commands with RustWiz metadata integration.
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

use crate::commands::state::AppState;
use crate::core::download::download_file;
use crate::core::modplatform::{
    curseforge::CurseForgeClient, 
    modrinth::ModrinthClient,
};
use crate::core::rustwiz::{
    self, ModToml, ModTomlExtended, OxideMetadata,
    HashFormat, Side,
};
use tauri::{State, Emitter, AppHandle};
use std::sync::Arc;
use std::path::Path;
use tokio::sync::Semaphore;

// =============================================================================
// RustWiz Metadata Helper
// =============================================================================

/// Create RustWiz metadata (.pw.toml) for a downloaded mod
/// 
/// This creates packwiz-compatible metadata that enables:
/// - Mod update checking via platform APIs
/// - Modpack export to .mrpack or CurseForge formats
/// 
/// Metadata is stored in mods/.index/<slug>.pw.toml following Prism Launcher's approach.
fn create_mod_metadata(
    mods_dir: &Path,
    filename: &str,
    name: &str,
    download_url: &str,
    hash: &str,
    hash_format: HashFormat,
    platform: &str,
    project_id: &str,
    version_id: &str,
    icon_url: Option<String>,
    description: Option<String>,
    mc_versions: Option<Vec<String>>,
    loaders: Option<Vec<String>>,
) {
    // Create base mod toml
    let mut mod_toml = ModToml::new(
        name.to_string(),
        format!("mods/{}", filename),
        download_url.to_string(),
        hash.to_string(),
        hash_format,
    ).with_side(Side::Both);
    
    // Add update source based on platform
    match platform.to_lowercase().as_str() {
        "modrinth" => {
            mod_toml = mod_toml.with_modrinth_update(project_id.to_string(), version_id.to_string());
        }
        "curseforge" => {
            if let (Ok(pid), Ok(fid)) = (project_id.parse::<u32>(), version_id.parse::<u32>()) {
                mod_toml = mod_toml.with_curseforge_update(pid, fid);
            }
        }
        _ => {}
    }
    
    // Create extended toml with OxideLauncher metadata
    let mut extended = ModTomlExtended::from_packwiz(mod_toml);
    
    // Add oxide metadata with all available info
    let has_oxide_data = icon_url.is_some() 
        || description.is_some()
        || mc_versions.as_ref().map_or(false, |v| !v.is_empty())
        || loaders.as_ref().map_or(false, |v| !v.is_empty());
    
    if has_oxide_data {
        extended.oxide = Some(OxideMetadata {
            icon_url,
            description,
            mc_versions: mc_versions.unwrap_or_default(),
            loaders: loaders.unwrap_or_default(),
            ..Default::default()
        });
    }
    
    // Write the .pw.toml file to .index folder
    let toml_filename = rustwiz::mod_toml_filename(filename);
    let index_dir = rustwiz::index_dir(mods_dir);
    let toml_path = index_dir.join(&toml_filename);
    
    if let Err(e) = rustwiz::write_mod_toml(&toml_path, &extended) {
        tracing::warn!("Failed to write mod metadata: {}", e);
    }
}

/// Progress tracking for batch downloads
#[derive(Debug, Clone, serde::Serialize)]
pub struct ModDownloadProgress {
    pub downloaded: u32,
    pub total: u32,
    pub current_file: String,
}

/// Request for batch mod download
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ModDownloadRequest {
    pub mod_id: String,
    pub version_id: String,
    pub platform: String,
}

#[tauri::command]
pub async fn download_mod(
    state: State<'_, AppState>,
    instance_id: String,
    mod_id: String,
    platform: Option<String>,
) -> Result<(), String> {
    let platform = platform.unwrap_or_else(|| "modrinth".to_string());
    
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let loader_name = instance.mod_loader.as_ref()
        .map(|ml| format!("{:?}", ml.loader_type).to_lowercase())
        .unwrap_or_else(|| "vanilla".to_string());
    
    let mods_dir = instance.mods_dir();
    std::fs::create_dir_all(&mods_dir).map_err(|e| format!("Failed to create mods directory: {}", e))?;
    
    match platform.to_lowercase().as_str() {
        "curseforge" => {
            let client = CurseForgeClient::new();
            if !client.has_api_key() {
                return Err("CurseForge API key not configured".to_string());
            }
            
            let mod_id_num: u32 = mod_id.parse()
                .map_err(|_| "Invalid CurseForge mod ID".to_string())?;
            
            // Fetch mod details to get icon_url and description
            let mod_info = client.get_mod(mod_id_num)
                .await
                .map_err(|e| format!("Failed to get mod info: {}", e))?;
            
            let files = client.get_files(
                mod_id_num,
                Some(&instance.minecraft_version),
                Some(&loader_name),
            ).await.map_err(|e| format!("Failed to get mod files: {}", e))?;
            
            if files.is_empty() {
                return Err("No compatible mod version found".to_string());
            }
            
            let version = &files[0];
            if version.files.is_empty() {
                return Err("No files available for this mod".to_string());
            }
            
            let file = &version.files[0];
            
            let download_url = if file.url.is_empty() {
                let file_id: u32 = version.id.parse()
                    .map_err(|_| "Invalid file ID".to_string())?;
                client.get_download_url(mod_id_num, file_id)
                    .await
                    .map_err(|e| format!("Failed to get download URL: {}", e))?
            } else {
                file.url.clone()
            };
            
            let file_path = mods_dir.join(&file.filename);
            
            download_file(&download_url, &file_path, None)
                .await
                .map_err(|e| format!("Failed to download mod: {}", e))?;
            
            // Create RustWiz metadata with icon_url and description
            let hash = rustwiz::compute_file_hash(&file_path, HashFormat::Sha512)
                .unwrap_or_default();
            
            create_mod_metadata(
                &mods_dir,
                &file.filename,
                &version.name,
                &download_url,
                &hash,
                HashFormat::Sha512,
                "curseforge",
                &mod_id,
                &version.id,
                mod_info.icon_url.clone(),
                Some(mod_info.description.clone()),
                Some(vec![instance.minecraft_version.clone()]),
                Some(vec![loader_name.clone()]),
            );
        },
        _ => {
            let client = ModrinthClient::new();
            
            // Fetch project details to get icon_url and description
            let project = client.get_project(&mod_id)
                .await
                .map_err(|e| format!("Failed to get project info: {}", e))?;
            
            let versions = client.get_versions(
                &mod_id,
                Some(&[instance.minecraft_version.clone()]),
                Some(&[loader_name.clone()]),
            ).await.map_err(|e| format!("Failed to get mod versions: {}", e))?;
            
            if versions.is_empty() {
                return Err("No compatible mod version found".to_string());
            }
            
            let version = &versions[0];
            if version.files.is_empty() {
                return Err("No files available for this mod".to_string());
            }
            
            let file = &version.files[0];
            let file_path = mods_dir.join(&file.filename);
            
            download_file(&file.url, &file_path, None)
                .await
                .map_err(|e| format!("Failed to download mod: {}", e))?;
            
            // Create RustWiz metadata with icon_url and description
            let hash = file.sha512.clone()
                .unwrap_or_else(|| {
                    rustwiz::compute_file_hash(&file_path, HashFormat::Sha512)
                        .unwrap_or_default()
                });
            
            create_mod_metadata(
                &mods_dir,
                &file.filename,
                &version.name,
                &file.url,
                &hash,
                HashFormat::Sha512,
                "modrinth",
                &mod_id,
                &version.id,
                project.icon_url.clone(),
                Some(project.description.clone()),
                Some(vec![instance.minecraft_version.clone()]),
                Some(vec![loader_name]),
            );
        }
    }
    
    Ok(())
}

#[tauri::command]
pub async fn download_mod_version(
    state: State<'_, AppState>,
    instance_id: String,
    mod_id: String,
    version_id: String,
    platform: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let mods_dir = instance.mods_dir();
    let mc_version = instance.minecraft_version.clone();
    let loader_name = instance.mod_loader.as_ref()
        .map(|ml| format!("{:?}", ml.loader_type).to_lowercase());
    
    std::fs::create_dir_all(&mods_dir).map_err(|e| format!("Failed to create mods directory: {}", e))?;
    
    match platform.to_lowercase().as_str() {
        "curseforge" => {
            let client = CurseForgeClient::new();
            if !client.has_api_key() {
                return Err("CurseForge API key not configured".to_string());
            }
            
            let mod_id_num: u32 = mod_id.parse()
                .map_err(|_| "Invalid CurseForge mod ID".to_string())?;
            let file_id: u32 = version_id.parse()
                .map_err(|_| "Invalid CurseForge file ID".to_string())?;
            
            // Fetch mod details to get icon_url and description
            let mod_info = client.get_mod(mod_id_num)
                .await
                .map_err(|e| format!("Failed to get mod info: {}", e))?;
            
            let version = client.get_file(mod_id_num, file_id)
                .await
                .map_err(|e| format!("Failed to get file info: {}", e))?;
            
            if version.files.is_empty() {
                return Err("No files available for this version".to_string());
            }
            
            let file = &version.files[0];
            
            let download_url = if file.url.is_empty() {
                client.get_download_url(mod_id_num, file_id)
                    .await
                    .map_err(|e| format!("Failed to get download URL: {}", e))?
            } else {
                file.url.clone()
            };
            
            let file_path = mods_dir.join(&file.filename);
            
            download_file(&download_url, &file_path, None)
                .await
                .map_err(|e| format!("Failed to download mod: {}", e))?;
            
            // Create RustWiz metadata with icon_url and description
            let hash = rustwiz::compute_file_hash(&file_path, HashFormat::Sha512)
                .unwrap_or_default();
            
            create_mod_metadata(
                &mods_dir,
                &file.filename,
                &version.name,
                &download_url,
                &hash,
                HashFormat::Sha512,
                "curseforge",
                &mod_id,
                &version_id,
                mod_info.icon_url.clone(),
                Some(mod_info.description.clone()),
                Some(vec![mc_version.clone()]),
                loader_name.clone().map(|l| vec![l]),
            );
        },
        _ => {
            let client = ModrinthClient::new();
            
            // Fetch project details to get icon_url and description
            let project = client.get_project(&mod_id)
                .await
                .map_err(|e| format!("Failed to get project info: {}", e))?;
            
            let version = client.get_version(&version_id)
                .await
                .map_err(|e| format!("Failed to get version info: {}", e))?;
            
            if version.files.is_empty() {
                return Err("No files available for this version".to_string());
            }
            
            let file = version.files.iter()
                .find(|f| f.primary)
                .unwrap_or(&version.files[0]);
            
            let file_path = mods_dir.join(&file.filename);
            
            download_file(&file.url, &file_path, None)
                .await
                .map_err(|e| format!("Failed to download mod: {}", e))?;
            
            // Create RustWiz metadata with icon_url and description
            let hash = file.sha512.clone()
                .unwrap_or_else(|| {
                    rustwiz::compute_file_hash(&file_path, HashFormat::Sha512)
                        .unwrap_or_default()
                });
            
            create_mod_metadata(
                &mods_dir,
                &file.filename,
                &version.name,
                &file.url,
                &hash,
                HashFormat::Sha512,
                "modrinth",
                &mod_id,
                &version_id,
                project.icon_url.clone(),
                Some(project.description.clone()),
                Some(vec![mc_version.clone()]),
                loader_name.map(|l| vec![l]),
            );
        }
    }
    
    Ok(())
}

/// Batch download multiple mods in parallel
#[tauri::command]
pub async fn download_mods_batch(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
    mods: Vec<ModDownloadRequest>,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let mods_dir = instance.mods_dir();
    let mc_version = instance.minecraft_version.clone();
    let loader_name = instance.mod_loader.as_ref()
        .map(|ml| format!("{:?}", ml.loader_type).to_lowercase());
    
    std::fs::create_dir_all(&mods_dir).map_err(|e| format!("Failed to create mods directory: {}", e))?;
    
    // Get max concurrent downloads from config (default to 6)
    let max_concurrent = {
        let config = state.config.lock().unwrap();
        config.network.max_concurrent_downloads
    };
    
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let total = mods.len() as u32;
    let downloaded = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let mods_dir = Arc::new(mods_dir);
    let mc_version = Arc::new(mc_version);
    let loader_name = Arc::new(loader_name);
    
    let mut handles = Vec::new();
    
    for mod_req in mods {
        let semaphore = Arc::clone(&semaphore);
        let downloaded = Arc::clone(&downloaded);
        let mods_dir = Arc::clone(&mods_dir);
        let mc_version = Arc::clone(&mc_version);
        let loader_name = Arc::clone(&loader_name);
        let app_handle = app.clone();
        
        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.map_err(|e| e.to_string())?;
            
            let result = download_single_mod(
                &mods_dir,
                &mod_req.mod_id,
                &mod_req.version_id,
                &mod_req.platform,
                &mc_version,
                loader_name.as_deref(),
            ).await;
            
            let count = downloaded.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            
            // Emit progress event
            let _ = app_handle.emit("mod-download-progress", ModDownloadProgress {
                downloaded: count,
                total,
                current_file: mod_req.mod_id.clone(),
            });
            
            result
        });
        
        handles.push(handle);
    }
    
    // Wait for all downloads to complete
    let results: Vec<_> = futures::future::join_all(handles).await;
    
    // Check for errors
    let errors: Vec<String> = results.into_iter()
        .filter_map(|r| match r {
            Ok(Ok(())) => None,
            Ok(Err(e)) => Some(e),
            Err(e) => Some(format!("Task failed: {}", e)),
        })
        .collect();
    
    if !errors.is_empty() {
        return Err(format!("Some downloads failed: {}", errors.join(", ")));
    }
    
    Ok(())
}

/// Internal function to download a single mod
async fn download_single_mod(
    mods_dir: &std::path::Path,
    mod_id: &str,
    version_id: &str,
    platform: &str,
    mc_version: &str,
    loader_name: Option<&str>,
) -> Result<(), String> {
    match platform.to_lowercase().as_str() {
        "curseforge" => {
            let client = CurseForgeClient::new();
            if !client.has_api_key() {
                return Err("CurseForge API key not configured".to_string());
            }
            
            let mod_id_num: u32 = mod_id.parse()
                .map_err(|_| "Invalid CurseForge mod ID".to_string())?;
            let file_id: u32 = version_id.parse()
                .map_err(|_| "Invalid CurseForge file ID".to_string())?;
            
            // Fetch mod details to get icon_url and description
            let mod_info = client.get_mod(mod_id_num)
                .await
                .map_err(|e| format!("Failed to get mod info: {}", e))?;
            
            let version = client.get_file(mod_id_num, file_id)
                .await
                .map_err(|e| format!("Failed to get file info: {}", e))?;
            
            if version.files.is_empty() {
                return Err("No files available for this version".to_string());
            }
            
            let file = &version.files[0];
            
            let download_url = if file.url.is_empty() {
                client.get_download_url(mod_id_num, file_id)
                    .await
                    .map_err(|e| format!("Failed to get download URL: {}", e))?
            } else {
                file.url.clone()
            };
            
            let file_path = mods_dir.join(&file.filename);
            
            download_file(&download_url, &file_path, None)
                .await
                .map_err(|e| format!("Failed to download mod: {}", e))?;
            
            // Create RustWiz metadata with icon_url and description
            let hash = rustwiz::compute_file_hash(&file_path, HashFormat::Sha512)
                .unwrap_or_default();
            
            create_mod_metadata(
                mods_dir,
                &file.filename,
                &version.name,
                &download_url,
                &hash,
                HashFormat::Sha512,
                "curseforge",
                mod_id,
                version_id,
                mod_info.icon_url.clone(),
                Some(mod_info.description.clone()),
                Some(vec![mc_version.to_string()]),
                loader_name.map(|l| vec![l.to_string()]),
            );
        },
        _ => {
            let client = ModrinthClient::new();
            
            // Fetch project details to get icon_url and description
            let project = client.get_project(mod_id)
                .await
                .map_err(|e| format!("Failed to get project info: {}", e))?;
            
            let version = client.get_version(version_id)
                .await
                .map_err(|e| format!("Failed to get version info: {}", e))?;
            
            if version.files.is_empty() {
                return Err("No files available for this version".to_string());
            }
            
            let file = version.files.iter()
                .find(|f| f.primary)
                .unwrap_or(&version.files[0]);
            
            let file_path = mods_dir.join(&file.filename);
            
            download_file(&file.url, &file_path, None)
                .await
                .map_err(|e| format!("Failed to download mod: {}", e))?;
            
            // Create RustWiz metadata with icon_url and description
            let hash = file.sha512.clone()
                .unwrap_or_else(|| {
                    rustwiz::compute_file_hash(&file_path, HashFormat::Sha512)
                        .unwrap_or_default()
                });
            
            create_mod_metadata(
                mods_dir,
                &file.filename,
                &version.name,
                &file.url,
                &hash,
                HashFormat::Sha512,
                "modrinth",
                mod_id,
                version_id,
                project.icon_url.clone(),
                Some(project.description.clone()),
                Some(vec![mc_version.to_string()]),
                loader_name.map(|l| vec![l.to_string()]),
            );
        }
    }
    
    Ok(())
}

#[tauri::command]
pub async fn add_local_mod(
    state: State<'_, AppState>,
    instance_id: String,
    file_path: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let mods_dir = instance.mods_dir();
    std::fs::create_dir_all(&mods_dir).map_err(|e| e.to_string())?;
    
    let source = std::path::Path::new(&file_path);
    let filename = source.file_name()
        .ok_or("Invalid file path")?
        .to_str()
        .ok_or("Invalid filename")?;
    
    let dest = mods_dir.join(filename);
    
    std::fs::copy(source, dest).map_err(|e| format!("Failed to copy mod: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn add_local_mod_from_bytes(
    state: State<'_, AppState>,
    instance_id: String,
    filename: String,
    data: String,
) -> Result<(), String> {
    use base64::{Engine as _, engine::general_purpose};
    
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let mods_dir = instance.mods_dir();
    std::fs::create_dir_all(&mods_dir).map_err(|e| e.to_string())?;
    
    let bytes = general_purpose::STANDARD.decode(data)
        .map_err(|e| format!("Failed to decode file data: {}", e))?;
    
    let dest = mods_dir.join(&filename);
    
    std::fs::write(dest, bytes)
        .map_err(|e| format!("Failed to write mod file: {}", e))?;
    
    Ok(())
}
