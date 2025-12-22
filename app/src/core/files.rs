//! File system utilities with recycle bin support.
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

use std::path::Path;
use crate::core::error::{OxideError, Result};
use tracing::{debug, warn};

/// Delete a file, optionally moving to recycle bin.
/// 
/// # Arguments
/// * `path` - Path to the file to delete
/// * `use_recycle_bin` - If true, move to recycle bin; if false, permanently delete
pub fn delete_file<P: AsRef<Path>>(path: P, use_recycle_bin: bool) -> Result<()> {
    let path = path.as_ref();
    
    if !path.exists() {
        return Ok(());
    }
    
    if !path.is_file() {
        return Err(OxideError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Path is not a file: {:?}", path),
        )));
    }
    
    if use_recycle_bin {
        debug!("Moving file to recycle bin: {:?}", path);
        trash::delete(path).map_err(|e| {
            warn!("Failed to move to recycle bin, falling back to permanent delete: {}", e);
            OxideError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to move to recycle bin: {}", e),
            ))
        })?;
    } else {
        debug!("Permanently deleting file: {:?}", path);
        std::fs::remove_file(path)?;
    }
    
    Ok(())
}

/// Delete a directory and all its contents, optionally moving to recycle bin.
/// 
/// # Arguments
/// * `path` - Path to the directory to delete
/// * `use_recycle_bin` - If true, move to recycle bin; if false, permanently delete
pub fn delete_directory<P: AsRef<Path>>(path: P, use_recycle_bin: bool) -> Result<()> {
    let path = path.as_ref();
    
    if !path.exists() {
        return Ok(());
    }
    
    if !path.is_dir() {
        return Err(OxideError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Path is not a directory: {:?}", path),
        )));
    }
    
    if use_recycle_bin {
        debug!("Moving directory to recycle bin: {:?}", path);
        trash::delete(path).map_err(|e| {
            warn!("Failed to move to recycle bin, falling back to permanent delete: {}", e);
            OxideError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to move to recycle bin: {}", e),
            ))
        })?;
    } else {
        debug!("Permanently deleting directory: {:?}", path);
        std::fs::remove_dir_all(path)?;
    }
    
    Ok(())
}

/// Delete a path (file or directory), optionally moving to recycle bin.
/// 
/// # Arguments
/// * `path` - Path to delete
/// * `use_recycle_bin` - If true, move to recycle bin; if false, permanently delete
pub fn delete_path<P: AsRef<Path>>(path: P, use_recycle_bin: bool) -> Result<()> {
    let path = path.as_ref();
    
    if !path.exists() {
        return Ok(());
    }
    
    if path.is_dir() {
        delete_directory(path, use_recycle_bin)
    } else {
        delete_file(path, use_recycle_bin)
    }
}
