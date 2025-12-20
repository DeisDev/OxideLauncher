//! Resource listing commands for resource packs and shaders.
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

use super::types::{ResourcePackInfo, ShaderPackInfo};
use crate::commands::state::AppState;
use crate::commands::utils::format_file_size;
use std::io::Read;
use std::path::Path;
use tauri::State;

/// Extract pack.png and pack.mcmeta from a resource pack
fn extract_pack_metadata(pack_path: &Path, cache_dir: &Path) -> (Option<String>, Option<String>) {
    let mut icon_path = None;
    let mut description = None;
    
    // Determine the base name for caching
    let pack_name = pack_path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown");
    
    if pack_path.is_file() {
        // It's a zip file
        if let Ok(file) = std::fs::File::open(pack_path) {
            if let Ok(mut archive) = zip::ZipArchive::new(file) {
                // Try to extract pack.png
                if let Ok(mut png_file) = archive.by_name("pack.png") {
                    let mut png_data = Vec::new();
                    if png_file.read_to_end(&mut png_data).is_ok() {
                        // Create cache directory if needed
                        let _ = std::fs::create_dir_all(cache_dir);
                        let cached_icon = cache_dir.join(format!("{}.png", pack_name));
                        if std::fs::write(&cached_icon, &png_data).is_ok() {
                            icon_path = Some(cached_icon.to_string_lossy().to_string());
                        }
                    }
                }
                
                // Try to extract pack.mcmeta for description
                if let Ok(mut mcmeta_file) = archive.by_name("pack.mcmeta") {
                    let mut mcmeta_data = String::new();
                    if mcmeta_file.read_to_string(&mut mcmeta_data).is_ok() {
                        description = parse_pack_description(&mcmeta_data);
                    }
                }
            }
        }
    } else if pack_path.is_dir() {
        // It's a folder-based resource pack
        let png_path = pack_path.join("pack.png");
        if png_path.exists() {
            if let Ok(png_data) = std::fs::read(&png_path) {
                let _ = std::fs::create_dir_all(cache_dir);
                let cached_icon = cache_dir.join(format!("{}.png", pack_name));
                if std::fs::write(&cached_icon, &png_data).is_ok() {
                    icon_path = Some(cached_icon.to_string_lossy().to_string());
                }
            }
        }
        
        let mcmeta_path = pack_path.join("pack.mcmeta");
        if mcmeta_path.exists() {
            if let Ok(mcmeta_data) = std::fs::read_to_string(&mcmeta_path) {
                description = parse_pack_description(&mcmeta_data);
            }
        }
    }
    
    (icon_path, description)
}

/// Parse description from pack.mcmeta JSON
fn parse_pack_description(mcmeta: &str) -> Option<String> {
    // Parse the JSON
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(mcmeta) {
        if let Some(pack) = json.get("pack") {
            if let Some(desc) = pack.get("description") {
                // Description can be a string or a complex JSON text component
                return Some(extract_text_from_json_component(desc));
            }
        }
    }
    None
}

/// Extract plain text from Minecraft JSON text component
fn extract_text_from_json_component(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            arr.iter().map(extract_text_from_json_component).collect::<Vec<_>>().join("")
        }
        serde_json::Value::Object(obj) => {
            let mut result = String::new();
            // Extract "text" field
            if let Some(text) = obj.get("text") {
                if let serde_json::Value::String(s) = text {
                    result.push_str(s);
                }
            }
            // Handle "extra" array for nested components
            if let Some(extra) = obj.get("extra") {
                result.push_str(&extract_text_from_json_component(extra));
            }
            result
        }
        _ => String::new(),
    }
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
    
    // Create cache directory for icons
    let cache_dir = instance.game_dir().join(".cache").join("icons").join("resourcepacks");
    
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
                
                // Extract icon and description from pack
                let (icon_path, description) = extract_pack_metadata(&path, &cache_dir);
                
                packs.push(ResourcePackInfo {
                    filename: filename.clone(),
                    name: filename.trim_end_matches(".zip").to_string(),
                    description,
                    size: format_file_size(size),
                    enabled: true,
                    icon_path,
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
