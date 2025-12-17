//! Instance import/export commands

use crate::commands::state::AppState;
use crate::core::instance::{
    export_instance as core_export_instance, ExportOptions,
    import_instance as core_import_instance, detect_import_type, ImportOptions, ImportType,
    ModLoader, ModLoaderType, ManagedPack, ModpackPlatform, Instance,
    install_modloader_for_instance, FileToDownload,
};
use crate::core::modplatform::curseforge::CurseForgeClient;
use crate::core::download::download_file;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::State;

// =============================================================================
// Export Types
// =============================================================================

/// Export options for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptionsRequest {
    pub include_saves: bool,
    pub include_screenshots: bool,
    pub include_logs: bool,
    pub include_crash_reports: bool,
    pub include_resource_packs: bool,
    pub include_shader_packs: bool,
    pub include_mods: bool,
    pub include_configs: bool,
    pub include_game_settings: bool,
}

impl From<ExportOptionsRequest> for ExportOptions {
    fn from(req: ExportOptionsRequest) -> Self {
        ExportOptions {
            include_saves: req.include_saves,
            include_screenshots: req.include_screenshots,
            include_logs: req.include_logs,
            include_crash_reports: req.include_crash_reports,
            include_resource_packs: req.include_resource_packs,
            include_shader_packs: req.include_shader_packs,
            include_mods: req.include_mods,
            include_configs: req.include_configs,
            include_game_settings: req.include_game_settings,
        }
    }
}

// =============================================================================
// Import Types
// =============================================================================

/// Import result for frontend
#[derive(Debug, Clone, Serialize)]
pub struct ImportResultInfo {
    pub instance_id: String,
    pub name: String,
    pub minecraft_version: String,
    pub mod_loader_type: Option<String>,
    pub mod_loader_version: Option<String>,
    pub files_to_download: usize,
    pub warnings: Vec<String>,
    /// Files that need manual download due to CurseForge restrictions
    pub blocked_files: Vec<BlockedFileInfo>,
}

/// Info about a blocked file that needs manual download
#[derive(Debug, Clone, Serialize)]
pub struct BlockedFileInfo {
    pub project_id: String,
    pub file_id: String,
    pub filename: String,
}

/// Detected import type info
#[derive(Debug, Clone, Serialize)]
pub struct ImportTypeInfo {
    pub format_type: String,
    pub display_name: String,
}

// =============================================================================
// Legacy Export Command
// =============================================================================

/// Legacy export command (simple zip export)
#[tauri::command]
pub async fn export_instance(
    state: State<'_, AppState>,
    instance_id: String,
    export_path: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Create zip archive
    let file = std::fs::File::create(&export_path)
        .map_err(|e| format!("Failed to create export file: {}", e))?;
    
    let mut zip = zip::ZipWriter::new(file);
    let options: zip::write::FileOptions<()> = zip::write::FileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    
    // Add all files from instance directory
    for entry in walkdir::WalkDir::new(&instance.path) {
        let entry = entry.map_err(|e| format!("Failed to read directory: {}", e))?;
        let path = entry.path();
        let name = path.strip_prefix(&instance.path)
            .map_err(|e| format!("Failed to get relative path: {}", e))?;
        
        if path.is_file() {
            zip.start_file(name.to_string_lossy().to_string(), options)
                .map_err(|e| format!("Failed to add file to zip: {}", e))?;
            let mut f = std::fs::File::open(path)
                .map_err(|e| format!("Failed to open file: {}", e))?;
            std::io::copy(&mut f, &mut zip)
                .map_err(|e| format!("Failed to write file to zip: {}", e))?;
        }
    }
    
    zip.finish().map_err(|e| format!("Failed to finish zip: {}", e))?;
    
    Ok(())
}

// =============================================================================
// Export Commands
// =============================================================================

#[tauri::command]
pub async fn export_instance_to_file(
    state: State<'_, AppState>,
    instance_id: String,
    output_path: String,
    options: ExportOptionsRequest,
) -> Result<(), String> {
    // Clone instance to avoid holding mutex across await
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let output = PathBuf::from(output_path);
    let export_options: ExportOptions = options.into();
    
    core_export_instance(&instance, &output, &export_options, None)
        .await
        .map_err(|e| format!("Export failed: {}", e))?;
    
    tracing::info!("Exported instance {} to {}", instance_id, output.display());
    
    Ok(())
}

// =============================================================================
// Import Commands
// =============================================================================

#[tauri::command]
pub async fn detect_import_format(
    archive_path: String,
) -> Result<ImportTypeInfo, String> {
    let path = PathBuf::from(archive_path);
    
    let import_type = detect_import_type(&path)
        .map_err(|e| format!("Failed to detect format: {}", e))?;
    
    let (format_type, display_name) = match import_type {
        ImportType::OxideLauncher => ("oxide", "OxideLauncher"),
        ImportType::Modrinth => ("modrinth", "Modrinth (.mrpack)"),
        ImportType::CurseForge => ("curseforge", "CurseForge"),
        ImportType::Prism => ("prism", "Prism Launcher"),
        ImportType::Technic => ("technic", "Technic"),
        ImportType::ATLauncher => ("atlauncher", "ATLauncher"),
        ImportType::FTBApp => ("ftbapp", "FTB App"),
        ImportType::Unknown => ("unknown", "Unknown Format"),
    };
    
    Ok(ImportTypeInfo {
        format_type: format_type.to_string(),
        display_name: display_name.to_string(),
    })
}

#[tauri::command]
pub async fn import_instance_from_file(
    state: State<'_, AppState>,
    archive_path: String,
    name_override: Option<String>,
) -> Result<ImportResultInfo, String> {
    // Get instances_dir without holding mutex across await
    let instances_dir = {
        let config = state.config.lock().unwrap();
        config.instances_dir()
    };
    
    let path = PathBuf::from(archive_path);
    let options = ImportOptions {
        name_override: name_override.clone(),
        instances_dir: instances_dir.clone(),
    };
    
    let result = core_import_instance(&path, &options, None)
        .await
        .map_err(|e| format!("Import failed: {}", e))?;
    
    // Create the actual instance
    let new_id = uuid::Uuid::new_v4().to_string();
    let instance_path = instances_dir.join(&new_id);
    let game_dir = instance_path.join(".minecraft");
    
    // Create directories
    std::fs::create_dir_all(&game_dir)
        .map_err(|e| format!("Failed to create instance directory: {}", e))?;
    
    // Move overrides to game directory
    if let Some(overrides_path) = &result.overrides_path {
        if overrides_path.exists() {
            // Copy all files from temp to game_dir
            copy_dir_all(overrides_path, &game_dir)
                .map_err(|e| format!("Failed to copy overrides: {}", e))?;
            
            // Clean up temp
            let _ = std::fs::remove_dir_all(overrides_path);
        }
    }
    
    // Create mod loader
    let mod_loader = result.mod_loader.as_ref().map(|(loader_type, version)| {
        let lt = match loader_type.as_str() {
            "forge" => ModLoaderType::Forge,
            "neoforge" => ModLoaderType::NeoForge,
            "fabric" => ModLoaderType::Fabric,
            "quilt" => ModLoaderType::Quilt,
            "liteloader" => ModLoaderType::LiteLoader,
            _ => ModLoaderType::Fabric,
        };
        ModLoader {
            loader_type: lt,
            version: version.clone(),
        }
    });
    
    // Create managed pack
    let managed_pack = result.managed_pack.as_ref().map(|mp| {
        let platform = match mp.platform.as_str() {
            "modrinth" => ModpackPlatform::Modrinth,
            "curseforge" => ModpackPlatform::CurseForge,
            "atlauncher" => ModpackPlatform::ATLauncher,
            "technic" => ModpackPlatform::Technic,
            "ftb" => ModpackPlatform::FTB,
            _ => ModpackPlatform::Modrinth,
        };
        ManagedPack {
            platform,
            pack_id: mp.pack_id.clone(),
            pack_name: mp.pack_name.clone(),
            version_id: mp.version_id.clone(),
            version_name: mp.version_name.clone(),
        }
    });
    
    // Create instance settings
    let settings = crate::core::instance::InstanceSettings {
        jvm_args: result.settings.jvm_args.clone(),
        game_args: result.settings.game_args.clone(),
        min_memory: result.settings.min_memory,
        max_memory: result.settings.max_memory,
        window_width: result.settings.window_width,
        window_height: result.settings.window_height,
        fullscreen: result.settings.fullscreen,
        ..Default::default()
    };
    
    // Determine icon
    let icon = result.icon.as_ref().map(|i| {
        match i {
            crate::core::instance::OxideIcon::Default { name } => name.clone(),
            crate::core::instance::OxideIcon::Custom { filename, .. } => format!("custom:{}", filename),
        }
    }).unwrap_or_else(|| "grass".to_string());
    
    // Create the instance
    let instance = Instance::new(
        result.name.clone(),
        instance_path.clone(),
        result.minecraft_version.clone(),
    );
    
    // Create a modified instance with all our settings
    let instance = Instance {
        id: new_id.clone(),
        mod_loader: mod_loader.clone(),
        managed_pack,
        settings,
        icon,
        notes: result.notes.clone(),
        total_played_seconds: result.playtime,
        ..instance
    };
    
    // Save the instance
    instance.save()
        .map_err(|e| format!("Failed to save instance: {}", e))?;
    
    // Install modloader if present
    if instance.mod_loader.is_some() {
        tracing::info!("Installing modloader for imported instance...");
        
        // Get libraries directory (scope to ensure mutex is dropped before await)
        let libraries_dir = {
            let config = state.config.lock().unwrap();
            config.libraries_dir()
        };
        
        install_modloader_for_instance(&instance, &libraries_dir)
            .await
            .map_err(|e| format!("Failed to install modloader: {}", e))?;
        
        tracing::info!("Modloader installed successfully");
    }
    
    // Add to state (using block to ensure lock is dropped before async ops)
    {
        let mut instances = state.instances.lock().unwrap();
        instances.push(instance.clone());
    }
    
    // Download files that need API resolution (CurseForge modpacks)
    let (download_warnings, blocked_files) = if !result.files_to_download.is_empty() {
        tracing::info!("Downloading {} modpack files...", result.files_to_download.len());
        let mods_dir = game_dir.join("mods");
        std::fs::create_dir_all(&mods_dir)
            .map_err(|e| format!("Failed to create mods directory: {}", e))?;
        
        let dl_result = download_curseforge_files(&result.files_to_download, &mods_dir).await;
        (dl_result.warnings, dl_result.blocked_files)
    } else {
        (Vec::new(), Vec::new())
    };
    
    // Prepare warnings
    let mut warnings = Vec::new();
    if !download_warnings.is_empty() {
        warnings.extend(download_warnings);
    }
    
    let result_info = ImportResultInfo {
        instance_id: new_id,
        name: result.name,
        minecraft_version: result.minecraft_version,
        mod_loader_type: mod_loader.as_ref().map(|m| m.loader_type.name().to_string()),
        mod_loader_version: mod_loader.as_ref().map(|m| m.version.clone()),
        files_to_download: result.files_to_download.len(),
        warnings,
        blocked_files,
    };
    
    tracing::info!("Imported instance: {}", result_info.name);
    
    Ok(result_info)
}

/// Import an instance from a URL (downloads first, then imports)
#[tauri::command]
pub async fn import_instance_from_url(
    state: State<'_, AppState>,
    url: String,
    name_override: Option<String>,
) -> Result<ImportResultInfo, String> {
    // Get temp and instances directories
    let (temp_dir, instances_dir) = {
        let config = state.config.lock().unwrap();
        (config.data_dir().join("temp"), config.instances_dir())
    };
    
    // Create temp directory
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp directory: {}", e))?;
    
    tracing::info!("Downloading modpack from URL: {}", url);
    
    // Parse URL to get filename
    let url_parsed = url.parse::<reqwest::Url>()
        .map_err(|e| format!("Invalid URL: {}", e))?;
    
    // Get filename from URL or use default
    let filename = url_parsed.path_segments()
        .and_then(|segments| segments.last())
        .map(|s| s.to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "modpack.zip".to_string());
    
    let download_path = temp_dir.join(&filename);
    
    // Download the file
    let client = reqwest::Client::new();
    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to download: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Download failed with status: {}", response.status()));
    }
    
    let bytes = response.bytes()
        .await
        .map_err(|e| format!("Failed to read download: {}", e))?;
    
    std::fs::write(&download_path, &bytes)
        .map_err(|e| format!("Failed to save downloaded file: {}", e))?;
    
    tracing::info!("Downloaded {} bytes to {:?}", bytes.len(), download_path);
    
    // Now import from the downloaded file
    let options = ImportOptions {
        name_override: name_override.clone(),
        instances_dir: instances_dir.clone(),
    };
    
    let result = core_import_instance(&download_path, &options, None)
        .await
        .map_err(|e| format!("Import failed: {}", e))?;
    
    // Clean up downloaded file
    let _ = std::fs::remove_file(&download_path);
    
    // Create the actual instance (same logic as import_instance_from_file)
    let new_id = uuid::Uuid::new_v4().to_string();
    let instance_path = instances_dir.join(&new_id);
    let game_dir = instance_path.join(".minecraft");
    
    std::fs::create_dir_all(&game_dir)
        .map_err(|e| format!("Failed to create instance directory: {}", e))?;
    
    // Move overrides to game directory
    if let Some(overrides_path) = &result.overrides_path {
        if overrides_path.exists() {
            copy_dir_all(overrides_path, &game_dir)
                .map_err(|e| format!("Failed to copy overrides: {}", e))?;
            let _ = std::fs::remove_dir_all(overrides_path);
        }
    }
    
    // Create mod loader
    let mod_loader = result.mod_loader.as_ref().map(|(loader_type, version)| {
        let lt = match loader_type.as_str() {
            "forge" => ModLoaderType::Forge,
            "neoforge" => ModLoaderType::NeoForge,
            "fabric" => ModLoaderType::Fabric,
            "quilt" => ModLoaderType::Quilt,
            "liteloader" => ModLoaderType::LiteLoader,
            _ => ModLoaderType::Fabric,
        };
        ModLoader {
            loader_type: lt,
            version: version.clone(),
        }
    });
    
    // Create managed pack
    let managed_pack = result.managed_pack.as_ref().map(|mp| {
        let platform = match mp.platform.as_str() {
            "modrinth" => ModpackPlatform::Modrinth,
            "curseforge" => ModpackPlatform::CurseForge,
            "atlauncher" => ModpackPlatform::ATLauncher,
            "technic" => ModpackPlatform::Technic,
            "ftb" => ModpackPlatform::FTB,
            _ => ModpackPlatform::Modrinth,
        };
        ManagedPack {
            platform,
            pack_id: mp.pack_id.clone(),
            pack_name: mp.pack_name.clone(),
            version_id: mp.version_id.clone(),
            version_name: mp.version_name.clone(),
        }
    });
    
    // Create instance settings
    let settings = crate::core::instance::InstanceSettings {
        jvm_args: result.settings.jvm_args.clone(),
        game_args: result.settings.game_args.clone(),
        min_memory: result.settings.min_memory,
        max_memory: result.settings.max_memory,
        window_width: result.settings.window_width,
        window_height: result.settings.window_height,
        fullscreen: result.settings.fullscreen,
        ..Default::default()
    };
    
    // Determine icon
    let icon = result.icon.as_ref().map(|i| {
        match i {
            crate::core::instance::OxideIcon::Default { name } => name.clone(),
            crate::core::instance::OxideIcon::Custom { filename, .. } => format!("custom:{}", filename),
        }
    }).unwrap_or_else(|| "grass".to_string());
    
    // Create the instance
    let instance = Instance::new(
        result.name.clone(),
        instance_path.clone(),
        result.minecraft_version.clone(),
    );
    
    let instance = Instance {
        id: new_id.clone(),
        mod_loader: mod_loader.clone(),
        managed_pack,
        settings,
        icon,
        notes: result.notes.clone(),
        total_played_seconds: result.playtime,
        ..instance
    };
    
    // Save the instance
    instance.save()
        .map_err(|e| format!("Failed to save instance: {}", e))?;
    
    // Install modloader if present
    if instance.mod_loader.is_some() {
        tracing::info!("Installing modloader for imported instance...");
        
        let libraries_dir = {
            let config = state.config.lock().unwrap();
            config.libraries_dir()
        };
        
        install_modloader_for_instance(&instance, &libraries_dir)
            .await
            .map_err(|e| format!("Failed to install modloader: {}", e))?;
        
        tracing::info!("Modloader installed successfully");
    }
    
    // Add to state (using block to ensure lock is dropped before async ops)
    {
        let mut instances = state.instances.lock().unwrap();
        instances.push(instance.clone());
    }
    
    // Download files that need API resolution (CurseForge modpacks)
    let (download_warnings, blocked_files) = if !result.files_to_download.is_empty() {
        tracing::info!("Downloading {} modpack files from URL import...", result.files_to_download.len());
        let mods_dir = game_dir.join("mods");
        std::fs::create_dir_all(&mods_dir)
            .map_err(|e| format!("Failed to create mods directory: {}", e))?;
        
        let dl_result = download_curseforge_files(&result.files_to_download, &mods_dir).await;
        (dl_result.warnings, dl_result.blocked_files)
    } else {
        (Vec::new(), Vec::new())
    };
    
    // Prepare warnings
    let mut warnings = Vec::new();
    if !download_warnings.is_empty() {
        warnings.extend(download_warnings);
    }
    
    let result_info = ImportResultInfo {
        instance_id: new_id,
        name: result.name,
        minecraft_version: result.minecraft_version,
        mod_loader_type: mod_loader.as_ref().map(|m| m.loader_type.name().to_string()),
        mod_loader_version: mod_loader.as_ref().map(|m| m.version.clone()),
        files_to_download: result.files_to_download.len(),
        warnings,
        blocked_files,
    };
    
    tracing::info!("Imported instance from URL: {}", result_info.name);
    
    Ok(result_info)
}

/// Copy directory recursively
fn copy_dir_all(src: &PathBuf, dst: &PathBuf) -> std::io::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }
    
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            std::fs::copy(&src_path, &dst_path)?;
        }
    }
    
    Ok(())
}

/// Result of downloading CurseForge files
struct CurseForgeDownloadResult {
    warnings: Vec<String>,
    blocked_files: Vec<BlockedFileInfo>,
}

/// Download files from CurseForge API
/// Returns warnings and info about blocked files that need manual download
async fn download_curseforge_files(
    files: &[FileToDownload],
    mods_dir: &PathBuf,
) -> CurseForgeDownloadResult {
    let mut warnings = Vec::new();
    let mut blocked_files = Vec::new();
    let client = CurseForgeClient::new();
    
    if !client.has_api_key() {
        warnings.push("CurseForge API key not configured - cannot download modpack files".to_string());
        return CurseForgeDownloadResult { warnings, blocked_files };
    }
    
    let mut downloaded = 0;
    let mut blocked = 0;
    
    for file in files {
        // Only process files with CurseForge platform info
        if let Some(ref platform_info) = file.platform_info {
            if platform_info.platform != "curseforge" {
                // Try to download from URLs if available
                if !file.urls.is_empty() {
                    if let Err(e) = download_from_urls(&file.urls, mods_dir, &file.path).await {
                        warnings.push(format!("Failed to download {}: {}", file.path, e));
                    } else {
                        downloaded += 1;
                    }
                }
                continue;
            }
            
            let project_id: u32 = match platform_info.project_id.parse() {
                Ok(id) => id,
                Err(_) => {
                    warnings.push(format!("Invalid project ID: {}", platform_info.project_id));
                    continue;
                }
            };
            
            let file_id: u32 = match platform_info.file_id.parse() {
                Ok(id) => id,
                Err(_) => {
                    warnings.push(format!("Invalid file ID: {}", platform_info.file_id));
                    continue;
                }
            };
            
            // Try to get the download URL from CurseForge API
            match client.get_download_url(project_id, file_id).await {
                Ok(download_url) if !download_url.is_empty() => {
                    // Get file info to get the correct filename
                    let filename = match client.get_file(project_id, file_id).await {
                        Ok(file_info) => file_info.files.first()
                            .map(|f| f.filename.clone())
                            .unwrap_or_else(|| format!("{}.jar", file_id)),
                        Err(_) => format!("{}.jar", file_id),
                    };
                    
                    let dest_path = mods_dir.join(&filename);
                    
                    match download_file(&download_url, &dest_path, None).await {
                        Ok(_) => {
                            tracing::debug!("Downloaded: {}", filename);
                            downloaded += 1;
                        }
                        Err(e) => {
                            warnings.push(format!("Failed to download {}: {}", filename, e));
                        }
                    }
                }
                Ok(_) => {
                    // Empty download URL means blocked mod
                    blocked += 1;
                    // Try to get the project name for a better warning
                    let filename = match client.get_file(project_id, file_id).await {
                        Ok(file_info) => file_info.files.first()
                            .map(|f| f.filename.clone())
                            .unwrap_or_else(|| format!("{}.jar", file_id)),
                        Err(_) => format!("{}.jar", file_id),
                    };
                    
                    blocked_files.push(BlockedFileInfo {
                        project_id: project_id.to_string(),
                        file_id: file_id.to_string(),
                        filename,
                    });
                }
                Err(e) => {
                    // API error - could be blocked or actual error
                    blocked += 1;
                    let filename = match client.get_file(project_id, file_id).await {
                        Ok(file_info) => file_info.files.first()
                            .map(|f| f.filename.clone())
                            .unwrap_or_else(|| format!("{}.jar", file_id)),
                        Err(_) => format!("{}.jar", file_id),
                    };
                    
                    blocked_files.push(BlockedFileInfo {
                        project_id: project_id.to_string(),
                        file_id: file_id.to_string(),
                        filename,
                    });
                    
                    tracing::warn!(
                        "Could not get download URL for project {} file {}: {}",
                        project_id, file_id, e
                    );
                }
            }
        } else if !file.urls.is_empty() {
            // Non-CurseForge file with URLs
            if let Err(e) = download_from_urls(&file.urls, mods_dir, &file.path).await {
                warnings.push(format!("Failed to download {}: {}", file.path, e));
            } else {
                downloaded += 1;
            }
        }
    }
    
    if downloaded > 0 || blocked > 0 {
        tracing::info!(
            "Downloaded {} files, {} blocked mods",
            downloaded, blocked
        );
    }
    
    CurseForgeDownloadResult { warnings, blocked_files }
}

/// Download a file from a list of URLs (tries each in order)
async fn download_from_urls(
    urls: &[String],
    mods_dir: &PathBuf,
    relative_path: &str,
) -> Result<(), String> {
    let filename = std::path::Path::new(relative_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(relative_path);
    
    let dest_path = mods_dir.join(filename);
    
    for url in urls {
        match download_file(url, &dest_path, None).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                tracing::warn!("Failed to download from {}: {}", url, e);
            }
        }
    }
    
    Err(format!("All download URLs failed for {}", relative_path))
}
