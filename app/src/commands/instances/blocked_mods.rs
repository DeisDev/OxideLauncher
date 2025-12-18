//! Blocked mods handling for CurseForge modpacks
//! 
//! Some CurseForge mods don't allow third-party launchers to download them directly.
//! This module handles detecting blocked mods and watching for manually downloaded files.

use crate::commands::state::AppState;
use crate::core::modplatform::curseforge::CurseForgeClient;
use crate::core::instance::{FileToDownload, PlatformFileInfo};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{State, AppHandle, Emitter};
use tokio::sync::Mutex;
use sha1::{Sha1, Digest};

/// Information about a blocked mod that needs manual download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedMod {
    /// Mod/project name
    pub name: String,
    /// URL to the mod's page on CurseForge
    pub website_url: String,
    /// Expected file hash (if available)
    pub hash: Option<String>,
    /// Hash algorithm (sha1, md5, etc.)
    pub hash_algo: Option<String>,
    /// Expected filename
    pub filename: String,
    /// CurseForge project ID
    pub project_id: u32,
    /// CurseForge file ID
    pub file_id: u32,
    /// Target folder relative to instance (e.g., "mods", "resourcepacks")
    pub target_folder: String,
    /// Whether this mod has been found
    pub matched: bool,
    /// Local path where the mod was found
    pub local_path: Option<String>,
}

/// Result of resolving blocked mods
#[allow(dead_code)] // Placeholder for future UI integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedModsResult {
    /// List of blocked mods that need manual download
    pub blocked_mods: Vec<BlockedMod>,
    /// Number of files that were downloaded successfully
    pub downloaded_count: usize,
    /// Total number of files to process
    pub total_count: usize,
}

/// State for tracking active blocked mods watchers
#[allow(dead_code)] // Placeholder for future file watcher feature
pub struct BlockedModsWatcherState {
    /// Map of instance ID to blocked mods being watched
    watchers: HashMap<String, Vec<BlockedMod>>,
    /// Active watcher handles
    watcher_handles: HashMap<String, notify::RecommendedWatcher>,
}

impl Default for BlockedModsWatcherState {
    fn default() -> Self {
        Self {
            watchers: HashMap::new(),
            watcher_handles: HashMap::new(),
        }
    }
}

/// Resolve blocked mod information from CurseForge API
/// Takes files that couldn't be downloaded and gets their project info
pub async fn resolve_blocked_mods(
    files: &[FileToDownload],
    target_folder: &str,
) -> Vec<BlockedMod> {
    let client = CurseForgeClient::new();
    let mut blocked_mods = Vec::new();
    
    if !client.has_api_key() {
        return blocked_mods;
    }
    
    for file in files {
        if let Some(ref platform_info) = file.platform_info {
            if platform_info.platform != "curseforge" {
                continue;
            }
            
            let project_id: u32 = match platform_info.project_id.parse() {
                Ok(id) => id,
                Err(_) => continue,
            };
            
            let file_id: u32 = match platform_info.file_id.parse() {
                Ok(id) => id,
                Err(_) => continue,
            };
            
            // Try to get download URL - if empty or error, it's blocked
            let is_blocked = match client.get_download_url(project_id, file_id).await {
                Ok(url) => url.is_empty(),
                Err(_) => true,
            };
            
            if !is_blocked {
                continue;
            }
            
            // Get project info for name and URL
            let (name, base_url) = match client.get_mod(project_id).await {
                Ok(project) => {
                    let url = project.links.website
                        .unwrap_or_else(|| format!("https://www.curseforge.com/minecraft/mc-mods/{}", project.slug));
                    (project.title, url)
                }
                Err(_) => (
                    format!("Unknown Mod ({})", project_id),
                    format!("https://www.curseforge.com/minecraft/mc-mods/{}", project_id),
                ),
            };
            
            // Construct direct file download URL (like Prism does)
            // This links to the exact file page instead of just the project page
            let website_url = format!("{}/download/{}", base_url, file_id);
            
            // Get file info for filename and hash
            let (filename, hash, hash_algo) = match client.get_file(project_id, file_id).await {
                Ok(version) => {
                    let filename = version.files.first()
                        .map(|f| f.filename.clone())
                        .unwrap_or_else(|| format!("{}.jar", file_id));
                    
                    // Get SHA1 hash from first file
                    let hash = version.files.first()
                        .and_then(|f| f.sha1.clone());
                    
                    (filename, hash, Some("sha1".to_string()))
                }
                Err(_) => (format!("{}.jar", file_id), None, None),
            };
            
            blocked_mods.push(BlockedMod {
                name,
                website_url,
                hash,
                hash_algo,
                filename,
                project_id,
                file_id,
                target_folder: target_folder.to_string(),
                matched: false,
                local_path: None,
            });
        }
    }
    
    blocked_mods
}

/// Scan a directory for files matching blocked mods
/// Uses both filename matching and hash verification
pub fn scan_for_blocked_mods(
    blocked_mods: &mut [BlockedMod],
    scan_dirs: &[PathBuf],
    recursive: bool,
) -> Vec<usize> {
    let mut matched_indices = Vec::new();
    
    for dir in scan_dirs {
        if !dir.exists() || !dir.is_dir() {
            continue;
        }
        
        let entries: Vec<_> = if recursive {
            walkdir::WalkDir::new(dir)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
                .map(|e| e.path().to_path_buf())
                .collect()
        } else {
            std::fs::read_dir(dir)
                .into_iter()
                .flatten()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().map(|t| t.is_file()).unwrap_or(false))
                .map(|e| e.path())
                .collect()
        };
        
        for entry_path in entries {
            let filename = match entry_path.file_name().and_then(|n| n.to_str()) {
                Some(f) => f,
                None => continue,
            };
            
            // Check each unmatched blocked mod
            for (idx, blocked_mod) in blocked_mods.iter_mut().enumerate() {
                if blocked_mod.matched {
                    continue;
                }
                
                // First check: filename match (case-insensitive)
                if !filename.eq_ignore_ascii_case(&blocked_mod.filename) 
                    && !lax_filename_compare(filename, &blocked_mod.filename) {
                    continue;
                }
                
                // Second check: hash verification (if we have a hash)
                if let (Some(ref expected_hash), Some(ref algo)) = (&blocked_mod.hash, &blocked_mod.hash_algo) {
                    if algo.to_lowercase() == "sha1" {
                        if let Ok(actual_hash) = compute_sha1_hash(&entry_path) {
                            if !expected_hash.eq_ignore_ascii_case(&actual_hash) {
                                tracing::debug!(
                                    "Hash mismatch for {}: expected {}, got {}",
                                    filename, expected_hash, actual_hash
                                );
                                continue;
                            }
                        }
                    }
                }
                
                // Match found!
                blocked_mod.matched = true;
                blocked_mod.local_path = Some(entry_path.to_string_lossy().to_string());
                matched_indices.push(idx);
                
                tracing::info!("Found blocked mod: {} at {:?}", blocked_mod.name, entry_path);
                break;
            }
        }
    }
    
    matched_indices
}

/// Lax filename comparison that ignores separators and case
fn lax_filename_compare(a: &str, b: &str) -> bool {
    let normalize = |s: &str| -> String {
        s.to_lowercase()
            .replace(['-', '+', '.', '_'], " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    
    normalize(a) == normalize(b)
}

/// Compute SHA1 hash of a file
fn compute_sha1_hash(path: &PathBuf) -> std::io::Result<String> {
    let data = std::fs::read(path)?;
    let mut hasher = Sha1::new();
    hasher.update(&data);
    let result = hasher.finalize();
    Ok(hex::encode(result))
}

/// Check if all blocked mods have been matched
pub fn all_mods_matched(blocked_mods: &[BlockedMod]) -> bool {
    blocked_mods.iter().all(|m| m.matched)
}

/// Copy matched blocked mods to their target folders in the instance
pub fn copy_matched_mods(
    blocked_mods: &[BlockedMod],
    instance_game_dir: &PathBuf,
) -> Result<Vec<String>, String> {
    let mut copied = Vec::new();
    
    for blocked_mod in blocked_mods {
        if !blocked_mod.matched {
            continue;
        }
        
        let local_path = match &blocked_mod.local_path {
            Some(p) => PathBuf::from(p),
            None => continue,
        };
        
        if !local_path.exists() {
            continue;
        }
        
        let target_dir = instance_game_dir.join(&blocked_mod.target_folder);
        std::fs::create_dir_all(&target_dir)
            .map_err(|e| format!("Failed to create target directory: {}", e))?;
        
        let target_path = target_dir.join(&blocked_mod.filename);
        
        // Copy the file
        std::fs::copy(&local_path, &target_path)
            .map_err(|e| format!("Failed to copy {}: {}", blocked_mod.filename, e))?;
        
        copied.push(blocked_mod.filename.clone());
        tracing::info!("Copied blocked mod {} to {:?}", blocked_mod.filename, target_path);
    }
    
    Ok(copied)
}

// =============================================================================
// Tauri Commands
// =============================================================================

/// Simple blocked file info from import result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimpleBlockedFileInfo {
    pub project_id: String,
    pub file_id: String,
    pub filename: String,
}

/// Resolve blocked files from simple info (project_id, file_id) to full BlockedMod info
/// This is called after import to get the full info needed for the BlockedModsDialog
#[tauri::command]
pub async fn resolve_blocked_files(
    blocked_files: Vec<SimpleBlockedFileInfo>,
    target_folder: String,
) -> Result<Vec<BlockedMod>, String> {
    let client = CurseForgeClient::new();
    let mut blocked_mods = Vec::new();
    
    if !client.has_api_key() {
        return Ok(blocked_mods);
    }
    
    for file in blocked_files {
        let project_id: u32 = match file.project_id.parse() {
            Ok(id) => id,
            Err(_) => continue,
        };
        
        let file_id: u32 = match file.file_id.parse() {
            Ok(id) => id,
            Err(_) => continue,
        };
        
        // Get project info for name and URL
        let (name, base_url) = match client.get_mod(project_id).await {
            Ok(project) => {
                let url = project.links.website
                    .unwrap_or_else(|| format!("https://www.curseforge.com/minecraft/mc-mods/{}", project.slug));
                (project.title, url)
            }
            Err(_) => (
                format!("Unknown Mod ({})", project_id),
                format!("https://www.curseforge.com/minecraft/mc-mods/{}", project_id),
            ),
        };
        
        // Construct direct file download URL (like Prism does)
        // This links to the exact file page instead of just the project page
        let website_url = format!("{}/download/{}", base_url, file_id);
        
        // Get file info for filename and hash
        let (filename, hash, hash_algo) = match client.get_file(project_id, file_id).await {
            Ok(version) => {
                let filename = version.files.first()
                    .map(|f| f.filename.clone())
                    .unwrap_or_else(|| file.filename.clone());
                
                // Get SHA1 hash from first file
                let hash = version.files.first()
                    .and_then(|f| f.sha1.clone());
                
                (filename, hash, Some("sha1".to_string()))
            }
            Err(_) => (file.filename.clone(), None, None),
        };
        
        blocked_mods.push(BlockedMod {
            name,
            website_url,
            hash,
            hash_algo,
            filename,
            project_id,
            file_id,
            target_folder: target_folder.to_string(),
            matched: false,
            local_path: None,
        });
    }
    
    Ok(blocked_mods)
}

/// Get list of blocked mods for files that couldn't be downloaded
#[tauri::command]
pub async fn get_blocked_mods_info(
    files: Vec<FileToDownloadInfo>,
    target_folder: String,
) -> Result<Vec<BlockedMod>, String> {
    let files_to_download: Vec<FileToDownload> = files.into_iter()
        .map(|f| f.into())
        .collect();
    
    Ok(resolve_blocked_mods(&files_to_download, &target_folder).await)
}

/// Simplified FileToDownload for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileToDownloadInfo {
    pub path: String,
    pub urls: Vec<String>,
    pub platform_info: Option<PlatformInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfo {
    pub platform: String,
    pub project_id: String,
    pub file_id: String,
}

impl From<FileToDownloadInfo> for FileToDownload {
    fn from(info: FileToDownloadInfo) -> Self {
        FileToDownload {
            path: info.path,
            urls: info.urls,
            size: 0,
            hash_sha1: None,
            hash_sha512: None,
            platform_info: info.platform_info.map(|p| PlatformFileInfo {
                platform: p.platform,
                project_id: p.project_id,
                file_id: p.file_id,
            }),
        }
    }
}

/// Scan downloads folder for blocked mods
#[tauri::command]
pub async fn scan_for_blocked_mod_files(
    state: State<'_, AppState>,
    blocked_mods: Vec<BlockedMod>,
    additional_paths: Vec<String>,
) -> Result<Vec<BlockedMod>, String> {
    let mut mods = blocked_mods;
    
    // Get configured downloads directory
    let downloads_dir = {
        let config = state.config.lock().unwrap();
        config.downloads_dir()
    };
    
    let recursive = {
        let config = state.config.lock().unwrap();
        config.network.downloads_dir_watch_recursive
    };
    
    // Build list of directories to scan
    let mut scan_dirs = vec![downloads_dir];
    for path in additional_paths {
        scan_dirs.push(PathBuf::from(path));
    }
    
    // Scan for matches
    scan_for_blocked_mods(&mut mods, &scan_dirs, recursive);
    
    Ok(mods)
}

/// Copy matched blocked mods to the instance
#[tauri::command]
pub async fn copy_blocked_mods_to_instance(
    state: State<'_, AppState>,
    instance_id: String,
    blocked_mods: Vec<BlockedMod>,
) -> Result<Vec<String>, String> {
    let instance_path = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .map(|i| i.path.clone())
            .ok_or_else(|| "Instance not found".to_string())?
    };
    
    let game_dir = instance_path.join(".minecraft");
    copy_matched_mods(&blocked_mods, &game_dir)
}

/// Start watching directories for blocked mod files
#[tauri::command]
pub async fn start_blocked_mods_watcher(
    app: AppHandle,
    state: State<'_, AppState>,
    session_id: String,
    blocked_mods: Vec<BlockedMod>,
    additional_paths: Vec<String>,
) -> Result<(), String> {
    use notify::{Watcher, RecursiveMode, Event, EventKind};
    
    let downloads_dir = {
        let config = state.config.lock().unwrap();
        config.downloads_dir()
    };
    
    let recursive = {
        let config = state.config.lock().unwrap();
        config.network.downloads_dir_watch_recursive
    };
    
    // Build watch paths
    let mut watch_paths = vec![downloads_dir];
    for path in additional_paths {
        watch_paths.push(PathBuf::from(path));
    }
    
    // Create shared state for the watcher
    let blocked_mods = Arc::new(Mutex::new(blocked_mods));
    let watch_paths_clone = watch_paths.clone();
    let app_clone = app.clone();
    let session_id_clone = session_id.clone();
    let blocked_mods_clone = blocked_mods.clone();
    
    // Create watcher
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            match event.kind {
                EventKind::Create(_) | EventKind::Modify(_) => {
                    // Spawn a task to scan for new matches
                    let blocked_mods = blocked_mods_clone.clone();
                    let watch_paths = watch_paths_clone.clone();
                    let app = app_clone.clone();
                    let session_id = session_id_clone.clone();
                    
                    tokio::spawn(async move {
                        let mut mods = blocked_mods.lock().await;
                        let matched = scan_for_blocked_mods(&mut mods, &watch_paths, true);
                        
                        if !matched.is_empty() {
                            // Emit event with updated mods
                            let _ = app.emit("blocked-mods-updated", serde_json::json!({
                                "session_id": session_id,
                                "blocked_mods": mods.clone(),
                                "all_matched": all_mods_matched(&mods),
                            }));
                        }
                    });
                }
                _ => {}
            }
        }
    }).map_err(|e| format!("Failed to create watcher: {}", e))?;
    
    // Watch the directories
    let mode = if recursive { RecursiveMode::Recursive } else { RecursiveMode::NonRecursive };
    for path in &watch_paths {
        if path.exists() {
            watcher.watch(path, mode)
                .map_err(|e| format!("Failed to watch {}: {}", path.display(), e))?;
        }
    }
    
    // Do an initial scan
    {
        let mut mods = blocked_mods.lock().await;
        scan_for_blocked_mods(&mut mods, &watch_paths, recursive);
        
        // Emit initial state
        let _ = app.emit("blocked-mods-updated", serde_json::json!({
            "session_id": session_id,
            "blocked_mods": mods.clone(),
            "all_matched": all_mods_matched(&mods),
        }));
    }
    
    // Keep watcher alive - in a real app you'd store this somewhere
    // For now we'll leak it (not ideal but works for the demo)
    std::mem::forget(watcher);
    
    Ok(())
}

/// Get the configured downloads directory
#[tauri::command]
pub async fn get_downloads_dir(
    state: State<'_, AppState>,
) -> Result<String, String> {
    let downloads_dir = {
        let config = state.config.lock().unwrap();
        config.downloads_dir()
    };
    
    Ok(downloads_dir.to_string_lossy().to_string())
}
