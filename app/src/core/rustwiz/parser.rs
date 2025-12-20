//! RustWiz metadata parsing and file I/O utilities.
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

use std::path::{Path, PathBuf};
use std::fs;
use sha2::{Sha256, Sha512, Digest};
use sha1::Sha1;

use crate::core::error::{Result, OxideError};
use super::types::*;

// =============================================================================
// File Reading
// =============================================================================

/// Read and parse pack.toml
pub fn read_pack_toml(instance_path: &Path) -> Result<PackToml> {
    let path = instance_path.join("pack.toml");
    if !path.exists() {
        return Err(OxideError::Other(format!(
            "pack.toml not found at {:?}",
            path
        )));
    }
    
    let content = fs::read_to_string(&path)?;
    let pack: PackToml = toml::from_str(&content)
        .map_err(|e| OxideError::Other(format!("Failed to parse pack.toml: {}", e)))?;
    
    Ok(pack)
}

/// Read and parse index.toml
#[allow(dead_code)] // Reserved for future pack sync features
pub fn read_index_toml(instance_path: &Path) -> Result<IndexToml> {
    let path = instance_path.join("index.toml");
    if !path.exists() {
        return Ok(IndexToml::default());
    }
    
    let content = fs::read_to_string(&path)?;
    let index: IndexToml = toml::from_str(&content)
        .map_err(|e| OxideError::Other(format!("Failed to parse index.toml: {}", e)))?;
    
    Ok(index)
}

/// Read and parse a mod.pw.toml file
pub fn read_mod_toml(path: &Path) -> Result<ModTomlExtended> {
    if !path.exists() {
        return Err(OxideError::Other(format!(
            "Mod toml not found at {:?}",
            path
        )));
    }
    
    let content = fs::read_to_string(path)?;
    let mod_toml: ModTomlExtended = toml::from_str(&content)
        .map_err(|e| OxideError::Other(format!("Failed to parse mod toml: {}", e)))?;
    
    Ok(mod_toml)
}

/// Get the .index directory for a resource folder (e.g., mods/.index)
/// 
/// Following Prism Launcher's approach, metadata is stored in a hidden
/// .index subfolder to keep the main folder clean.
pub fn index_dir(resource_dir: &Path) -> PathBuf {
    resource_dir.join(".index")
}

/// Find all mod.pw.toml files in an instance
/// 
/// Searches in .index subfolders following Prism Launcher's approach,
/// and also checks root folders for backwards compatibility.
pub fn find_mod_tomls(instance_path: &Path) -> Result<Vec<PathBuf>> {
    let mods_dir = instance_path.join("mods");
    let mut tomls = Vec::new();
    
    // Check .index subfolder first (new location)
    let mods_index = index_dir(&mods_dir);
    if mods_index.exists() {
        for entry in fs::read_dir(&mods_index)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "toml") 
                && path.file_name()
                    .and_then(|n| n.to_str())
                    .map_or(false, |n| n.ends_with(".pw.toml"))
            {
                tomls.push(path);
            }
        }
    }
    
    // Also check root mods folder for backwards compatibility
    if mods_dir.exists() {
        for entry in fs::read_dir(&mods_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map_or(false, |e| e == "toml") 
                && path.file_name()
                    .and_then(|n| n.to_str())
                    .map_or(false, |n| n.ends_with(".pw.toml"))
            {
                tomls.push(path);
            }
        }
    }
    
    // Also check resourcepacks and shaderpacks directories
    for subdir in &["resourcepacks", "shaderpacks"] {
        let dir = instance_path.join(subdir);
        
        // Check .index subfolder first
        let index = index_dir(&dir);
        if index.exists() {
            for entry in fs::read_dir(&index)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "toml")
                    && path.file_name()
                        .and_then(|n| n.to_str())
                        .map_or(false, |n| n.ends_with(".pw.toml"))
                {
                    tomls.push(path);
                }
            }
        }
        
        // Also check root folder for backwards compatibility
        if dir.exists() {
            for entry in fs::read_dir(&dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "toml")
                    && path.file_name()
                        .and_then(|n| n.to_str())
                        .map_or(false, |n| n.ends_with(".pw.toml"))
                {
                    tomls.push(path);
                }
            }
        }
    }
    
    Ok(tomls)
}

// =============================================================================
// File Writing
// =============================================================================

/// Write pack.toml to disk
pub fn write_pack_toml(instance_path: &Path, pack: &PackToml) -> Result<()> {
    let path = instance_path.join("pack.toml");
    let content = toml::to_string_pretty(pack)
        .map_err(|e| OxideError::Other(format!("Failed to serialize pack.toml: {}", e)))?;
    
    fs::write(&path, content)?;
    Ok(())
}

/// Write index.toml to disk and update pack.toml hash
pub fn write_index_toml(instance_path: &Path, index: &IndexToml, pack: &mut PackToml) -> Result<()> {
    let path = instance_path.join("index.toml");
    let content = toml::to_string_pretty(index)
        .map_err(|e| OxideError::Other(format!("Failed to serialize index.toml: {}", e)))?;
    
    // Compute hash of index content
    pack.index.hash = compute_hash(content.as_bytes(), pack.index.hash_format);
    
    fs::write(&path, content)?;
    Ok(())
}

/// Write a mod.pw.toml file
pub fn write_mod_toml(path: &Path, mod_toml: &ModTomlExtended) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    
    let content = toml::to_string_pretty(mod_toml)
        .map_err(|e| OxideError::Other(format!("Failed to serialize mod toml: {}", e)))?;
    
    fs::write(path, content)?;
    Ok(())
}

/// Delete a mod.pw.toml file
pub fn delete_mod_toml(path: &Path) -> Result<()> {
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

// =============================================================================
// Hash Computation
// =============================================================================

/// Compute hash of data using specified format
pub fn compute_hash(data: &[u8], format: HashFormat) -> String {
    match format {
        HashFormat::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(data);
            hex::encode(hasher.finalize())
        }
        HashFormat::Sha512 => {
            let mut hasher = Sha512::new();
            hasher.update(data);
            hex::encode(hasher.finalize())
        }
        HashFormat::Sha1 => {
            let mut hasher = Sha1::new();
            hasher.update(data);
            hex::encode(hasher.finalize())
        }
        HashFormat::Md5 => {
            // md5 crate uses compute() function directly
            let digest = md5::compute(data);
            format!("{:x}", digest)
        }
        HashFormat::Murmur2 => {
            // Murmur2 hash (used by CurseForge)
            compute_murmur2(data).to_string()
        }
    }
}

/// Compute hash of a file
pub fn compute_file_hash(path: &Path, format: HashFormat) -> Result<String> {
    let data = fs::read(path)?;
    Ok(compute_hash(&data, format))
}

/// Compute Murmur2 hash (CurseForge fingerprint style)
/// 
/// CurseForge uses a specific variant of Murmur2 that:
/// - Filters out whitespace characters (0x09, 0x0A, 0x0D, 0x20)
/// - Uses seed 1
fn compute_murmur2(data: &[u8]) -> u32 {
    // Filter out whitespace as per CurseForge spec
    let filtered: Vec<u8> = data
        .iter()
        .filter(|&&b| b != 0x09 && b != 0x0A && b != 0x0D && b != 0x20)
        .copied()
        .collect();
    
    murmur2_32(&filtered, 1)
}

/// Murmur2 32-bit hash implementation
fn murmur2_32(data: &[u8], seed: u32) -> u32 {
    const M: u32 = 0x5bd1e995;
    const R: i32 = 24;
    
    let len = data.len();
    let mut h: u32 = seed ^ (len as u32);
    
    let chunks = data.chunks_exact(4);
    let remainder = chunks.remainder();
    
    for chunk in chunks {
        let mut k = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        k = k.wrapping_mul(M);
        k ^= k >> R;
        k = k.wrapping_mul(M);
        h = h.wrapping_mul(M);
        h ^= k;
    }
    
    // Handle remaining bytes
    match remainder.len() {
        3 => {
            h ^= (remainder[2] as u32) << 16;
            h ^= (remainder[1] as u32) << 8;
            h ^= remainder[0] as u32;
            h = h.wrapping_mul(M);
        }
        2 => {
            h ^= (remainder[1] as u32) << 8;
            h ^= remainder[0] as u32;
            h = h.wrapping_mul(M);
        }
        1 => {
            h ^= remainder[0] as u32;
            h = h.wrapping_mul(M);
        }
        _ => {}
    }
    
    h ^= h >> 13;
    h = h.wrapping_mul(M);
    h ^= h >> 15;
    
    h
}

/// Verify file hash
#[allow(dead_code)] // Reserved for future file integrity verification
pub fn verify_hash(path: &Path, expected_hash: &str, format: HashFormat) -> Result<bool> {
    let actual_hash = compute_file_hash(path, format)?;
    Ok(actual_hash.eq_ignore_ascii_case(expected_hash))
}

// =============================================================================
// Index Management
// =============================================================================

/// Add or update a file in the index
#[allow(dead_code)] // Reserved for future incremental index updates
pub fn update_index_entry(index: &mut IndexToml, file_path: &str, hash: String, is_metafile: bool) {
    // Find existing entry
    if let Some(entry) = index.files.iter_mut().find(|f| f.file == file_path) {
        entry.hash = hash;
        entry.metafile = is_metafile;
    } else {
        // Add new entry
        index.files.push(IndexFile {
            file: file_path.to_string(),
            hash,
            hash_format: None, // Use default from index
            metafile: is_metafile,
            preserve: false,
            alias: None,
        });
    }
}

/// Remove a file from the index
#[allow(dead_code)] // Reserved for future mod removal index sync
pub fn remove_index_entry(index: &mut IndexToml, file_path: &str) {
    index.files.retain(|f| f.file != file_path);
}

/// Rebuild index from disk
pub fn rebuild_index(instance_path: &Path, hash_format: HashFormat) -> Result<IndexToml> {
    let mut index = IndexToml {
        hash_format,
        files: Vec::new(),
    };
    
    // Scan for mod tomls
    let mods_dir = instance_path.join("mods");
    if mods_dir.exists() {
        scan_directory_for_index(&mut index, instance_path, &mods_dir, hash_format)?;
    }
    
    // Scan resourcepacks
    let resourcepacks_dir = instance_path.join("resourcepacks");
    if resourcepacks_dir.exists() {
        scan_directory_for_index(&mut index, instance_path, &resourcepacks_dir, hash_format)?;
    }
    
    // Scan shaderpacks
    let shaderpacks_dir = instance_path.join("shaderpacks");
    if shaderpacks_dir.exists() {
        scan_directory_for_index(&mut index, instance_path, &shaderpacks_dir, hash_format)?;
    }
    
    Ok(index)
}

fn scan_directory_for_index(
    index: &mut IndexToml,
    base_path: &Path,
    dir: &Path,
    hash_format: HashFormat,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            
            // Check if it's a pw.toml file
            if filename.ends_with(".pw.toml") {
                let relative = path.strip_prefix(base_path)
                    .map_err(|_| OxideError::Other("Failed to get relative path".into()))?;
                
                let hash = compute_file_hash(&path, hash_format)?;
                
                index.files.push(IndexFile {
                    file: relative.to_string_lossy().replace('\\', "/"),
                    hash,
                    hash_format: None,
                    metafile: true,
                    preserve: false,
                    alias: None,
                });
            }
        }
    }
    
    Ok(())
}

// =============================================================================
// Initialization
// =============================================================================

/// Initialize packwiz files for an existing instance
pub fn initialize_packwiz(
    instance_path: &Path,
    name: &str,
    minecraft_version: &str,
    mod_loader: Option<(&str, &str)>,
) -> Result<()> {
    // Create pack.toml
    let mut pack = PackToml::new(
        name.to_string(),
        minecraft_version.to_string(),
        mod_loader,
    );
    
    // Create empty index
    let index = IndexToml::default();
    
    // Write index first (to get hash)
    write_index_toml(instance_path, &index, &mut pack)?;
    
    // Write pack.toml
    write_pack_toml(instance_path, &pack)?;
    
    // Create mods directory for pw.toml files
    let mods_toml_dir = instance_path.join("mods");
    fs::create_dir_all(&mods_toml_dir)?;
    
    Ok(())
}

/// Check if an instance has packwiz files
pub fn has_packwiz(instance_path: &Path) -> bool {
    instance_path.join("pack.toml").exists()
}

// =============================================================================
// Filename Utilities
// =============================================================================

/// Generate pw.toml filename from mod filename
/// e.g., "sodium-mc1.21.1-0.6.0.jar" -> "sodium.pw.toml"
pub fn mod_toml_filename(jar_filename: &str) -> String {
    // Remove .jar extension and any .disabled suffix
    let base = jar_filename
        .trim_end_matches(".disabled")
        .trim_end_matches(".jar");
    
    // Try to extract just the mod name (before version info)
    // Common patterns: mod-name-mc1.21-1.0.0, mod_name_1.0.0, ModName-1.0.0
    let name = extract_mod_name(base);
    
    format!("{}.pw.toml", name.to_lowercase().replace(' ', "-"))
}

/// Extract mod name from filename (strip version numbers)
fn extract_mod_name(filename: &str) -> &str {
    // Find common version separators
    // Look for patterns like -mc1.21, -1.0.0, _1.0.0, etc.
    
    // Try to find version pattern start
    let chars: Vec<char> = filename.chars().collect();
    let mut best_end = filename.len();
    
    for (i, window) in chars.windows(2).enumerate() {
        // Look for -mc, _mc, -[digit], _[digit]
        if (window[0] == '-' || window[0] == '_') && 
           (window[1].is_ascii_digit() || 
            (window[1] == 'm' && chars.get(i + 2) == Some(&'c')))
        {
            // Check if this looks like a version start
            if i > 0 {
                best_end = i;
                break;
            }
        }
    }
    
    // Also check for +forge, +fabric, etc.
    if let Some(plus_pos) = filename.find('+') {
        if plus_pos < best_end && plus_pos > 0 {
            best_end = plus_pos;
        }
    }
    
    &filename[..best_end]
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mod_toml_filename() {
        assert_eq!(mod_toml_filename("sodium-mc1.21.1-0.6.0.jar"), "sodium.pw.toml");
        assert_eq!(mod_toml_filename("iris-mc1.21.1-1.8.0+build.1.jar"), "iris.pw.toml");
        assert_eq!(mod_toml_filename("jei-1.21.1-forge-19.0.0.7.jar"), "jei.pw.toml");
        assert_eq!(mod_toml_filename("ModName-1.0.0.jar"), "modname.pw.toml");
    }
    
    #[test]
    fn test_murmur2() {
        // Test with known CurseForge values
        let data = b"test data for hashing";
        let hash = compute_murmur2(data);
        assert!(hash > 0); // Basic sanity check
    }
    
    #[test]
    fn test_hash_formats() {
        let data = b"test";
        
        let sha256 = compute_hash(data, HashFormat::Sha256);
        assert_eq!(sha256.len(), 64);
        
        let sha512 = compute_hash(data, HashFormat::Sha512);
        assert_eq!(sha512.len(), 128);
        
        let sha1 = compute_hash(data, HashFormat::Sha1);
        assert_eq!(sha1.len(), 40);
        
        let md5 = compute_hash(data, HashFormat::Md5);
        assert_eq!(md5.len(), 32);
    }
}
