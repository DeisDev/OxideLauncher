//! Instance component commands for mod loader management.
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
use crate::core::instance::{ModLoader, ModLoaderType};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Component information for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub uid: String,
    pub name: String,
    pub version: String,
    pub component_type: String,
    pub enabled: bool,
    pub removable: bool,
    pub version_changeable: bool,
    pub customizable: bool,
    pub revertible: bool,
    pub custom: bool,
    pub order: i32,
    pub problems: Vec<ComponentProblemInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentProblemInfo {
    pub severity: String,
    pub description: String,
}

#[tauri::command]
pub async fn get_instance_components(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<ComponentInfo>, String> {
    use crate::core::instance::{build_component_list, ComponentType, ProblemSeverity};
    
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Build component list from instance data
    let components = build_component_list(
        &instance.minecraft_version,
        instance.mod_loader.as_ref(),
    );
    
    let component_infos: Vec<ComponentInfo> = components.components.iter().map(|c| {
        ComponentInfo {
            uid: c.uid.clone(),
            name: c.name.clone(),
            version: c.version.clone(),
            component_type: match c.component_type {
                ComponentType::Minecraft => "minecraft".to_string(),
                ComponentType::ModLoader => "mod_loader".to_string(),
                ComponentType::Library => "library".to_string(),
                ComponentType::Agent => "agent".to_string(),
                ComponentType::JarMod => "jar_mod".to_string(),
                ComponentType::Mappings => "mappings".to_string(),
                ComponentType::Other => "other".to_string(),
            },
            enabled: c.enabled,
            removable: c.removable,
            version_changeable: c.version_changeable,
            customizable: c.customizable,
            revertible: c.revertible,
            custom: c.custom,
            order: c.order,
            problems: c.problems.iter().map(|p| ComponentProblemInfo {
                severity: match p.severity {
                    ProblemSeverity::None => "none".to_string(),
                    ProblemSeverity::Warning => "warning".to_string(),
                    ProblemSeverity::Error => "error".to_string(),
                },
                description: p.description.clone(),
            }).collect(),
        }
    }).collect();
    
    Ok(component_infos)
}

#[tauri::command]
pub async fn remove_instance_component(
    state: State<'_, AppState>,
    instance_id: String,
    component_uid: String,
) -> Result<(), String> {
    let mut instances = state.instances.lock().unwrap();
    let instance = instances.iter_mut()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Handle mod loader removal
    match component_uid.as_str() {
        "net.minecraftforge" | "net.neoforged" | "net.fabricmc.fabric-loader" | 
        "org.quiltmc.quilt-loader" | "com.mumfrey.liteloader" => {
            instance.mod_loader = None;
            instance.save().map_err(|e| format!("Failed to save instance: {}", e))?;
            Ok(())
        }
        "net.minecraft" => {
            Err("Cannot remove the Minecraft component".to_string())
        }
        "net.fabricmc.intermediary" => {
            Err("Cannot remove intermediary mappings while mod loader is installed".to_string())
        }
        _ => {
            Err(format!("Unknown component: {}", component_uid))
        }
    }
}

#[tauri::command]
pub async fn change_component_version(
    state: State<'_, AppState>,
    instance_id: String,
    component_uid: String,
    new_version: String,
) -> Result<(), String> {
    let mut instances = state.instances.lock().unwrap();
    let instance = instances.iter_mut()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    match component_uid.as_str() {
        "net.minecraft" => {
            instance.minecraft_version = new_version;
            instance.save().map_err(|e| format!("Failed to save instance: {}", e))?;
            Ok(())
        }
        "net.minecraftforge" | "net.neoforged" | "net.fabricmc.fabric-loader" | 
        "org.quiltmc.quilt-loader" | "com.mumfrey.liteloader" => {
            if let Some(ref mut loader) = instance.mod_loader {
                loader.version = new_version;
                instance.save().map_err(|e| format!("Failed to save instance: {}", e))?;
                Ok(())
            } else {
                Err("No mod loader installed".to_string())
            }
        }
        _ => {
            Err(format!("Cannot change version of component: {}", component_uid))
        }
    }
}

#[tauri::command]
pub async fn install_mod_loader(
    state: State<'_, AppState>,
    instance_id: String,
    loader_type: String,
    loader_version: String,
) -> Result<(), String> {
    let mut instances = state.instances.lock().unwrap();
    let instance = instances.iter_mut()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let loader_type = match loader_type.to_lowercase().as_str() {
        "forge" => ModLoaderType::Forge,
        "neoforge" => ModLoaderType::NeoForge,
        "fabric" => ModLoaderType::Fabric,
        "quilt" => ModLoaderType::Quilt,
        "liteloader" => ModLoaderType::LiteLoader,
        _ => return Err(format!("Unknown loader type: {}", loader_type)),
    };
    
    instance.mod_loader = Some(ModLoader {
        loader_type,
        version: loader_version,
    });
    
    instance.save().map_err(|e| format!("Failed to save instance: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn move_component_up(
    state: State<'_, AppState>,
    instance_id: String,
    component_uid: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Load component list
    let mut component_list = crate::core::instance::ComponentList::load(&instance.path)
        .map_err(|e| format!("Failed to load components: {}", e))?;
    
    // Move the component up
    if component_list.move_up(&component_uid) {
        // Save the updated list
        component_list.save(&instance.path)
            .map_err(|e| format!("Failed to save components: {}", e))?;
        tracing::info!("Moved component {} up in instance {}", component_uid, instance_id);
        Ok(())
    } else {
        Err("Component cannot be moved up (not found or already at top)".to_string())
    }
}

#[tauri::command]
pub async fn move_component_down(
    state: State<'_, AppState>,
    instance_id: String,
    component_uid: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Load component list
    let mut component_list = crate::core::instance::ComponentList::load(&instance.path)
        .map_err(|e| format!("Failed to load components: {}", e))?;
    
    // Move the component down
    if component_list.move_down(&component_uid) {
        // Save the updated list
        component_list.save(&instance.path)
            .map_err(|e| format!("Failed to save components: {}", e))?;
        tracing::info!("Moved component {} down in instance {}", component_uid, instance_id);
        Ok(())
    } else {
        Err("Component cannot be moved down (not found or already at bottom)".to_string())
    }
}

#[tauri::command]
pub async fn add_empty_component(
    state: State<'_, AppState>,
    instance_id: String,
    name: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Load component list
    let mut component_list = crate::core::instance::ComponentList::load(&instance.path)
        .map_err(|e| format!("Failed to load components: {}", e))?;
    
    // Create a unique ID for the custom component
    let uid = format!("custom.{}", uuid::Uuid::new_v4());
    
    // Add the empty custom component
    let component = crate::core::instance::InstanceComponent {
        uid,
        name,
        version: "1.0".to_string(),
        component_type: crate::core::instance::ComponentType::Other,
        enabled: true,
        removable: true,
        version_changeable: true,
        customizable: true,
        revertible: false,
        custom: true,
        order: component_list.components.len() as i32,
        problems: Vec::new(),
    };
    
    component_list.add(component);
    
    // Save the updated list
    component_list.save(&instance.path)
        .map_err(|e| format!("Failed to save components: {}", e))?;
    
    tracing::info!("Added empty component to instance {}", instance_id);
    
    Ok(())
}

#[tauri::command]
pub async fn customize_component(
    state: State<'_, AppState>,
    instance_id: String,
    component_uid: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Load component list
    let mut component_list = crate::core::instance::ComponentList::load(&instance.path)
        .map_err(|e| format!("Failed to load components: {}", e))?;
    
    // Find and mark the component as custom
    if let Some(component) = component_list.get_mut(&component_uid) {
        if !component.customizable {
            return Err("This component cannot be customized".to_string());
        }
        component.custom = true;
        component.revertible = true;
        
        // Save the updated list
        component_list.save(&instance.path)
            .map_err(|e| format!("Failed to save components: {}", e))?;
        
        tracing::info!("Customized component {} in instance {}", component_uid, instance_id);
        Ok(())
    } else {
        Err("Component not found".to_string())
    }
}

#[tauri::command]
pub async fn revert_component(
    state: State<'_, AppState>,
    instance_id: String,
    component_uid: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Load component list
    let mut component_list = crate::core::instance::ComponentList::load(&instance.path)
        .map_err(|e| format!("Failed to load components: {}", e))?;
    
    // Find and revert the component
    if let Some(component) = component_list.get_mut(&component_uid) {
        if !component.revertible {
            return Err("This component cannot be reverted".to_string());
        }
        component.custom = false;
        component.revertible = false;
        
        // Save the updated list
        component_list.save(&instance.path)
            .map_err(|e| format!("Failed to save components: {}", e))?;
        
        tracing::info!("Reverted component {} in instance {}", component_uid, instance_id);
        Ok(())
    } else {
        Err("Component not found".to_string())
    }
}
