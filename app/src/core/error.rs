//! Error types and handling for the launcher.
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

use thiserror::Error;

/// Main error type for the launcher
#[derive(Error, Debug)]
#[allow(dead_code)] // Error variants will be used as features are completed
pub enum OxideError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Download error: {0}")]
    Download(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Instance error: {0}")]
    Instance(String),

    #[error("Launch error: {0}")]
    Launch(String),

    #[error("Java error: {0}")]
    Java(String),

    #[error("Version error: {0}")]
    Version(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Mod platform error: {0}")]
    ModPlatform(String),

    #[error("Modloader error: {0}")]
    Modloader(String),

    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("{0}")]
    Other(String),
}

impl From<String> for OxideError {
    fn from(s: String) -> Self {
        OxideError::Other(s)
    }
}

impl From<&str> for OxideError {
    fn from(s: &str) -> Self {
        OxideError::Other(s.to_string())
    }
}

pub type Result<T> = std::result::Result<T, OxideError>;
