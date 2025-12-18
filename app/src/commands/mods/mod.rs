//! Mod management commands
//! 
//! This module provides commands for searching, downloading, and managing mods.
//! It's organized into submodules for better maintainability:
//! - `types`: Shared type definitions
//! - `search`: Search and discovery commands
//! - `download`: Download commands (including batch parallel downloads)
//! - `listing`: Installed mod listing and management commands

pub mod types;
pub mod search;
pub mod download;
pub mod listing;

// Re-export all commands - using wildcard to include __cmd__ symbols for tauri
pub use search::*;
pub use download::*;
pub use listing::*;
