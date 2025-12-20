//! Step-based game launch system.
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

mod step;
mod task;
pub mod steps;

pub use step::{LaunchStep, LaunchStepResult};
#[allow(unused_imports)] // Part of public API
pub use task::{LaunchTask, LaunchProgress, LaunchState};
#[allow(unused_imports)] // Re-exports for convenience
pub use steps::*;

#[allow(unused_imports)] // May be used by steps
use crate::core::error::Result;
use crate::core::instance::Instance;
use crate::core::accounts::AuthSession;
use crate::core::config::Config;
use crate::core::minecraft::version::LaunchFeatures;

/// Context passed to launch steps containing all necessary information
#[derive(Clone)]
pub struct LaunchContext {
    /// The instance being launched
    pub instance: Instance,
    
    /// The authentication session
    pub auth_session: AuthSession,
    
    /// Global configuration
    pub config: Config,
    
    /// Launch features for conditional arguments
    pub features: LaunchFeatures,
    
    /// Path to the Java executable (set by CheckJava step)
    pub java_path: Option<std::path::PathBuf>,
    
    /// Java version string (set by CheckJava step)
    pub java_version: Option<String>,
    
    /// Java architecture (set by CheckJava step)
    pub java_architecture: Option<String>,
    
    /// Path to natives directory
    pub natives_dir: std::path::PathBuf,
    
    /// Path to libraries directory
    pub libraries_dir: std::path::PathBuf,
    
    /// Path to assets directory
    pub assets_dir: std::path::PathBuf,
    
    /// Whether launch was aborted
    #[allow(dead_code)] // Used by abort functionality
    pub aborted: bool,
}

impl LaunchContext {
    /// Create a new launch context
    #[allow(dead_code)]
    pub fn new(instance: Instance, auth_session: AuthSession, config: Config) -> Self {
        Self::with_features(instance, auth_session, config, LaunchFeatures::normal())
    }
    
    /// Create a new launch context with specific features
    pub fn with_features(instance: Instance, auth_session: AuthSession, config: Config, features: LaunchFeatures) -> Self {
        let game_dir = instance.game_dir();
        let natives_dir = game_dir.join("natives");
        let libraries_dir = config.libraries_dir();
        let assets_dir = config.assets_dir();
        
        Self {
            instance,
            auth_session,
            config,
            features,
            java_path: None,
            java_version: None,
            java_architecture: None,
            natives_dir,
            libraries_dir,
            assets_dir,
            aborted: false,
        }
    }
}

/// Message levels for log output
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Variants used for log categorization
pub enum MessageLevel {
    /// Launcher status messages
    Launcher,
    /// Debug information
    Debug,
    /// Informational messages
    Info,
    /// Warning messages
    Warning,
    /// Error messages
    Error,
    /// Fatal errors
    Fatal,
    /// Game output
    Game,
}

impl std::fmt::Display for MessageLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageLevel::Launcher => write!(f, "LAUNCHER"),
            MessageLevel::Debug => write!(f, "DEBUG"),
            MessageLevel::Info => write!(f, "INFO"),
            MessageLevel::Warning => write!(f, "WARN"),
            MessageLevel::Error => write!(f, "ERROR"),
            MessageLevel::Fatal => write!(f, "FATAL"),
            MessageLevel::Game => write!(f, "GAME"),
        }
    }
}
