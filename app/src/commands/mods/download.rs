//! Mod download commands

use crate::commands::state::AppState;
use crate::core::download::download_file;
use crate::core::modplatform::{
    curseforge::CurseForgeClient, 
    modrinth::ModrinthClient, 
    types::*,
};
use super::types::*;
use tauri::{State, Emitter, AppHandle};
use std::sync::Arc;
use tokio::sync::Semaphore;

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
            
            let metadata = ModMetadata {
                mod_id: mod_id.clone(),
                name: version.name.clone(),
                version: version.version_number.clone(),
                provider: "CurseForge".to_string(),
                icon_url: None,
            };
            
            let metadata_path = mods_dir.join(format!("{}.metadata.json", file.filename));
            if let Ok(json) = serde_json::to_string_pretty(&metadata) {
                let _ = std::fs::write(metadata_path, json);
            }
        },
        _ => {
            let client = ModrinthClient::new();
            
            let versions = client.get_versions(
                &mod_id,
                Some(&[instance.minecraft_version.clone()]),
                Some(&[loader_name]),
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
            
            let metadata = ModMetadata {
                mod_id: mod_id.clone(),
                name: version.name.clone(),
                version: version.version_number.clone(),
                provider: "Modrinth".to_string(),
                icon_url: None,
            };
            
            let metadata_path = mods_dir.join(format!("{}.metadata.json", file.filename));
            if let Ok(json) = serde_json::to_string_pretty(&metadata) {
                let _ = std::fs::write(metadata_path, json);
            }
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
            
            let metadata = ModMetadata {
                mod_id: mod_id.clone(),
                name: version.name.clone(),
                version: version.version_number.clone(),
                provider: "CurseForge".to_string(),
                icon_url: None,
            };
            
            let metadata_path = mods_dir.join(format!("{}.metadata.json", file.filename));
            if let Ok(json) = serde_json::to_string_pretty(&metadata) {
                let _ = std::fs::write(metadata_path, json);
            }
        },
        _ => {
            let client = ModrinthClient::new();
            
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
            
            let metadata = ModMetadata {
                mod_id: mod_id.clone(),
                name: version.name.clone(),
                version: version.version_number.clone(),
                provider: "Modrinth".to_string(),
                icon_url: None,
            };
            
            let metadata_path = mods_dir.join(format!("{}.metadata.json", file.filename));
            if let Ok(json) = serde_json::to_string_pretty(&metadata) {
                let _ = std::fs::write(metadata_path, json);
            }
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
    
    let mut handles = Vec::new();
    
    for mod_req in mods {
        let semaphore = Arc::clone(&semaphore);
        let downloaded = Arc::clone(&downloaded);
        let mods_dir = Arc::clone(&mods_dir);
        let app_handle = app.clone();
        
        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.map_err(|e| e.to_string())?;
            
            let result = download_single_mod(
                &mods_dir,
                &mod_req.mod_id,
                &mod_req.version_id,
                &mod_req.platform,
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
            
            let metadata = ModMetadata {
                mod_id: mod_id.to_string(),
                name: version.name.clone(),
                version: version.version_number.clone(),
                provider: "CurseForge".to_string(),
                icon_url: None,
            };
            
            let metadata_path = mods_dir.join(format!("{}.metadata.json", file.filename));
            if let Ok(json) = serde_json::to_string_pretty(&metadata) {
                let _ = std::fs::write(metadata_path, json);
            }
        },
        _ => {
            let client = ModrinthClient::new();
            
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
            
            let metadata = ModMetadata {
                mod_id: mod_id.to_string(),
                name: version.name.clone(),
                version: version.version_number.clone(),
                provider: "Modrinth".to_string(),
                icon_url: None,
            };
            
            let metadata_path = mods_dir.join(format!("{}.metadata.json", file.filename));
            if let Ok(json) = serde_json::to_string_pretty(&metadata) {
                let _ = std::fs::write(metadata_path, json);
            }
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
