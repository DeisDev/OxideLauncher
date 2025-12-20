//! Instance management module for Minecraft instances.
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
