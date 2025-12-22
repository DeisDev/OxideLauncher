//! Jar mod management commands.
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

use crate::commands::state::AppState;
use crate::core::files;
use serde::{Deserialize, Serialize};
use std::fs;
use tauri::State;

/// Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub file: String,
    pub args: Option<String>,
}

// =============================================================================
// Jar Mods
// =============================================================================

#[tauri::command]
pub async fn add_jar_mod(
    state: State<'_, AppState>,
    instance_id: String,
    jar_path: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Create jar mods directory if it doesn't exist
    let jar_mods_dir = instance.path.join("jarmods");
    fs::create_dir_all(&jar_mods_dir)
        .map_err(|e| format!("Failed to create jarmods directory: {}", e))?;
    
    // Copy the jar file to the jarmods directory
    let source_path = std::path::PathBuf::from(&jar_path);
    let file_name = source_path.file_name()
        .ok_or_else(|| "Invalid jar path".to_string())?;
    let dest_path = jar_mods_dir.join(file_name);
    
    fs::copy(&source_path, &dest_path)
        .map_err(|e| format!("Failed to copy jar mod: {}", e))?;
    
    tracing::info!("Added jar mod: {:?}", dest_path);
    
    Ok(())
}

#[tauri::command]
pub async fn get_jar_mods(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<String>, String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let jar_mods_dir = instance.path.join("jarmods");
    
    if !jar_mods_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut jar_mods = Vec::new();
    if let Ok(entries) = fs::read_dir(&jar_mods_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".jar") || name.ends_with(".zip") {
                    jar_mods.push(name.to_string());
                }
            }
        }
    }
    
    Ok(jar_mods)
}

#[tauri::command]
pub async fn remove_jar_mod(
    state: State<'_, AppState>,
    instance_id: String,
    jar_name: String,
) -> Result<(), String> {
    // Get recycle bin setting from config
    let use_recycle_bin = {
        let config = state.config.lock().unwrap();
        config.files.use_recycle_bin
    };
    
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let jar_path = instance.path.join("jarmods").join(&jar_name);
    
    if jar_path.exists() {
        files::delete_file(&jar_path, use_recycle_bin)
            .map_err(|e| format!("Failed to remove jar mod: {}", e))?;
        
        if use_recycle_bin {
            tracing::info!("Moved jar mod to recycle bin: {:?}", jar_path);
        } else {
            tracing::info!("Permanently removed jar mod: {:?}", jar_path);
        }
    }
    
    Ok(())
}

// =============================================================================
// Java Agents
// =============================================================================

#[tauri::command]
pub async fn add_java_agent(
    state: State<'_, AppState>,
    instance_id: String,
    agent_path: String,
    agent_args: Option<String>,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Create agents directory if it doesn't exist  
    let agents_dir = instance.path.join("agents");
    fs::create_dir_all(&agents_dir)
        .map_err(|e| format!("Failed to create agents directory: {}", e))?;
    
    // Copy the agent jar to the agents directory
    let source_path = std::path::PathBuf::from(&agent_path);
    let file_name = source_path.file_name()
        .ok_or_else(|| "Invalid agent path".to_string())?;
    let dest_path = agents_dir.join(file_name);
    
    fs::copy(&source_path, &dest_path)
        .map_err(|e| format!("Failed to copy agent: {}", e))?;
    
    // Store agent configuration
    let agents_config_path = instance.path.join("agents.json");
    let mut agents_config: Vec<AgentConfig> = if agents_config_path.exists() {
        let content = fs::read_to_string(&agents_config_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };
    
    agents_config.push(AgentConfig {
        file: file_name.to_string_lossy().to_string(),
        args: agent_args,
    });
    
    let config_content = serde_json::to_string_pretty(&agents_config)
        .map_err(|e| format!("Failed to serialize agents config: {}", e))?;
    fs::write(&agents_config_path, config_content)
        .map_err(|e| format!("Failed to write agents config: {}", e))?;
    
    tracing::info!("Added Java agent: {:?}", dest_path);
    
    Ok(())
}

#[tauri::command]
pub async fn get_java_agents(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<AgentConfig>, String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let agents_config_path = instance.path.join("agents.json");
    
    if !agents_config_path.exists() {
        return Ok(Vec::new());
    }
    
    let content = fs::read_to_string(&agents_config_path)
        .map_err(|e| format!("Failed to read agents config: {}", e))?;
    let agents: Vec<AgentConfig> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse agents config: {}", e))?;
    
    Ok(agents)
}

#[tauri::command]
pub async fn remove_java_agent(
    state: State<'_, AppState>,
    instance_id: String,
    agent_file: String,
) -> Result<(), String> {
    // Get recycle bin setting from config
    let use_recycle_bin = {
        let config = state.config.lock().unwrap();
        config.files.use_recycle_bin
    };
    
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Remove agent file
    let agent_path = instance.path.join("agents").join(&agent_file);
    if agent_path.exists() {
        files::delete_file(&agent_path, use_recycle_bin)
            .map_err(|e| format!("Failed to remove agent file: {}", e))?;
    }
    
    // Update agents config
    let agents_config_path = instance.path.join("agents.json");
    if agents_config_path.exists() {
        let content = fs::read_to_string(&agents_config_path).unwrap_or_default();
        let mut agents: Vec<AgentConfig> = serde_json::from_str(&content).unwrap_or_default();
        agents.retain(|a| a.file != agent_file);
        
        let config_content = serde_json::to_string_pretty(&agents)
            .map_err(|e| format!("Failed to serialize agents config: {}", e))?;
        fs::write(&agents_config_path, config_content)
            .map_err(|e| format!("Failed to write agents config: {}", e))?;
    }
    
    tracing::info!("Removed Java agent: {}", agent_file);
    
    Ok(())
}

// =============================================================================
// Custom Minecraft Jar
// =============================================================================

#[tauri::command]
pub async fn replace_minecraft_jar(
    state: State<'_, AppState>,
    instance_id: String,
    jar_path: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Create patches directory if it doesn't exist
    let patches_dir = instance.path.join("patches");
    fs::create_dir_all(&patches_dir)
        .map_err(|e| format!("Failed to create patches directory: {}", e))?;
    
    // Copy the custom jar
    let source_path = std::path::PathBuf::from(&jar_path);
    let dest_path = patches_dir.join("custom.jar");
    
    fs::copy(&source_path, &dest_path)
        .map_err(|e| format!("Failed to copy custom jar: {}", e))?;
    
    // Create a marker file to indicate custom jar is in use
    let marker_path = patches_dir.join("custom_jar.json");
    let marker = serde_json::json!({
        "type": "custom_jar",
        "file": "custom.jar",
        "original_name": source_path.file_name().map(|n| n.to_string_lossy().to_string())
    });
    
    fs::write(&marker_path, marker.to_string())
        .map_err(|e| format!("Failed to write marker file: {}", e))?;
    
    tracing::info!("Replaced Minecraft jar for instance {}", instance_id);
    
    Ok(())
}

#[tauri::command]
pub async fn revert_minecraft_jar(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    // Get recycle bin setting from config
    let use_recycle_bin = {
        let config = state.config.lock().unwrap();
        config.files.use_recycle_bin
    };
    
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let patches_dir = instance.path.join("patches");
    
    // Remove custom jar and marker
    let custom_jar = patches_dir.join("custom.jar");
    let marker_path = patches_dir.join("custom_jar.json");
    
    if custom_jar.exists() {
        files::delete_file(&custom_jar, use_recycle_bin)
            .map_err(|e| format!("Failed to remove custom jar: {}", e))?;
    }
    
    if marker_path.exists() {
        // Marker file is just metadata, can be permanently deleted
        fs::remove_file(&marker_path)
            .map_err(|e| format!("Failed to remove marker file: {}", e))?;
    }
    
    tracing::info!("Reverted Minecraft jar for instance {}", instance_id);
    
    Ok(())
}

#[tauri::command]
pub async fn has_custom_minecraft_jar(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<bool, String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let marker_path = instance.path.join("patches").join("custom_jar.json");
    Ok(marker_path.exists())
}
