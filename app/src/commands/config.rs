//! Configuration management commands

use super::state::AppState;
use crate::core::config::Config;
use tauri::State;

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<Config, String> {
    let config = state.config.lock().unwrap();
    Ok(config.clone())
}

#[tauri::command]
pub async fn update_config(
    state: State<'_, AppState>,
    config: Config,
) -> Result<(), String> {
    // Save to file
    config.save().map_err(|e| e.to_string())?;
    
    let mut app_config = state.config.lock().unwrap();
    *app_config = config;
    Ok(())
}

#[tauri::command]
pub async fn get_logs_directory(state: State<'_, AppState>) -> Result<String, String> {
    let config = state.config.lock().unwrap();
    let logs_dir = config.logs_dir();
    Ok(logs_dir.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn open_logs_directory(state: State<'_, AppState>) -> Result<(), String> {
    let config = state.config.lock().unwrap();
    let logs_dir = config.logs_dir();
    
    // Create directory if it doesn't exist
    std::fs::create_dir_all(&logs_dir).map_err(|e| e.to_string())?;
    
    // Open in file explorer
    open::that(logs_dir).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn open_data_directory(state: State<'_, AppState>) -> Result<(), String> {
    let config = state.config.lock().unwrap();
    let data_dir = config.data_dir();
    
    // Create directory if it doesn't exist
    std::fs::create_dir_all(&data_dir).map_err(|e| e.to_string())?;
    
    // Open in file explorer
    open::that(data_dir).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn open_launcher_folder(state: State<'_, AppState>, folder_type: String) -> Result<(), String> {
    let config = state.config.lock().unwrap();
    
    let folder_path = match folder_type.as_str() {
        "instances" => config.instances_dir(),
        "logs" => config.logs_dir(),
        "java" => config.java_dir(),
        "assets" => config.assets_dir(),
        "libraries" => config.libraries_dir(),
        "icons" => config.icons_dir(),
        "cache" => config.cache_dir(),
        "meta" => config.meta_dir(),
        "data" => config.data_dir(),
        _ => return Err(format!("Unknown folder type: {}", folder_type)),
    };
    
    // Create directory if it doesn't exist
    std::fs::create_dir_all(&folder_path).map_err(|e| e.to_string())?;
    
    // Open in file explorer
    open::that(folder_path).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn open_external_url(url: String) -> Result<(), String> {
    // Validate URL starts with http:// or https://
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err("Invalid URL: must start with http:// or https://".to_string());
    }
    
    open::that(url).map_err(|e| e.to_string())?;
    Ok(())
}
