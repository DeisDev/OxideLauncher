//! Minecraft and mod loader version commands

use crate::core::{
    minecraft::version::{fetch_version_manifest, VersionType},
    modloaders,
};
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

#[tauri::command]
pub async fn get_minecraft_versions(
    show_releases: bool,
    show_snapshots: bool,
    show_old: bool,
) -> Result<Vec<MinecraftVersionInfo>, String> {
    let manifest = fetch_version_manifest()
        .await
        .map_err(|e| format!("Failed to fetch version manifest: {}", e))?;
    
    let versions: Vec<MinecraftVersionInfo> = manifest.versions
        .into_iter()
        .filter(|v| {
            match v.version_type {
                VersionType::Release => show_releases,
                VersionType::Snapshot => show_snapshots,
                VersionType::OldAlpha | VersionType::OldBeta => show_old,
            }
        })
        .map(|v| MinecraftVersionInfo {
            id: v.id,
            version_type: format!("{:?}", v.version_type),
            release_time: v.release_time.to_rfc3339(),
        })
        .collect();
    
    Ok(versions)
}

#[tauri::command]
pub async fn get_forge_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    let versions = modloaders::get_forge_versions(&minecraft_version)
        .await
        .map_err(|e| format!("Failed to fetch Forge versions: {}", e))?;
    
    Ok(versions.into_iter().map(|v| LoaderVersionInfo {
        version: v.version,
        recommended: v.recommended,
    }).collect())
}

#[tauri::command]
pub async fn get_fabric_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    let versions = modloaders::get_fabric_versions(&minecraft_version)
        .await
        .map_err(|e| format!("Failed to fetch Fabric versions: {}", e))?;
    
    Ok(versions.into_iter().map(|v| LoaderVersionInfo {
        version: v.version,
        recommended: v.stable,
    }).collect())
}

#[tauri::command]
pub async fn get_quilt_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    let versions = modloaders::get_quilt_versions(&minecraft_version)
        .await
        .map_err(|e| format!("Failed to fetch Quilt versions: {}", e))?;
    
    Ok(versions.into_iter().enumerate().map(|(idx, v)| LoaderVersionInfo {
        version: v.version,
        recommended: idx == 0,
    }).collect())
}

#[tauri::command]
pub async fn get_neoforge_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    let versions = modloaders::get_neoforge_versions(&minecraft_version)
        .await
        .map_err(|e| format!("Failed to fetch NeoForge versions: {}", e))?;
    
    Ok(versions.into_iter().map(|v| LoaderVersionInfo {
        version: v.version,
        recommended: v.recommended,
    }).collect())
}

#[tauri::command]
pub async fn get_liteloader_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    let versions = modloaders::get_liteloader_versions(&minecraft_version)
        .await
        .map_err(|e| format!("Failed to fetch LiteLoader versions: {}", e))?;
    
    Ok(versions.into_iter().enumerate().map(|(idx, v)| LoaderVersionInfo {
        version: v.version,
        recommended: idx == 0,
    }).collect())
}
