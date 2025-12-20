//! Java management Tauri commands.
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
use tauri::Emitter;
use tokio::sync::mpsc;

/// Serializable Java installation info for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaInstallationInfo {
    pub id: String,
    pub path: String,
    pub version: String,
    pub major_version: u32,
    pub arch: String,
    pub vendor: String,
    pub is_64bit: bool,
    pub is_managed: bool,
    pub recommended: bool,
}

impl From<crate::core::java::JavaInstallation> for JavaInstallationInfo {
    fn from(install: crate::core::java::JavaInstallation) -> Self {
        Self {
            id: install.id.clone(),
            path: install.path.to_string_lossy().to_string(),
            version: install.version.to_string(),
            major_version: install.version.major,
            arch: install.arch.to_string(),
            vendor: install.vendor.clone(),
            is_64bit: install.arch.is_64bit(),
            is_managed: install.is_managed,
            recommended: install.recommended,
        }
    }
}

/// Available Java version for download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableJavaInfo {
    pub major: u32,
    pub name: String,
    pub is_lts: bool,
}

/// Detect all Java installations on the system
#[tauri::command]
pub async fn detect_java() -> Result<Vec<JavaInstallationInfo>, String> {
    use crate::core::java::detection::detect_java_installations;
    
    let installations = detect_java_installations();
    
    Ok(installations.into_iter().map(JavaInstallationInfo::from).collect())
}

/// Find Java that meets a version requirement
#[tauri::command]
pub async fn find_java_for_minecraft(minecraft_version: String) -> Result<Option<JavaInstallationInfo>, String> {
    use crate::core::java::detection::find_java_for_minecraft;
    
    if let Some(java) = find_java_for_minecraft(&minecraft_version) {
        Ok(Some(JavaInstallationInfo::from(java)))
    } else {
        Ok(None)
    }
}

/// Get required Java version for a Minecraft version
#[tauri::command]
pub fn get_required_java(minecraft_version: String) -> u32 {
    crate::core::java::detection::get_required_java_version(&minecraft_version)
}

/// Validate a Java installation
#[tauri::command]
pub async fn validate_java(java_path: String) -> Result<JavaInstallationInfo, String> {
    use crate::core::java::checker::JavaChecker;
    use std::path::PathBuf;
    
    let path = PathBuf::from(&java_path);
    
    if !path.exists() {
        return Err("Java executable does not exist".to_string());
    }
    
    let checker = JavaChecker::new(path);
    let result = checker.check().await;
    
    if result.valid {
        if let Some(installation) = result.to_installation() {
            return Ok(JavaInstallationInfo::from(installation));
        }
    }
    
    Err(result.error.unwrap_or_else(|| "Java validation failed".to_string()))
}

/// Fetch available Java versions for download
#[tauri::command]
pub async fn fetch_available_java_versions() -> Result<Vec<AvailableJavaInfo>, String> {
    use crate::core::java::download::fetch_adoptium_versions;
    
    let versions = fetch_adoptium_versions()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(versions.into_iter().map(|v| AvailableJavaInfo {
        major: v.major,
        name: v.name,
        is_lts: v.is_lts,
    }).collect())
}

/// Download and install Java
#[tauri::command]
pub async fn download_java(
    major_version: u32,
    app: tauri::AppHandle,
) -> Result<JavaInstallationInfo, String> {
    use crate::core::java::download::{fetch_adoptium_download, download_java as do_download};
    
    // Fetch download metadata
    let metadata = fetch_adoptium_download(major_version)
        .await
        .map_err(|e| e.to_string())?;
    
    // Create progress channel
    let (tx, mut rx) = mpsc::channel(100);
    
    // Spawn progress event emitter
    let app_clone = app.clone();
    tokio::spawn(async move {
        while let Some(progress) = rx.recv().await {
            let _ = app_clone.emit("java-download-progress", &progress);
        }
    });
    
    // Download Java
    let installation = do_download(&metadata, Some(tx))
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(JavaInstallationInfo::from(installation))
}

/// Get the path to the managed Java directory
#[tauri::command]
pub fn get_java_install_dir() -> Result<String, String> {
    use crate::core::java::download::get_java_install_dir;
    
    let dir = get_java_install_dir().map_err(|e| e.to_string())?;
    Ok(dir.to_string_lossy().to_string())
}

/// Delete a managed Java installation
#[tauri::command]
pub async fn delete_java(java_path: String) -> Result<(), String> {
    use crate::core::java::download::delete_java_installation;
    use crate::core::java::install::JavaInstallation;
    use std::path::PathBuf;
    
    // Create a minimal installation struct for deletion
    let mut installation = JavaInstallation::default();
    installation.path = PathBuf::from(&java_path);
    installation.is_managed = true;
    
    delete_java_installation(&installation)
        .await
        .map_err(|e| e.to_string())
}
/// Java compatibility check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaCompatibilityResult {
    /// Whether the Java is compatible
    pub compatible: bool,
    /// The Java major version
    pub java_major: u32,
    /// The required Java major version
    pub required_major: u32,
    /// Minimum compatible version
    pub min_compatible: u32,
    /// Maximum compatible version
    pub max_compatible: u32,
    /// Human-readable message
    pub message: String,
}

/// Check if a Java version is compatible with a Minecraft version
#[tauri::command]
pub fn check_java_compatibility(
    java_major_version: u32,
    minecraft_version: String,
) -> JavaCompatibilityResult {
    let required = crate::core::java::detection::get_required_java_version(&minecraft_version);
    let (min_compatible, max_compatible) = get_java_compat_range(required);
    
    let compatible = java_major_version >= min_compatible && java_major_version <= max_compatible;
    
    let message = if compatible {
        format!("Java {} is compatible with Minecraft {}", java_major_version, minecraft_version)
    } else if java_major_version < min_compatible {
        format!(
            "Java {} is too old for Minecraft {}. Requires Java {} or newer.",
            java_major_version, minecraft_version, min_compatible
        )
    } else {
        format!(
            "Java {} may not be compatible with Minecraft {}. Recommended: Java {}",
            java_major_version, minecraft_version, required
        )
    };
    
    JavaCompatibilityResult {
        compatible,
        java_major: java_major_version,
        required_major: required,
        min_compatible,
        max_compatible,
        message,
    }
}

/// Get the compatible Java version range (min, max) for a required version
fn get_java_compat_range(required_major: u32) -> (u32, u32) {
    match required_major {
        8 => (8, 8),
        16 => (16, 17),
        17 => (17, 21),
        21 => (21, 25),
        _ => (required_major, required_major + 4),
    }
}

/// Find the best Java for an instance, with option to auto-download
#[tauri::command]
pub async fn find_best_java_for_instance(
    minecraft_version: String,
    auto_download: bool,
    app: tauri::AppHandle,
) -> Result<Option<JavaInstallationInfo>, String> {
    use crate::core::java::detection::find_java_for_minecraft;
    use crate::core::java::download::{fetch_adoptium_download, download_java as do_download};
    
    // First try to find existing compatible Java
    if let Some(java) = find_java_for_minecraft(&minecraft_version) {
        return Ok(Some(JavaInstallationInfo::from(java)));
    }
    
    // No compatible Java found - check if we should auto-download
    if !auto_download {
        return Ok(None);
    }
    
    // Get required version and download
    let required = crate::core::java::detection::get_required_java_version(&minecraft_version);
    
    // Emit event that we're starting auto-download
    let _ = app.emit("java-auto-download-started", &required);
    
    // Fetch download metadata
    let metadata = fetch_adoptium_download(required)
        .await
        .map_err(|e| format!("Failed to get Java download info: {}", e))?;
    
    // Create progress channel
    let (tx, mut rx) = tokio::sync::mpsc::channel(100);
    
    // Spawn progress event emitter
    let app_clone = app.clone();
    tokio::spawn(async move {
        while let Some(progress) = rx.recv().await {
            let _ = app_clone.emit("java-download-progress", &progress);
        }
    });
    
    // Download Java
    let installation = do_download(&metadata, Some(tx))
        .await
        .map_err(|e| format!("Failed to download Java: {}", e))?;
    
    // Emit completion
    let _ = app.emit("java-auto-download-completed", &required);
    
    Ok(Some(JavaInstallationInfo::from(installation)))
}