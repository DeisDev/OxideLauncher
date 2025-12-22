//! Version management Tauri commands for Minecraft and mod loaders.
//!
//! This module provides version listing for Minecraft and modloaders using
//! the Oxide Launcher meta server (PrismLauncher format).
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

use crate::core::meta::MetaClient;
use serde::{Deserialize, Serialize};

/// Minecraft version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftVersionInfo {
    pub id: String,
    pub version_type: String,
    pub release_time: String,
}

/// Mod loader version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderVersionInfo {
    pub version: String,
    pub recommended: bool,
}

// ============================================================================
// Tauri Commands
// ============================================================================

#[tauri::command]
pub async fn get_minecraft_versions(
    show_releases: bool,
    show_snapshots: bool,
    show_betas: bool,
    show_alphas: bool,
    show_experimental: bool,
) -> Result<Vec<MinecraftVersionInfo>, String> {
    let client = MetaClient::default();
    let versions = client
        .get_minecraft_versions_filtered(show_releases, show_snapshots, show_betas, show_alphas, show_experimental)
        .await
        .map_err(|e| format!("Failed to fetch Minecraft versions: {}", e))?;

    Ok(versions
        .into_iter()
        .map(|v| MinecraftVersionInfo {
            id: v.version,
            version_type: v.version_type.unwrap_or_else(|| "release".to_string()),
            release_time: v.release_time,
        })
        .collect())
}

#[tauri::command]
pub async fn get_forge_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    let client = MetaClient::default();
    let versions = client
        .get_forge_versions(&minecraft_version)
        .await
        .map_err(|e| format!("Failed to fetch Forge versions: {}", e))?;

    Ok(versions
        .into_iter()
        .map(|v| LoaderVersionInfo {
            version: v.version,
            recommended: v.recommended,
        })
        .collect())
}

#[tauri::command]
pub async fn get_fabric_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    let client = MetaClient::default();
    let versions = client
        .get_fabric_versions(&minecraft_version)
        .await
        .map_err(|e| format!("Failed to fetch Fabric versions: {}", e))?;

    Ok(versions
        .into_iter()
        .map(|v| LoaderVersionInfo {
            version: v.version,
            recommended: v.recommended,
        })
        .collect())
}

#[tauri::command]
pub async fn get_quilt_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    let client = MetaClient::default();
    let versions = client
        .get_quilt_versions(&minecraft_version)
        .await
        .map_err(|e| format!("Failed to fetch Quilt versions: {}", e))?;

    // Quilt versions are all beta (recommended=false), so treat first as recommended
    let mut result: Vec<LoaderVersionInfo> = versions
        .into_iter()
        .enumerate()
        .map(|(idx, v)| LoaderVersionInfo {
            version: v.version,
            // If meta marks it recommended, use that; otherwise first is recommended
            recommended: v.recommended || idx == 0,
        })
        .collect();
    
    // Ensure only one is marked recommended
    if result.iter().filter(|v| v.recommended).count() > 1 {
        let mut found_first = false;
        for item in result.iter_mut() {
            if item.recommended {
                if found_first {
                    item.recommended = false;
                } else {
                    found_first = true;
                }
            }
        }
    }
    
    Ok(result)
}

#[tauri::command]
pub async fn get_neoforge_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    let client = MetaClient::default();
    let versions = client
        .get_neoforge_versions(&minecraft_version)
        .await
        .map_err(|e| format!("Failed to fetch NeoForge versions: {}", e))?;

    Ok(versions
        .into_iter()
        .map(|v| LoaderVersionInfo {
            version: v.version,
            recommended: v.recommended,
        })
        .collect())
}

#[tauri::command]
pub async fn get_liteloader_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    let client = MetaClient::default();
    let versions = client
        .get_liteloader_versions(&minecraft_version)
        .await
        .map_err(|e| format!("Failed to fetch LiteLoader versions: {}", e))?;

    Ok(versions
        .into_iter()
        .enumerate()
        .map(|(idx, v)| LoaderVersionInfo {
            version: v.version,
            // LiteLoader: if meta marks recommended, use it; otherwise first is recommended
            recommended: v.recommended || idx == 0,
        })
        .collect())
}
