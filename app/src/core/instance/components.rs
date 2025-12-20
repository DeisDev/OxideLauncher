//! Instance component management for version stacks and mod loaders.
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

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A component in an instance's version stack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceComponent {
    /// Unique identifier for this component
    pub uid: String,
    
    /// Display name
    pub name: String,
    
    /// Version string
    pub version: String,
    
    /// Component type
    pub component_type: ComponentType,
    
    /// Whether this component is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    
    /// Whether this component can be removed
    #[serde(default)]
    pub removable: bool,
    
    /// Whether this component's version can be changed
    #[serde(default)]
    pub version_changeable: bool,
    
    /// Whether this component can be customized
    #[serde(default)]
    pub customizable: bool,
    
    /// Whether this component can be reverted to default
    #[serde(default)]
    pub revertible: bool,
    
    /// Whether this is a custom (user-modified) component
    #[serde(default)]
    pub custom: bool,
    
    /// Order in the component stack (lower = earlier)
    #[serde(default)]
    pub order: i32,
    
    /// Any problems with this component
    #[serde(default)]
    pub problems: Vec<ComponentProblem>,
}

fn default_true() -> bool {
    true
}

/// Types of components
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    /// The Minecraft game itself
    Minecraft,
    /// Mod loader (Forge, Fabric, etc.)
    ModLoader,
    /// A library dependency
    Library,
    /// Java agent
    Agent,
    /// Custom jar modification
    JarMod,
    /// Intermediary mappings (for Fabric)
    Mappings,
    /// Other/unknown
    Other,
}

impl ComponentType {
    #[allow(dead_code)]
    pub fn display_name(&self) -> &'static str {
        match self {
            ComponentType::Minecraft => "Minecraft",
            ComponentType::ModLoader => "Mod Loader",
            ComponentType::Library => "Library",
            ComponentType::Agent => "Java Agent",
            ComponentType::JarMod => "Jar Mod",
            ComponentType::Mappings => "Mappings",
            ComponentType::Other => "Other",
        }
    }
}

/// Problem severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProblemSeverity {
    None,
    Warning,
    Error,
}

/// A problem with a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentProblem {
    pub severity: ProblemSeverity,
    pub description: String,
}

/// Component list for an instance
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentList {
    pub components: Vec<InstanceComponent>,
}

impl ComponentList {
    /// Load component list from instance directory
    pub fn load(instance_path: &PathBuf) -> crate::core::error::Result<Self> {
        let components_file = instance_path.join("mmc-pack.json");
        
        if components_file.exists() {
            // Try to load existing component file
            let content = std::fs::read_to_string(&components_file)?;
            let list: ComponentList = serde_json::from_str(&content)?;
            Ok(list)
        } else {
            // Return empty list
            Ok(Self::default())
        }
    }
    
    /// Save component list to instance directory
    pub fn save(&self, instance_path: &PathBuf) -> crate::core::error::Result<()> {
        let components_file = instance_path.join("mmc-pack.json");
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(components_file, content)?;
        Ok(())
    }
    
    /// Get component by uid
    #[allow(dead_code)]
    pub fn get(&self, uid: &str) -> Option<&InstanceComponent> {
        self.components.iter().find(|c| c.uid == uid)
    }
    
    /// Get mutable component by uid
    pub fn get_mut(&mut self, uid: &str) -> Option<&mut InstanceComponent> {
        self.components.iter_mut().find(|c| c.uid == uid)
    }
    
    /// Add a component
    pub fn add(&mut self, component: InstanceComponent) {
        self.components.push(component);
    }
    
    /// Remove a component by uid
    #[allow(dead_code)]
    pub fn remove(&mut self, uid: &str) -> bool {
        if let Some(pos) = self.components.iter().position(|c| c.uid == uid) {
            self.components.remove(pos);
            true
        } else {
            false
        }
    }
    
    /// Move a component up in the list
    pub fn move_up(&mut self, uid: &str) -> bool {
        if let Some(pos) = self.components.iter().position(|c| c.uid == uid) {
            if pos > 0 {
                self.components.swap(pos, pos - 1);
                return true;
            }
        }
        false
    }
    
    /// Move a component down in the list
    pub fn move_down(&mut self, uid: &str) -> bool {
        if let Some(pos) = self.components.iter().position(|c| c.uid == uid) {
            if pos < self.components.len() - 1 {
                self.components.swap(pos, pos + 1);
                return true;
            }
        }
        false
    }
    
    /// Get the Minecraft component
    #[allow(dead_code)]
    pub fn get_minecraft(&self) -> Option<&InstanceComponent> {
        self.components.iter().find(|c| c.component_type == ComponentType::Minecraft)
    }
    
    /// Get all mod loader components
    #[allow(dead_code)]
    pub fn get_mod_loaders(&self) -> Vec<&InstanceComponent> {
        self.components.iter()
            .filter(|c| c.component_type == ComponentType::ModLoader)
            .collect()
    }
}

/// Build a component list from instance data
pub fn build_component_list(
    minecraft_version: &str,
    mod_loader: Option<&super::types::ModLoader>,
) -> ComponentList {
    let mut components = Vec::new();
    
    // Minecraft component
    components.push(InstanceComponent {
        uid: "net.minecraft".to_string(),
        name: "Minecraft".to_string(),
        version: minecraft_version.to_string(),
        component_type: ComponentType::Minecraft,
        enabled: true,
        removable: false,
        version_changeable: true,
        customizable: true,
        revertible: false,
        custom: false,
        order: 0,
        problems: Vec::new(),
    });
    
    // Mod loader component (if present)
    if let Some(loader) = mod_loader {
        let (uid, name) = match loader.loader_type {
            super::types::ModLoaderType::Forge => ("net.minecraftforge", "Forge"),
            super::types::ModLoaderType::NeoForge => ("net.neoforged", "NeoForge"),
            super::types::ModLoaderType::Fabric => ("net.fabricmc.fabric-loader", "Fabric Loader"),
            super::types::ModLoaderType::Quilt => ("org.quiltmc.quilt-loader", "Quilt Loader"),
            super::types::ModLoaderType::LiteLoader => ("com.mumfrey.liteloader", "LiteLoader"),
        };
        
        // Add intermediary mappings for Fabric/Quilt
        if matches!(loader.loader_type, super::types::ModLoaderType::Fabric | super::types::ModLoaderType::Quilt) {
            components.push(InstanceComponent {
                uid: "net.fabricmc.intermediary".to_string(),
                name: "Intermediary Mappings".to_string(),
                version: minecraft_version.to_string(),
                component_type: ComponentType::Mappings,
                enabled: true,
                removable: false,
                version_changeable: false,
                customizable: false,
                revertible: false,
                custom: false,
                order: 1,
                problems: Vec::new(),
            });
        }
        
        components.push(InstanceComponent {
            uid: uid.to_string(),
            name: name.to_string(),
            version: loader.version.clone(),
            component_type: ComponentType::ModLoader,
            enabled: true,
            removable: true,
            version_changeable: true,
            customizable: true,
            revertible: false,
            custom: false,
            order: if matches!(loader.loader_type, super::types::ModLoaderType::Fabric | super::types::ModLoaderType::Quilt) { 2 } else { 1 },
            problems: Vec::new(),
        });
    }
    
    // Add LWJGL 3 library component
    // LWJGL version is tied to Minecraft version
    let lwjgl_version = get_lwjgl_version_for_minecraft(minecraft_version);
    components.push(InstanceComponent {
        uid: "org.lwjgl3".to_string(),
        name: "LWJGL 3".to_string(),
        version: lwjgl_version,
        component_type: ComponentType::Library,
        enabled: true,
        removable: false,
        version_changeable: false,
        customizable: false,
        revertible: false,
        custom: false,
        order: 100, // Libraries come after main components
        problems: Vec::new(),
    });
    
    ComponentList { components }
}

/// Get LWJGL version for a Minecraft version
/// This is a simplified mapping - in reality, this would be read from version manifests
fn get_lwjgl_version_for_minecraft(mc_version: &str) -> String {
    // Minecraft 1.19+ uses LWJGL 3.3.x
    // Minecraft 1.13-1.18 uses LWJGL 3.2.x
    // Minecraft 1.12 and older uses LWJGL 2.x
    let version_parts: Vec<&str> = mc_version.split('.').collect();
    if version_parts.len() >= 2 {
        if let (Ok(major), Ok(minor)) = (version_parts[0].parse::<u32>(), version_parts[1].parse::<u32>()) {
            if major == 1 {
                if minor >= 19 {
                    return "3.3.1".to_string();
                } else if minor >= 13 {
                    return "3.2.3".to_string();
                } else {
                    return "2.9.4".to_string();
                }
            }
        }
    }
    "3.3.1".to_string() // Default to latest
}
