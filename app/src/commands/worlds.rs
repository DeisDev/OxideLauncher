//! World management commands

use super::state::AppState;
use crate::core::minecraft::world;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

/// World information for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldInfo {
    pub folder_name: String,
    pub name: String,
    pub seed: Option<i64>,
    pub game_type: String,
    pub hardcore: bool,
    pub last_played: Option<String>,
    pub size: String,
    pub has_icon: bool,
}

/// List worlds for an instance
#[tauri::command]
pub async fn list_worlds(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<WorldInfo>, String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let saves_dir = instance.game_dir().join("saves");
    let worlds = world::list_worlds(&saves_dir);
    
    let world_infos: Vec<WorldInfo> = worlds.into_iter().map(|w| {
        let last_played = w.formatted_last_played();
        let size = w.formatted_size();
        WorldInfo {
            folder_name: w.folder_name,
            name: w.name,
            seed: w.seed,
            game_type: w.game_type.to_string(),
            hardcore: w.hardcore,
            last_played,
            size,
            has_icon: w.has_icon,
        }
    }).collect();
    
    Ok(world_infos)
}

/// Delete a world
#[tauri::command]
pub async fn delete_world(
    state: State<'_, AppState>,
    instance_id: String,
    folder_name: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let saves_dir = instance.game_dir().join("saves");
    world::delete_world(&saves_dir, &folder_name)
        .map_err(|e| e.to_string())
}

/// Export a world to a ZIP file
#[tauri::command]
pub async fn export_world(
    state: State<'_, AppState>,
    instance_id: String,
    folder_name: String,
    output_path: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let saves_dir = instance.game_dir().join("saves");
    let output = PathBuf::from(output_path);
    
    world::export_world(&saves_dir, &folder_name, &output)
        .map_err(|e| e.to_string())
}

/// Copy/duplicate a world
#[tauri::command]
pub async fn copy_world(
    state: State<'_, AppState>,
    instance_id: String,
    folder_name: String,
    new_name: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let saves_dir = instance.game_dir().join("saves");
    world::copy_world(&saves_dir, &folder_name, &new_name)
        .map_err(|e| e.to_string())
}

/// Get world icon as base64
#[tauri::command]
pub async fn get_world_icon(
    state: State<'_, AppState>,
    instance_id: String,
    folder_name: String,
) -> Result<Option<String>, String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let saves_dir = instance.game_dir().join("saves");
    Ok(world::get_world_icon(&saves_dir, &folder_name))
}

/// Open saves folder
#[tauri::command]
pub async fn open_saves_folder(
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
    
    let saves_dir = instance.game_dir().join("saves");
    
    // Create if doesn't exist
    if !saves_dir.exists() {
        std::fs::create_dir_all(&saves_dir).map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&saves_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&saves_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&saves_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(())
}
