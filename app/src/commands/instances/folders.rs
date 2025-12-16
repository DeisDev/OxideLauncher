//! Folder opening utility commands

use crate::commands::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn open_instance_folder(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    open_folder(&instance.path)?;
    
    Ok(())
}

#[tauri::command]
pub async fn open_instance_logs_folder(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // The logs folder is at {instance}/.minecraft/logs
    let logs_path = instance.path.join(".minecraft").join("logs");
    
    // Create the folder if it doesn't exist
    std::fs::create_dir_all(&logs_path)
        .map_err(|e| format!("Failed to create logs folder: {}", e))?;
    
    open_folder(&logs_path)?;
    
    Ok(())
}

#[tauri::command]
pub async fn open_minecraft_folder(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let minecraft_dir = instance.game_dir();
    
    // Ensure directory exists
    std::fs::create_dir_all(&minecraft_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;
    
    open_folder(&minecraft_dir)?;
    
    Ok(())
}

#[tauri::command]
pub async fn open_libraries_folder(
    state: State<'_, AppState>,
    _instance_id: String,
) -> Result<(), String> {
    // Libraries are stored globally, not per-instance
    let config = state.config.lock().unwrap();
    let libraries_dir = config.data_dir().join("libraries");
    
    // Ensure directory exists
    std::fs::create_dir_all(&libraries_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;
    
    open_folder(&libraries_dir)?;
    
    Ok(())
}

/// Open a folder in the system file explorer
fn open_folder(path: &std::path::Path) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(path.to_str().unwrap())
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(())
}
