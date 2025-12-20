//! Pre-launch command step.
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
use std::process::{Command, Stdio};
use tracing::{debug, info, warn};

use crate::core::launch::{LaunchContext, LaunchStep, LaunchStepResult};

/// Step that runs a pre-launch command
pub struct PreLaunchCommandStep {
    status: Option<String>,
    progress: f32,
    #[allow(dead_code)] // Reserved for abort functionality
    aborted: bool,
}

impl PreLaunchCommandStep {
    pub fn new() -> Self {
        Self {
            status: None,
            progress: 0.0,
            aborted: false,
        }
    }
    
    /// Substitute variables in command string
    fn substitute_variables(&self, command: &str, context: &LaunchContext) -> String {
        let instance = &context.instance;
        let game_dir = instance.game_dir();
        
        command
            .replace("$INST_NAME", &instance.name)
            .replace("$INST_ID", &instance.id)
            .replace("$INST_DIR", &instance.path.to_string_lossy())
            .replace("$INST_MC_DIR", &game_dir.to_string_lossy())
            .replace("$INST_JAVA", &context.java_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default())
            .replace("$INST_JAVA_ARGS", &instance.settings.jvm_args
                .as_ref()
                .cloned()
                .unwrap_or_default())
    }
}

#[async_trait]
impl LaunchStep for PreLaunchCommandStep {
    fn name(&self) -> &'static str {
        "Pre-Launch Command"
    }
    
    fn description(&self) -> &'static str {
        "Runs a custom command before launching the game"
    }
    
    async fn execute(&mut self, context: &mut LaunchContext) -> LaunchStepResult {
        // Use instance-specific command if set, otherwise fall back to global config
        let command = match &context.instance.settings.pre_launch_command {
            Some(cmd) if !cmd.trim().is_empty() => cmd.clone(),
            _ => {
                // Check global config
                match &context.config.commands.pre_launch {
                    Some(cmd) if !cmd.trim().is_empty() => cmd.clone(),
                    _ => {
                        debug!("No pre-launch command configured");
                        return LaunchStepResult::Success;
                    }
                }
            }
        };
        
        // Substitute variables
        let cmd = self.substitute_variables(&command, context);
        
        self.status = Some(format!("Running: {}", cmd));
        self.progress = 0.0;
        
        info!("Running pre-launch command: {}", cmd);
        
        // Parse command
        let (program, args) = if cfg!(target_os = "windows") {
            ("cmd".to_string(), vec!["/C".to_string(), cmd.clone()])
        } else {
            ("sh".to_string(), vec!["-c".to_string(), cmd.clone()])
        };
        
        // Run command
        let game_dir = context.instance.game_dir();
        
        let result = Command::new(&program)
            .args(&args)
            .current_dir(&game_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();
        
        self.progress = 0.8;
        
        match result {
            Ok(output) => {
                if !output.stdout.is_empty() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    debug!("Pre-launch stdout: {}", stdout);
                }
                if !output.stderr.is_empty() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    debug!("Pre-launch stderr: {}", stderr);
                }
                
                if output.status.success() {
                    info!("Pre-launch command completed successfully");
                    self.status = Some("Pre-launch command completed".to_string());
                    self.progress = 1.0;
                    LaunchStepResult::Success
                } else {
                    let code = output.status.code().unwrap_or(-1);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    
                    warn!("Pre-launch command failed with exit code {}", code);
                    
                    LaunchStepResult::Failed(format!(
                        "Pre-launch command failed with exit code {}.\n\
                         Command: {}\n\
                         Error: {}",
                        code, cmd, stderr.trim()
                    ))
                }
            }
            Err(e) => {
                LaunchStepResult::Failed(format!(
                    "Failed to run pre-launch command: {}\nCommand: {}",
                    e, cmd
                ))
            }
        }
    }
    
    fn can_abort(&self) -> bool {
        true
    }
    
    async fn abort(&mut self) -> bool {
        self.aborted = true;
        true
    }
    
    fn progress(&self) -> f32 {
        self.progress
    }
    
    fn status(&self) -> Option<String> {
        self.status.clone()
    }
}

impl Default for PreLaunchCommandStep {
    fn default() -> Self {
        Self::new()
    }
}
