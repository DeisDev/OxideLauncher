//! Instance export functionality

use std::path::{Path, PathBuf};
use std::io::{Read, Write};
use std::fs::File;
use chrono::Utc;
use sha2::{Sha256, Digest};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use zip::write::FileOptions;
use zip::ZipWriter;
use walkdir::WalkDir;

use crate::core::error::Result;
use super::types::{Instance, ModLoaderType};
use super::transfer::{
    OxideManifest, OxideInstanceMetadata, OxideIcon, OxideModLoader,
    OxideInstanceSettings, OxideManagedPack,
};

/// Options for exporting an instance
#[derive(Debug, Clone)]
pub struct ExportOptions {
    /// Include saves/worlds
    pub include_saves: bool,
    
    /// Include screenshots
    pub include_screenshots: bool,
    
    /// Include logs
    pub include_logs: bool,
    
    /// Include crash reports
    pub include_crash_reports: bool,
    
    /// Include resource packs
    pub include_resource_packs: bool,
    
    /// Include shader packs
    pub include_shader_packs: bool,
    
    /// Include mods
    pub include_mods: bool,
    
    /// Include configs
    pub include_configs: bool,
    
    /// Include options.txt and other game settings
    pub include_game_settings: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            include_saves: true,
            include_screenshots: false,
            include_logs: false,
            include_crash_reports: false,
            include_resource_packs: true,
            include_shader_packs: true,
            include_mods: true,
            include_configs: true,
            include_game_settings: true,
        }
    }
}

/// Progress callback type that's Send + Sync
pub type ProgressCallback = std::sync::Arc<dyn Fn(f32, &str) + Send + Sync>;

/// Export an instance to OxideLauncher format (.oxide)
pub async fn export_instance(
    instance: &Instance,
    output_path: &Path,
    options: &ExportOptions,
    progress_callback: Option<ProgressCallback>,
) -> Result<()> {
    if let Some(ref cb) = progress_callback {
        cb(0.0, "Preparing export...");
    }
    
    // Create manifest
    let manifest = create_manifest(instance)?;
    
    // Create ZIP file
    let file = File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    
    let zip_options = FileOptions::<()>::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(6));
    
    if let Some(ref cb) = progress_callback {
        cb(0.1, "Writing manifest...");
    }
    
    // Write manifest
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    zip.start_file("oxide.manifest.json", zip_options.clone())?;
    zip.write_all(manifest_json.as_bytes())?;
    
    // Export custom icon if present
    if let OxideIcon::Custom { data, filename } = &manifest.instance.icon {
        if let Some(ref cb) = progress_callback {
            cb(0.15, "Writing icon...");
        }
        
        let icon_path = format!("icon/{}", filename);
        zip.start_file(&icon_path, zip_options.clone())?;
        
        if let Ok(icon_data) = BASE64.decode(data) {
            zip.write_all(&icon_data)?;
        }
    }
    
    // Collect files to export
    let game_dir = instance.game_dir();
    let files_to_export = collect_files_to_export(&game_dir, options)?;
    
    let total_files = files_to_export.len();
    
    if let Some(ref cb) = progress_callback {
        cb(0.2, &format!("Exporting {} files...", total_files));
    }
    
    // Export files
    for (idx, (relative_path, absolute_path)) in files_to_export.iter().enumerate() {
        if let Some(ref cb) = progress_callback {
            let progress = 0.2 + (0.8 * (idx as f32 / total_files as f32));
            let filename = relative_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("file");
            cb(progress, &format!("Exporting: {}", filename));
        }
        
        // Read file
        let mut file_data = Vec::new();
        let mut file = File::open(absolute_path)?;
        file.read_to_end(&mut file_data)?;
        
        // Write to ZIP
        let zip_path = format!("data/{}", relative_path.to_string_lossy().replace("\\", "/"));
        zip.start_file(&zip_path, zip_options.clone())?;
        zip.write_all(&file_data)?;
    }
    
    // Finalize ZIP
    zip.finish()?;
    
    if let Some(ref cb) = progress_callback {
        cb(1.0, "Export complete!");
    }
    
    Ok(())
}

/// Create the export manifest
fn create_manifest(instance: &Instance) -> Result<OxideManifest> {
    // Prepare icon
    let icon = if instance.icon.starts_with("custom:") || instance.icon.contains('/') || instance.icon.contains('\\') {
        // Try to load custom icon
        let icon_path = PathBuf::from(&instance.icon.replace("custom:", ""));
        if icon_path.exists() {
            let mut data = Vec::new();
            File::open(&icon_path)
                .and_then(|mut f| f.read_to_end(&mut data))
                .ok()
                .map(|_| OxideIcon::Custom {
                    data: BASE64.encode(&data),
                    filename: icon_path.file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("icon.png")
                        .to_string(),
                })
                .unwrap_or(OxideIcon::Default { name: "default".to_string() })
        } else {
            OxideIcon::Default { name: "default".to_string() }
        }
    } else {
        OxideIcon::Default { name: instance.icon.clone() }
    };
    
    // Prepare mod loader
    let mod_loader = instance.mod_loader.as_ref().map(|ml| OxideModLoader {
        loader_type: match ml.loader_type {
            ModLoaderType::Forge => "forge".to_string(),
            ModLoaderType::NeoForge => "neoforge".to_string(),
            ModLoaderType::Fabric => "fabric".to_string(),
            ModLoaderType::Quilt => "quilt".to_string(),
            ModLoaderType::LiteLoader => "liteloader".to_string(),
        },
        version: ml.version.clone(),
    });
    
    // Prepare settings
    let settings = OxideInstanceSettings {
        jvm_args: instance.settings.jvm_args.clone(),
        game_args: instance.settings.game_args.clone(),
        min_memory: instance.settings.min_memory,
        max_memory: instance.settings.max_memory,
        window_width: instance.settings.window_width,
        window_height: instance.settings.window_height,
        fullscreen: instance.settings.fullscreen,
    };
    
    // Prepare managed pack
    let managed_pack = instance.managed_pack.as_ref().map(|mp| OxideManagedPack {
        platform: format!("{:?}", mp.platform).to_lowercase(),
        pack_id: mp.pack_id.clone(),
        pack_name: mp.pack_name.clone(),
        version_id: mp.version_id.clone(),
        version_name: mp.version_name.clone(),
    });
    
    Ok(OxideManifest {
        format_version: 1,
        instance: OxideInstanceMetadata {
            original_id: instance.id.clone(),
            name: instance.name.clone(),
            icon,
            minecraft_version: instance.minecraft_version.clone(),
            mod_loader,
            notes: instance.notes.clone(),
            total_played_seconds: instance.total_played_seconds,
            created_at: instance.created_at,
            settings,
            managed_pack,
        },
        files: Vec::new(), // Files are stored separately in the archive
        exported_at: Utc::now(),
        launcher_version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Collect files to export based on options
fn collect_files_to_export(game_dir: &Path, options: &ExportOptions) -> Result<Vec<(PathBuf, PathBuf)>> {
    let mut files = Vec::new();
    
    if !game_dir.exists() {
        return Ok(files);
    }
    
    // Define what folders/files to include
    let mut include_patterns: Vec<&str> = vec![];
    let exclude_patterns: Vec<&str> = vec![".fabric", ".mixin.out"];
    
    if options.include_mods {
        include_patterns.push("mods");
    }
    if options.include_configs {
        include_patterns.push("config");
    }
    if options.include_resource_packs {
        include_patterns.push("resourcepacks");
    }
    if options.include_shader_packs {
        include_patterns.push("shaderpacks");
    }
    if options.include_saves {
        include_patterns.push("saves");
    }
    if options.include_screenshots {
        include_patterns.push("screenshots");
    }
    if options.include_logs {
        include_patterns.push("logs");
    }
    if options.include_crash_reports {
        include_patterns.push("crash-reports");
    }
    if options.include_game_settings {
        include_patterns.push("options.txt");
        include_patterns.push("optionsof.txt");
        include_patterns.push("servers.dat");
    }
    
    for entry in WalkDir::new(game_dir)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        if !path.is_file() {
            continue;
        }
        
        // Get relative path
        let relative = match path.strip_prefix(game_dir) {
            Ok(r) => r,
            Err(_) => continue,
        };
        
        let relative_str = relative.to_string_lossy();
        
        // Check if matches include patterns
        let should_include = include_patterns.iter().any(|pattern| {
            relative_str.starts_with(pattern) || relative_str == *pattern
        });
        
        if !should_include {
            continue;
        }
        
        // Check if matches exclude patterns
        let should_exclude = exclude_patterns.iter().any(|pattern| {
            relative_str.contains(pattern)
        });
        
        if should_exclude {
            continue;
        }
        
        files.push((relative.to_path_buf(), path.to_path_buf()));
    }
    
    Ok(files)
}

/// Calculate SHA256 hash of a file
#[allow(dead_code)]
fn hash_file(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];
    
    loop {
        let bytes_read = file.read(&mut buffer)?;
        
        if bytes_read == 0 {
            break;
        }
        
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(hex::encode(hasher.finalize()))
}
