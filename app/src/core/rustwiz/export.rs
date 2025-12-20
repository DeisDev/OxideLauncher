//! RustWiz metadata export to Modrinth and CurseForge modpack formats.
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
use std::fs::{self, File};
use std::io::{Read, Write};
use std::collections::HashMap;
use chrono::Utc;
use zip::write::FileOptions;
use zip::ZipWriter;
use walkdir::WalkDir;

use crate::core::error::{Result, OxideError};
use super::types::*;
use super::parser::{read_pack_toml, read_mod_toml, find_mod_tomls, compute_file_hash};

// =============================================================================
// Export Options
// =============================================================================

/// Options for exporting a modpack
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Include config files
    pub include_configs: bool,
    
    /// Include resource packs
    pub include_resourcepacks: bool,
    
    /// Include shader packs
    pub include_shaderpacks: bool,
    
    /// Include worlds/saves
    pub include_saves: bool,
    
    /// Pack version override
    pub version: Option<String>,
    
    /// Pack author override
    pub author: Option<String>,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_configs: true,
            include_resourcepacks: true,
            include_shaderpacks: true,
            include_saves: false,
            version: None,
            author: None,
        }
    }
}

/// Progress callback type
pub type ProgressCallback = std::sync::Arc<dyn Fn(f32, &str) + Send + Sync>;

// =============================================================================
// Modrinth Export (.mrpack)
// =============================================================================

/// Export to Modrinth format (.mrpack)
pub async fn export_modrinth(
    instance_path: &Path,
    output_path: &Path,
    options: &ExportOptions,
    progress: Option<ProgressCallback>,
) -> Result<()> {
    if let Some(ref cb) = progress {
        cb(0.0, "Reading pack metadata...");
    }
    
    let pack = read_pack_toml(instance_path)?;
    let game_dir = instance_path.join(".minecraft");
    
    // Build modrinth.index.json
    let mut modrinth_files = Vec::new();
    let mut overrides_files: Vec<(PathBuf, String)> = Vec::new(); // (source_path, relative_path)
    
    if let Some(ref cb) = progress {
        cb(0.1, "Processing mod metadata...");
    }
    
    // Process mod tomls to get download URLs
    let mod_tomls = find_mod_tomls(instance_path)?;
    
    for (idx, toml_path) in mod_tomls.iter().enumerate() {
        if let Some(ref cb) = progress {
            let progress_pct = 0.1 + (0.4 * (idx as f32 / mod_tomls.len() as f32));
            cb(progress_pct, &format!("Processing mod {}...", idx + 1));
        }
        
        let mod_toml = read_mod_toml(toml_path)?;
        
        // Compute file hashes
        let jar_path = game_dir.join(&mod_toml.packwiz.filename);
        
        if !jar_path.exists() {
            tracing::warn!("Mod file not found: {:?}", jar_path);
            continue;
        }
        
        let sha1 = compute_file_hash(&jar_path, HashFormat::Sha1)?;
        let sha512 = compute_file_hash(&jar_path, HashFormat::Sha512)?;
        let file_size = fs::metadata(&jar_path)?.len();
        
        // Determine environment based on side
        let env = match mod_toml.packwiz.side {
            Side::Both => ModrinthEnv {
                client: "required".to_string(),
                server: "required".to_string(),
            },
            Side::Client => ModrinthEnv {
                client: "required".to_string(),
                server: "unsupported".to_string(),
            },
            Side::Server => ModrinthEnv {
                client: "unsupported".to_string(),
                server: "required".to_string(),
            },
        };
        
        modrinth_files.push(ModrinthExportFile {
            path: mod_toml.packwiz.filename.clone(),
            hashes: ModrinthExportHashes {
                sha1,
                sha512,
            },
            env: Some(env),
            downloads: vec![mod_toml.packwiz.download.url.clone()],
            file_size,
        });
    }
    
    if let Some(ref cb) = progress {
        cb(0.5, "Collecting override files...");
    }
    
    // Collect override files (configs, etc.)
    if options.include_configs {
        collect_overrides(&game_dir.join("config"), "overrides/config", &mut overrides_files)?;
    }
    
    if options.include_resourcepacks {
        collect_overrides(&game_dir.join("resourcepacks"), "overrides/resourcepacks", &mut overrides_files)?;
    }
    
    if options.include_shaderpacks {
        collect_overrides(&game_dir.join("shaderpacks"), "overrides/shaderpacks", &mut overrides_files)?;
    }
    
    if options.include_saves {
        collect_overrides(&game_dir.join("saves"), "overrides/saves", &mut overrides_files)?;
    }
    
    // Build dependencies
    let mut dependencies = HashMap::new();
    dependencies.insert("minecraft".to_string(), pack.versions.minecraft.clone());
    
    if let Some(ref v) = pack.versions.fabric {
        dependencies.insert("fabric-loader".to_string(), v.clone());
    }
    if let Some(ref v) = pack.versions.forge {
        dependencies.insert("forge".to_string(), v.clone());
    }
    if let Some(ref v) = pack.versions.neoforge {
        dependencies.insert("neoforge".to_string(), v.clone());
    }
    if let Some(ref v) = pack.versions.quilt {
        dependencies.insert("quilt-loader".to_string(), v.clone());
    }
    
    // Generate version ID
    let version_id = options.version.clone()
        .or(pack.version.clone())
        .unwrap_or_else(|| Utc::now().format("%Y%m%d-%H%M%S").to_string());
    
    // Create modrinth index
    let modrinth_index = ModrinthExportIndex {
        format_version: 1,
        game: "minecraft".to_string(),
        version_id,
        name: pack.name.clone(),
        summary: pack.description.clone(),
        files: modrinth_files,
        dependencies,
    };
    
    if let Some(ref cb) = progress {
        cb(0.7, "Creating archive...");
    }
    
    // Write .mrpack file
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    
    let zip_options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(6));
    
    // Write modrinth.index.json
    let index_json = serde_json::to_string_pretty(&modrinth_index)
        .map_err(|e| OxideError::Other(format!("Failed to serialize index: {}", e)))?;
    
    zip.start_file("modrinth.index.json", zip_options.clone())?;
    zip.write_all(index_json.as_bytes())?;
    
    if let Some(ref cb) = progress {
        cb(0.8, "Writing override files...");
    }
    
    // Write override files
    for (idx, (source_path, relative_path)) in overrides_files.iter().enumerate() {
        if let Some(ref cb) = progress {
            let progress_pct = 0.8 + (0.2 * (idx as f32 / overrides_files.len().max(1) as f32));
            cb(progress_pct, &format!("Writing: {}", relative_path));
        }
        
        let mut file_data = Vec::new();
        let mut file = File::open(source_path)?;
        file.read_to_end(&mut file_data)?;
        
        zip.start_file(relative_path, zip_options.clone())?;
        zip.write_all(&file_data)?;
    }
    
    zip.finish()?;
    
    if let Some(ref cb) = progress {
        cb(1.0, "Export complete!");
    }
    
    Ok(())
}

// =============================================================================
// CurseForge Export
// =============================================================================

/// Export to CurseForge format
pub async fn export_curseforge(
    instance_path: &Path,
    output_path: &Path,
    options: &ExportOptions,
    progress: Option<ProgressCallback>,
) -> Result<()> {
    if let Some(ref cb) = progress {
        cb(0.0, "Reading pack metadata...");
    }
    
    let pack = read_pack_toml(instance_path)?;
    let game_dir = instance_path.join(".minecraft");
    
    let mut cf_files = Vec::new();
    let mut overrides_files: Vec<(PathBuf, String)> = Vec::new();
    
    if let Some(ref cb) = progress {
        cb(0.1, "Processing mod metadata...");
    }
    
    // Process mod tomls
    let mod_tomls = find_mod_tomls(instance_path)?;
    
    for toml_path in &mod_tomls {
        let mod_toml = read_mod_toml(toml_path)?;
        
        // Only include mods with CurseForge update info
        if let Some(ref update) = mod_toml.packwiz.update {
            if let Some(ref cf) = update.curseforge {
                cf_files.push(CurseForgeExportFile {
                    project_id: cf.project_id,
                    file_id: cf.file_id,
                    required: mod_toml.packwiz.option.as_ref().map_or(true, |o| !o.optional),
                });
            } else {
                // Mod doesn't have CurseForge source - add to overrides
                let jar_path = game_dir.join(&mod_toml.packwiz.filename);
                if jar_path.exists() {
                    let override_path = format!("overrides/{}", mod_toml.packwiz.filename);
                    overrides_files.push((jar_path, override_path));
                }
            }
        } else {
            // No update source - add to overrides
            let jar_path = game_dir.join(&mod_toml.packwiz.filename);
            if jar_path.exists() {
                let override_path = format!("overrides/{}", mod_toml.packwiz.filename);
                overrides_files.push((jar_path, override_path));
            }
        }
    }
    
    if let Some(ref cb) = progress {
        cb(0.5, "Collecting override files...");
    }
    
    // Collect override files
    if options.include_configs {
        collect_overrides(&game_dir.join("config"), "overrides/config", &mut overrides_files)?;
    }
    
    // Build mod loaders
    let mut mod_loaders = Vec::new();
    
    if let Some(ref v) = pack.versions.forge {
        mod_loaders.push(CurseForgeExportModLoader {
            id: format!("forge-{}", v),
            primary: true,
        });
    }
    if let Some(ref v) = pack.versions.neoforge {
        mod_loaders.push(CurseForgeExportModLoader {
            id: format!("neoforge-{}", v),
            primary: true,
        });
    }
    if let Some(ref v) = pack.versions.fabric {
        mod_loaders.push(CurseForgeExportModLoader {
            id: format!("fabric-{}", v),
            primary: true,
        });
    }
    
    // Create manifest
    let manifest = CurseForgeExportManifest {
        manifest_type: "minecraftModpack".to_string(),
        manifest_version: 1,
        name: pack.name.clone(),
        version: options.version.clone()
            .or(pack.version.clone())
            .unwrap_or_else(|| "1.0.0".to_string()),
        author: options.author.clone()
            .or(pack.author.clone())
            .unwrap_or_default(),
        minecraft: CurseForgeExportMinecraft {
            version: pack.versions.minecraft.clone(),
            mod_loaders,
        },
        files: cf_files,
        overrides: "overrides".to_string(),
    };
    
    if let Some(ref cb) = progress {
        cb(0.7, "Creating archive...");
    }
    
    // Write zip file
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    
    let zip_options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(6));
    
    // Write manifest.json
    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| OxideError::Other(format!("Failed to serialize manifest: {}", e)))?;
    
    zip.start_file("manifest.json", zip_options.clone())?;
    zip.write_all(manifest_json.as_bytes())?;
    
    if let Some(ref cb) = progress {
        cb(0.8, "Writing override files...");
    }
    
    // Write override files
    for (source_path, relative_path) in &overrides_files {
        let mut file_data = Vec::new();
        let mut file = File::open(source_path)?;
        file.read_to_end(&mut file_data)?;
        
        zip.start_file(relative_path, zip_options.clone())?;
        zip.write_all(&file_data)?;
    }
    
    zip.finish()?;
    
    if let Some(ref cb) = progress {
        cb(1.0, "Export complete!");
    }
    
    Ok(())
}

// =============================================================================
// Packwiz Export (TOML format for git/hosting)
// =============================================================================

/// Export as a packwiz pack (for hosting or git)
pub fn export_packwiz(
    instance_path: &Path,
    output_dir: &Path,
    progress: Option<ProgressCallback>,
) -> Result<()> {
    if let Some(ref cb) = progress {
        cb(0.0, "Copying packwiz files...");
    }
    
    // Create output directory
    fs::create_dir_all(output_dir)?;
    
    // Copy pack.toml
    let pack_src = instance_path.join("pack.toml");
    if pack_src.exists() {
        fs::copy(&pack_src, output_dir.join("pack.toml"))?;
    }
    
    // Copy index.toml
    let index_src = instance_path.join("index.toml");
    if index_src.exists() {
        fs::copy(&index_src, output_dir.join("index.toml"))?;
    }
    
    if let Some(ref cb) = progress {
        cb(0.3, "Copying mod metadata...");
    }
    
    // Copy mods directory (pw.toml files only)
    let mods_src = instance_path.join("mods");
    let mods_dst = output_dir.join("mods");
    
    if mods_src.exists() {
        fs::create_dir_all(&mods_dst)?;
        
        for entry in fs::read_dir(&mods_src)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.extension().map_or(false, |e| e == "toml") {
                let filename = path.file_name().unwrap();
                fs::copy(&path, mods_dst.join(filename))?;
            }
        }
    }
    
    if let Some(ref cb) = progress {
        cb(1.0, "Export complete!");
    }
    
    Ok(())
}

// =============================================================================
// Helper Types
// =============================================================================

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ModrinthExportIndex {
    format_version: u32,
    game: String,
    version_id: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    files: Vec<ModrinthExportFile>,
    dependencies: HashMap<String, String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ModrinthExportFile {
    path: String,
    hashes: ModrinthExportHashes,
    #[serde(skip_serializing_if = "Option::is_none")]
    env: Option<ModrinthEnv>,
    downloads: Vec<String>,
    file_size: u64,
}

#[derive(Debug, serde::Serialize)]
struct ModrinthExportHashes {
    sha1: String,
    sha512: String,
}

#[derive(Debug, serde::Serialize)]
struct ModrinthEnv {
    client: String,
    server: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CurseForgeExportManifest {
    manifest_type: String,
    manifest_version: u32,
    name: String,
    version: String,
    author: String,
    minecraft: CurseForgeExportMinecraft,
    files: Vec<CurseForgeExportFile>,
    overrides: String,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CurseForgeExportMinecraft {
    version: String,
    mod_loaders: Vec<CurseForgeExportModLoader>,
}

#[derive(Debug, serde::Serialize)]
struct CurseForgeExportModLoader {
    id: String,
    primary: bool,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct CurseForgeExportFile {
    #[serde(rename = "projectID")]
    project_id: u32,
    #[serde(rename = "fileID")]
    file_id: u32,
    required: bool,
}

// =============================================================================
// Utilities
// =============================================================================

/// Collect files from a directory for overrides
fn collect_overrides(
    source_dir: &Path,
    override_prefix: &str,
    files: &mut Vec<(PathBuf, String)>,
) -> Result<()> {
    if !source_dir.exists() {
        return Ok(());
    }
    
    for entry in WalkDir::new(source_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        
        if path.is_file() {
            // Skip pw.toml files (those are metadata, not content)
            if path.extension().map_or(false, |e| e == "toml") {
                continue;
            }
            
            // Skip jar files (those should be downloaded, not bundled)
            if path.extension().map_or(false, |e| e == "jar") {
                continue;
            }
            
            let relative = path.strip_prefix(source_dir)
                .map_err(|_| OxideError::Other("Failed to get relative path".into()))?;
            
            let override_path = format!("{}/{}", override_prefix, relative.to_string_lossy().replace('\\', "/"));
            
            files.push((path.to_path_buf(), override_path));
        }
    }
    
    Ok(())
}
