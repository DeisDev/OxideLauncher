//! Tauri command handlers organized by domain
//!
//! Each submodule contains related commands for a specific domain.

mod state;
pub mod instances;
pub mod accounts;
pub mod config;
pub mod versions;
pub mod mods;
pub mod java;
pub mod worlds;
pub mod resources;
pub mod screenshots;
pub mod shortcuts;

// Re-export state types for use in main.rs
pub use state::AppState;

// Helper utilities shared across modules
pub(crate) mod utils {
    /// Format file size in human-readable format
    pub fn format_file_size(bytes: u64) -> String {
        const KB: u64 = 1024;
        const MB: u64 = KB * 1024;
        const GB: u64 = MB * 1024;
        
        if bytes >= GB {
            format!("{:.2} GB", bytes as f64 / GB as f64)
        } else if bytes >= MB {
            format!("{:.2} MB", bytes as f64 / MB as f64)
        } else if bytes >= KB {
            format!("{:.2} KB", bytes as f64 / KB as f64)
        } else {
            format!("{} B", bytes)
        }
    }
}
