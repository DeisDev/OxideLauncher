//! Instance management commands
//!
//! This module is organized into submodules by domain:
//! - `crud` - Basic CRUD operations (get, create, delete, rename, copy)
//! - `launch` - Launch management (launch, kill, status, logs)
//! - `components` - Component management (mod loaders, ordering)
//! - `jarmods` - Jar mods and Java agents
//! - `folders` - Folder opening utilities
//! - `transfer` - Import/Export functionality
//! - `settings` - Instance settings management
//! - `blocked_mods` - Blocked mod handling for CurseForge

mod crud;
mod launch;
mod components;
mod jarmods;
mod folders;
mod transfer;
mod settings;
pub mod blocked_mods;

// Re-export all commands for registration in main.rs
pub use crud::*;
pub use launch::*;
pub use components::*;
pub use jarmods::*;
pub use folders::*;
pub use transfer::*;
pub use settings::*;
pub use blocked_mods::*;

use crate::core::instance::{Instance, ModLoader, ModLoaderType};
use serde::{Deserialize, Serialize};

// =============================================================================
// Shared Types
// =============================================================================

/// Serializable instance information for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub id: String,
    pub name: String,
    pub minecraft_version: String,
    pub mod_loader: String,
    pub mod_loader_version: Option<String>,
    pub icon: Option<String>,
    pub last_played: Option<String>,
    pub total_played_seconds: u64,
    pub group: Option<String>,
}

impl From<&Instance> for InstanceInfo {
    fn from(inst: &Instance) -> Self {
        let (mod_loader, mod_loader_version) = match &inst.mod_loader {
            Some(ml) => (ml.loader_type.name().to_string(), Some(ml.version.clone())),
            None => ("Vanilla".to_string(), None),
        };
        
        // Convert custom icon path to asset URL or keep as-is for default icons
        let icon = if inst.icon.starts_with("custom:") {
            // Extract filename from "custom:icon.png" format
            let filename = inst.icon.trim_start_matches("custom:");
            let icon_path = inst.path.join(filename);
            if icon_path.exists() {
                // Convert to asset:// URL that Tauri can serve
                // Use convertFileSrc on frontend instead, just provide the path
                Some(icon_path.to_string_lossy().to_string())
            } else {
                // Custom icon doesn't exist, use default
                None
            }
        } else if inst.icon == "default" {
            None
        } else {
            // Named default icon
            Some(inst.icon.clone())
        };
        
        InstanceInfo {
            id: inst.id.clone(),
            name: inst.name.clone(),
            minecraft_version: inst.minecraft_version.clone(),
            mod_loader,
            mod_loader_version,
            icon,
            last_played: inst.last_played.map(|dt| dt.to_string()),
            total_played_seconds: inst.total_played_seconds,
            group: inst.group.clone(),
        }
    }
}

/// Request payload for creating a new instance
#[derive(Debug, Clone, Deserialize)]
pub struct CreateInstanceRequest {
    pub name: String,
    pub minecraft_version: String,
    pub mod_loader_type: String,
    pub loader_version: Option<String>,
    pub group: Option<String>,
}

/// Instance settings update payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceSettingsUpdate {
    pub name: Option<String>,
    pub java_path: Option<String>,
    pub java_args: Option<String>,
    pub min_memory: Option<u32>,
    pub max_memory: Option<u32>,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub skip_java_compatibility_check: Option<bool>,
    pub close_launcher_on_launch: Option<bool>,
    pub quit_launcher_on_exit: Option<bool>,
    pub prelaunch_command: Option<String>,
    pub postexit_command: Option<String>,
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Parse mod loader type from string
pub(crate) fn parse_mod_loader(loader_type: &str, version: Option<String>) -> Option<ModLoader> {
    let version = version.unwrap_or_else(|| "latest".to_string());
    match loader_type {
        "Forge" => Some(ModLoader {
            loader_type: ModLoaderType::Forge,
            version,
        }),
        "NeoForge" => Some(ModLoader {
            loader_type: ModLoaderType::NeoForge,
            version,
        }),
        "Fabric" => Some(ModLoader {
            loader_type: ModLoaderType::Fabric,
            version,
        }),
        "Quilt" => Some(ModLoader {
            loader_type: ModLoaderType::Quilt,
            version,
        }),
        "LiteLoader" => Some(ModLoader {
            loader_type: ModLoaderType::LiteLoader,
            version,
        }),
        _ => None,
    }
}
