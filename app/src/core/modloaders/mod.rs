//! Modloader API clients and installers

pub mod forge;
pub mod neoforge;
pub mod fabric;
pub mod quilt;
pub mod liteloader;
pub mod profile;
pub mod installer;
pub mod processor;

// Re-export commonly used items (only what's actually used elsewhere)
pub use forge::get_forge_versions;
pub use neoforge::get_neoforge_versions;
pub use fabric::get_fabric_versions;
pub use quilt::get_quilt_versions;
pub use liteloader::get_liteloader_versions;
pub use profile::ModloaderProfile;
pub use installer::{InstallProgress, install_modloader, get_installer};
