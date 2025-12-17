//! Mod management commands

use super::state::AppState;
use crate::core::download::download_file;
use crate::core::modplatform::{
    curseforge::CurseForgeClient, 
    modrinth::ModrinthClient, 
    types::*,
};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Search result for mod browsing
#[derive(Debug, Clone, Serialize)]
pub struct ModSearchResult {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub downloads: u64,
    pub icon_url: Option<String>,
    pub project_type: String,
    pub platform: String,
}

/// Metadata stored for downloaded mods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModMetadata {
    pub mod_id: String,
    pub name: String,
    pub version: String,
    pub provider: String,
    pub icon_url: Option<String>,
}

/// Information about an installed mod
#[derive(Debug, Clone, Serialize)]
pub struct InstalledMod {
    pub filename: String,
    pub name: String,
    pub version: Option<String>,
    pub enabled: bool,
    pub size: u64,
    pub modified: Option<String>,
    pub provider: Option<String>,
    pub icon_url: Option<String>,
    pub homepage: Option<String>,
    pub issues_url: Option<String>,
    pub source_url: Option<String>,
}

#[tauri::command]
pub async fn search_mods(
    query: String,
    minecraft_version: String,
    mod_loader: String,
    platform: Option<String>,
) -> Result<Vec<ModSearchResult>, String> {
    let platform = platform.unwrap_or_else(|| "modrinth".to_string());
    let loaders = if mod_loader != "Vanilla" { 
        vec![mod_loader.to_lowercase()] 
    } else { 
        vec![] 
    };
    
    let search_query = SearchQuery {
        query: query.clone(),
        resource_type: Some(ResourceType::Mod),
        categories: vec![],
        game_versions: vec![minecraft_version.clone()],
        loaders: loaders.clone(),
        sort: SortOrder::Relevance,
        limit: 20,
        offset: 0,
    };
    
    match platform.to_lowercase().as_str() {
        "curseforge" => {
            let client = CurseForgeClient::new();
            if !client.has_api_key() {
                return Err("CurseForge API key not configured. Please add your API key in settings.".to_string());
            }
            
            let results = client.search(&search_query)
                .await
                .map_err(|e| format!("Failed to search CurseForge: {}", e))?;
            
            Ok(results.hits.into_iter().map(|hit| ModSearchResult {
                id: hit.id,
                name: hit.title,
                description: hit.description,
                author: hit.author,
                downloads: hit.downloads,
                icon_url: hit.icon_url,
                project_type: format!("{:?}", hit.resource_type),
                platform: "CurseForge".to_string(),
            }).collect())
        },
        _ => {
            let client = ModrinthClient::new();
            
            let results = client.search(&search_query)
                .await
                .map_err(|e| format!("Failed to search Modrinth: {}", e))?;
            
            Ok(results.hits.into_iter().map(|hit| ModSearchResult {
                id: hit.id,
                name: hit.title,
                description: hit.description,
                author: hit.author,
                downloads: hit.downloads,
                icon_url: hit.icon_url,
                project_type: format!("{:?}", hit.resource_type),
                platform: "Modrinth".to_string(),
            }).collect())
        }
    }
}

#[tauri::command]
pub async fn download_mod(
    state: State<'_, AppState>,
    instance_id: String,
    mod_id: String,
    platform: Option<String>,
) -> Result<(), String> {
    let platform = platform.unwrap_or_else(|| "modrinth".to_string());
    
    // Get instance
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let loader_name = instance.mod_loader.as_ref()
        .map(|ml| format!("{:?}", ml.loader_type).to_lowercase())
        .unwrap_or_else(|| "vanilla".to_string());
    
    let mods_dir = instance.mods_dir();
    std::fs::create_dir_all(&mods_dir).map_err(|e| format!("Failed to create mods directory: {}", e))?;
    
    match platform.to_lowercase().as_str() {
        "curseforge" => {
            let client = CurseForgeClient::new();
            if !client.has_api_key() {
                return Err("CurseForge API key not configured".to_string());
            }
            
            let mod_id_num: u32 = mod_id.parse()
                .map_err(|_| "Invalid CurseForge mod ID".to_string())?;
            
            let files = client.get_files(
                mod_id_num,
                Some(&instance.minecraft_version),
                Some(&loader_name),
            ).await.map_err(|e| format!("Failed to get mod files: {}", e))?;
            
            if files.is_empty() {
                return Err("No compatible mod version found".to_string());
            }
            
            let version = &files[0];
            if version.files.is_empty() {
                return Err("No files available for this mod".to_string());
            }
            
            let file = &version.files[0];
            
            let download_url = if file.url.is_empty() {
                let file_id: u32 = version.id.parse()
                    .map_err(|_| "Invalid file ID".to_string())?;
                client.get_download_url(mod_id_num, file_id)
                    .await
                    .map_err(|e| format!("Failed to get download URL: {}", e))?
            } else {
                file.url.clone()
            };
            
            let file_path = mods_dir.join(&file.filename);
            
            download_file(&download_url, &file_path, None)
                .await
                .map_err(|e| format!("Failed to download mod: {}", e))?;
            
            // Save metadata
            let metadata = ModMetadata {
                mod_id: mod_id.clone(),
                name: version.name.clone(),
                version: version.version_number.clone(),
                provider: "CurseForge".to_string(),
                icon_url: None,
            };
            
            let metadata_path = mods_dir.join(format!("{}.metadata.json", file.filename));
            if let Ok(json) = serde_json::to_string_pretty(&metadata) {
                let _ = std::fs::write(metadata_path, json);
            }
        },
        _ => {
            let client = ModrinthClient::new();
            
            let versions = client.get_versions(
                &mod_id,
                Some(&[instance.minecraft_version.clone()]),
                Some(&[loader_name]),
            ).await.map_err(|e| format!("Failed to get mod versions: {}", e))?;
            
            if versions.is_empty() {
                return Err("No compatible mod version found".to_string());
            }
            
            let version = &versions[0];
            if version.files.is_empty() {
                return Err("No files available for this mod".to_string());
            }
            
            let file = &version.files[0];
            let file_path = mods_dir.join(&file.filename);
            
            download_file(&file.url, &file_path, None)
                .await
                .map_err(|e| format!("Failed to download mod: {}", e))?;
            
            // Save metadata
            let metadata = ModMetadata {
                mod_id: mod_id.clone(),
                name: version.name.clone(),
                version: version.version_number.clone(),
                provider: "Modrinth".to_string(),
                icon_url: None,
            };
            
            let metadata_path = mods_dir.join(format!("{}.metadata.json", file.filename));
            if let Ok(json) = serde_json::to_string_pretty(&metadata) {
                let _ = std::fs::write(metadata_path, json);
            }
        }
    }
    
    Ok(())
}

#[tauri::command]
pub async fn get_installed_mods(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<InstalledMod>, String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let mods_dir = instance.mods_dir();
    
    tracing::debug!("Loading installed mods from: {:?}", mods_dir);
    
    if !mods_dir.exists() {
        tracing::debug!("Mods directory does not exist");
        return Ok(Vec::new());
    }
    
    let mut mods = Vec::new();
    
    let entries: Vec<_> = std::fs::read_dir(&mods_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .collect();
    
    tracing::info!("Found {} files in mods directory", entries.len());
    
    for entry in entries {
        let path = entry.path();
        
        if path.is_file() {
            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            
            tracing::debug!("Processing file: {}", filename);
            
            // Skip metadata files
            if filename.ends_with(".metadata.json") {
                tracing::debug!("Skipping metadata file: {}", filename);
                continue;
            }
            
            // Only process .jar files (enabled or disabled)
            if !filename.ends_with(".jar") && !filename.ends_with(".jar.disabled") {
                continue;
            }
            
            let enabled = !filename.ends_with(".disabled");
            let base_filename = filename.trim_end_matches(".disabled").to_string();
            
            let file_meta = entry.metadata().ok();
            let size = file_meta.as_ref().map(|m| m.len()).unwrap_or(0);
            let modified = file_meta.and_then(|m| m.modified().ok()).map(|t| {
                let datetime: chrono::DateTime<chrono::Utc> = t.into();
                datetime.format("%Y-%m-%d %H:%M").to_string()
            });
            
            // Try to load metadata from .metadata.json file
            let metadata_path = mods_dir.join(format!("{}.metadata.json", base_filename));
            let metadata: Option<ModMetadata> = if metadata_path.exists() {
                std::fs::read_to_string(&metadata_path)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
            } else {
                None
            };
            
            let (name, version, provider, icon_url, homepage, issues_url, source_url) = if let Some(meta) = metadata {
                (meta.name, Some(meta.version), Some(meta.provider), meta.icon_url, None, None, None)
            } else {
                // Try to parse mod metadata from JAR file
                use crate::core::modplatform::mod_parser::{parse_mod_jar, extract_mod_icon};
                
                let jar_path = if enabled {
                    mods_dir.join(&base_filename)
                } else {
                    mods_dir.join(format!("{}.disabled", base_filename))
                };
                
                if let Some(jar_details) = parse_mod_jar(&jar_path) {
                    tracing::debug!(
                        "Parsed mod '{}': name='{}', version='{}', homepage={:?}, issues={:?}, source={:?}, icon_path={:?}",
                        base_filename,
                        jar_details.name,
                        jar_details.version,
                        jar_details.homepage,
                        jar_details.issues_url,
                        jar_details.source_url,
                        jar_details.icon_path
                    );
                    
                    let name = if !jar_details.name.is_empty() {
                        jar_details.name
                    } else {
                        base_filename.trim_end_matches(".jar").to_string()
                    };
                    let version = if !jar_details.version.is_empty() && jar_details.version != "unknown" {
                        Some(jar_details.version)
                    } else {
                        None
                    };
                    
                    // Extract icon from JAR if available
                    let icon_url = extract_mod_icon(&jar_path);
                    if icon_url.is_some() {
                        tracing::info!("Successfully extracted icon for mod '{}'", base_filename);
                    } else {
                        tracing::info!("No icon found for mod '{}' (icon_path was: {:?})", base_filename, jar_details.icon_path);
                    }
                    
                    // Log URL extraction results
                    if jar_details.homepage.is_some() || jar_details.issues_url.is_some() || jar_details.source_url.is_some() {
                        tracing::info!(
                            "Mod '{}' URLs: homepage={:?}, issues={:?}, source={:?}",
                            base_filename,
                            jar_details.homepage,
                            jar_details.issues_url,
                            jar_details.source_url
                        );
                    } else {
                        tracing::info!("No URLs found in mod '{}'", base_filename);
                    }
                    
                    (name, version, jar_details.loader_type, icon_url, jar_details.homepage, jar_details.issues_url, jar_details.source_url)
                } else {
                    tracing::info!("Could not parse mod metadata from JAR: {}", base_filename);
                    let name = base_filename.trim_end_matches(".jar").to_string();
                    (name, None, None, None, None, None, None)
                }
            };
            
            tracing::info!(
                "Returning mod '{}': icon={}, homepage={}, issues={}, source={}",
                name,
                icon_url.is_some(),
                homepage.is_some(),
                issues_url.is_some(),
                source_url.is_some()
            );
            
            mods.push(InstalledMod {
                filename: base_filename,
                name,
                version,
                enabled,
                size,
                modified,
                provider,
                icon_url,
                homepage,
                issues_url,
                source_url,
            });
        }
    }
    
    mods.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    
    Ok(mods)
}

#[tauri::command]
pub async fn toggle_mod(
    state: State<'_, AppState>,
    instance_id: String,
    filename: String,
    enabled: bool,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let mods_dir = instance.mods_dir();
    let current_path = if enabled {
        mods_dir.join(format!("{}.disabled", filename))
    } else {
        mods_dir.join(&filename)
    };
    
    let new_path = if enabled {
        mods_dir.join(&filename)
    } else {
        mods_dir.join(format!("{}.disabled", filename))
    };
    
    std::fs::rename(current_path, new_path)
        .map_err(|e| format!("Failed to toggle mod: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn delete_mod(
    state: State<'_, AppState>,
    instance_id: String,
    filename: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let mods_dir = instance.mods_dir();
    let mod_path = mods_dir.join(&filename);
    let disabled_path = mods_dir.join(format!("{}.disabled", filename));
    let metadata_path = mods_dir.join(format!("{}.metadata.json", filename));
    
    let _ = std::fs::remove_file(mod_path);
    let _ = std::fs::remove_file(disabled_path);
    let _ = std::fs::remove_file(metadata_path);
    
    Ok(())
}

#[tauri::command]
pub async fn delete_mods(
    state: State<'_, AppState>,
    instance_id: String,
    filenames: Vec<String>,
) -> Result<(), String> {
    for filename in filenames {
        delete_mod(state.clone(), instance_id.clone(), filename).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn enable_mods(
    state: State<'_, AppState>,
    instance_id: String,
    filenames: Vec<String>,
) -> Result<(), String> {
    for filename in filenames {
        toggle_mod(state.clone(), instance_id.clone(), filename, true).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn disable_mods(
    state: State<'_, AppState>,
    instance_id: String,
    filenames: Vec<String>,
) -> Result<(), String> {
    for filename in filenames {
        toggle_mod(state.clone(), instance_id.clone(), filename, false).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn open_mods_folder(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let mods_dir = instance.mods_dir();
    std::fs::create_dir_all(&mods_dir).map_err(|e| e.to_string())?;
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&mods_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&mods_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&mods_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn open_configs_folder(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let config_dir = instance.game_dir().join("config");
    std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&config_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&config_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&config_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn add_local_mod(
    state: State<'_, AppState>,
    instance_id: String,
    file_path: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let mods_dir = instance.mods_dir();
    std::fs::create_dir_all(&mods_dir).map_err(|e| e.to_string())?;
    
    let source = std::path::Path::new(&file_path);
    let filename = source.file_name()
        .ok_or("Invalid file path")?
        .to_str()
        .ok_or("Invalid filename")?;
    
    let dest = mods_dir.join(filename);
    
    std::fs::copy(source, dest).map_err(|e| format!("Failed to copy mod: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn add_local_mod_from_bytes(
    state: State<'_, AppState>,
    instance_id: String,
    filename: String,
    data: String,
) -> Result<(), String> {
    use base64::{Engine as _, engine::general_purpose};
    
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let mods_dir = instance.mods_dir();
    std::fs::create_dir_all(&mods_dir).map_err(|e| e.to_string())?;
    
    // Decode base64 data
    let bytes = general_purpose::STANDARD.decode(data)
        .map_err(|e| format!("Failed to decode file data: {}", e))?;
    
    let dest = mods_dir.join(&filename);
    
    std::fs::write(dest, bytes)
        .map_err(|e| format!("Failed to write mod file: {}", e))?;
    
    Ok(())
}

// Enhanced mod search result with more details
#[derive(Debug, Clone, Serialize)]
pub struct ModSearchResultDetailed {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub downloads: u64,
    pub follows: u32,
    pub icon_url: Option<String>,
    pub project_type: String,
    pub platform: String,
    pub categories: Vec<String>,
    pub date_created: String,
    pub date_modified: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModDetailsResponse {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub body: String,
    pub author: String,
    pub downloads: u64,
    pub follows: u32,
    pub icon_url: Option<String>,
    pub source_url: Option<String>,
    pub issues_url: Option<String>,
    pub wiki_url: Option<String>,
    pub discord_url: Option<String>,
    pub donation_urls: Vec<DonationLink>,
    pub gallery: Vec<GalleryImageResponse>,
    pub categories: Vec<String>,
    pub versions: Vec<String>,
    pub loaders: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DonationLink {
    pub platform: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GalleryImageResponse {
    pub url: String,
    pub title: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModVersionResponse {
    pub id: String,
    pub version_number: String,
    pub name: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub date_published: String,
    pub downloads: u64,
    pub files: Vec<ModFileResponse>,
    pub dependencies: Vec<ModDependencyResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModFileResponse {
    pub filename: String,
    pub url: String,
    pub size: u64,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModDependencyResponse {
    pub project_id: String,
    pub dependency_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ModSearchResponse {
    pub mods: Vec<ModSearchResultDetailed>,
    pub total_hits: u32,
    pub offset: u32,
    pub limit: u32,
}

#[tauri::command]
pub async fn search_mods_detailed(
    query: String,
    minecraft_version: String,
    mod_loader: String,
    platform: String,
    sort_by: String,
    limit: u32,
    offset: Option<u32>,
    categories: Option<Vec<String>>,
    client_side: Option<String>,
    server_side: Option<String>,
) -> Result<ModSearchResponse, String> {
    use crate::core::modplatform::types::SortOrder;
    
    let offset_val = offset.unwrap_or(0);
    
    let loaders = if mod_loader != "vanilla" && !mod_loader.is_empty() { 
        vec![mod_loader.to_lowercase()] 
    } else { 
        vec![] 
    };
    
    let sort = match sort_by.as_str() {
        "relevance" => SortOrder::Relevance,
        "downloads" => SortOrder::Downloads,
        "follows" => SortOrder::Follows,
        "newest" => SortOrder::Newest,
        "updated" => SortOrder::Updated,
        _ => SortOrder::Downloads,
    };
    
    let category_list = categories.unwrap_or_default();
    
    let search_query = SearchQuery {
        query: query.clone(),
        resource_type: Some(ResourceType::Mod),
        categories: category_list,
        game_versions: vec![minecraft_version.clone()],
        loaders: loaders.clone(),
        sort,
        limit,
        offset: offset_val,
    };
    
    match platform.to_lowercase().as_str() {
        "curseforge" => {
            let client = CurseForgeClient::new();
            if !client.has_api_key() {
                return Err("CurseForge API key not configured. Please add your API key in settings.".to_string());
            }
            
            let results = client.search(&search_query)
                .await
                .map_err(|e| format!("Failed to search CurseForge: {}", e))?;
            
            Ok(ModSearchResponse {
                mods: results.hits.into_iter().map(|hit| ModSearchResultDetailed {
                    id: hit.id,
                    slug: hit.slug,
                    name: hit.title,
                    description: hit.description,
                    author: hit.author,
                    downloads: hit.downloads,
                    follows: hit.follows,
                    icon_url: hit.icon_url,
                    project_type: format!("{:?}", hit.resource_type),
                    platform: "curseforge".to_string(),
                    categories: hit.categories,
                    date_created: hit.date_created.to_rfc3339(),
                    date_modified: hit.date_modified.to_rfc3339(),
                }).collect(),
                total_hits: results.total_hits,
                offset: results.offset,
                limit: results.limit,
            })
        },
        _ => {
            let client = ModrinthClient::new();
            
            let results = client.search_with_environment(&search_query, client_side.as_deref(), server_side.as_deref())
                .await
                .map_err(|e| format!("Failed to search Modrinth: {}", e))?;
            
            Ok(ModSearchResponse {
                mods: results.hits.into_iter().map(|hit| ModSearchResultDetailed {
                    id: hit.id,
                    slug: hit.slug,
                    name: hit.title,
                    description: hit.description,
                    author: hit.author,
                    downloads: hit.downloads,
                    follows: hit.follows,
                    icon_url: hit.icon_url,
                    project_type: format!("{:?}", hit.resource_type),
                    platform: "modrinth".to_string(),
                    categories: hit.categories,
                    date_created: hit.date_created.to_rfc3339(),
                    date_modified: hit.date_modified.to_rfc3339(),
                }).collect(),
                total_hits: results.total_hits,
                offset: results.offset,
                limit: results.limit,
            })
        }
    }
}

#[tauri::command]
pub async fn get_mod_details(
    mod_id: String,
    platform: String,
) -> Result<ModDetailsResponse, String> {
    match platform.to_lowercase().as_str() {
        "curseforge" => {
            let client = CurseForgeClient::new();
            if !client.has_api_key() {
                return Err("CurseForge API key not configured".to_string());
            }
            
            let mod_id_num: u32 = mod_id.parse()
                .map_err(|_| "Invalid CurseForge mod ID".to_string())?;
            
            let project = client.get_mod(mod_id_num)
                .await
                .map_err(|e| format!("Failed to get mod details: {}", e))?;
            
            Ok(ModDetailsResponse {
                id: project.id,
                slug: project.slug,
                name: project.title,
                description: project.description,
                body: project.body,
                author: project.author,
                downloads: project.downloads,
                follows: project.followers,
                icon_url: project.icon_url,
                source_url: project.links.source,
                issues_url: project.links.issues,
                wiki_url: project.links.wiki,
                discord_url: project.links.discord,
                donation_urls: vec![],
                gallery: project.gallery.into_iter().map(|g| GalleryImageResponse {
                    url: g.url,
                    title: g.title.unwrap_or_default(),
                    description: g.description.unwrap_or_default(),
                }).collect(),
                categories: project.categories,
                versions: project.versions,
                loaders: project.loaders,
            })
        },
        _ => {
            let client = ModrinthClient::new();
            
            let project = client.get_project(&mod_id)
                .await
                .map_err(|e| format!("Failed to get mod details: {}", e))?;
            
            Ok(ModDetailsResponse {
                id: project.id,
                slug: project.slug,
                name: project.title,
                description: project.description,
                body: project.body,
                author: project.author,
                downloads: project.downloads,
                follows: project.followers,
                icon_url: project.icon_url,
                source_url: project.links.source,
                issues_url: project.links.issues,
                wiki_url: project.links.wiki,
                discord_url: project.links.discord,
                donation_urls: vec![],
                gallery: project.gallery.into_iter().map(|g| GalleryImageResponse {
                    url: g.url,
                    title: g.title.unwrap_or_default(),
                    description: g.description.unwrap_or_default(),
                }).collect(),
                categories: project.categories,
                versions: project.versions,
                loaders: project.loaders,
            })
        }
    }
}

#[tauri::command]
pub async fn get_mod_versions(
    mod_id: String,
    platform: String,
    minecraft_version: String,
    mod_loader: String,
) -> Result<Vec<ModVersionResponse>, String> {
    let loaders = if mod_loader != "vanilla" && !mod_loader.is_empty() { 
        vec![mod_loader.to_lowercase()] 
    } else { 
        vec![] 
    };
    
    match platform.to_lowercase().as_str() {
        "curseforge" => {
            let client = CurseForgeClient::new();
            if !client.has_api_key() {
                return Err("CurseForge API key not configured".to_string());
            }
            
            let mod_id_num: u32 = mod_id.parse()
                .map_err(|_| "Invalid CurseForge mod ID".to_string())?;
            
            let versions = client.get_files(
                mod_id_num,
                Some(&minecraft_version),
                loaders.first().map(|s| s.as_str()),
            ).await.map_err(|e| format!("Failed to get mod versions: {}", e))?;
            
            Ok(versions.into_iter().map(|v| ModVersionResponse {
                id: v.id,
                version_number: v.version_number,
                name: v.name,
                game_versions: v.game_versions,
                loaders: v.loaders,
                date_published: v.date_published.to_rfc3339(),
                downloads: v.downloads,
                files: v.files.into_iter().map(|f| ModFileResponse {
                    filename: f.filename,
                    url: f.url,
                    size: f.size,
                    primary: f.primary,
                }).collect(),
                dependencies: v.dependencies.into_iter()
                    .filter(|d| d.project_id.is_some())
                    .map(|d| ModDependencyResponse {
                        project_id: d.project_id.unwrap_or_default(),
                        dependency_type: match d.dependency_type {
                            crate::core::modplatform::types::DependencyType::Required => "required".to_string(),
                            crate::core::modplatform::types::DependencyType::Optional => "optional".to_string(),
                            crate::core::modplatform::types::DependencyType::Incompatible => "incompatible".to_string(),
                            crate::core::modplatform::types::DependencyType::Embedded => "embedded".to_string(),
                            crate::core::modplatform::types::DependencyType::Unknown => "unknown".to_string(),
                        },
                    }).collect(),
            }).collect())
        },
        _ => {
            let client = ModrinthClient::new();
            
            let versions = client.get_versions(
                &mod_id,
                Some(&[minecraft_version]),
                if loaders.is_empty() { None } else { Some(&loaders) },
            ).await.map_err(|e| format!("Failed to get mod versions: {}", e))?;
            
            Ok(versions.into_iter().map(|v| ModVersionResponse {
                id: v.id,
                version_number: v.version_number,
                name: v.name,
                game_versions: v.game_versions,
                loaders: v.loaders,
                date_published: v.date_published.to_rfc3339(),
                downloads: v.downloads,
                files: v.files.into_iter().map(|f| ModFileResponse {
                    filename: f.filename,
                    url: f.url,
                    size: f.size,
                    primary: f.primary,
                }).collect(),
                dependencies: v.dependencies.into_iter()
                    .filter(|d| d.project_id.is_some())
                    .map(|d| ModDependencyResponse {
                        project_id: d.project_id.unwrap_or_default(),
                        dependency_type: match d.dependency_type {
                            crate::core::modplatform::types::DependencyType::Required => "required".to_string(),
                            crate::core::modplatform::types::DependencyType::Optional => "optional".to_string(),
                            crate::core::modplatform::types::DependencyType::Incompatible => "incompatible".to_string(),
                            crate::core::modplatform::types::DependencyType::Embedded => "embedded".to_string(),
                            crate::core::modplatform::types::DependencyType::Unknown => "unknown".to_string(),
                        },
                    }).collect(),
            }).collect())
        }
    }
}

#[tauri::command]
pub async fn download_mod_version(
    state: State<'_, AppState>,
    instance_id: String,
    mod_id: String,
    version_id: String,
    platform: String,
) -> Result<(), String> {
    // Get instance
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let mods_dir = instance.mods_dir();
    std::fs::create_dir_all(&mods_dir).map_err(|e| format!("Failed to create mods directory: {}", e))?;
    
    match platform.to_lowercase().as_str() {
        "curseforge" => {
            let client = CurseForgeClient::new();
            if !client.has_api_key() {
                return Err("CurseForge API key not configured".to_string());
            }
            
            let mod_id_num: u32 = mod_id.parse()
                .map_err(|_| "Invalid CurseForge mod ID".to_string())?;
            let file_id: u32 = version_id.parse()
                .map_err(|_| "Invalid CurseForge file ID".to_string())?;
            
            let version = client.get_file(mod_id_num, file_id)
                .await
                .map_err(|e| format!("Failed to get file info: {}", e))?;
            
            if version.files.is_empty() {
                return Err("No files available for this version".to_string());
            }
            
            let file = &version.files[0];
            
            let download_url = if file.url.is_empty() {
                client.get_download_url(mod_id_num, file_id)
                    .await
                    .map_err(|e| format!("Failed to get download URL: {}", e))?
            } else {
                file.url.clone()
            };
            
            let file_path = mods_dir.join(&file.filename);
            
            download_file(&download_url, &file_path, None)
                .await
                .map_err(|e| format!("Failed to download mod: {}", e))?;
            
            // Save metadata
            let metadata = ModMetadata {
                mod_id: mod_id.clone(),
                name: version.name.clone(),
                version: version.version_number.clone(),
                provider: "CurseForge".to_string(),
                icon_url: None,
            };
            
            let metadata_path = mods_dir.join(format!("{}.metadata.json", file.filename));
            if let Ok(json) = serde_json::to_string_pretty(&metadata) {
                let _ = std::fs::write(metadata_path, json);
            }
        },
        _ => {
            let client = ModrinthClient::new();
            
            let version = client.get_version(&version_id)
                .await
                .map_err(|e| format!("Failed to get version info: {}", e))?;
            
            if version.files.is_empty() {
                return Err("No files available for this version".to_string());
            }
            
            let file = version.files.iter()
                .find(|f| f.primary)
                .unwrap_or(&version.files[0]);
            
            let file_path = mods_dir.join(&file.filename);
            
            download_file(&file.url, &file_path, None)
                .await
                .map_err(|e| format!("Failed to download mod: {}", e))?;
            
            // Save metadata
            let metadata = ModMetadata {
                mod_id: mod_id.clone(),
                name: version.name.clone(),
                version: version.version_number.clone(),
                provider: "Modrinth".to_string(),
                icon_url: None,
            };
            
            let metadata_path = mods_dir.join(format!("{}.metadata.json", file.filename));
            if let Ok(json) = serde_json::to_string_pretty(&metadata) {
                let _ = std::fs::write(metadata_path, json);
            }
        }
    }
    
    Ok(())
}
