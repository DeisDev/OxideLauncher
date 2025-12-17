//! Instance import/export commands

use crate::commands::state::AppState;
use crate::core::config::Config;
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
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tauri::{State, AppHandle, Emitter};

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

/// Download progress event for modpack installation
#[derive(Debug, Clone, Serialize)]
pub struct ModpackDownloadProgress {
    /// Number of files downloaded so far
    pub downloaded: usize,
    /// Total number of files to download
    pub total: usize,
    /// Bytes downloaded so far across all files
    pub bytes_downloaded: u64,
    /// Current download speed in bytes per second
    pub speed_bps: u64,
    /// Name of the file currently being downloaded (if any)
    pub current_file: Option<String>,
}

/// Detected import type info
#[derive(Debug, Clone, Serialize)]
pub struct ImportTypeInfo {
    pub format_type: String,
    pub display_name: String,
}

// =============================================================================
// Helper Functions
// =============================================================================

/// Download an icon from a URL and save it to the instance directory
async fn download_icon(client: &reqwest::Client, url: &str, instance_path: &std::path::Path) -> Result<String, String> {
    // Download the icon
    let response = client.get(url)
        .send()
        .await
        .map_err(|e| format!("Failed to download icon: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Icon download failed with status: {}", response.status()));
    }
    
    // Get content type to determine extension
    let content_type = response.headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("image/png");
    
    let ext = match content_type {
        t if t.contains("png") => "png",
        t if t.contains("jpeg") || t.contains("jpg") => "jpg",
        t if t.contains("gif") => "gif",
        t if t.contains("webp") => "webp",
        _ => "png",
    };
    
    let bytes = response.bytes()
        .await
        .map_err(|e| format!("Failed to read icon data: {}", e))?;
    
    // Save the icon
    let icon_filename = format!("icon.{}", ext);
    let icon_path = instance_path.join(&icon_filename);
    
    std::fs::write(&icon_path, &bytes)
        .map_err(|e| format!("Failed to save icon: {}", e))?;
    
    tracing::info!("Downloaded icon to {:?}", icon_path);
    Ok(icon_filename)
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
    app: AppHandle,
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
        // Pass game_dir and let the function determine correct subdirectory for each file
        let dl_result = download_curseforge_files(&result.files_to_download, &game_dir, Some(&app)).await;
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
    app: AppHandle,
    url: String,
    name_override: Option<String>,
    icon_url: Option<String>,
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
    
    // Create HTTP client
    let client = reqwest::Client::new();
    
    // Download the file
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
    
    // Download and save icon if provided
    let downloaded_icon = if let Some(ref icon_url_str) = icon_url {
        match download_icon(&client, icon_url_str, &instance_path).await {
            Ok(icon_filename) => Some(format!("custom:{}", icon_filename)),
            Err(e) => {
                tracing::warn!("Failed to download icon: {}", e);
                None
            }
        }
    } else {
        None
    };
    
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
    
    // Determine icon - prefer downloaded icon, then result icon, then default
    let icon = downloaded_icon
        .or_else(|| result.icon.as_ref().map(|i| {
            match i {
                crate::core::instance::OxideIcon::Default { name } => name.clone(),
                crate::core::instance::OxideIcon::Custom { filename, .. } => format!("custom:{}", filename),
            }
        }))
        .unwrap_or_else(|| "grass".to_string());
    
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
        // Pass game_dir and let the function determine correct subdirectory for each file
        let dl_result = download_curseforge_files(&result.files_to_download, &game_dir, Some(&app)).await;
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
/// game_dir is the .minecraft directory, files will be placed in appropriate subdirectories
/// based on their path (e.g., mods/, resourcepacks/, shaderpacks/)
async fn download_curseforge_files(
    files: &[FileToDownload],
    game_dir: &PathBuf,
    app: Option<&AppHandle>,
) -> CurseForgeDownloadResult {
    let mut warnings = Vec::new();
    let mut blocked_files = Vec::new();
    let client = CurseForgeClient::new();
    
    if !client.has_api_key() {
        warnings.push("CurseForge API key not configured - cannot download modpack files".to_string());
        return CurseForgeDownloadResult { warnings, blocked_files };
    }
    
    // Phase 1: Resolve all download URLs in parallel
    // Collect tasks: (url, dest_path, is_blocked, blocked_info)
    let resolve_futures: Vec<_> = files.iter().map(|file| {
        let game_dir = game_dir.clone();
        let file = file.clone();
        
        async move {
            // Create client inside async block since CurseForgeClient isn't Clone
            let client = CurseForgeClient::new();
            
            // Determine the target directory based on the file path
            let target_dir = if let Some(parent) = std::path::Path::new(&file.path).parent() {
                let parent_str = parent.to_string_lossy();
                if !parent_str.is_empty() {
                    game_dir.join(parent_str.as_ref())
                } else {
                    game_dir.join("mods")
                }
            } else {
                game_dir.join("mods")
            };
            
            // Ensure target directory exists
            if let Err(e) = std::fs::create_dir_all(&target_dir) {
                return Err(format!("Failed to create directory {}: {}", target_dir.display(), e));
            }
            
            if let Some(ref platform_info) = file.platform_info {
                if platform_info.platform != "curseforge" {
                    // Non-CurseForge file with URLs
                    if !file.urls.is_empty() {
                        let filename = std::path::Path::new(&file.path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&file.path);
                        return Ok(Some((file.urls[0].clone(), target_dir.join(filename), None)));
                    }
                    return Ok(None);
                }
                
                let project_id: u32 = match platform_info.project_id.parse() {
                    Ok(id) => id,
                    Err(_) => return Err(format!("Invalid project ID: {}", platform_info.project_id)),
                };
                
                let file_id: u32 = match platform_info.file_id.parse() {
                    Ok(id) => id,
                    Err(_) => return Err(format!("Invalid file ID: {}", platform_info.file_id)),
                };
                
                // Get download URL
                match client.get_download_url(project_id, file_id).await {
                    Ok(download_url) if !download_url.is_empty() => {
                        // Get filename
                        let filename = match client.get_file(project_id, file_id).await {
                            Ok(file_info) => file_info.files.first()
                                .map(|f| f.filename.clone())
                                .unwrap_or_else(|| format!("{}.jar", file_id)),
                            Err(_) => format!("{}.jar", file_id),
                        };
                        Ok(Some((download_url, target_dir.join(&filename), None)))
                    }
                    Ok(_) | Err(_) => {
                        // Blocked or error - get filename for blocked_files
                        let filename = match client.get_file(project_id, file_id).await {
                            Ok(file_info) => file_info.files.first()
                                .map(|f| f.filename.clone())
                                .unwrap_or_else(|| format!("{}.jar", file_id)),
                            Err(_) => format!("{}.jar", file_id),
                        };
                        
                        let blocked_info = BlockedFileInfo {
                            project_id: project_id.to_string(),
                            file_id: file_id.to_string(),
                            filename,
                        };
                        Ok(Some(("".to_string(), PathBuf::new(), Some(blocked_info))))
                    }
                }
            } else if !file.urls.is_empty() {
                let filename = std::path::Path::new(&file.path)
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or(&file.path);
                Ok(Some((file.urls[0].clone(), target_dir.join(filename), None)))
            } else {
                Ok(None)
            }
        }
    }).collect();
    
    // Run URL resolution in parallel (limited concurrency)
    let resolved: Vec<_> = futures::future::join_all(resolve_futures).await;
    
    // Collect download tasks and blocked files
    let mut download_tasks = Vec::new();
    
    for result in resolved {
        match result {
            Ok(Some((url, dest, blocked_info))) => {
                if let Some(info) = blocked_info {
                    blocked_files.push(info);
                } else if !url.is_empty() {
                    download_tasks.push((url, dest));
                }
            }
            Ok(None) => {}
            Err(e) => warnings.push(e),
        }
    }
    
    // Phase 2: Download all files in parallel
    // Use a semaphore to limit concurrent downloads (respect config setting)
    let config = Config::load().unwrap_or_default();
    let max_concurrent = config.network.max_concurrent_downloads;
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(max_concurrent));
    
    // Progress tracking
    let total_files = download_tasks.len();
    let downloaded_count = Arc::new(AtomicUsize::new(0));
    let bytes_downloaded = Arc::new(AtomicU64::new(0));
    let start_time = Instant::now();
    
    // Emit initial progress
    if let Some(app) = app {
        let _ = app.emit("modpack-download-progress", ModpackDownloadProgress {
            downloaded: 0,
            total: total_files,
            bytes_downloaded: 0,
            speed_bps: 0,
            current_file: None,
        });
    }
    
    let download_futures: Vec<_> = download_tasks.into_iter().map(|(url, dest)| {
        let sem = semaphore.clone();
        let downloaded_count = downloaded_count.clone();
        let bytes_downloaded = bytes_downloaded.clone();
        let app_opt = app.cloned();
        let total = total_files;
        
        async move {
            let _permit = sem.acquire().await.unwrap();
            let filename = dest.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown")
                .to_string();
            
            match download_file(&url, &dest, None).await {
                Ok(_) => {
                    // Update counters
                    let new_count = downloaded_count.fetch_add(1, Ordering::SeqCst) + 1;
                    
                    // Approximate bytes - we don't have exact size info here
                    // Get actual file size after download
                    if let Ok(metadata) = std::fs::metadata(&dest) {
                        bytes_downloaded.fetch_add(metadata.len(), Ordering::SeqCst);
                    }
                    
                    // Emit progress event
                    if let Some(app) = &app_opt {
                        let elapsed = start_time.elapsed().as_secs_f64();
                        let total_bytes = bytes_downloaded.load(Ordering::SeqCst);
                        let speed = if elapsed > 0.0 { (total_bytes as f64 / elapsed) as u64 } else { 0 };
                        
                        let _ = app.emit("modpack-download-progress", ModpackDownloadProgress {
                            downloaded: new_count,
                            total,
                            bytes_downloaded: total_bytes,
                            speed_bps: speed,
                            current_file: Some(filename.clone()),
                        });
                    }
                    
                    tracing::debug!("Downloaded: {}", dest.display());
                    Ok(())
                }
                Err(e) => Err(format!("Failed to download {}: {}", dest.display(), e)),
            }
        }
    }).collect();
    
    let download_results: Vec<_> = futures::future::join_all(download_futures).await;
    
    let mut downloaded = 0;
    for result in download_results {
        match result {
            Ok(_) => downloaded += 1,
            Err(e) => warnings.push(e),
        }
    }
    
    let blocked = blocked_files.len();
    
    if downloaded > 0 || blocked > 0 {
        tracing::info!(
            "Downloaded {} files in parallel, {} blocked mods",
            downloaded, blocked
        );
    }
    
    // Emit final progress
    if let Some(app) = app {
        let elapsed = start_time.elapsed().as_secs_f64();
        let total_bytes = bytes_downloaded.load(Ordering::SeqCst);
        let speed = if elapsed > 0.0 { (total_bytes as f64 / elapsed) as u64 } else { 0 };
        
        let _ = app.emit("modpack-download-progress", ModpackDownloadProgress {
            downloaded,
            total: total_files,
            bytes_downloaded: total_bytes,
            speed_bps: speed,
            current_file: None,
        });
    }
    
    CurseForgeDownloadResult { warnings, blocked_files }
}
