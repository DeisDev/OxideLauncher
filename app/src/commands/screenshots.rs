//! Screenshot management commands

use super::state::AppState;
use super::utils::format_file_size;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Screenshot information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotInfo {
    pub filename: String,
    pub path: String,
    pub timestamp: Option<String>,
    pub size: String,
}

/// List screenshots for an instance
#[tauri::command]
pub async fn list_screenshots(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<ScreenshotInfo>, String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let screenshots_dir = instance.game_dir().join("screenshots");
    if !screenshots_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut screenshots = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(&screenshots_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let filename = entry.file_name().to_string_lossy().to_string();
            
            // Only include PNG files
            if !filename.to_lowercase().ends_with(".png") {
                continue;
            }
            
            let metadata = entry.metadata().ok();
            let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            let timestamp = metadata.and_then(|m| m.modified().ok())
                .map(|t| {
                    let datetime: chrono::DateTime<chrono::Local> = t.into();
                    datetime.format("%Y-%m-%d %H:%M").to_string()
                });
            
            screenshots.push(ScreenshotInfo {
                filename: filename.clone(),
                path: path.to_string_lossy().to_string(),
                timestamp,
                size: format_file_size(size),
            });
        }
    }
    
    // Sort by filename (which includes timestamp for Minecraft screenshots)
    screenshots.sort_by(|a, b| b.filename.cmp(&a.filename));
    Ok(screenshots)
}

/// Delete a screenshot
#[tauri::command]
pub async fn delete_screenshot(
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
    
    let screenshot_path = instance.game_dir().join("screenshots").join(&filename);
    
    if !screenshot_path.exists() {
        return Err(format!("Screenshot '{}' not found", filename));
    }
    
    std::fs::remove_file(&screenshot_path).map_err(|e| e.to_string())
}

/// Open screenshots folder in file explorer
#[tauri::command]
pub async fn open_screenshots_folder(
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
    
    let screenshots_dir = instance.game_dir().join("screenshots");
    
    // Create directory if it doesn't exist
    if !screenshots_dir.exists() {
        std::fs::create_dir_all(&screenshots_dir).map_err(|e| e.to_string())?;
    }
    
    open::that(&screenshots_dir).map_err(|e| e.to_string())
}
