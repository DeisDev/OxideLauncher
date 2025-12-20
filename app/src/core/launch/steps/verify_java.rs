//! Verify Java installation step.
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
use tracing::{info, warn};

use crate::core::launch::{LaunchContext, LaunchStep, LaunchStepResult};
use crate::core::java;

/// Step that verifies Java is compatible with the Minecraft version
pub struct VerifyJavaStep {
    status: Option<String>,
    progress: f32,
}

impl VerifyJavaStep {
    pub fn new() -> Self {
        Self {
            status: None,
            progress: 0.0,
        }
    }
    
    /// Parse major version from Java version string
    fn parse_major_version(version: &str) -> Option<u32> {
        // Handle both old format (1.8.0_XXX) and new format (11.0.X, 17.0.X)
        let version = version.trim();
        
        if version.starts_with("1.") {
            // Old format: 1.8.0_XXX -> major is 8
            version.split('.').nth(1)?.parse().ok()
        } else {
            // New format: 17.0.1 -> major is 17
            version.split('.').next()?.parse().ok()
        }
    }
    
    /// Get compatible Java major versions for a Minecraft version
    fn get_compatible_java_majors(minecraft_version: &str) -> Vec<u32> {
        let required = java::get_required_java_version(minecraft_version);
        
        // Return a list of compatible versions
        // Generally newer Java versions are forward compatible
        match required {
            8 => vec![8], // Minecraft < 1.17 strictly needs Java 8
            16 => vec![16, 17], // 1.17 snapshots
            17 => vec![17, 18, 19, 20, 21], // 1.17-1.20.4
            21 => vec![21, 22, 23, 24, 25], // 1.20.5+
            _ => vec![required],
        }
    }
}

#[async_trait]
impl LaunchStep for VerifyJavaStep {
    fn name(&self) -> &'static str {
        "Verify Java"
    }
    
    fn description(&self) -> &'static str {
        "Verifies Java version compatibility with Minecraft"
    }
    
    async fn execute(&mut self, context: &mut LaunchContext) -> LaunchStepResult {
        self.status = Some("Verifying Java compatibility...".to_string());
        self.progress = 0.0;
        
        let instance = &context.instance;
        
        // Get stored Java version from CheckJava step
        let java_version = match &context.java_version {
            Some(v) => v.clone(),
            None => {
                return LaunchStepResult::Failed(
                    "Java version not determined. CheckJava step may have failed.".to_string()
                );
            }
        };
        
        let java_architecture = context.java_architecture.as_deref().unwrap_or("64");
        
        // Check if user wants to ignore compatibility
        if instance.settings.skip_java_compatibility_check {
            warn!(
                "Java compatibility check skipped by user for instance '{}'",
                instance.name
            );
            self.status = Some("Java compatibility check skipped".to_string());
            self.progress = 1.0;
            return LaunchStepResult::Success;
        }
        
        // Check architecture and memory
        self.progress = 0.3;
        let max_memory = instance.settings.max_memory.unwrap_or(context.config.memory.max_memory);
        
        if java_architecture == "32" && max_memory > 2048 {
            warn!(
                "32-bit Java detected with {}MB max memory allocation. \
                 32-bit Java cannot use more than ~2048MB.",
                max_memory
            );
            // Don't fail, just warn - the game might still run with reduced memory
        }
        
        // Parse Java major version
        self.progress = 0.5;
        let java_major = match Self::parse_major_version(&java_version) {
            Some(v) => v,
            None => {
                warn!("Could not parse Java version: {}", java_version);
                // Don't fail if we can't parse - let the game try to run
                self.status = Some("Could not verify Java version".to_string());
                self.progress = 1.0;
                return LaunchStepResult::Success;
            }
        };
        
        // Get compatible versions for this Minecraft version
        let compatible_majors = Self::get_compatible_java_majors(&instance.minecraft_version);
        
        self.progress = 0.8;
        
        if compatible_majors.contains(&java_major) {
            info!(
                "Java {} is compatible with Minecraft {}",
                java_major, instance.minecraft_version
            );
            self.status = Some(format!(
                "Java {} is compatible with Minecraft {}",
                java_major, instance.minecraft_version
            ));
            self.progress = 1.0;
            return LaunchStepResult::Success;
        }
        
        // Check if this is globally ignored
        if context.config.java.skip_compatibility_check {
            warn!(
                "Java {} may not be compatible with Minecraft {}, but global check is disabled",
                java_major, instance.minecraft_version
            );
            self.status = Some("Java compatibility check disabled globally".to_string());
            self.progress = 1.0;
            return LaunchStepResult::Success;
        }
        
        // Java version is not compatible
        let required = java::get_required_java_version(&instance.minecraft_version);
        
        LaunchStepResult::Failed(format!(
            "Java {} is not compatible with Minecraft {}.\n\n\
             This version of Minecraft requires Java {}.\n\n\
             You can:\n\
             - Download a compatible Java version from the Java settings\n\
             - Enable 'Skip Java compatibility check' in instance settings\n\
             - Enable auto-install Java to automatically download the correct version",
            java_major, instance.minecraft_version, required
        ))
    }
    
    fn progress(&self) -> f32 {
        self.progress
    }
    
    fn status(&self) -> Option<String> {
        self.status.clone()
    }
}

impl Default for VerifyJavaStep {
    fn default() -> Self {
        Self::new()
    }
}
