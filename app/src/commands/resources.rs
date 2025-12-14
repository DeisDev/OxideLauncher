//! Resource pack and shader pack management commands

use super::state::AppState;
use super::utils::format_file_size;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Resource pack information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePackInfo {
    pub filename: String,
    pub name: String,
    pub description: Option<String>,
    pub size: String,
    pub enabled: bool,
}

/// Shader pack information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderPackInfo {
    pub filename: String,
    pub name: String,
    pub size: String,
}

/// List resource packs for an instance
#[tauri::command]
pub async fn list_resource_packs(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<ResourcePackInfo>, String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let resourcepacks_dir = instance.game_dir().join("resourcepacks");
    if !resourcepacks_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut packs = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(&resourcepacks_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let filename = entry.file_name().to_string_lossy().to_string();
            
            // Skip disabled packs marker files
            if filename.starts_with('.') {
                continue;
            }
            
            // Check if it's a ZIP or folder
            let is_valid = path.is_dir() || 
                filename.to_lowercase().ends_with(".zip");
            
            if is_valid {
                let size = if path.is_file() {
                    entry.metadata().map(|m| m.len()).unwrap_or(0)
                } else {
                    0
                };
                
                packs.push(ResourcePackInfo {
                    filename: filename.clone(),
                    name: filename.trim_end_matches(".zip").to_string(),
                    description: None,
                    size: format_file_size(size),
                    enabled: true,
                });
            }
        }
    }
    
    packs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(packs)
}

/// Delete a resource pack
#[tauri::command]
pub async fn delete_resource_pack(
    state: State<'_, AppState>,
    instance_id: String,
    filename: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let pack_path = instance.game_dir().join("resourcepacks").join(&filename);
    
    if !pack_path.exists() {
        return Err(format!("Resource pack '{}' not found", filename));
    }
    
    if pack_path.is_dir() {
        std::fs::remove_dir_all(&pack_path).map_err(|e| e.to_string())?;
    } else {
        std::fs::remove_file(&pack_path).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

/// List shader packs for an instance
#[tauri::command]
pub async fn list_shader_packs(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<ShaderPackInfo>, String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let shaderpacks_dir = instance.game_dir().join("shaderpacks");
    if !shaderpacks_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut packs = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(&shaderpacks_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let filename = entry.file_name().to_string_lossy().to_string();
            
            // Skip hidden files
            if filename.starts_with('.') {
                continue;
            }
            
            // Check if it's a ZIP or folder
            let is_valid = path.is_dir() || 
                filename.to_lowercase().ends_with(".zip");
            
            if is_valid {
                let size = if path.is_file() {
                    entry.metadata().map(|m| m.len()).unwrap_or(0)
                } else {
                    0
                };
                
                packs.push(ShaderPackInfo {
                    filename: filename.clone(),
                    name: filename.trim_end_matches(".zip").to_string(),
                    size: format_file_size(size),
                });
            }
        }
    }
    
    packs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(packs)
}

/// Delete a shader pack
#[tauri::command]
pub async fn delete_shader_pack(
    state: State<'_, AppState>,
    instance_id: String,
    filename: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let pack_path = instance.game_dir().join("shaderpacks").join(&filename);
    
    if !pack_path.exists() {
        return Err(format!("Shader pack '{}' not found", filename));
    }
    
    if pack_path.is_dir() {
        std::fs::remove_dir_all(&pack_path).map_err(|e| e.to_string())?;
    } else {
        std::fs::remove_file(&pack_path).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

/// Open resource packs folder
#[tauri::command]
pub async fn open_resourcepacks_folder(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let resourcepacks_dir = instance.game_dir().join("resourcepacks");
    
    // Create if doesn't exist
    if !resourcepacks_dir.exists() {
        std::fs::create_dir_all(&resourcepacks_dir).map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&resourcepacks_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&resourcepacks_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&resourcepacks_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(())
}

/// Open shader packs folder
#[tauri::command]
pub async fn open_shaderpacks_folder(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let shaderpacks_dir = instance.game_dir().join("shaderpacks");
    
    // Create if doesn't exist
    if !shaderpacks_dir.exists() {
        std::fs::create_dir_all(&shaderpacks_dir).map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&shaderpacks_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&shaderpacks_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&shaderpacks_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(())
}
