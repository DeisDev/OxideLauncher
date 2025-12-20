//! Mod command type definitions.
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

/// Search result for mod browsing
#[derive(Debug, Clone, Serialize)]
pub struct ModSearchResult {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub project_type: String,
    pub platform: String,
}

/// Metadata stored for downloaded mods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModMetadata {
    pub mod_id: String,
    pub name: String,
    pub version: String,
    pub provider: String,
    pub icon_url: Option<String>,
}

/// Information about an installed mod
#[derive(Debug, Clone, Serialize)]
pub struct InstalledMod {
    pub filename: String,
    pub name: String,
    pub version: Option<String>,
    pub enabled: bool,
    pub size: u64,
    pub modified: Option<String>,
    pub provider: Option<String>,
    pub icon_url: Option<String>,
    pub homepage: Option<String>,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
}

/// Enhanced mod search result with more details
#[derive(Debug, Clone, Serialize)]
pub struct ModSearchResultDetailed {
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

#[derive(Debug, Clone, Serialize)]
pub struct ModDetailsResponse {
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
    pub donation_urls: Vec<DonationLink>,
    pub gallery: Vec<GalleryImageResponse>,
    pub categories: Vec<String>,
    pub versions: Vec<String>,
    pub loaders: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DonationLink {
    pub platform: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GalleryImageResponse {
    pub url: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModVersionResponse {
    pub id: String,
    pub version_number: String,
    pub name: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub date_published: String,
    pub downloads: u64,
    pub files: Vec<ModFileResponse>,
    pub dependencies: Vec<ModDependencyResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModFileResponse {
    pub filename: String,
    pub url: String,
    pub size: u64,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModDependencyResponse {
    pub project_id: String,
    pub dependency_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModSearchResponse {
    pub mods: Vec<ModSearchResultDetailed>,
    pub total_hits: u32,
    pub offset: u32,
    pub limit: u32,
}

/// Request for batch downloading mods
#[allow(dead_code)] // Placeholder for future batch download queue feature
#[derive(Debug, Clone, Deserialize)]
pub struct ModDownloadRequest {
    pub mod_id: String,
    pub version_id: String,
    pub platform: String,
}

/// Progress update for batch downloads
#[allow(dead_code)] // Placeholder for future download queue UI
#[derive(Debug, Clone, Serialize)]
pub struct ModDownloadProgress {
    pub downloaded: u32,
    pub total: u32,
    pub current_file: String,
}
