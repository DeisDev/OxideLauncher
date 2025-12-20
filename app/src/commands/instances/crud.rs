//! Instance CRUD operations (create, read, update, delete).
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

use super::{CreateInstanceRequest, InstanceInfo, parse_mod_loader};
use crate::commands::state::AppState;
use crate::core::instance::{setup_instance, Instance};
use tauri::State;

#[tauri::command]
pub async fn get_instances(state: State<'_, AppState>) -> Result<Vec<InstanceInfo>, String> {
    let instances = state.instances.lock().unwrap();
    Ok(instances.iter().map(InstanceInfo::from).collect())
}

#[tauri::command]
pub async fn get_instance_details(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<InstanceInfo, String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    Ok(InstanceInfo::from(instance))
}

#[tauri::command]
pub async fn create_instance(
    state: State<'_, AppState>,
    request: CreateInstanceRequest,
) -> Result<String, String> {
    let mod_loader = parse_mod_loader(&request.mod_loader_type, request.loader_version.clone());
    
    let instance_id = uuid::Uuid::new_v4().to_string();
    let instance_path = state.data_dir.join("instances").join(&instance_id);
    
    // Create instance directory structure
    std::fs::create_dir_all(&instance_path)
        .map_err(|e| format!("Failed to create instance directory: {}", e))?;
    
    let game_dir = instance_path.join(".minecraft");
    std::fs::create_dir_all(&game_dir)
        .map_err(|e| format!("Failed to create game directory: {}", e))?;
    std::fs::create_dir_all(game_dir.join("mods"))
        .map_err(|e| format!("Failed to create mods directory: {}", e))?;
    std::fs::create_dir_all(game_dir.join("resourcepacks"))
        .map_err(|e| format!("Failed to create resourcepacks directory: {}", e))?;
    std::fs::create_dir_all(game_dir.join("saves"))
        .map_err(|e| format!("Failed to create saves directory: {}", e))?;
    std::fs::create_dir_all(game_dir.join("screenshots"))
        .map_err(|e| format!("Failed to create screenshots directory: {}", e))?;
    
    let mut instance = Instance::new(
        request.name,
        instance_path,
        request.minecraft_version.clone(),
    );
    
    instance.mod_loader = mod_loader;
    
    // Set group if provided
    if let Some(group) = request.group {
        if !group.is_empty() {
            instance.group = Some(group);
        }
    }
    
    // Save instance to file
    instance.save()
        .map_err(|e| format!("Failed to save instance: {}", e))?;
    
    // Clone values before moving instance
    let instance_clone = instance.clone();
    let data_dir_clone = state.data_dir.clone();
    let id_clone = instance_id.clone();
    
    // Add to state
    {
        let mut instances = state.instances.lock().unwrap();
        instances.push(instance);
    }
    
    // Setup instance (download files) in background
    tokio::spawn(async move {
        match setup_instance(&instance_clone, &data_dir_clone, None).await {
            Ok(_) => println!("Instance {} setup complete", id_clone),
            Err(e) => eprintln!("Failed to setup instance {}: {}", id_clone, e),
        }
    });
    
    Ok(instance_id)
}

#[tauri::command]
pub async fn delete_instance(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let mut instances = state.instances.lock().unwrap();
    
    // Find the instance to get its path before removing
    if let Some(instance) = instances.iter().find(|i| i.id == instance_id) {
        // Delete the instance directory from disk
        let instance_path = instance.path.clone();
        
        if instance_path.exists() {
            std::fs::remove_dir_all(&instance_path)
                .map_err(|e| format!("Failed to delete instance directory: {}", e))?;
            tracing::info!("Deleted instance directory: {:?}", instance_path);
        }
    }
    
    // Remove from memory
    instances.retain(|i| i.id != instance_id);
    
    Ok(())
}

#[tauri::command]
pub async fn rename_instance(
    state: State<'_, AppState>,
    instance_id: String,
    new_name: String,
) -> Result<(), String> {
    let mut instances = state.instances.lock().unwrap();
    if let Some(instance) = instances.iter_mut().find(|i| i.id == instance_id) {
        instance.name = new_name;
        instance.save().map_err(|e| format!("Failed to save instance: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn change_instance_icon(
    state: State<'_, AppState>,
    instance_id: String,
    icon: String,
) -> Result<(), String> {
    let mut instances = state.instances.lock().unwrap();
    if let Some(instance) = instances.iter_mut().find(|i| i.id == instance_id) {
        instance.icon = icon;
        instance.save().map_err(|e| format!("Failed to save instance: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn copy_instance(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<String, String> {
    let instances = state.instances.lock().unwrap();
    let original = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let new_id = uuid::Uuid::new_v4().to_string();
    let new_path = state.data_dir.join("instances").join(&new_id);
    
    // Copy directory
    let copy_options = fs_extra::dir::CopyOptions::new();
    fs_extra::dir::copy(&original.path, &new_path, &copy_options)
        .map_err(|e| format!("Failed to copy instance: {}", e))?;
    
    let mut new_instance = original.clone();
    new_instance.id = new_id.clone();
    new_instance.name = format!("{} (Copy)", original.name);
    new_instance.path = new_path;
    new_instance.created_at = chrono::Utc::now();
    
    new_instance.save().map_err(|e| format!("Failed to save instance: {}", e))?;
    
    drop(instances);
    let mut instances = state.instances.lock().unwrap();
    instances.push(new_instance);
    
    Ok(new_id)
}

#[tauri::command]
pub async fn change_instance_group(
    state: State<'_, AppState>,
    instance_id: String,
    group: Option<String>,
) -> Result<(), String> {
    let mut instances = state.instances.lock().unwrap();
    if let Some(instance) = instances.iter_mut().find(|i| i.id == instance_id) {
        instance.group = group;
        instance.save().map_err(|e| format!("Failed to save instance: {}", e))?;
    }
    Ok(())
}
