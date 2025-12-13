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
    let mut app_config = state.config.lock().unwrap();
    *app_config = config;
    Ok(())
}
