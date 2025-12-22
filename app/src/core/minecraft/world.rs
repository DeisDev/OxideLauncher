//! Minecraft world/save parsing and management.
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

use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use crate::core::error::{OxideError, Result};
use crate::core::files;

/// Represents a Minecraft game type/mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GameType {
    Survival,
    Creative,
    Adventure,
    Spectator,
    Unknown,
}

impl From<i32> for GameType {
    fn from(value: i32) -> Self {
        match value {
            0 => GameType::Survival,
            1 => GameType::Creative,
            2 => GameType::Adventure,
            3 => GameType::Spectator,
            _ => GameType::Unknown,
        }
    }
}

impl std::fmt::Display for GameType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameType::Survival => write!(f, "Survival"),
            GameType::Creative => write!(f, "Creative"),
            GameType::Adventure => write!(f, "Adventure"),
            GameType::Spectator => write!(f, "Spectator"),
            GameType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Represents a Minecraft world/save
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    /// World folder name (directory name)
    pub folder_name: String,
    /// Display name from level.dat
    pub name: String,
    /// Full path to the world folder
    pub path: PathBuf,
    /// World seed (if available)
    pub seed: Option<i64>,
    /// Game type/mode
    pub game_type: GameType,
    /// Whether hardcore mode is enabled
    pub hardcore: bool,
    /// Last played timestamp (Unix epoch milliseconds)
    pub last_played: Option<i64>,
    /// Size of the world folder in bytes
    pub size: u64,
    /// Whether the world has an icon
    pub has_icon: bool,
    /// Icon path if available
    pub icon_path: Option<PathBuf>,
}

impl World {
    /// Load world information from a world directory
    pub fn from_path(path: &Path) -> Option<Self> {
        if !path.is_dir() {
            return None;
        }
        
        let folder_name = path.file_name()?.to_string_lossy().to_string();
        let level_dat = path.join("level.dat");
        
        if !level_dat.exists() {
            debug!("No level.dat found in {:?}", path);
            return None;
        }
        
        // Calculate folder size
        let size = calculate_dir_size(path).unwrap_or(0);
        
        // Check for icon
        let icon_path = path.join("icon.png");
        let has_icon = icon_path.exists();
        
        // Try to parse level.dat for metadata
        let (name, seed, game_type, hardcore, last_played) = match parse_level_dat(&level_dat) {
            Ok(data) => data,
            Err(e) => {
                debug!("Failed to parse level.dat for {:?}: {}", path, e);
                (folder_name.clone(), None, GameType::Unknown, false, None)
            }
        };
        
        Some(World {
            folder_name,
            name,
            path: path.to_path_buf(),
            seed,
            game_type,
            hardcore,
            last_played,
            size,
            has_icon,
            icon_path: if has_icon { Some(icon_path) } else { None },
        })
    }
    
    /// Get the formatted size string
    pub fn formatted_size(&self) -> String {
        format_size(self.size)
    }
    
    /// Get the formatted last played time
    pub fn formatted_last_played(&self) -> Option<String> {
        self.last_played.map(|ts| {
            let datetime = chrono::DateTime::from_timestamp_millis(ts);
            datetime
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".to_string())
        })
    }
}

/// List all worlds in a saves directory
pub fn list_worlds(saves_dir: &Path) -> Vec<World> {
    if !saves_dir.exists() || !saves_dir.is_dir() {
        return Vec::new();
    }
    
    let mut worlds = Vec::new();
    
    match fs::read_dir(saves_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(world) = World::from_path(&path) {
                        worlds.push(world);
                    }
                }
            }
        }
        Err(e) => {
            warn!("Failed to read saves directory {:?}: {}", saves_dir, e);
        }
    }
    
    // Sort by last played (most recent first)
    worlds.sort_by(|a, b| {
        b.last_played.unwrap_or(0).cmp(&a.last_played.unwrap_or(0))
    });
    
    worlds
}

/// Delete a world by folder name
pub fn delete_world(saves_dir: &Path, folder_name: &str, use_recycle_bin: bool) -> Result<()> {
    let world_path = saves_dir.join(folder_name);
    
    if !world_path.exists() {
        return Err(OxideError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("World '{}' not found", folder_name),
        )));
    }
    
    if !world_path.is_dir() {
        return Err(OxideError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("'{}' is not a directory", folder_name),
        )));
    }
    
    // Verify it's a valid world (has level.dat)
    if !world_path.join("level.dat").exists() {
        return Err(OxideError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("'{}' does not appear to be a valid world", folder_name),
        )));
    }
    
    if use_recycle_bin {
        info!("Moving world to recycle bin: {:?}", world_path);
    } else {
        info!("Permanently deleting world: {:?}", world_path);
    }
    
    files::delete_directory(&world_path, use_recycle_bin)?;
    
    Ok(())
}

/// Export a world to a ZIP file
pub fn export_world(saves_dir: &Path, folder_name: &str, output_path: &Path) -> Result<()> {
    let world_path = saves_dir.join(folder_name);
    
    if !world_path.exists() {
        return Err(OxideError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("World '{}' not found", folder_name),
        )));
    }
    
    info!("Exporting world {:?} to {:?}", world_path, output_path);
    
    let file = fs::File::create(output_path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    
    // Add all files from the world directory
    add_directory_to_zip(&mut zip, &world_path, folder_name, options)?;
    
    zip.finish()?;
    info!("World exported successfully");
    
    Ok(())
}

/// Copy/duplicate a world
pub fn copy_world(saves_dir: &Path, folder_name: &str, new_name: &str) -> Result<()> {
    let source_path = saves_dir.join(folder_name);
    let dest_path = saves_dir.join(new_name);
    
    if !source_path.exists() {
        return Err(OxideError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("World '{}' not found", folder_name),
        )));
    }
    
    if dest_path.exists() {
        return Err(OxideError::Io(std::io::Error::new(
            std::io::ErrorKind::AlreadyExists,
            format!("World '{}' already exists", new_name),
        )));
    }
    
    info!("Copying world {:?} to {:?}", source_path, dest_path);
    copy_dir_recursive(&source_path, &dest_path)?;
    
    Ok(())
}

/// Get the icon data for a world (as base64)
pub fn get_world_icon(saves_dir: &Path, folder_name: &str) -> Option<String> {
    let icon_path = saves_dir.join(folder_name).join("icon.png");
    
    if !icon_path.exists() {
        return None;
    }
    
    match fs::read(&icon_path) {
        Ok(data) => {
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            Some(STANDARD.encode(&data))
        }
        Err(e) => {
            warn!("Failed to read world icon: {}", e);
            None
        }
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Parse level.dat NBT file for world metadata
/// Returns (name, seed, game_type, hardcore, last_played)
fn parse_level_dat(path: &Path) -> Result<(String, Option<i64>, GameType, bool, Option<i64>)> {
    let mut file = fs::File::open(path)?;
    let mut compressed_data = Vec::new();
    file.read_to_end(&mut compressed_data)?;
    
    // level.dat is gzip compressed NBT
    let mut decoder = flate2::read::GzDecoder::new(&compressed_data[..]);
    let mut nbt_data = Vec::new();
    decoder.read_to_end(&mut nbt_data)?;
    
    // Parse NBT - we'll use a simple approach since we just need a few values
    // The structure is: Compound "Data" containing the world info
    let (name, seed, game_type, hardcore, last_played) = parse_nbt_world_data(&nbt_data);
    
    Ok((name, seed, game_type, hardcore, last_played))
}

/// Simple NBT parser to extract world data
/// This is a simplified parser that looks for specific tags
fn parse_nbt_world_data(data: &[u8]) -> (String, Option<i64>, GameType, bool, Option<i64>) {
    let mut name = String::new();
    let mut seed = None;
    let mut game_type = GameType::Unknown;
    let mut hardcore = false;
    let mut last_played = None;
    
    // Simple string search for common patterns in NBT data
    // This is a hack but works for basic metadata extraction
    
    // Look for LevelName
    if let Some(level_name) = find_nbt_string(data, b"LevelName") {
        name = level_name;
    }
    
    // Look for RandomSeed (8-byte long)
    if let Some(pos) = find_tag_position(data, b"RandomSeed") {
        if pos + 8 <= data.len() {
            let bytes: [u8; 8] = data[pos..pos+8].try_into().unwrap_or([0; 8]);
            seed = Some(i64::from_be_bytes(bytes));
        }
    }
    
    // Look for GameType (4-byte int)
    if let Some(pos) = find_tag_position(data, b"GameType") {
        if pos + 4 <= data.len() {
            let bytes: [u8; 4] = data[pos..pos+4].try_into().unwrap_or([0; 4]);
            game_type = GameType::from(i32::from_be_bytes(bytes));
        }
    }
    
    // Look for hardcore (1-byte)
    if let Some(pos) = find_tag_position(data, b"hardcore") {
        if pos < data.len() {
            hardcore = data[pos] != 0;
        }
    }
    
    // Look for LastPlayed (8-byte long)
    if let Some(pos) = find_tag_position(data, b"LastPlayed") {
        if pos + 8 <= data.len() {
            let bytes: [u8; 8] = data[pos..pos+8].try_into().unwrap_or([0; 8]);
            last_played = Some(i64::from_be_bytes(bytes));
        }
    }
    
    if name.is_empty() {
        name = "Unknown World".to_string();
    }
    
    (name, seed, game_type, hardcore, last_played)
}

/// Find an NBT string value by tag name
fn find_nbt_string(data: &[u8], tag_name: &[u8]) -> Option<String> {
    let pos = data.windows(tag_name.len())
        .position(|window| window == tag_name)?;
    
    // After the tag name, there should be the string data
    // NBT strings are length-prefixed (2 bytes big-endian)
    let string_start = pos + tag_name.len();
    if string_start + 2 > data.len() {
        return None;
    }
    
    let length = u16::from_be_bytes([data[string_start], data[string_start + 1]]) as usize;
    let string_data_start = string_start + 2;
    
    if string_data_start + length > data.len() {
        return None;
    }
    
    String::from_utf8(data[string_data_start..string_data_start + length].to_vec()).ok()
}

/// Find the position after a tag name where the value starts
fn find_tag_position(data: &[u8], tag_name: &[u8]) -> Option<usize> {
    let pos = data.windows(tag_name.len())
        .position(|window| window == tag_name)?;
    
    // Return position after tag name
    Some(pos + tag_name.len())
}

/// Calculate the total size of a directory
fn calculate_dir_size(path: &Path) -> Result<u64> {
    let mut total = 0u64;
    
    for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            total += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    
    Ok(total)
}

/// Format size in human-readable form
fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let path = entry.path();
        let dest_path = dst.join(entry.file_name());
        
        if path.is_dir() {
            copy_dir_recursive(&path, &dest_path)?;
        } else {
            fs::copy(&path, &dest_path)?;
        }
    }
    
    Ok(())
}

/// Add a directory to a ZIP file recursively
fn add_directory_to_zip<W: std::io::Write + std::io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    dir_path: &Path,
    _prefix: &str, // Reserved for future use with custom prefixes
    options: zip::write::SimpleFileOptions,
) -> Result<()> {
    for entry in walkdir::WalkDir::new(dir_path) {
        let entry = entry.map_err(|e| OxideError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            e.to_string(),
        )))?;
        
        let path = entry.path();
        let relative_path = path.strip_prefix(dir_path.parent().unwrap_or(dir_path))
            .unwrap_or(path);
        let name = relative_path.to_string_lossy();
        
        if path.is_file() {
            zip.start_file(name.to_string(), options)?;
            let mut file = fs::File::open(path)?;
            std::io::copy(&mut file, zip)?;
        } else if path.is_dir() && path != dir_path {
            zip.add_directory(format!("{}/", name), options)?;
        }
    }
    
    Ok(())
}
