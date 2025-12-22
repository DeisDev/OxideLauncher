//! Modloader API clients and installers.
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

pub mod forge;
pub mod neoforge;
pub mod fabric;
pub mod quilt;
pub mod liteloader;
pub mod profile;
pub mod installer;
pub mod processor;

// Re-export commonly used items
// Note: Version listing is now handled by meta server (commands/versions.rs)
// The get_*_versions functions in each module are used internally by installers
pub use profile::{ModloaderProfile, LauncherType};
pub use installer::{InstallProgress, install_modloader, get_installer};
