//! Resource pack and shader pack management commands

use super::state::AppState;
use super::utils::format_file_size;
use crate::core::download::download_file;
use crate::core::modplatform::{
    curseforge::CurseForgeClient,
    modrinth::ModrinthClient,
    types::*,
};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Resource pack information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePackInfo {
    pub filename: String,
    pub name: String,
    pub description: Option<String>,
    pub size: String,
    pub enabled: bool,
}

/// Shader pack information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderPackInfo {
    pub filename: String,
    pub name: String,
    pub size: String,
}

/// List resource packs for an instance
#[tauri::command]
pub async fn list_resource_packs(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<ResourcePackInfo>, String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let resourcepacks_dir = instance.game_dir().join("resourcepacks");
    if !resourcepacks_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut packs = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(&resourcepacks_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let filename = entry.file_name().to_string_lossy().to_string();
            
            // Skip disabled packs marker files
            if filename.starts_with('.') {
                continue;
            }
            
            // Check if it's a ZIP or folder
            let is_valid = path.is_dir() || 
                filename.to_lowercase().ends_with(".zip");
            
            if is_valid {
                let size = if path.is_file() {
                    entry.metadata().map(|m| m.len()).unwrap_or(0)
                } else {
                    0
                };
                
                packs.push(ResourcePackInfo {
                    filename: filename.clone(),
                    name: filename.trim_end_matches(".zip").to_string(),
                    description: None,
                    size: format_file_size(size),
                    enabled: true,
                });
            }
        }
    }
    
    packs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(packs)
}

/// Delete a resource pack
#[tauri::command]
pub async fn delete_resource_pack(
    state: State<'_, AppState>,
    instance_id: String,
    filename: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let pack_path = instance.game_dir().join("resourcepacks").join(&filename);
    
    if !pack_path.exists() {
        return Err(format!("Resource pack '{}' not found", filename));
    }
    
    if pack_path.is_dir() {
        std::fs::remove_dir_all(&pack_path).map_err(|e| e.to_string())?;
    } else {
        std::fs::remove_file(&pack_path).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

/// List shader packs for an instance
#[tauri::command]
pub async fn list_shader_packs(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<ShaderPackInfo>, String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let shaderpacks_dir = instance.game_dir().join("shaderpacks");
    if !shaderpacks_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut packs = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(&shaderpacks_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let filename = entry.file_name().to_string_lossy().to_string();
            
            // Skip hidden files
            if filename.starts_with('.') {
                continue;
            }
            
            // Check if it's a ZIP or folder
            let is_valid = path.is_dir() || 
                filename.to_lowercase().ends_with(".zip");
            
            if is_valid {
                let size = if path.is_file() {
                    entry.metadata().map(|m| m.len()).unwrap_or(0)
                } else {
                    0
                };
                
                packs.push(ShaderPackInfo {
                    filename: filename.clone(),
                    name: filename.trim_end_matches(".zip").to_string(),
                    size: format_file_size(size),
                });
            }
        }
    }
    
    packs.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(packs)
}

/// Delete a shader pack
#[tauri::command]
pub async fn delete_shader_pack(
    state: State<'_, AppState>,
    instance_id: String,
    filename: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let pack_path = instance.game_dir().join("shaderpacks").join(&filename);
    
    if !pack_path.exists() {
        return Err(format!("Shader pack '{}' not found", filename));
    }
    
    if pack_path.is_dir() {
        std::fs::remove_dir_all(&pack_path).map_err(|e| e.to_string())?;
    } else {
        std::fs::remove_file(&pack_path).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

/// Open resource packs folder
#[tauri::command]
pub async fn open_resourcepacks_folder(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let resourcepacks_dir = instance.game_dir().join("resourcepacks");
    
    // Create if doesn't exist
    if !resourcepacks_dir.exists() {
        std::fs::create_dir_all(&resourcepacks_dir).map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&resourcepacks_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&resourcepacks_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&resourcepacks_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(())
}

/// Open shader packs folder
#[tauri::command]
pub async fn open_shaderpacks_folder(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let shaderpacks_dir = instance.game_dir().join("shaderpacks");
    
    // Create if doesn't exist
    if !shaderpacks_dir.exists() {
        std::fs::create_dir_all(&shaderpacks_dir).map_err(|e| e.to_string())?;
    }
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&shaderpacks_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&shaderpacks_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&shaderpacks_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(())
}

// ============================================================================
// Search and Download Functions for Resource Packs and Shaders
// ============================================================================

/// Search result for resource browsing
#[derive(Debug, Clone, Serialize)]
pub struct ResourceSearchResult {
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

/// Resource version response
#[derive(Debug, Clone, Serialize)]
pub struct ResourceVersionResponse {
    pub id: String,
    pub version_number: String,
    pub name: String,
    pub game_versions: Vec<String>,
    pub date_published: String,
    pub downloads: u64,
    pub files: Vec<ResourceFileResponse>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceFileResponse {
    pub filename: String,
    pub url: String,
    pub size: u64,
    pub primary: bool,
}

/// Resource details response
#[derive(Debug, Clone, Serialize)]
pub struct ResourceDetailsResponse {
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
    pub gallery: Vec<ResourceGalleryImage>,
    pub categories: Vec<String>,
    pub versions: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ResourceGalleryImage {
    pub url: String,
    pub title: String,
    pub description: String,
}

/// Search for resource packs
#[tauri::command]
pub async fn search_resource_packs(
    query: String,
    minecraft_version: String,
    platform: String,
    sort_by: String,
    limit: u32,
) -> Result<Vec<ResourceSearchResult>, String> {
    search_resources(query, minecraft_version, platform, sort_by, limit, ResourceType::ResourcePack).await
}

/// Search for shader packs
#[tauri::command]
pub async fn search_shader_packs(
    query: String,
    minecraft_version: String,
    platform: String,
    sort_by: String,
    limit: u32,
) -> Result<Vec<ResourceSearchResult>, String> {
    search_resources(query, minecraft_version, platform, sort_by, limit, ResourceType::ShaderPack).await
}

/// Internal search function for resources
async fn search_resources(
    query: String,
    minecraft_version: String,
    platform: String,
    sort_by: String,
    limit: u32,
    resource_type: ResourceType,
) -> Result<Vec<ResourceSearchResult>, String> {
    let sort = match sort_by.as_str() {
        "relevance" => SortOrder::Relevance,
        "downloads" => SortOrder::Downloads,
        "follows" => SortOrder::Follows,
        "newest" => SortOrder::Newest,
        "updated" => SortOrder::Updated,
        _ => SortOrder::Downloads,
    };
    
    let search_query = SearchQuery {
        query: query.clone(),
        resource_type: Some(resource_type),
        categories: vec![],
        game_versions: vec![minecraft_version.clone()],
        loaders: vec![],
        sort,
        limit,
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
            
            Ok(results.hits.into_iter().map(|hit| ResourceSearchResult {
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
            }).collect())
        },
        _ => {
            let client = ModrinthClient::new();
            
            let results = client.search(&search_query)
                .await
                .map_err(|e| format!("Failed to search Modrinth: {}", e))?;
            
            Ok(results.hits.into_iter().map(|hit| ResourceSearchResult {
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
            }).collect())
        }
    }
}

/// Get resource pack details
#[tauri::command]
pub async fn get_resource_pack_details(
    resource_id: String,
    platform: String,
) -> Result<ResourceDetailsResponse, String> {
    get_resource_details(resource_id, platform).await
}

/// Get shader pack details
#[tauri::command]
pub async fn get_shader_pack_details(
    resource_id: String,
    platform: String,
) -> Result<ResourceDetailsResponse, String> {
    get_resource_details(resource_id, platform).await
}

/// Internal function to get resource details
async fn get_resource_details(
    resource_id: String,
    platform: String,
) -> Result<ResourceDetailsResponse, String> {
    match platform.to_lowercase().as_str() {
        "curseforge" => {
            let client = CurseForgeClient::new();
            if !client.has_api_key() {
                return Err("CurseForge API key not configured".to_string());
            }
            
            let id_num: u32 = resource_id.parse()
                .map_err(|_| "Invalid CurseForge ID".to_string())?;
            
            let project = client.get_mod(id_num)
                .await
                .map_err(|e| format!("Failed to get details: {}", e))?;
            
            Ok(ResourceDetailsResponse {
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
                gallery: project.gallery.into_iter().map(|g| ResourceGalleryImage {
                    url: g.url,
                    title: g.title.unwrap_or_default(),
                    description: g.description.unwrap_or_default(),
                }).collect(),
                categories: project.categories,
                versions: project.versions,
            })
        },
        _ => {
            let client = ModrinthClient::new();
            
            let project = client.get_project(&resource_id)
                .await
                .map_err(|e| format!("Failed to get details: {}", e))?;
            
            Ok(ResourceDetailsResponse {
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
                gallery: project.gallery.into_iter().map(|g| ResourceGalleryImage {
                    url: g.url,
                    title: g.title.unwrap_or_default(),
                    description: g.description.unwrap_or_default(),
                }).collect(),
                categories: project.categories,
                versions: project.versions,
            })
        }
    }
}

/// Get resource pack versions
#[tauri::command]
pub async fn get_resource_pack_versions(
    resource_id: String,
    platform: String,
    minecraft_version: String,
) -> Result<Vec<ResourceVersionResponse>, String> {
    get_resource_versions(resource_id, platform, minecraft_version).await
}

/// Get shader pack versions
#[tauri::command]
pub async fn get_shader_pack_versions(
    resource_id: String,
    platform: String,
    minecraft_version: String,
) -> Result<Vec<ResourceVersionResponse>, String> {
    get_resource_versions(resource_id, platform, minecraft_version).await
}

/// Internal function to get resource versions
async fn get_resource_versions(
    resource_id: String,
    platform: String,
    minecraft_version: String,
) -> Result<Vec<ResourceVersionResponse>, String> {
    match platform.to_lowercase().as_str() {
        "curseforge" => {
            let client = CurseForgeClient::new();
            if !client.has_api_key() {
                return Err("CurseForge API key not configured".to_string());
            }
            
            let id_num: u32 = resource_id.parse()
                .map_err(|_| "Invalid CurseForge ID".to_string())?;
            
            let versions = client.get_files(id_num, Some(&minecraft_version), None)
                .await
                .map_err(|e| format!("Failed to get versions: {}", e))?;
            
            Ok(versions.into_iter().map(|v| ResourceVersionResponse {
                id: v.id,
                version_number: v.version_number,
                name: v.name,
                game_versions: v.game_versions,
                date_published: v.date_published.to_rfc3339(),
                downloads: v.downloads,
                files: v.files.into_iter().map(|f| ResourceFileResponse {
                    filename: f.filename,
                    url: f.url,
                    size: f.size,
                    primary: f.primary,
                }).collect(),
            }).collect())
        },
        _ => {
            let client = ModrinthClient::new();
            
            let versions = client.get_versions(
                &resource_id,
                Some(&[minecraft_version]),
                None,
            ).await.map_err(|e| format!("Failed to get versions: {}", e))?;
            
            Ok(versions.into_iter().map(|v| ResourceVersionResponse {
                id: v.id,
                version_number: v.version_number,
                name: v.name,
                game_versions: v.game_versions,
                date_published: v.date_published.to_rfc3339(),
                downloads: v.downloads,
                files: v.files.into_iter().map(|f| ResourceFileResponse {
                    filename: f.filename,
                    url: f.url,
                    size: f.size,
                    primary: f.primary,
                }).collect(),
            }).collect())
        }
    }
}

/// Download a resource pack version
#[tauri::command]
pub async fn download_resource_pack_version(
    state: State<'_, AppState>,
    instance_id: String,
    resource_id: String,
    version_id: String,
    platform: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let resourcepacks_dir = instance.game_dir().join("resourcepacks");
    std::fs::create_dir_all(&resourcepacks_dir).map_err(|e| format!("Failed to create directory: {}", e))?;
    
    download_resource(&resourcepacks_dir, resource_id, version_id, platform).await
}

/// Download a shader pack version
#[tauri::command]
pub async fn download_shader_pack_version(
    state: State<'_, AppState>,
    instance_id: String,
    resource_id: String,
    version_id: String,
    platform: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let shaderpacks_dir = instance.game_dir().join("shaderpacks");
    std::fs::create_dir_all(&shaderpacks_dir).map_err(|e| format!("Failed to create directory: {}", e))?;
    
    download_resource(&shaderpacks_dir, resource_id, version_id, platform).await
}

/// Internal function to download a resource
async fn download_resource(
    dest_dir: &std::path::Path,
    resource_id: String,
    version_id: String,
    platform: String,
) -> Result<(), String> {
    match platform.to_lowercase().as_str() {
        "curseforge" => {
            let client = CurseForgeClient::new();
            if !client.has_api_key() {
                return Err("CurseForge API key not configured".to_string());
            }
            
            let id_num: u32 = resource_id.parse()
                .map_err(|_| "Invalid CurseForge ID".to_string())?;
            let file_id: u32 = version_id.parse()
                .map_err(|_| "Invalid CurseForge file ID".to_string())?;
            
            let version = client.get_file(id_num, file_id)
                .await
                .map_err(|e| format!("Failed to get file info: {}", e))?;
            
            if version.files.is_empty() {
                return Err("No files available for this version".to_string());
            }
            
            let file = &version.files[0];
            
            let download_url = if file.url.is_empty() {
                client.get_download_url(id_num, file_id)
                    .await
                    .map_err(|e| format!("Failed to get download URL: {}", e))?
            } else {
                file.url.clone()
            };
            
            let file_path = dest_dir.join(&file.filename);
            
            download_file(&download_url, &file_path, None)
                .await
                .map_err(|e| format!("Failed to download: {}", e))?;
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
            
            let file_path = dest_dir.join(&file.filename);
            
            download_file(&file.url, &file_path, None)
                .await
                .map_err(|e| format!("Failed to download: {}", e))?;
        }
    }
    
    Ok(())
}

/// Add a local resource pack from file path
#[tauri::command]
pub async fn add_local_resource_pack(
    state: State<'_, AppState>,
    instance_id: String,
    file_path: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let resourcepacks_dir = instance.game_dir().join("resourcepacks");
    std::fs::create_dir_all(&resourcepacks_dir).map_err(|e| e.to_string())?;
    
    let source = std::path::Path::new(&file_path);
    let filename = source.file_name()
        .ok_or("Invalid file path")?
        .to_str()
        .ok_or("Invalid filename")?;
    
    let dest = resourcepacks_dir.join(filename);
    
    std::fs::copy(source, dest).map_err(|e| format!("Failed to copy file: {}", e))?;
    
    Ok(())
}

/// Add a local shader pack from file path
#[tauri::command]
pub async fn add_local_shader_pack(
    state: State<'_, AppState>,
    instance_id: String,
    file_path: String,
) -> Result<(), String> {
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let shaderpacks_dir = instance.game_dir().join("shaderpacks");
    std::fs::create_dir_all(&shaderpacks_dir).map_err(|e| e.to_string())?;
    
    let source = std::path::Path::new(&file_path);
    let filename = source.file_name()
        .ok_or("Invalid file path")?
        .to_str()
        .ok_or("Invalid filename")?;
    
    let dest = shaderpacks_dir.join(filename);
    
    std::fs::copy(source, dest).map_err(|e| format!("Failed to copy file: {}", e))?;
    
    Ok(())
}
