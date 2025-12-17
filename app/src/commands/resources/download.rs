//! Download commands for resource packs and shader packs

use super::types::{ResourceDownloadProgress, ResourceDownloadRequest};
use crate::commands::state::AppState;
use crate::core::download::download_file;
use crate::core::modplatform::{curseforge::CurseForgeClient, modrinth::ModrinthClient};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::Semaphore;

/// Download a resource pack version
#[tauri::command]
pub async fn download_resource_pack_version(
    state: State<'_, AppState>,
    instance_id: String,
    resource_id: String,
    version_id: String,
    platform: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances
            .iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };

    let resourcepacks_dir = instance.game_dir().join("resourcepacks");
    std::fs::create_dir_all(&resourcepacks_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    download_resource(&resourcepacks_dir, resource_id, version_id, platform).await
}

/// Download a shader pack version
#[tauri::command]
pub async fn download_shader_pack_version(
    state: State<'_, AppState>,
    instance_id: String,
    resource_id: String,
    version_id: String,
    platform: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances
            .iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };

    let shaderpacks_dir = instance.game_dir().join("shaderpacks");
    std::fs::create_dir_all(&shaderpacks_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    download_resource(&shaderpacks_dir, resource_id, version_id, platform).await
}

/// Internal function to download a resource
pub(crate) async fn download_resource(
    dest_dir: &std::path::Path,
    resource_id: String,
    version_id: String,
    platform: String,
) -> Result<(), String> {
    match platform.to_lowercase().as_str() {
        "curseforge" => {
            let client = CurseForgeClient::new();
            if !client.has_api_key() {
                return Err("CurseForge API key not configured".to_string());
            }

            let id_num: u32 = resource_id
                .parse()
                .map_err(|_| "Invalid CurseForge ID".to_string())?;
            let file_id: u32 = version_id
                .parse()
                .map_err(|_| "Invalid CurseForge file ID".to_string())?;

            let version = client
                .get_file(id_num, file_id)
                .await
                .map_err(|e| format!("Failed to get file info: {}", e))?;

            if version.files.is_empty() {
                return Err("No files available for this version".to_string());
            }

            let file = &version.files[0];

            let download_url = if file.url.is_empty() {
                client
                    .get_download_url(id_num, file_id)
                    .await
                    .map_err(|e| format!("Failed to get download URL: {}", e))?
            } else {
                file.url.clone()
            };

            let file_path = dest_dir.join(&file.filename);

            download_file(&download_url, &file_path, None)
                .await
                .map_err(|e| format!("Failed to download: {}", e))?;
        }
        _ => {
            let client = ModrinthClient::new();

            let version = client
                .get_version(&version_id)
                .await
                .map_err(|e| format!("Failed to get version info: {}", e))?;

            if version.files.is_empty() {
                return Err("No files available for this version".to_string());
            }

            let file = version
                .files
                .iter()
                .find(|f| f.primary)
                .unwrap_or(&version.files[0]);

            let file_path = dest_dir.join(&file.filename);

            download_file(&file.url, &file_path, None)
                .await
                .map_err(|e| format!("Failed to download: {}", e))?;
        }
    }

    Ok(())
}

/// Add a local resource pack from file path
#[tauri::command]
pub async fn add_local_resource_pack(
    state: State<'_, AppState>,
    instance_id: String,
    file_path: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances
            .iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };

    let resourcepacks_dir = instance.game_dir().join("resourcepacks");
    std::fs::create_dir_all(&resourcepacks_dir).map_err(|e| e.to_string())?;

    let source = std::path::Path::new(&file_path);
    let filename = source
        .file_name()
        .ok_or("Invalid file path")?
        .to_str()
        .ok_or("Invalid filename")?;

    let dest = resourcepacks_dir.join(filename);

    std::fs::copy(source, dest).map_err(|e| format!("Failed to copy file: {}", e))?;

    Ok(())
}

/// Add a local shader pack from file path
#[tauri::command]
pub async fn add_local_shader_pack(
    state: State<'_, AppState>,
    instance_id: String,
    file_path: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances
            .iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };

    let shaderpacks_dir = instance.game_dir().join("shaderpacks");
    std::fs::create_dir_all(&shaderpacks_dir).map_err(|e| e.to_string())?;

    let source = std::path::Path::new(&file_path);
    let filename = source
        .file_name()
        .ok_or("Invalid file path")?
        .to_str()
        .ok_or("Invalid filename")?;

    let dest = shaderpacks_dir.join(filename);

    std::fs::copy(source, dest).map_err(|e| format!("Failed to copy file: {}", e))?;

    Ok(())
}

/// Add a local resource pack from bytes (for drag and drop)
#[tauri::command]
pub async fn add_local_resource_pack_from_bytes(
    state: State<'_, AppState>,
    instance_id: String,
    filename: String,
    data: String,
) -> Result<(), String> {
    use base64::{engine::general_purpose, Engine as _};

    let instance = {
        let instances = state.instances.lock().unwrap();
        instances
            .iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };

    let resourcepacks_dir = instance.game_dir().join("resourcepacks");
    std::fs::create_dir_all(&resourcepacks_dir).map_err(|e| e.to_string())?;

    // Decode base64 data
    let bytes = general_purpose::STANDARD
        .decode(data)
        .map_err(|e| format!("Failed to decode file data: {}", e))?;

    let dest = resourcepacks_dir.join(&filename);

    std::fs::write(dest, bytes).map_err(|e| format!("Failed to write resource pack file: {}", e))?;

    Ok(())
}

/// Add a local shader pack from bytes (for drag and drop)
#[tauri::command]
pub async fn add_local_shader_pack_from_bytes(
    state: State<'_, AppState>,
    instance_id: String,
    filename: String,
    data: String,
) -> Result<(), String> {
    use base64::{engine::general_purpose, Engine as _};

    let instance = {
        let instances = state.instances.lock().unwrap();
        instances
            .iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };

    let shaderpacks_dir = instance.game_dir().join("shaderpacks");
    std::fs::create_dir_all(&shaderpacks_dir).map_err(|e| e.to_string())?;

    // Decode base64 data
    let bytes = general_purpose::STANDARD
        .decode(data)
        .map_err(|e| format!("Failed to decode file data: {}", e))?;

    let dest = shaderpacks_dir.join(&filename);

    std::fs::write(dest, bytes).map_err(|e| format!("Failed to write shader pack file: {}", e))?;

    Ok(())
}

/// Batch download multiple resource packs in parallel
#[tauri::command]
pub async fn download_resource_packs_batch(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
    resources: Vec<ResourceDownloadRequest>,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances
            .iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };

    let resourcepacks_dir = instance.game_dir().join("resourcepacks");
    std::fs::create_dir_all(&resourcepacks_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    download_resources_batch_internal(
        app,
        state,
        resourcepacks_dir,
        resources,
        "resource-download-progress",
    )
    .await
}

/// Batch download multiple shader packs in parallel
#[tauri::command]
pub async fn download_shader_packs_batch(
    app: AppHandle,
    state: State<'_, AppState>,
    instance_id: String,
    resources: Vec<ResourceDownloadRequest>,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances
            .iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };

    let shaderpacks_dir = instance.game_dir().join("shaderpacks");
    std::fs::create_dir_all(&shaderpacks_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;

    download_resources_batch_internal(
        app,
        state,
        shaderpacks_dir,
        resources,
        "resource-download-progress",
    )
    .await
}

/// Internal function to batch download resources in parallel
async fn download_resources_batch_internal(
    app: AppHandle,
    state: State<'_, AppState>,
    dest_dir: std::path::PathBuf,
    resources: Vec<ResourceDownloadRequest>,
    event_name: &'static str,
) -> Result<(), String> {
    // Get max concurrent downloads from config
    let max_concurrent = {
        let config = state.config.lock().unwrap();
        config.network.max_concurrent_downloads
    };

    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let total = resources.len() as u32;
    let downloaded = Arc::new(std::sync::atomic::AtomicU32::new(0));
    let dest_dir = Arc::new(dest_dir);

    let mut handles = Vec::new();

    for resource_req in resources {
        let semaphore = Arc::clone(&semaphore);
        let downloaded = Arc::clone(&downloaded);
        let dest_dir = Arc::clone(&dest_dir);
        let app_handle = app.clone();
        let event = event_name;

        let handle = tokio::spawn(async move {
            let _permit = semaphore.acquire().await.map_err(|e| e.to_string())?;

            let result = download_resource(
                &dest_dir,
                resource_req.resource_id.clone(),
                resource_req.version_id.clone(),
                resource_req.platform.clone(),
            )
            .await;

            let count = downloaded.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;

            // Emit progress event
            let _ = app_handle.emit(
                event,
                ResourceDownloadProgress {
                    downloaded: count,
                    total,
                    current_file: resource_req.resource_id.clone(),
                },
            );

            result
        });

        handles.push(handle);
    }

    // Wait for all downloads to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // Check for errors
    let errors: Vec<String> = results
        .into_iter()
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
