//! Extract natives step.
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

use async_trait::async_trait;
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};
use zip::ZipArchive;

use crate::core::launch::{LaunchContext, LaunchStep, LaunchStepResult};
use crate::core::minecraft::version::{fetch_version_manifest, fetch_version_data};

/// Step that extracts native libraries
pub struct ExtractNativesStep {
    status: Option<String>,
    progress: f32,
}

impl ExtractNativesStep {
    pub fn new() -> Self {
        Self {
            status: None,
            progress: 0.0,
        }
    }
    
    /// Extract a native JAR to the output directory
    fn extract_native_jar(&self, jar_path: &Path, output_dir: &Path, apply_jnilib_hack: bool) -> io::Result<()> {
        let file = File::open(jar_path)?;
        let mut archive = ZipArchive::new(file)?;
        
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let name = entry.name().to_string();
            
            // Skip non-native files
            if name.ends_with('/') {
                continue; // Directory
            }
            
            // Skip META-INF
            if name.starts_with("META-INF/") {
                continue;
            }
            
            // Only extract native libraries
            let is_native = name.ends_with(".dll") 
                || name.ends_with(".so") 
                || name.ends_with(".dylib")
                || name.ends_with(".jnilib");
            
            if !is_native {
                continue;
            }
            
            // Apply .jnilib -> .dylib hack for macOS
            let output_name = if apply_jnilib_hack && name.ends_with(".jnilib") {
                name.replace(".jnilib", ".dylib")
            } else {
                name.clone()
            };
            
            let output_path = output_dir.join(&output_name);
            
            // Create parent directories
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Extract file
            let mut output_file = File::create(&output_path)?;
            io::copy(&mut entry, &mut output_file)?;
            
            debug!("Extracted native: {} -> {:?}", name, output_path);
        }
        
        Ok(())
    }
    
    /// Get list of native JARs for the current platform
    async fn get_native_jars(&self, context: &LaunchContext) -> Result<Vec<PathBuf>, String> {
        let manifest = fetch_version_manifest().await
            .map_err(|e| format!("Failed to fetch version manifest: {}", e))?;
        
        let version_info = manifest.get_version(&context.instance.minecraft_version)
            .ok_or_else(|| format!("Version {} not found", context.instance.minecraft_version))?;
        
        let version_data = fetch_version_data(version_info).await
            .map_err(|e| format!("Failed to fetch version data: {}", e))?;
        
        let mut native_jars = Vec::new();
        let libraries_dir = &context.libraries_dir;
        
        // Determine current OS
        let os_name = if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "osx"
        } else {
            "linux"
        };
        
        for library in &version_data.libraries {
            // Check if this library has natives for our OS
            if let Some(ref natives) = library.natives {
                let native_key = match os_name {
                    "windows" => natives.get("windows"),
                    "osx" => natives.get("osx").or_else(|| natives.get("macos")),
                    "linux" => natives.get("linux"),
                    _ => None,
                };
                
                if let Some(classifier) = native_key {
                    // Build path to native JAR
                    if let Some(ref downloads) = library.downloads {
                        if let Some(ref classifiers) = downloads.classifiers {
                            // Replace ${arch} in classifier
                            let arch = if cfg!(target_arch = "x86_64") {
                                "64"
                            } else {
                                "32"
                            };
                            let actual_classifier = classifier.replace("${arch}", arch);
                            
                            if let Some(artifact) = classifiers.get(&actual_classifier) {
                                let path = libraries_dir.join(&artifact.path);
                                if path.exists() {
                                    native_jars.push(path);
                                } else {
                                    warn!("Native JAR not found: {:?}", path);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(native_jars)
    }
}

#[async_trait]
impl LaunchStep for ExtractNativesStep {
    fn name(&self) -> &'static str {
        "Extract Natives"
    }
    
    fn description(&self) -> &'static str {
        "Extracts native libraries required by Minecraft"
    }
    
    async fn execute(&mut self, context: &mut LaunchContext) -> LaunchStepResult {
        self.status = Some("Finding native libraries...".to_string());
        self.progress = 0.0;
        
        let native_jars = match self.get_native_jars(context).await {
            Ok(jars) => jars,
            Err(e) => return LaunchStepResult::Failed(e),
        };
        
        if native_jars.is_empty() {
            info!("No native libraries to extract");
            self.status = Some("No natives to extract".to_string());
            self.progress = 1.0;
            return LaunchStepResult::Success;
        }
        
        info!("Extracting {} native JARs", native_jars.len());
        
        let output_dir = &context.natives_dir;
        
        // Clean existing natives
        if output_dir.exists() {
            if let Err(e) = fs::remove_dir_all(output_dir) {
                warn!("Failed to clean natives directory: {}", e);
            }
        }
        fs::create_dir_all(output_dir).ok();
        
        // Check if jnilib hack is needed (Java 8+)
        let java_major = context.java_version
            .as_ref()
            .and_then(|v| {
                if v.starts_with("1.") {
                    v.split('.').nth(1)?.parse::<u32>().ok()
                } else {
                    v.split('.').next()?.parse::<u32>().ok()
                }
            })
            .unwrap_or(8);
        
        let apply_jnilib_hack = java_major >= 8;
        
        // Extract each native JAR
        for (i, jar_path) in native_jars.iter().enumerate() {
            self.status = Some(format!(
                "Extracting {}...",
                jar_path.file_name().unwrap_or_default().to_string_lossy()
            ));
            self.progress = i as f32 / native_jars.len() as f32;
            
            if let Err(e) = self.extract_native_jar(jar_path, output_dir, apply_jnilib_hack) {
                let reason = format!(
                    "Couldn't extract native jar '{}' to destination '{:?}': {}",
                    jar_path.display(), output_dir, e
                );
                return LaunchStepResult::Failed(reason);
            }
        }
        
        info!("Native libraries extracted successfully");
        self.status = Some("Natives extracted".to_string());
        self.progress = 1.0;
        
        LaunchStepResult::Success
    }
    
    async fn finalize(&mut self, _context: &mut LaunchContext) {
        // NOTE: We intentionally do NOT clean up natives here.
        // The finalize method is called immediately after the game process spawns,
        // but the game is still running and needs these native libraries.
        // The natives directory is cleaned at the START of extraction instead,
        // ensuring fresh natives for each launch without breaking running games.
    }
    
    fn progress(&self) -> f32 {
        self.progress
    }
    
    fn status(&self) -> Option<String> {
        self.status.clone()
    }
}

impl Default for ExtractNativesStep {
    fn default() -> Self {
        Self::new()
    }
}
