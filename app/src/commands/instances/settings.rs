//! Instance settings commands for per-instance configuration.
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

use super::InstanceSettingsUpdate;
use crate::commands::state::AppState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

/// Instance settings returned to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceSettingsResponse {
    pub java_path: Option<String>,
    pub memory_min_mb: u32,
    pub memory_max_mb: u32,
    pub java_args: String,
    pub game_args: String,
    pub window_width: u32,
    pub window_height: u32,
    pub start_maximized: bool,
    pub fullscreen: bool,
    pub console_mode: String,
    pub pre_launch_hook: Option<String>,
    pub post_exit_hook: Option<String>,
    pub enable_analytics: bool,
    pub enable_logging: bool,
    pub game_dir_override: Option<String>,
    pub skip_java_compatibility_check: bool,
    pub wrapper_command: Option<String>,
}

#[tauri::command]
pub async fn get_instance_settings(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<InstanceSettingsResponse, String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Load global config for defaults
    let config = crate::core::config::Config::load().unwrap_or_default();
    
    Ok(InstanceSettingsResponse {
        java_path: instance.settings.java_path.as_ref().map(|p| p.to_string_lossy().to_string()),
        memory_min_mb: instance.settings.min_memory.unwrap_or(config.memory.min_memory),
        memory_max_mb: instance.settings.max_memory.unwrap_or(config.memory.max_memory),
        java_args: instance.settings.jvm_args.clone().unwrap_or_default(),
        game_args: instance.settings.game_args.clone().unwrap_or_default(),
        window_width: instance.settings.window_width.unwrap_or(config.minecraft.window_width),
        window_height: instance.settings.window_height.unwrap_or(config.minecraft.window_height),
        start_maximized: instance.settings.fullscreen,
        fullscreen: instance.settings.fullscreen,
        console_mode: "on_error".to_string(), // Default
        pre_launch_hook: instance.settings.pre_launch_command.clone(),
        post_exit_hook: instance.settings.post_exit_command.clone(),
        enable_analytics: false,
        enable_logging: true,
        game_dir_override: None, // Not available in current InstanceSettings struct
        skip_java_compatibility_check: instance.settings.skip_java_compatibility_check,
        wrapper_command: instance.settings.wrapper_command.clone(),
    })
}

#[tauri::command]
pub async fn update_instance_settings(
    state: State<'_, AppState>,
    instance_id: String,
    settings: InstanceSettingsUpdate,
) -> Result<(), String> {
    let mut instances = state.instances.lock().unwrap();
    let instance = instances.iter_mut()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Update name if provided
    if let Some(name) = settings.name {
        if !name.is_empty() {
            instance.name = name;
        }
    }
    
    // Update settings
    if let Some(java_path) = settings.java_path {
        instance.settings.java_path = if java_path.is_empty() { None } else { Some(PathBuf::from(java_path)) };
    }
    if let Some(java_args) = settings.java_args {
        instance.settings.jvm_args = if java_args.is_empty() { None } else { Some(java_args) };
    }
    if let Some(min) = settings.min_memory {
        instance.settings.min_memory = Some(min);
    }
    if let Some(max) = settings.max_memory {
        instance.settings.max_memory = Some(max);
    }
    if let Some(width) = settings.window_width {
        instance.settings.window_width = Some(width);
    }
    if let Some(height) = settings.window_height {
        instance.settings.window_height = Some(height);
    }
    if let Some(skip) = settings.skip_java_compatibility_check {
        instance.settings.skip_java_compatibility_check = skip;
    }
    if let Some(close) = settings.close_launcher_on_launch {
        instance.settings.close_launcher_on_launch = close;
    }
    if let Some(quit) = settings.quit_launcher_on_exit {
        instance.settings.quit_launcher_on_exit = quit;
    }
    if let Some(cmd) = settings.prelaunch_command {
        instance.settings.pre_launch_command = if cmd.is_empty() { None } else { Some(cmd) };
    }
    if let Some(cmd) = settings.postexit_command {
        instance.settings.post_exit_command = if cmd.is_empty() { None } else { Some(cmd) };
    }
    
    // Save instance to disk
    let instance_clone = instance.clone();
    drop(instances);
    
    instance_clone.save().map_err(|e| e.to_string())?;
    
    Ok(())
}
