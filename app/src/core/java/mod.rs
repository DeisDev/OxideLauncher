//! Java detection, management, and downloading.
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

pub mod version;
pub mod metadata;
pub mod install;
pub mod detection;
pub mod checker;
pub mod download;

// Public API re-exports - may not all be used internally but are part of the public module interface
#[allow(unused_imports)]
pub use version::JavaVersion;
#[allow(unused_imports)]
pub use metadata::{JavaMetadata, DownloadType};
#[allow(unused_imports)]
pub use install::{JavaInstallation, JavaArch};
#[allow(unused_imports)] // Functions used through commands module
pub use detection::{detect_java_installations, find_java_for_version, get_required_java_version};
#[allow(unused_imports)]
pub use checker::{JavaChecker, JavaCheckResult};
#[allow(unused_imports)]
pub use download::{download_java, fetch_adoptium_versions, AvailableJavaVersion, JavaDownloadProgress};
