//! Instance management
//! 
//! Handles Minecraft instance creation, loading, saving, and management.

mod types;
mod list;
mod create;
mod setup;
mod components;
mod transfer;
mod export;
mod import;

pub use types::*;
#[allow(unused_imports)] // Will be used as features are completed
pub use list::InstanceList;
#[allow(unused_imports)]
pub use create::create_instance;
#[allow(unused_imports)]
pub use setup::{setup_instance, SetupProgress, install_modloader_for_instance};
pub use components::*;
pub use transfer::*;
pub use export::{export_instance, ExportOptions};
pub use import::{import_instance, detect_import_type, ImportOptions};
