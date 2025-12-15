//! Post-launch command step - runs a custom command after the game exits

use async_trait::async_trait;
use std::process::{Command, Stdio};
use tracing::{debug, info, warn};

use crate::core::launch::{LaunchContext, LaunchStep, LaunchStepResult};

/// Step that runs a post-exit command
pub struct PostLaunchCommandStep {
    status: Option<String>,
    progress: f32,
}

impl PostLaunchCommandStep {
    pub fn new() -> Self {
        Self {
            status: None,
            progress: 0.0,
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
impl LaunchStep for PostLaunchCommandStep {
    fn name(&self) -> &'static str {
        "Post-Exit Command"
    }
    
    fn description(&self) -> &'static str {
        "Runs a custom command after the game exits"
    }
    
    async fn execute(&mut self, context: &mut LaunchContext) -> LaunchStepResult {
        // Use instance-specific command if set, otherwise fall back to global config
        let command = match &context.instance.settings.post_exit_command {
            Some(cmd) if !cmd.trim().is_empty() => cmd.clone(),
            _ => {
                // Check global config
                match &context.config.commands.post_exit {
                    Some(cmd) if !cmd.trim().is_empty() => cmd.clone(),
                    _ => {
                        debug!("No post-exit command configured");
                        return LaunchStepResult::Success;
                    }
                }
            }
        };
        
        // Substitute variables
        let cmd = self.substitute_variables(&command, context);
        
        self.status = Some(format!("Running: {}", cmd));
        self.progress = 0.0;
        
        info!("Running post-exit command: {}", cmd);
        
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
                    debug!("Post-exit stdout: {}", stdout);
                }
                if !output.stderr.is_empty() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    debug!("Post-exit stderr: {}", stderr);
                }
                
                if output.status.success() {
                    info!("Post-exit command completed successfully");
                    self.status = Some("Post-exit command completed".to_string());
                    self.progress = 1.0;
                    LaunchStepResult::Success
                } else {
                    let code = output.status.code().unwrap_or(-1);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    
                    warn!("Post-exit command failed with exit code {}", code);
                    
                    // Post-exit command failure is non-fatal
                    self.status = Some(format!("Post-exit command failed (exit code {})", code));
                    self.progress = 1.0;
                    
                    // Log but don't fail the launch
                    info!("Post-exit command error: {}", stderr.trim());
                    LaunchStepResult::Success
                }
            }
            Err(e) => {
                // Post-exit command failure is non-fatal
                warn!("Failed to run post-exit command: {}", e);
                self.status = Some("Post-exit command failed".to_string());
                self.progress = 1.0;
                LaunchStepResult::Success
            }
        }
    }
    
    fn progress(&self) -> f32 {
        self.progress
    }
    
    fn status(&self) -> Option<String> {
        self.status.clone()
    }
}

impl Default for PostLaunchCommandStep {
    fn default() -> Self {
        Self::new()
    }
}
