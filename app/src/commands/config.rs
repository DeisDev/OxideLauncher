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
