//! Instance creation and initialization.
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

#![allow(dead_code)] // Creation pipeline will be used as features are completed

use std::path::PathBuf;
use crate::core::error::{OxideError, Result};
use super::{Instance, InstanceConfig};

/// Create a new instance
pub async fn create_instance(
    config: InstanceConfig,
    instances_dir: &PathBuf,
) -> Result<Instance> {
    // Generate instance directory name (sanitized)
    let dir_name = sanitize_name(&config.name);
    let mut instance_path = instances_dir.join(&dir_name);
    
    // Handle duplicate names
    let mut counter = 1;
    while instance_path.exists() {
        instance_path = instances_dir.join(format!("{}_{}", dir_name, counter));
        counter += 1;
    }

    // Create instance directory
    std::fs::create_dir_all(&instance_path)?;

    // Create the basic directory structure
    let game_dir = instance_path.join(".minecraft");
    std::fs::create_dir_all(&game_dir)?;
    std::fs::create_dir_all(game_dir.join("mods"))?;
    std::fs::create_dir_all(game_dir.join("resourcepacks"))?;
    std::fs::create_dir_all(game_dir.join("saves"))?;

    // Create the instance
    let mut instance = Instance::new(
        config.name.clone(),
        instance_path,
        config.minecraft_version.clone(),
    );

    instance.icon = config.icon;
    instance.group = config.group;
    instance.mod_loader = config.mod_loader;

    // Handle copy from existing instance
    if let Some(source_path) = &config.copy_from {
        copy_instance_files(source_path, &instance.path)?;
    }

    // Handle modpack import
    if let Some(import) = &config.import_modpack {
        import_modpack(&mut instance, import).await?;
    }

    // Save instance configuration
    instance.save()?;

    Ok(instance)
}

/// Sanitize a name for use as a directory name
fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim()
        .to_string()
}

/// Copy files from one instance to another
fn copy_instance_files(source: &PathBuf, dest: &PathBuf) -> Result<()> {
    let options = fs_extra::dir::CopyOptions {
        overwrite: true,
        skip_exist: false,
        buffer_size: 64000,
        copy_inside: true,
        content_only: true,
        depth: 0,
    };

    // Copy the entire source directory contents to destination
    if source.exists() {
        fs_extra::dir::copy(source, dest, &options)
            .map_err(|e| OxideError::Instance(format!("Failed to copy instance: {}", e)))?;
    }

    Ok(())
}

/// Import a modpack
async fn import_modpack(instance: &mut Instance, import: &super::ImportModpack) -> Result<()> {
    match &import.source {
        super::ModpackSource::File(path) => {
            // Determine file type and handle accordingly
            let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            
            match extension {
                "mrpack" => {
                    // Modrinth modpack format
                    import_mrpack(instance, path).await?;
                }
                "zip" => {
                    // Could be CurseForge or other format
                    import_zip_modpack(instance, path).await?;
                }
                _ => {
                    return Err(OxideError::Instance(
                        format!("Unsupported modpack format: {}", extension)
                    ));
                }
            }
        }
        super::ModpackSource::Modrinth { project_id, version_id } => {
            import_modrinth_modpack(instance, project_id, version_id).await?;
        }
        super::ModpackSource::CurseForge { project_id, file_id } => {
            import_curseforge_modpack(instance, *project_id, *file_id).await?;
        }
        super::ModpackSource::Url(url) => {
            import_url_modpack(instance, url).await?;
        }
    }

    Ok(())
}

/// Import a Modrinth modpack (.mrpack)
async fn import_mrpack(instance: &mut Instance, path: &PathBuf) -> Result<()> {
    use std::io::Read;
    
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Read modrinth.index.json
    let mut index_content = String::new();
    {
        let mut index_file = archive.by_name("modrinth.index.json")
            .map_err(|_| OxideError::Instance("Invalid mrpack: missing modrinth.index.json".into()))?;
        index_file.read_to_string(&mut index_content)?;
    }

    let index: serde_json::Value = serde_json::from_str(&index_content)?;

    // Extract game version
    if let Some(deps) = index.get("dependencies").and_then(|d| d.as_object()) {
        if let Some(mc_version) = deps.get("minecraft").and_then(|v| v.as_str()) {
            instance.minecraft_version = mc_version.to_string();
        }

        // Check for mod loaders
        if let Some(fabric_version) = deps.get("fabric-loader").and_then(|v| v.as_str()) {
            instance.mod_loader = Some(super::ModLoader {
                loader_type: super::ModLoaderType::Fabric,
                version: fabric_version.to_string(),
            });
        } else if let Some(quilt_version) = deps.get("quilt-loader").and_then(|v| v.as_str()) {
            instance.mod_loader = Some(super::ModLoader {
                loader_type: super::ModLoaderType::Quilt,
                version: quilt_version.to_string(),
            });
        } else if let Some(forge_version) = deps.get("forge").and_then(|v| v.as_str()) {
            instance.mod_loader = Some(super::ModLoader {
                loader_type: super::ModLoaderType::Forge,
                version: forge_version.to_string(),
            });
        } else if let Some(neoforge_version) = deps.get("neoforge").and_then(|v| v.as_str()) {
            instance.mod_loader = Some(super::ModLoader {
                loader_type: super::ModLoaderType::NeoForge,
                version: neoforge_version.to_string(),
            });
        }
    }

    // Extract overrides
    let game_dir = instance.game_dir();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        // Check for overrides directories
        let target_path = if name.starts_with("overrides/") {
            Some(game_dir.join(name.strip_prefix("overrides/").unwrap()))
        } else if name.starts_with("client-overrides/") {
            Some(game_dir.join(name.strip_prefix("client-overrides/").unwrap()))
        } else {
            None
        };

        if let Some(target) = target_path {
            if file.is_dir() {
                std::fs::create_dir_all(&target)?;
            } else {
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut output = std::fs::File::create(&target)?;
                std::io::copy(&mut file, &mut output)?;
            }
        }
    }

    // Set managed pack info
    if let Some(name) = index.get("name").and_then(|n| n.as_str()) {
        if let Some(version) = index.get("versionId").and_then(|v| v.as_str()) {
            instance.managed_pack = Some(super::ManagedPack {
                platform: super::ModpackPlatform::Modrinth,
                pack_id: String::new(), // Unknown from local file
                pack_name: name.to_string(),
                version_id: String::new(),
                version_name: version.to_string(),
            });
        }
    }

    // TODO: Download files from the files array
    // This would require implementing the download system first

    Ok(())
}

/// Import a generic zip modpack
async fn import_zip_modpack(instance: &mut Instance, path: &PathBuf) -> Result<()> {
    use std::io::Read;
    
    let file = std::fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    // Check for manifest.json (CurseForge format)
    let has_cf_manifest = archive.by_name("manifest.json").is_ok();

    if has_cf_manifest {
        // Handle CurseForge format
        let mut manifest_content = String::new();
        archive.by_name("manifest.json")?.read_to_string(&mut manifest_content)?;
        
        let manifest: serde_json::Value = serde_json::from_str(&manifest_content)?;
        
        // Extract Minecraft version
        if let Some(mc) = manifest.get("minecraft") {
            if let Some(version) = mc.get("version").and_then(|v| v.as_str()) {
                instance.minecraft_version = version.to_string();
            }
            
            // Extract mod loader
            if let Some(loaders) = mc.get("modLoaders").and_then(|l| l.as_array()) {
                for loader in loaders {
                    if let Some(id) = loader.get("id").and_then(|i| i.as_str()) {
                        if id.starts_with("forge-") {
                            instance.mod_loader = Some(super::ModLoader {
                                loader_type: super::ModLoaderType::Forge,
                                version: id.strip_prefix("forge-").unwrap().to_string(),
                            });
                            break;
                        } else if id.starts_with("fabric-") {
                            instance.mod_loader = Some(super::ModLoader {
                                loader_type: super::ModLoaderType::Fabric,
                                version: id.strip_prefix("fabric-").unwrap().to_string(),
                            });
                            break;
                        } else if id.starts_with("neoforge-") {
                            instance.mod_loader = Some(super::ModLoader {
                                loader_type: super::ModLoaderType::NeoForge,
                                version: id.strip_prefix("neoforge-").unwrap().to_string(),
                            });
                            break;
                        }
                    }
                }
            }
        }

        // Set managed pack info
        if let Some(name) = manifest.get("name").and_then(|n| n.as_str()) {
            let version = manifest.get("version").and_then(|v| v.as_str()).unwrap_or("1.0");
            instance.managed_pack = Some(super::ManagedPack {
                platform: super::ModpackPlatform::CurseForge,
                pack_id: String::new(),
                pack_name: name.to_string(),
                version_id: String::new(),
                version_name: version.to_string(),
            });
        }
    }

    // Extract overrides
    let game_dir = instance.game_dir();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();

        // Check for overrides directories
        let target_path = if name.starts_with("overrides/") {
            Some(game_dir.join(name.strip_prefix("overrides/").unwrap()))
        } else {
            None
        };

        if let Some(target) = target_path {
            if file.is_dir() {
                std::fs::create_dir_all(&target)?;
            } else {
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent)?;
                }
                let mut output = std::fs::File::create(&target)?;
                std::io::copy(&mut file, &mut output)?;
            }
        }
    }

    Ok(())
}

/// Import a Modrinth modpack from the API
async fn import_modrinth_modpack(
    _instance: &mut Instance,
    _project_id: &str,
    _version_id: &str,
) -> Result<()> {
    // TODO: Implement Modrinth API integration
    // 1. Fetch version info from Modrinth API
    // 2. Download the mrpack file
    // 3. Call import_mrpack with the downloaded file
    
    tracing::warn!("Modrinth modpack import not yet implemented");
    Err(OxideError::Instance("Modrinth modpack import not yet implemented".into()))
}

/// Import a CurseForge modpack from the API
async fn import_curseforge_modpack(
    _instance: &mut Instance,
    _project_id: u32,
    _file_id: u32,
) -> Result<()> {
    // TODO: Implement CurseForge API integration
    // 1. Fetch mod info from CurseForge API (requires API key)
    // 2. Download the modpack file
    // 3. Call import_zip_modpack with the downloaded file
    
    tracing::warn!("CurseForge modpack import not yet implemented");
    Err(OxideError::Instance("CurseForge modpack import not yet implemented".into()))
}

/// Import a modpack from a URL
async fn import_url_modpack(_instance: &mut Instance, _url: &str) -> Result<()> {
    // TODO: Implement URL download and modpack detection
    // 1. Download the file to temp directory
    // 2. Detect format based on content/extension
    // 3. Call appropriate import function
    
    tracing::warn!("URL modpack import not yet implemented");
    Err(OxideError::Instance("URL modpack import not yet implemented".into()))
}
