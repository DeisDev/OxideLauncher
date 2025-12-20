//! RustWiz metadata system module for packwiz-compatible mod tracking.
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

pub mod types;
pub mod parser;
pub mod update_check;
pub mod export;

// Re-export commonly used types
pub use types::{
    ModToml, ModTomlExtended, OxideMetadata,
    HashFormat, Side,
    BatchUpdateResult,
};

pub use parser::{
    read_pack_toml, write_pack_toml,
    write_index_toml,
    write_mod_toml, delete_mod_toml,
    rebuild_index,
    mod_toml_filename,
    compute_file_hash,
    index_dir,
};

// Renamed functions for RustWiz branding
pub use parser::initialize_packwiz as initialize_pack;
pub use parser::has_packwiz as has_pack;

#[allow(unused_imports)] // check_instance_updates kept for backwards compatibility
pub use update_check::{check_instance_updates, check_instance_updates_with_info};

pub use export::{
    ExportOptions,
    export_modrinth, export_curseforge, export_packwiz,
};
