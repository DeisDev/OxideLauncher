//! Build script for OxideLaunch wrapper JAR compilation and Tauri setup.
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

use std::process::Command;
use std::path::PathBuf;

fn main() {
    // Build and copy OxideLaunch wrapper JAR
    build_wrapper_jar();
    
    // Skip icon embedding for now
    tauri_build::try_build(tauri_build::Attributes::new()).expect("failed to run build script");
}

/// Build the OxideLaunch wrapper JAR and copy it to the resources directory
fn build_wrapper_jar() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let wrapper_dir = manifest_dir.join("wrappers").join("oxide-launch");
    let build_script = wrapper_dir.join("build.bat");
    let wrapper_jar = wrapper_dir.join("build").join("OxideLaunch.jar");
    
    // Only build if the build script exists
    if !build_script.exists() {
        println!("cargo:warning=OxideLaunch build.bat not found at {:?}", build_script);
        return;
    }
    
    // Check if we need to rebuild (source files newer than JAR)
    let needs_rebuild = if wrapper_jar.exists() {
        let jar_modified = std::fs::metadata(&wrapper_jar)
            .and_then(|m| m.modified())
            .ok();
        
        // Check if any Java source file is newer than the JAR
        let src_dir = wrapper_dir.join("src");
        if src_dir.exists() {
            walkdir_check_newer(&src_dir, jar_modified)
        } else {
            true
        }
    } else {
        true
    };
    
    if needs_rebuild {
        println!("cargo:warning=Building OxideLaunch wrapper JAR...");
        
        // Run the build script
        let status = Command::new("cmd")
            .args(["/C", build_script.to_str().unwrap()])
            .current_dir(&wrapper_dir)
            .status();
        
        match status {
            Ok(s) if s.success() => {
                println!("cargo:warning=OxideLaunch wrapper JAR built successfully");
            }
            Ok(s) => {
                println!("cargo:warning=OxideLaunch build failed with status: {:?}", s);
            }
            Err(e) => {
                println!("cargo:warning=Failed to run OxideLaunch build: {}", e);
            }
        }
    }
    
    // Copy the JAR to the resources directory for bundling
    if wrapper_jar.exists() {
        // Copy to target directory for runtime access
        let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap_or_else(|_| ".".to_string()));
        let target_dir = out_dir.ancestors().nth(3).unwrap_or(&out_dir);
        let bin_dir = target_dir.join("bin");
        
        if let Err(e) = std::fs::create_dir_all(&bin_dir) {
            println!("cargo:warning=Failed to create bin directory: {}", e);
            return;
        }
        
        let dest_jar = bin_dir.join("OxideLaunch.jar");
        if let Err(e) = std::fs::copy(&wrapper_jar, &dest_jar) {
            println!("cargo:warning=Failed to copy OxideLaunch.jar: {}", e);
        } else {
            println!("cargo:warning=Copied OxideLaunch.jar to {:?}", dest_jar);
        }
    }
    
    // Re-run build script if wrapper source changes
    println!("cargo:rerun-if-changed=wrappers/oxide-launch/src");
    println!("cargo:rerun-if-changed=wrappers/oxide-launch/build.bat");
}

/// Check if any file in the directory tree is newer than the given time
fn walkdir_check_newer(dir: &PathBuf, reference_time: Option<std::time::SystemTime>) -> bool {
    let reference_time = match reference_time {
        Some(t) => t,
        None => return true,
    };
    
    fn check_dir(dir: &PathBuf, reference: std::time::SystemTime) -> bool {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if check_dir(&path, reference) {
                        return true;
                    }
                } else if path.extension().map_or(false, |ext| ext == "java") {
                    if let Ok(metadata) = std::fs::metadata(&path) {
                        if let Ok(modified) = metadata.modified() {
                            if modified > reference {
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }
    
    check_dir(dir, reference_time)
}
