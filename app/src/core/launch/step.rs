//! Launch step trait and types.
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
use std::process::Child;
use std::sync::{Arc, Mutex};
#[allow(unused_imports)] // May be used by implementations
use crate::core::error::Result;
use super::{LaunchContext, MessageLevel};

/// Result of a launch step execution
#[derive(Debug, Clone)]
#[allow(dead_code)] // Variants reserved for future use
pub enum LaunchStepResult {
    /// Step completed successfully
    Success,
    /// Step failed with an error
    Failed(String),
    /// Step was aborted
    Aborted,
    /// Step requires user interaction before proceeding
    WaitingForInput(String),
}

/// A step in the launch process
#[async_trait]
#[allow(dead_code)] // Trait methods part of public API
pub trait LaunchStep: Send + Sync {
    /// Get the name of this step for display purposes
    fn name(&self) -> &'static str;
    
    /// Get a description of what this step does
    fn description(&self) -> &'static str;
    
    /// Execute the step
    async fn execute(&mut self, context: &mut LaunchContext) -> LaunchStepResult;
    
    /// Called after the launch process completes (success or failure)
    /// Used for cleanup
    async fn finalize(&mut self, _context: &mut LaunchContext) {
        // Default: no cleanup needed
    }
    
    /// Check if this step can be aborted
    fn can_abort(&self) -> bool {
        true
    }
    
    /// Abort this step if currently running
    async fn abort(&mut self) -> bool {
        false // Default: cannot abort
    }
    
    /// Get current progress (0.0 to 1.0)
    fn progress(&self) -> f32 {
        0.0
    }
    
    /// Get status message
    fn status(&self) -> Option<String> {
        None
    }
    
    /// Get the launched game process (if this step launches a game)
    fn get_game_process(&self) -> Option<Arc<Mutex<Child>>> {
        None
    }
}

/// A log message from a launch step
#[derive(Debug, Clone)]
#[allow(dead_code)] // Fields used for log output
pub struct LogMessage {
    pub level: MessageLevel,
    pub message: String,
    pub source: String,
}

impl LogMessage {
    pub fn new(level: MessageLevel, message: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            source: source.into(),
        }
    }
}

/// Channel for sending log messages from launch steps
pub type LogSender = tokio::sync::mpsc::UnboundedSender<LogMessage>;
pub type LogReceiver = tokio::sync::mpsc::UnboundedReceiver<LogMessage>;

/// Create a new log channel
pub fn log_channel() -> (LogSender, LogReceiver) {
    tokio::sync::mpsc::unbounded_channel()
}
