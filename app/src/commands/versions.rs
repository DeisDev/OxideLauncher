//! Version management Tauri commands for Minecraft and mod loaders.
//!
//! This module provides two implementation paths:
//! - **Meta Server** (new): Uses self-hosted PrismLauncher-format meta server
//! - **Legacy** (fallback): Uses direct API calls to individual sources
//!
//! The meta server implementation is preferred when available. Set `USE_META_SERVER`
//! to `true` to enable it once the meta server is ready.
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

use crate::core::{
    meta::MetaClient,
    minecraft::version::{fetch_version_manifest, VersionType},
    modloaders,
};
use serde::{Deserialize, Serialize};

/// Toggle to switch between meta server and legacy implementations.
/// Set to `true` once the meta server at meta.oxidelauncher.org is ready.
const USE_META_SERVER: bool = false;

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
// Meta Server Implementation
// ============================================================================

mod meta_impl {
    use super::*;

    pub async fn get_minecraft_versions(
        show_releases: bool,
        show_snapshots: bool,
        show_old: bool,
    ) -> Result<Vec<MinecraftVersionInfo>, String> {
        let client = MetaClient::default();
        let versions = client
            .get_minecraft_versions_filtered(show_releases, show_snapshots, show_old)
            .await
            .map_err(|e| format!("Failed to fetch Minecraft versions from meta server: {}", e))?;

        Ok(versions
            .into_iter()
            .map(|v| MinecraftVersionInfo {
                id: v.version,
                version_type: v.version_type.unwrap_or_else(|| "release".to_string()),
                release_time: v.release_time,
            })
            .collect())
    }

    pub async fn get_forge_versions(minecraft_version: &str) -> Result<Vec<LoaderVersionInfo>, String> {
        let client = MetaClient::default();
        let versions = client
            .get_forge_versions(minecraft_version)
            .await
            .map_err(|e| format!("Failed to fetch Forge versions from meta server: {}", e))?;

        Ok(versions
            .into_iter()
            .map(|v| LoaderVersionInfo {
                version: v.version,
                recommended: v.recommended,
            })
            .collect())
    }

    pub async fn get_fabric_versions(minecraft_version: &str) -> Result<Vec<LoaderVersionInfo>, String> {
        let client = MetaClient::default();
        let versions = client
            .get_fabric_versions(minecraft_version)
            .await
            .map_err(|e| format!("Failed to fetch Fabric versions from meta server: {}", e))?;

        Ok(versions
            .into_iter()
            .map(|v| LoaderVersionInfo {
                version: v.version,
                recommended: v.recommended,
            })
            .collect())
    }

    pub async fn get_neoforge_versions(minecraft_version: &str) -> Result<Vec<LoaderVersionInfo>, String> {
        let client = MetaClient::default();
        let versions = client
            .get_neoforge_versions(minecraft_version)
            .await
            .map_err(|e| format!("Failed to fetch NeoForge versions from meta server: {}", e))?;

        Ok(versions
            .into_iter()
            .map(|v| LoaderVersionInfo {
                version: v.version,
                recommended: v.recommended,
            })
            .collect())
    }

    pub async fn get_quilt_versions(minecraft_version: &str) -> Result<Vec<LoaderVersionInfo>, String> {
        let client = MetaClient::default();
        let versions = client
            .get_quilt_versions(minecraft_version)
            .await
            .map_err(|e| format!("Failed to fetch Quilt versions from meta server: {}", e))?;

        Ok(versions
            .into_iter()
            .map(|v| LoaderVersionInfo {
                version: v.version,
                recommended: v.recommended,
            })
            .collect())
    }

    pub async fn get_liteloader_versions(minecraft_version: &str) -> Result<Vec<LoaderVersionInfo>, String> {
        let client = MetaClient::default();
        let versions = client
            .get_liteloader_versions(minecraft_version)
            .await
            .map_err(|e| format!("Failed to fetch LiteLoader versions from meta server: {}", e))?;

        Ok(versions
            .into_iter()
            .map(|v| LoaderVersionInfo {
                version: v.version,
                recommended: v.recommended,
            })
            .collect())
    }
}

// ============================================================================
// Legacy Implementation (direct API calls)
// ============================================================================

mod legacy_impl {
    use super::*;

    pub async fn get_minecraft_versions(
        show_releases: bool,
        show_snapshots: bool,
        show_old: bool,
    ) -> Result<Vec<MinecraftVersionInfo>, String> {
        let manifest = fetch_version_manifest()
            .await
            .map_err(|e| format!("Failed to fetch version manifest: {}", e))?;

        let versions: Vec<MinecraftVersionInfo> = manifest
            .versions
            .into_iter()
            .filter(|v| match v.version_type {
                VersionType::Release => show_releases,
                VersionType::Snapshot => show_snapshots,
                VersionType::OldAlpha | VersionType::OldBeta => show_old,
            })
            .map(|v| MinecraftVersionInfo {
                id: v.id,
                version_type: format!("{:?}", v.version_type),
                release_time: v.release_time.to_rfc3339(),
            })
            .collect();

        Ok(versions)
    }

    pub async fn get_forge_versions(minecraft_version: &str) -> Result<Vec<LoaderVersionInfo>, String> {
        let versions = modloaders::get_forge_versions(minecraft_version)
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

    pub async fn get_fabric_versions(minecraft_version: &str) -> Result<Vec<LoaderVersionInfo>, String> {
        let versions = modloaders::get_fabric_versions(minecraft_version)
            .await
            .map_err(|e| format!("Failed to fetch Fabric versions: {}", e))?;

        Ok(versions
            .into_iter()
            .map(|v| LoaderVersionInfo {
                version: v.version,
                recommended: v.stable,
            })
            .collect())
    }

    pub async fn get_neoforge_versions(minecraft_version: &str) -> Result<Vec<LoaderVersionInfo>, String> {
        let versions = modloaders::get_neoforge_versions(minecraft_version)
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

    pub async fn get_quilt_versions(minecraft_version: &str) -> Result<Vec<LoaderVersionInfo>, String> {
        let versions = modloaders::get_quilt_versions(minecraft_version)
            .await
            .map_err(|e| format!("Failed to fetch Quilt versions: {}", e))?;

        Ok(versions
            .into_iter()
            .enumerate()
            .map(|(idx, v)| LoaderVersionInfo {
                version: v.version,
                recommended: idx == 0,
            })
            .collect())
    }

    pub async fn get_liteloader_versions(minecraft_version: &str) -> Result<Vec<LoaderVersionInfo>, String> {
        let versions = modloaders::get_liteloader_versions(minecraft_version)
            .await
            .map_err(|e| format!("Failed to fetch LiteLoader versions: {}", e))?;

        Ok(versions
            .into_iter()
            .enumerate()
            .map(|(idx, v)| LoaderVersionInfo {
                version: v.version,
                recommended: idx == 0,
            })
            .collect())
    }
}

// ============================================================================
// Tauri Commands (dispatch to appropriate implementation)
// ============================================================================

#[tauri::command]
pub async fn get_minecraft_versions(
    show_releases: bool,
    show_snapshots: bool,
    show_old: bool,
) -> Result<Vec<MinecraftVersionInfo>, String> {
    if USE_META_SERVER {
        meta_impl::get_minecraft_versions(show_releases, show_snapshots, show_old).await
    } else {
        legacy_impl::get_minecraft_versions(show_releases, show_snapshots, show_old).await
    }
}

#[tauri::command]
pub async fn get_forge_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    if USE_META_SERVER {
        meta_impl::get_forge_versions(&minecraft_version).await
    } else {
        legacy_impl::get_forge_versions(&minecraft_version).await
    }
}

#[tauri::command]
pub async fn get_fabric_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    if USE_META_SERVER {
        meta_impl::get_fabric_versions(&minecraft_version).await
    } else {
        legacy_impl::get_fabric_versions(&minecraft_version).await
    }
}

#[tauri::command]
pub async fn get_quilt_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    if USE_META_SERVER {
        meta_impl::get_quilt_versions(&minecraft_version).await
    } else {
        legacy_impl::get_quilt_versions(&minecraft_version).await
    }
}

#[tauri::command]
pub async fn get_neoforge_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    if USE_META_SERVER {
        meta_impl::get_neoforge_versions(&minecraft_version).await
    } else {
        legacy_impl::get_neoforge_versions(&minecraft_version).await
    }
}

#[tauri::command]
pub async fn get_liteloader_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    if USE_META_SERVER {
        meta_impl::get_liteloader_versions(&minecraft_version).await
    } else {
        legacy_impl::get_liteloader_versions(&minecraft_version).await
    }
}
