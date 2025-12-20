//! Resource command type definitions for resource packs and shaders.
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

use serde::{Deserialize, Serialize};

/// Resource pack information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePackInfo {
    pub filename: String,
    pub name: String,
    pub description: Option<String>,
    pub size: String,
    pub enabled: bool,
    /// Path to the cached pack icon (extracted from pack.png inside the archive)
    pub icon_path: Option<String>,
}

/// Shader pack information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderPackInfo {
    pub filename: String,
    pub name: String,
    pub size: String,
}

/// Search result for resource browsing
#[derive(Debug, Clone, Serialize)]
pub struct ResourceSearchResult {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub downloads: u64,
    pub follows: u32,
    pub icon_url: Option<String>,
    pub project_type: String,
    pub platform: String,
    pub categories: Vec<String>,
    pub date_created: String,
    pub date_modified: String,
}

/// Resource version response
#[derive(Debug, Clone, Serialize)]
pub struct ResourceVersionResponse {
    pub id: String,
    pub version_number: String,
    pub name: String,
    pub game_versions: Vec<String>,
    pub date_published: String,
    pub downloads: u64,
    pub files: Vec<ResourceFileResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceFileResponse {
    pub filename: String,
    pub url: String,
    pub size: u64,
    pub primary: bool,
}

/// Resource details response
#[derive(Debug, Clone, Serialize)]
pub struct ResourceDetailsResponse {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub body: String,
    pub author: String,
    pub downloads: u64,
    pub follows: u32,
    pub icon_url: Option<String>,
    pub source_url: Option<String>,
    pub issues_url: Option<String>,
    pub wiki_url: Option<String>,
    pub discord_url: Option<String>,
    pub gallery: Vec<ResourceGalleryImage>,
    pub categories: Vec<String>,
    pub versions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceGalleryImage {
    pub url: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceSearchResponse {
    pub resources: Vec<ResourceSearchResult>,
    pub total_hits: u32,
    pub offset: u32,
    pub limit: u32,
}

/// Request for batch downloading resources
#[derive(Debug, Clone, Deserialize)]
pub struct ResourceDownloadRequest {
    pub resource_id: String,
    pub version_id: String,
    pub platform: String,
}

/// Progress update for batch downloads
#[derive(Debug, Clone, Serialize)]
pub struct ResourceDownloadProgress {
    pub downloaded: u32,
    pub total: u32,
    pub current_file: String,
}
