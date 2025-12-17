//! Mod search commands

use crate::core::modplatform::{
    curseforge::CurseForgeClient, 
    modrinth::ModrinthClient, 
    types::*,
};
use super::types::*;

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
pub async fn get_mod_categories(
    platform: String,
) -> Result<Vec<String>, String> {
    match platform.to_lowercase().as_str() {
        "curseforge" => {
            Ok(vec![
                "Adventure".to_string(),
                "Magic".to_string(),
                "Tech".to_string(),
                "Storage".to_string(),
                "Library".to_string(),
                "Utility".to_string(),
                "World Gen".to_string(),
                "Cosmetic".to_string(),
                "Food".to_string(),
                "Armor".to_string(),
                "Mobs".to_string(),
            ])
        },
        _ => {
            Ok(vec![
                "adventure".to_string(),
                "cursed".to_string(),
                "decoration".to_string(),
                "economy".to_string(),
                "equipment".to_string(),
                "food".to_string(),
                "game-mechanics".to_string(),
                "library".to_string(),
                "magic".to_string(),
                "management".to_string(),
                "minigame".to_string(),
                "mobs".to_string(),
                "optimization".to_string(),
                "social".to_string(),
                "storage".to_string(),
                "technology".to_string(),
                "transportation".to_string(),
                "utility".to_string(),
                "worldgen".to_string(),
            ])
        }
    }
}
