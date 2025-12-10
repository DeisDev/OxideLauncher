//! Instance management
//! 
//! Handles Minecraft instance creation, loading, saving, and management.

mod types;
mod list;
mod create;
mod setup;

pub use types::*;
pub use list::InstanceList;
pub use create::create_instance;
pub use setup::{setup_instance, SetupProgress};
