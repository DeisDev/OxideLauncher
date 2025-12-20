//! Launch task manager.
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

use std::process::Child;
use std::sync::{Arc, Mutex};

use crate::core::error::Result;
use super::{
    LaunchStep, LaunchStepResult, LaunchContext, MessageLevel,
    step::{LogMessage, LogSender, LogReceiver, log_channel},
};

/// Current state of the launch process
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Variants reserved for future use
pub enum LaunchState {
    /// Not started yet
    Idle,
    /// Currently running steps
    Running,
    /// Waiting for user input
    WaitingForInput,
    /// Ready to launch game
    ReadyToLaunch,
    /// Game is running
    GameRunning,
    /// Launch completed successfully
    Completed,
    /// Launch failed
    Failed(String),
    /// Launch was aborted
    Aborted,
}

/// Progress information for the launch process
#[derive(Debug, Clone)]
#[allow(dead_code)] // Part of public API for progress tracking
pub struct LaunchProgress {
    /// Current state
    pub state: LaunchState,
    /// Current step index (0-based)
    pub current_step: usize,
    /// Total number of steps
    pub total_steps: usize,
    /// Name of current step
    pub step_name: String,
    /// Progress within current step (0.0 to 1.0)
    pub step_progress: f32,
    /// Status message
    pub status: String,
}

impl Default for LaunchProgress {
    fn default() -> Self {
        Self {
            state: LaunchState::Idle,
            current_step: 0,
            total_steps: 0,
            step_name: String::new(),
            step_progress: 0.0,
            status: String::new(),
        }
    }
}

/// Manages the launch process by executing steps in sequence
pub struct LaunchTask {
    /// Launch steps to execute
    steps: Vec<Box<dyn LaunchStep>>,
    
    /// Launch context
    context: LaunchContext,
    
    /// Current step index
    current_step: usize,
    
    /// Current state
    state: LaunchState,
    
    /// Log sender
    log_sender: LogSender,
    
    /// Log receiver (moved out when task starts)
    log_receiver: Option<LogReceiver>,
    
    /// Whether abort was requested
    abort_requested: bool,
    
    /// Game process (if launched)
    game_process: Option<Arc<Mutex<Child>>>,
}

impl LaunchTask {
    /// Create a new launch task
    pub fn new(context: LaunchContext) -> Self {
        let (log_sender, log_receiver) = log_channel();
        
        Self {
            steps: Vec::new(),
            context,
            current_step: 0,
            state: LaunchState::Idle,
            log_sender,
            log_receiver: Some(log_receiver),
            abort_requested: false,
            game_process: None,
        }
    }
    
    /// Add a step to the end of the launch process
    pub fn append_step(&mut self, step: Box<dyn LaunchStep>) {
        self.steps.push(step);
    }
    
    /// Add a step to the beginning of the launch process
    #[allow(dead_code)] // Part of public API
    pub fn prepend_step(&mut self, step: Box<dyn LaunchStep>) {
        self.steps.insert(0, step);
    }
    
    /// Get the log receiver (can only be called once)
    pub fn take_log_receiver(&mut self) -> Option<LogReceiver> {
        self.log_receiver.take()
    }
    
    /// Get current progress
    #[allow(dead_code)] // Part of public API for progress tracking
    pub fn progress(&self) -> LaunchProgress {
        let step_name = if self.current_step < self.steps.len() {
            self.steps[self.current_step].name().to_string()
        } else {
            String::new()
        };
        
        let step_progress = if self.current_step < self.steps.len() {
            self.steps[self.current_step].progress()
        } else {
            0.0
        };
        
        let status = if self.current_step < self.steps.len() {
            self.steps[self.current_step].status().unwrap_or_default()
        } else {
            String::new()
        };
        
        LaunchProgress {
            state: self.state.clone(),
            current_step: self.current_step,
            total_steps: self.steps.len(),
            step_name,
            step_progress,
            status,
        }
    }
    
    /// Log a message
    pub fn log(&self, level: MessageLevel, message: impl Into<String>) {
        let _ = self.log_sender.send(LogMessage::new(
            level,
            message,
            "LaunchTask",
        ));
    }
    
    /// Log a line with the launcher source
    #[allow(dead_code)] // Part of public API for logging
    pub fn log_line(&self, message: impl Into<String>, level: MessageLevel) {
        self.log(level, message);
    }
    
    /// Execute the launch process
    pub async fn execute(&mut self) -> Result<()> {
        self.state = LaunchState::Running;
        self.current_step = 0;
        
        let total_steps = self.steps.len();
        self.log(MessageLevel::Launcher, format!(
            "Starting launch process with {} steps",
            total_steps
        ));
        
        while self.current_step < total_steps {
            if self.abort_requested {
                self.state = LaunchState::Aborted;
                self.log(MessageLevel::Launcher, "Launch aborted by user");
                self.finalize_steps(false).await;
                return Ok(());
            }
            
            // Get step info before mutable borrow
            let step_name = self.steps[self.current_step].name();
            let current = self.current_step;
            
            self.log(MessageLevel::Launcher, format!(
                "Executing step {}/{}: {}",
                current + 1,
                total_steps,
                step_name
            ));
            
            // Execute step
            let result = {
                let step = &mut self.steps[current];
                step.execute(&mut self.context).await
            };
            
            match result {
                LaunchStepResult::Success => {
                    self.log(MessageLevel::Launcher, format!(
                        "Step '{}' completed successfully",
                        step_name
                    ));
                    self.current_step += 1;
                }
                LaunchStepResult::Failed(error) => {
                    self.log(MessageLevel::Error, format!(
                        "Step '{}' failed: {}",
                        step_name, error
                    ));
                    self.state = LaunchState::Failed(error.clone());
                    self.finalize_steps(false).await;
                    return Err(crate::core::error::OxideError::Launch(error));
                }
                LaunchStepResult::Aborted => {
                    self.log(MessageLevel::Launcher, format!(
                        "Step '{}' was aborted",
                        step_name
                    ));
                    self.state = LaunchState::Aborted;
                    self.finalize_steps(false).await;
                    return Ok(());
                }
                LaunchStepResult::WaitingForInput(msg) => {
                    self.log(MessageLevel::Launcher, format!(
                        "Step '{}' waiting for input: {}",
                        step_name, msg
                    ));
                    self.state = LaunchState::WaitingForInput;
                    // The step should handle resuming
                }
            }
        }
        
        // Try to capture the game process from any step that launched it
        for step in &self.steps {
            if let Some(process) = step.get_game_process() {
                self.game_process = Some(process);
                break;
            }
        }
        
        self.state = LaunchState::Completed;
        self.log(MessageLevel::Launcher, "Launch process completed successfully");
        self.finalize_steps(true).await;
        
        Ok(())
    }
    
    /// Request abort of the launch process
    #[allow(dead_code)] // Part of public API for abort handling
    pub async fn abort(&mut self) -> bool {
        self.abort_requested = true;
        self.context.aborted = true;
        
        let current = self.current_step;
        let total = self.steps.len();
        
        if current < total {
            let step = &mut self.steps[current];
            if step.can_abort() {
                return step.abort().await;
            }
        }
        
        true
    }
    
    /// Finalize all steps
    async fn finalize_steps(&mut self, successful: bool) {
        self.log(MessageLevel::Launcher, format!(
            "Finalizing steps (successful: {})",
            successful
        ));
        
        for step in &mut self.steps {
            step.finalize(&mut self.context).await;
        }
    }
    
    /// Proceed after waiting for input
    #[allow(dead_code)] // Part of public API for interactive steps
    pub async fn proceed(&mut self) {
        if self.state == LaunchState::WaitingForInput {
            self.state = LaunchState::Running;
            self.current_step += 1;
        }
    }
    
    /// Get the instance being launched
    #[allow(dead_code)] // Part of public API
    pub fn instance(&self) -> &crate::core::instance::Instance {
        &self.context.instance
    }
    
    /// Get mutable access to the context
    #[allow(dead_code)] // Part of public API
    pub fn context_mut(&mut self) -> &mut LaunchContext {
        &mut self.context
    }
    
    /// Get the game process (if launched)
    pub fn take_game_process(&mut self) -> Option<Arc<Mutex<Child>>> {
        self.game_process.take()
    }
    
    /// Substitute variables in a command string
    #[allow(dead_code)] // Part of public API for command variable substitution
    pub fn substitute_variables(&self, command: &str) -> String {
        let instance = &self.context.instance;
        let game_dir = instance.game_dir();
        
        command
            .replace("$INST_NAME", &instance.name)
            .replace("$INST_ID", &instance.id)
            .replace("$INST_DIR", &instance.path.to_string_lossy())
            .replace("$INST_MC_DIR", &game_dir.to_string_lossy())
            .replace("$INST_JAVA", &self.context.java_path
                .as_ref()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default())
            .replace("$INST_JAVA_ARGS", &instance.settings.jvm_args
                .as_ref()
                .cloned()
                .unwrap_or_default())
    }
}
