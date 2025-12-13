//! Step-based game launch system
//! 
//! This module implements a launch system inspired by Prism Launcher's architecture.
//! The launch process is divided into discrete steps that execute sequentially,
//! with each step handling a specific part of the launch process.

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

/// Context passed to launch steps containing all necessary information
#[derive(Clone)]
pub struct LaunchContext {
    /// The instance being launched
    pub instance: Instance,
    
    /// The authentication session
    pub auth_session: AuthSession,
    
    /// Global configuration
    pub config: Config,
    
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
    pub fn new(instance: Instance, auth_session: AuthSession, config: Config) -> Self {
        let game_dir = instance.game_dir();
        let natives_dir = game_dir.join("natives");
        let libraries_dir = config.libraries_dir();
        let assets_dir = config.assets_dir();
        
        Self {
            instance,
            auth_session,
            config,
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
