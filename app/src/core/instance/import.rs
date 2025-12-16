//! Instance import functionality

use std::path::{Path, PathBuf};
use std::io::Read;
use std::fs::{self, File};
use zip::ZipArchive;
use std::sync::Arc;

use crate::core::error::Result;
use super::transfer::{
    ImportType, OxideManifest, OxideIcon, ModrinthIndex, CurseForgeManifest, 
    PrismInstanceConfig, PrismPackJson, ImportResult, FileToDownload, PlatformFileInfo,
    OxideInstanceSettings, OxideManagedPack,
};

/// Progress callback type that's Send + Sync
pub type ProgressCallback = Arc<dyn Fn(f32, &str) + Send + Sync>;

/// Import options
#[derive(Debug, Clone)]
pub struct ImportOptions {
    /// Name override (if None, uses name from import)
    pub name_override: Option<String>,
    
    /// Target instances directory
    pub instances_dir: PathBuf,
}

/// Detect the type of import file
pub fn detect_import_type(archive_path: &Path) -> Result<ImportType> {
    let file = File::open(archive_path)?;
    let archive = ZipArchive::new(file)?;
    
    // Collect file names
    let file_list: Vec<String> = (0..archive.len())
        .filter_map(|i| archive.name_for_index(i).map(|s| s.to_string()))
        .collect();
    
    Ok(ImportType::detect(&file_list))
}

/// Import an instance from any supported format
pub async fn import_instance(
    archive_path: &Path,
    options: &ImportOptions,
    progress_callback: Option<ProgressCallback>,
) -> Result<ImportResult> {
    if let Some(ref cb) = progress_callback {
        cb(0.0, "Detecting format...");
    }
    
    let import_type = detect_import_type(archive_path)?;
    
    if let Some(ref cb) = progress_callback {
        cb(0.1, &format!("Detected {} format", format_name(&import_type)));
    }
    
    match import_type {
        ImportType::OxideLauncher => import_oxide(archive_path, options, progress_callback).await,
        ImportType::Modrinth => import_modrinth(archive_path, options, progress_callback).await,
        ImportType::CurseForge => import_curseforge(archive_path, options, progress_callback).await,
        ImportType::Prism => import_prism(archive_path, options, progress_callback).await,
        ImportType::Unknown => Err("Unknown archive format".into()),
    }
}

fn format_name(import_type: &ImportType) -> &'static str {
    match import_type {
        ImportType::OxideLauncher => "OxideLauncher",
        ImportType::Modrinth => "Modrinth",
        ImportType::CurseForge => "CurseForge",
        ImportType::Prism => "Prism Launcher",
        ImportType::Unknown => "Unknown",
    }
}

/// Import from OxideLauncher format
async fn import_oxide(
    archive_path: &Path,
    options: &ImportOptions,
    progress_callback: Option<ProgressCallback>,
) -> Result<ImportResult> {
    if let Some(ref cb) = progress_callback {
        cb(0.15, "Reading manifest...");
    }
    
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;
    
    // Read manifest
    let manifest: OxideManifest = {
        let mut manifest_file = archive.by_name("oxide.manifest.json")?;
        let mut content = String::new();
        manifest_file.read_to_string(&mut content)?;
        serde_json::from_str(&content)?
    };
    
    let name = options.name_override.clone()
        .unwrap_or_else(|| manifest.instance.name.clone());
    
    // Create temporary extraction path
    let temp_dir = options.instances_dir.join("_temp_import");
    fs::create_dir_all(&temp_dir)?;
    
    if let Some(ref cb) = progress_callback {
        cb(0.2, "Extracting files...");
    }
    
    // Extract data files
    let total_files = archive.len();
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name_raw = file.name().to_string();
        
        // Skip manifest
        if name_raw == "oxide.manifest.json" {
            continue;
        }
        
        // Handle data files
        if name_raw.starts_with("data/") {
            if let Some(relative) = name_raw.strip_prefix("data/") {
                if relative.is_empty() || file.is_dir() {
                    continue;
                }
                
                if let Some(ref cb) = progress_callback {
                    let progress = 0.2 + (0.7 * (i as f32 / total_files as f32));
                    cb(progress, &format!("Extracting: {}", relative));
                }
                
                let target_path = temp_dir.join(relative);
                
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                
                let mut data = Vec::new();
                file.read_to_end(&mut data)?;
                fs::write(&target_path, &data)?;
            }
        }
    }
    
    // Prepare icon
    let icon = Some(manifest.instance.icon.clone());
    
    // Prepare mod loader
    let mod_loader = manifest.instance.mod_loader.as_ref()
        .map(|ml| (ml.loader_type.clone(), ml.version.clone()));
    
    if let Some(ref cb) = progress_callback {
        cb(1.0, "Import complete!");
    }
    
    Ok(ImportResult {
        name,
        minecraft_version: manifest.instance.minecraft_version,
        mod_loader,
        files_to_download: Vec::new(),
        overrides_path: Some(temp_dir),
        icon,
        playtime: manifest.instance.total_played_seconds,
        notes: manifest.instance.notes,
        managed_pack: manifest.instance.managed_pack,
        settings: manifest.instance.settings,
    })
}

/// Import from Modrinth format (.mrpack)
async fn import_modrinth(
    archive_path: &Path,
    options: &ImportOptions,
    progress_callback: Option<ProgressCallback>,
) -> Result<ImportResult> {
    if let Some(ref cb) = progress_callback {
        cb(0.15, "Reading Modrinth index...");
    }
    
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;
    
    // Read index
    let index: ModrinthIndex = {
        let mut index_file = archive.by_name("modrinth.index.json")?;
        let mut content = String::new();
        index_file.read_to_string(&mut content)?;
        serde_json::from_str(&content)?
    };
    
    let name = options.name_override.clone()
        .unwrap_or_else(|| index.name.clone());
    
    // Determine minecraft version and mod loader
    let minecraft_version = index.dependencies.get("minecraft")
        .cloned()
        .unwrap_or_else(|| "1.20.1".to_string());
    
    let mod_loader = if let Some(version) = index.dependencies.get("fabric-loader") {
        Some(("fabric".to_string(), version.clone()))
    } else if let Some(version) = index.dependencies.get("quilt-loader") {
        Some(("quilt".to_string(), version.clone()))
    } else if let Some(version) = index.dependencies.get("forge") {
        Some(("forge".to_string(), version.clone()))
    } else if let Some(version) = index.dependencies.get("neoforge") {
        Some(("neoforge".to_string(), version.clone()))
    } else {
        None
    };
    
    // Collect files to download
    let mut files_to_download: Vec<FileToDownload> = Vec::new();
    
    for mrfile in &index.files {
        // Skip client-unsupported files
        if let Some(ref env) = mrfile.env {
            if env.client == "unsupported" {
                continue;
            }
        }
        
        files_to_download.push(FileToDownload {
            path: mrfile.path.clone(),
            urls: mrfile.downloads.clone(),
            size: mrfile.file_size,
            hash_sha1: Some(mrfile.hashes.sha1.clone()),
            hash_sha512: Some(mrfile.hashes.sha512.clone()),
            platform_info: None,
        });
    }
    
    // Extract overrides
    if let Some(ref cb) = progress_callback {
        cb(0.4, "Extracting overrides...");
    }
    let temp_dir = options.instances_dir.join("_temp_import");
    fs::create_dir_all(&temp_dir)?;
    
    extract_overrides(&mut archive, &temp_dir, &["overrides/", "client-overrides/"])?;
    
    // Create managed pack info
    let managed_pack = Some(OxideManagedPack {
        platform: "modrinth".to_string(),
        pack_id: index.version_id.clone(),
        pack_name: index.name.clone(),
        version_id: index.version_id.clone(),
        version_name: index.version_id,
    });
    
    if let Some(ref cb) = progress_callback {
        cb(1.0, "Import complete - files pending download");
    }
    
    Ok(ImportResult {
        name,
        minecraft_version,
        mod_loader,
        files_to_download,
        overrides_path: Some(temp_dir),
        icon: None,
        playtime: 0,
        notes: index.summary.unwrap_or_default(),
        managed_pack,
        settings: OxideInstanceSettings::default(),
    })
}

/// Import from CurseForge format
async fn import_curseforge(
    archive_path: &Path,
    options: &ImportOptions,
    progress_callback: Option<ProgressCallback>,
) -> Result<ImportResult> {
    if let Some(ref cb) = progress_callback {
        cb(0.15, "Reading CurseForge manifest...");
    }
    
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;
    
    // Read manifest
    let manifest: CurseForgeManifest = {
        let mut manifest_file = archive.by_name("manifest.json")?;
        let mut content = String::new();
        manifest_file.read_to_string(&mut content)?;
        serde_json::from_str(&content)?
    };
    
    let name = options.name_override.clone()
        .unwrap_or_else(|| manifest.name.clone());
    
    // Determine mod loader
    let mod_loader = manifest.minecraft.mod_loaders.iter()
        .find(|ml| ml.primary)
        .or_else(|| manifest.minecraft.mod_loaders.first())
        .map(|ml| {
            // Parse loader ID like "forge-47.2.0" or "fabric-0.16.0"
            let parts: Vec<&str> = ml.id.splitn(2, '-').collect();
            if parts.len() == 2 {
                (parts[0].to_string(), parts[1].to_string())
            } else {
                ("forge".to_string(), ml.id.clone())
            }
        });
    
    // Files from CurseForge need to be fetched via API
    let mut files_to_download: Vec<FileToDownload> = Vec::new();
    
    for cf_file in &manifest.files {
        files_to_download.push(FileToDownload {
            path: format!("mods/cf_{}_{}.jar", cf_file.project_id, cf_file.file_id),
            urls: vec![], // CurseForge files need API resolution
            size: 0,
            hash_sha1: None,
            hash_sha512: None,
            platform_info: Some(PlatformFileInfo {
                platform: "curseforge".to_string(),
                project_id: cf_file.project_id.to_string(),
                file_id: cf_file.file_id.to_string(),
            }),
        });
    }
    
    // Extract overrides
    if let Some(ref cb) = progress_callback {
        cb(0.4, "Extracting overrides...");
    }
    let temp_dir = options.instances_dir.join("_temp_import");
    fs::create_dir_all(&temp_dir)?;
    
    let override_folder = format!("{}/", manifest.overrides);
    extract_overrides(&mut archive, &temp_dir, &[&override_folder])?;
    
    let managed_pack = Some(OxideManagedPack {
        platform: "curseforge".to_string(),
        pack_id: String::new(), // CurseForge manifest doesn't include project ID
        pack_name: manifest.name.clone(),
        version_id: manifest.version.clone(),
        version_name: manifest.version,
    });
    
    if let Some(ref cb) = progress_callback {
        cb(1.0, "Import complete - CurseForge files need API resolution");
    }
    
    Ok(ImportResult {
        name,
        minecraft_version: manifest.minecraft.version,
        mod_loader,
        files_to_download,
        overrides_path: Some(temp_dir),
        icon: None,
        playtime: 0,
        notes: String::new(),
        managed_pack,
        settings: OxideInstanceSettings::default(),
    })
}

/// Import from Prism Launcher format
async fn import_prism(
    archive_path: &Path,
    options: &ImportOptions,
    progress_callback: Option<ProgressCallback>,
) -> Result<ImportResult> {
    if let Some(ref cb) = progress_callback {
        cb(0.15, "Reading Prism instance config...");
    }
    
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;
    
    // Read instance.cfg (INI format)
    let config: PrismInstanceConfig = {
        let mut config_file = archive.by_name("instance.cfg")?;
        let mut content = String::new();
        config_file.read_to_string(&mut content)?;
        PrismInstanceConfig::parse(&content)
    };
    
    // Try to read mmc-pack.json for component info
    let pack_json: Option<PrismPackJson> = match archive.by_name("mmc-pack.json") {
        Ok(mut file) => {
            let mut content = String::new();
            file.read_to_string(&mut content).ok();
            serde_json::from_str(&content).ok()
        },
        Err(_) => None,
    };
    
    let name = options.name_override.clone()
        .unwrap_or_else(|| {
            if config.name.is_empty() {
                "Imported Instance".to_string()
            } else {
                config.name.clone()
            }
        });
    
    // Determine minecraft version and mod loader from components
    let mut minecraft_version = String::from("1.20.1");
    let mut mod_loader: Option<(String, String)> = None;
    
    if let Some(ref pack) = pack_json {
        for component in &pack.components {
            match component.uid.as_str() {
                "net.minecraft" => {
                    minecraft_version = component.version.clone();
                },
                "net.fabricmc.fabric-loader" => {
                    mod_loader = Some(("fabric".to_string(), component.version.clone()));
                },
                "org.quiltmc.quilt-loader" => {
                    mod_loader = Some(("quilt".to_string(), component.version.clone()));
                },
                "net.minecraftforge" => {
                    mod_loader = Some(("forge".to_string(), component.version.clone()));
                },
                "net.neoforged" => {
                    mod_loader = Some(("neoforge".to_string(), component.version.clone()));
                },
                _ => {}
            }
        }
    }
    
    // Extract .minecraft contents
    if let Some(ref cb) = progress_callback {
        cb(0.3, "Extracting instance files...");
    }
    let temp_dir = options.instances_dir.join("_temp_import");
    fs::create_dir_all(&temp_dir)?;
    
    extract_prism_minecraft(&mut archive, &temp_dir)?;
    
    // Parse playtime (Prism stores in seconds already)
    let playtime = config.total_time_played;
    
    // Try to get icon
    let icon = if !config.icon_key.is_empty() {
        Some(OxideIcon::Default { name: config.icon_key.clone() })
    } else {
        None
    };
    
    // Check for managed pack
    let managed_pack = if config.managed_pack_type.is_some() {
        Some(OxideManagedPack {
            platform: config.managed_pack_type.clone().unwrap_or_default(),
            pack_id: config.managed_pack_id.clone().unwrap_or_default(),
            pack_name: config.managed_pack_name.clone().unwrap_or_default(),
            version_id: config.managed_pack_version_id.clone().unwrap_or_default(),
            version_name: config.managed_pack_version_name.clone().unwrap_or_default(),
        })
    } else {
        None
    };
    
    if let Some(ref cb) = progress_callback {
        cb(1.0, "Import complete!");
    }
    
    Ok(ImportResult {
        name,
        minecraft_version,
        mod_loader,
        files_to_download: Vec::new(),
        overrides_path: Some(temp_dir),
        icon,
        playtime,
        notes: config.notes.clone(),
        managed_pack,
        settings: OxideInstanceSettings::default(),
    })
}

/// Extract override folders from archive
fn extract_overrides(
    archive: &mut ZipArchive<File>,
    target_dir: &Path,
    prefixes: &[&str],
) -> Result<()> {
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();
        
        // Check if this file is in an override folder
        for prefix in prefixes {
            if name.starts_with(prefix) {
                if let Some(relative) = name.strip_prefix(prefix) {
                    if relative.is_empty() {
                        continue;
                    }
                    
                    let target_path = target_dir.join(relative);
                    
                    if file.is_dir() {
                        fs::create_dir_all(&target_path)?;
                    } else {
                        if let Some(parent) = target_path.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        
                        let mut data = Vec::new();
                        file.read_to_end(&mut data)?;
                        fs::write(&target_path, &data)?;
                    }
                }
                break;
            }
        }
    }
    
    Ok(())
}

/// Extract Prism's .minecraft folder contents
fn extract_prism_minecraft(archive: &mut ZipArchive<File>, target_dir: &Path) -> Result<()> {
    // Prism exports have .minecraft/ at the root
    let prefixes = [".minecraft/", "minecraft/"];
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let name = file.name().to_string();
        
        for prefix in prefixes {
            if name.starts_with(prefix) {
                if let Some(relative) = name.strip_prefix(prefix) {
                    if relative.is_empty() {
                        continue;
                    }
                    
                    let target_path = target_dir.join(relative);
                    
                    if file.is_dir() {
                        fs::create_dir_all(&target_path)?;
                    } else {
                        if let Some(parent) = target_path.parent() {
                            fs::create_dir_all(parent)?;
                        }
                        
                        let mut data = Vec::new();
                        file.read_to_end(&mut data)?;
                        fs::write(&target_path, &data)?;
                    }
                }
                break;
            }
        }
    }
    
    Ok(())
}
