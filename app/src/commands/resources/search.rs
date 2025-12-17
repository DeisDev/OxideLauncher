//! Search and details commands for resource packs and shader packs

use super::types::{
    ResourceDetailsResponse, ResourceFileResponse, ResourceGalleryImage, 
    ResourceSearchResponse, ResourceSearchResult, ResourceVersionResponse,
};
use crate::core::modplatform::{
    curseforge::CurseForgeClient,
    modrinth::ModrinthClient,
    types::*,
};

/// Search for resource packs
#[tauri::command]
pub async fn search_resource_packs(
    query: String,
    minecraft_version: String,
    platform: String,
    sort_by: String,
    limit: u32,
    offset: Option<u32>,
) -> Result<ResourceSearchResponse, String> {
    search_resources(query, minecraft_version, platform, sort_by, limit, offset, ResourceType::ResourcePack).await
}

/// Search for shader packs
#[tauri::command]
pub async fn search_shader_packs(
    query: String,
    minecraft_version: String,
    platform: String,
    sort_by: String,
    limit: u32,
    offset: Option<u32>,
) -> Result<ResourceSearchResponse, String> {
    search_resources(query, minecraft_version, platform, sort_by, limit, offset, ResourceType::ShaderPack).await
}

/// Internal search function for resources
async fn search_resources(
    query: String,
    minecraft_version: String,
    platform: String,
    sort_by: String,
    limit: u32,
    offset: Option<u32>,
    resource_type: ResourceType,
) -> Result<ResourceSearchResponse, String> {
    let offset_val = offset.unwrap_or(0);
    
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
            
            Ok(ResourceSearchResponse {
                resources: results.hits.into_iter().map(|hit| ResourceSearchResult {
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
            
            let results = client.search(&search_query)
                .await
                .map_err(|e| format!("Failed to search Modrinth: {}", e))?;
            
            Ok(ResourceSearchResponse {
                resources: results.hits.into_iter().map(|hit| ResourceSearchResult {
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
