//! Modpack search and download commands

use crate::core::modplatform::{
    curseforge::CurseForgeClient, 
    modrinth::ModrinthClient, 
    types::*,
};
use serde::{Deserialize, Serialize};

/// Modpack search result
#[derive(Debug, Clone, Serialize)]
pub struct ModpackSearchResult {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub downloads: u64,
    pub follows: u32,
    pub icon_url: Option<String>,
    pub categories: Vec<String>,
    pub versions: Vec<String>,
    pub loaders: Vec<String>,
    pub date_created: String,
    pub date_modified: String,
    pub platform: String,
}

/// Modpack search response with pagination
#[derive(Debug, Clone, Serialize)]
pub struct ModpackSearchResponse {
    pub modpacks: Vec<ModpackSearchResult>,
    pub total_hits: u32,
    pub offset: u32,
    pub limit: u32,
}

/// Modpack version/file
#[derive(Debug, Clone, Serialize)]
pub struct ModpackVersion {
    pub id: String,
    pub project_id: String,
    pub name: String,
    pub version_number: String,
    pub changelog: Option<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub download_url: Option<String>,
    pub filename: String,
    pub size: u64,
    pub downloads: u64,
    pub date_published: String,
    pub version_type: String,
    pub platform: String,
}

/// Modpack details
#[derive(Debug, Clone, Serialize)]
pub struct ModpackDetails {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: String,
    pub body: String,
    pub author: String,
    pub icon_url: Option<String>,
    pub downloads: u64,
    pub followers: u32,
    pub categories: Vec<String>,
    pub versions: Vec<String>,
    pub loaders: Vec<String>,
    pub website_url: Option<String>,
    pub source_url: Option<String>,
    pub issues_url: Option<String>,
    pub wiki_url: Option<String>,
    pub discord_url: Option<String>,
    pub date_created: String,
    pub date_modified: String,
    pub platform: String,
}

/// Platform type for modpack search
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModpackPlatform {
    Modrinth,
    CurseForge,
    #[serde(rename = "atlauncher")]
    ATLauncher,
    Technic,
    #[serde(rename = "ftb-legacy")]
    FTBLegacy,
    #[serde(rename = "ftb-app")]
    FTBApp,
}

impl ModpackPlatform {
    pub fn display_name(&self) -> &'static str {
        match self {
            ModpackPlatform::Modrinth => "Modrinth",
            ModpackPlatform::CurseForge => "CurseForge",
            ModpackPlatform::ATLauncher => "ATLauncher",
            ModpackPlatform::Technic => "Technic",
            ModpackPlatform::FTBLegacy => "FTB Legacy",
            ModpackPlatform::FTBApp => "FTB App",
        }
    }
}

/// Search modpacks across platforms
#[tauri::command]
pub async fn search_modpacks(
    query: String,
    platform: String,
    minecraft_version: Option<String>,
    mod_loader: Option<String>,
    categories: Option<Vec<String>>,
    client_side: Option<String>,
    server_side: Option<String>,
    sort: Option<String>,
    offset: Option<u32>,
    limit: Option<u32>,
) -> Result<ModpackSearchResponse, String> {
    let offset = offset.unwrap_or(0);
    let limit = limit.unwrap_or(25);
    
    let sort_order = match sort.as_deref() {
        Some("downloads") => SortOrder::Downloads,
        Some("follows") | Some("popularity") => SortOrder::Follows,
        Some("newest") => SortOrder::Newest,
        Some("updated") => SortOrder::Updated,
        _ => SortOrder::Relevance,
    };
    
    let loaders = mod_loader
        .filter(|l| l != "Vanilla" && !l.is_empty())
        .map(|l| vec![l.to_lowercase()])
        .unwrap_or_default();
    
    let game_versions = minecraft_version
        .filter(|v| !v.is_empty())
        .map(|v| vec![v])
        .unwrap_or_default();
    
    let category_list = categories.unwrap_or_default();
    
    let search_query = SearchQuery {
        query: query.clone(),
        resource_type: Some(ResourceType::Modpack),
        categories: category_list.clone(),
        game_versions,
        loaders,
        sort: sort_order,
        offset,
        limit,
    };
    
    match platform.to_lowercase().as_str() {
        "curseforge" => search_curseforge_modpacks(&search_query).await,
        "modrinth" => search_modrinth_modpacks(&search_query, client_side, server_side).await,
        "atlauncher" => search_atlauncher_modpacks(&query, offset, limit).await,
        "technic" => search_technic_modpacks(&query, offset, limit).await,
        "ftb-legacy" => search_ftb_legacy_modpacks(&query, offset, limit).await,
        _ => Err(format!("Unsupported platform: {}", platform)),
    }
}

async fn search_modrinth_modpacks(
    query: &SearchQuery,
    client_side: Option<String>,
    server_side: Option<String>,
) -> Result<ModpackSearchResponse, String> {
    let client = ModrinthClient::new();
    
    // Build facets with environment filters
    let results = client.search_with_environment(query, client_side.as_deref(), server_side.as_deref())
        .await
        .map_err(|e| format!("Failed to search Modrinth: {}", e))?;
    
    Ok(ModpackSearchResponse {
        modpacks: results.hits.into_iter().map(|hit| ModpackSearchResult {
            id: hit.id,
            slug: hit.slug,
            name: hit.title,
            description: hit.description,
            author: hit.author,
            downloads: hit.downloads,
            follows: hit.follows,
            icon_url: hit.icon_url,
            categories: hit.categories,
            versions: hit.versions,
            loaders: hit.loaders,
            date_created: hit.date_created.to_rfc3339(),
            date_modified: hit.date_modified.to_rfc3339(),
            platform: "Modrinth".to_string(),
        }).collect(),
        total_hits: results.total_hits,
        offset: results.offset,
        limit: results.limit,
    })
}

async fn search_curseforge_modpacks(query: &SearchQuery) -> Result<ModpackSearchResponse, String> {
    let client = CurseForgeClient::new();
    
    if !client.has_api_key() {
        return Err("CurseForge API key not configured. Please add your API key in Settings > Advanced.".to_string());
    }
    
    let results = client.search(query)
        .await
        .map_err(|e| format!("Failed to search CurseForge: {}", e))?;
    
    Ok(ModpackSearchResponse {
        modpacks: results.hits.into_iter().map(|hit| ModpackSearchResult {
            id: hit.id,
            slug: hit.slug,
            name: hit.title,
            description: hit.description,
            author: hit.author,
            downloads: hit.downloads,
            follows: hit.follows,
            icon_url: hit.icon_url,
            categories: hit.categories,
            versions: hit.versions,
            loaders: hit.loaders,
            date_created: hit.date_created.to_rfc3339(),
            date_modified: hit.date_modified.to_rfc3339(),
            platform: "CurseForge".to_string(),
        }).collect(),
        total_hits: results.total_hits,
        offset: results.offset,
        limit: results.limit,
    })
}

// ATLauncher modpack search
async fn search_atlauncher_modpacks(query: &str, offset: u32, limit: u32) -> Result<ModpackSearchResponse, String> {
    let client = reqwest::Client::new();
    
    let response = client
        .get("https://download.nodecdn.net/containers/atl/launcher/json/packsnew.json")
        .header("User-Agent", format!("OxideLauncher/{}", env!("CARGO_PKG_VERSION")))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch ATLauncher packs: {}", e))?;
    
    let packs: Vec<ATLauncherPack> = response.json()
        .await
        .map_err(|e| format!("Failed to parse ATLauncher packs: {}", e))?;
    
    // Filter by query (case-insensitive)
    let query_lower = query.to_lowercase();
    let filtered: Vec<_> = packs.into_iter()
        .filter(|p| !p.system && p.pack_type == "public")
        .filter(|p| {
            if query.is_empty() {
                true
            } else {
                p.name.to_lowercase().contains(&query_lower) ||
                p.description.to_lowercase().contains(&query_lower)
            }
        })
        .collect();
    
    let total_hits = filtered.len() as u32;
    let paginated: Vec<_> = filtered.into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();
    
    Ok(ModpackSearchResponse {
        modpacks: paginated.into_iter().map(|pack| {
            let safe_name = pack.name.chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>()
                .to_lowercase();
            let icon_url = Some(format!("https://download.nodecdn.net/containers/atl/launcher/images/{}.png", safe_name));
            
            // Get latest version info
            let latest_version = pack.versions.first();
            let minecraft_version = latest_version.map(|v| v.minecraft.clone()).unwrap_or_default();
            
            ModpackSearchResult {
                id: pack.id.to_string(),
                slug: pack.safe_name.clone(),
                name: pack.name,
                description: pack.description,
                author: String::new(),
                downloads: 0,
                follows: 0,
                icon_url,
                categories: vec![],
                versions: if minecraft_version.is_empty() { vec![] } else { vec![minecraft_version] },
                loaders: vec![],
                date_created: String::new(),
                date_modified: String::new(),
                platform: "ATLauncher".to_string(),
            }
        }).collect(),
        total_hits,
        offset,
        limit,
    })
}

// Technic modpack search
async fn search_technic_modpacks(query: &str, offset: u32, limit: u32) -> Result<ModpackSearchResponse, String> {
    let client = reqwest::Client::new();
    
    // Use trending endpoint if no query, search endpoint otherwise
    let url = if query.is_empty() {
        format!("https://api.technicpack.net/trending?build=999")
    } else {
        format!("https://api.technicpack.net/search?build=999&q={}", urlencoding::encode(query))
    };
    
    let response = client
        .get(&url)
        .header("User-Agent", format!("OxideLauncher/{}", env!("CARGO_PKG_VERSION")))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch Technic packs: {}", e))?;
    
    let data: TechnicSearchResponse = response.json()
        .await
        .map_err(|e| format!("Failed to parse Technic response: {}", e))?;
    
    let packs = data.modpacks.unwrap_or_default();
    let total_hits = packs.len() as u32;
    
    let paginated: Vec<_> = packs.into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();
    
    Ok(ModpackSearchResponse {
        modpacks: paginated.into_iter().map(|pack| {
            ModpackSearchResult {
                id: pack.slug.clone(),
                slug: pack.slug,
                name: pack.name,
                description: pack.description.unwrap_or_default(),
                author: pack.author.unwrap_or_default(),
                downloads: 0,
                follows: 0,
                icon_url: pack.logo_url,
                categories: vec![],
                versions: pack.minecraft_version.map(|v| vec![v]).unwrap_or_default(),
                loaders: vec![],
                date_created: String::new(),
                date_modified: String::new(),
                platform: "Technic".to_string(),
            }
        }).collect(),
        total_hits,
        offset,
        limit,
    })
}

// FTB Legacy modpack search (using XML)
async fn search_ftb_legacy_modpacks(query: &str, offset: u32, limit: u32) -> Result<ModpackSearchResponse, String> {
    let client = reqwest::Client::new();
    
    // Fetch public modpacks XML
    let response = client
        .get("https://dist.creeper.host/FTB2/static/modpacks.xml")
        .header("User-Agent", format!("OxideLauncher/{}", env!("CARGO_PKG_VERSION")))
        .send()
        .await
        .map_err(|e| format!("Failed to fetch FTB Legacy packs: {}", e))?;
    
    let xml_text = response.text()
        .await
        .map_err(|e| format!("Failed to read FTB Legacy response: {}", e))?;
    
    // Parse XML using quick-xml
    let packs = parse_ftb_legacy_xml(&xml_text)?;
    
    // Filter by query
    let query_lower = query.to_lowercase();
    let filtered: Vec<_> = packs.into_iter()
        .filter(|p| {
            if query.is_empty() {
                true
            } else {
                p.name.to_lowercase().contains(&query_lower) ||
                p.description.to_lowercase().contains(&query_lower)
            }
        })
        .collect();
    
    let total_hits = filtered.len() as u32;
    let paginated: Vec<_> = filtered.into_iter()
        .skip(offset as usize)
        .take(limit as usize)
        .collect();
    
    Ok(ModpackSearchResponse {
        modpacks: paginated.into_iter().map(|pack| {
            ModpackSearchResult {
                id: pack.dir.clone(),
                slug: pack.dir.clone(),
                name: pack.name,
                description: pack.description,
                author: pack.author,
                downloads: 0,
                follows: 0,
                icon_url: Some(format!("https://dist.creeper.host/FTB2/static/{}.png", pack.logo)),
                categories: vec![],
                versions: if pack.mc_version.is_empty() { vec![] } else { vec![pack.mc_version] },
                loaders: vec![],
                date_created: String::new(),
                date_modified: String::new(),
                platform: "FTB Legacy".to_string(),
            }
        }).collect(),
        total_hits,
        offset,
        limit,
    })
}

fn parse_ftb_legacy_xml(xml: &str) -> Result<Vec<FTBLegacyPack>, String> {
    use quick_xml::Reader;
    use quick_xml::events::Event;
    
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    
    let mut packs = Vec::new();
    let mut buf = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) if e.name().as_ref() == b"modpack" => {
                let mut pack = FTBLegacyPack::default();
                
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let value = String::from_utf8_lossy(&attr.value).to_string();
                    
                    match key.as_str() {
                        "name" => pack.name = value,
                        "description" => pack.description = value,
                        "author" => pack.author = value,
                        "logo" => pack.logo = value,
                        "dir" => pack.dir = value,
                        "mcVersion" => pack.mc_version = value,
                        "version" => pack.current_version = value,
                        _ => {}
                    }
                }
                
                if !pack.name.is_empty() {
                    packs.push(pack);
                }
            }
            Ok(Event::Empty(e)) if e.name().as_ref() == b"modpack" => {
                let mut pack = FTBLegacyPack::default();
                
                for attr in e.attributes().flatten() {
                    let key = String::from_utf8_lossy(attr.key.as_ref()).to_string();
                    let value = String::from_utf8_lossy(&attr.value).to_string();
                    
                    match key.as_str() {
                        "name" => pack.name = value,
                        "description" => pack.description = value,
                        "author" => pack.author = value,
                        "logo" => pack.logo = value,
                        "dir" => pack.dir = value,
                        "mcVersion" => pack.mc_version = value,
                        "version" => pack.current_version = value,
                        _ => {}
                    }
                }
                
                if !pack.name.is_empty() {
                    packs.push(pack);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("Error parsing XML: {}", e)),
            _ => {}
        }
        buf.clear();
    }
    
    Ok(packs)
}

/// Get modpack details
#[tauri::command]
pub async fn get_modpack_details(
    modpack_id: String,
    platform: String,
) -> Result<ModpackDetails, String> {
    match platform.to_lowercase().as_str() {
        "curseforge" => get_curseforge_modpack_details(&modpack_id).await,
        "modrinth" => get_modrinth_modpack_details(&modpack_id).await,
        _ => Err(format!("Unsupported platform for details: {}", platform)),
    }
}

async fn get_modrinth_modpack_details(id: &str) -> Result<ModpackDetails, String> {
    let client = ModrinthClient::new();
    
    let project = client.get_project(id)
        .await
        .map_err(|e| format!("Failed to get modpack details: {}", e))?;
    
    Ok(ModpackDetails {
        id: project.id,
        slug: project.slug,
        name: project.title,
        description: project.description,
        body: project.body,
        author: project.author,
        icon_url: project.icon_url,
        downloads: project.downloads,
        followers: project.followers,
        categories: project.categories,
        versions: project.versions,
        loaders: project.loaders,
        website_url: project.links.website,
        source_url: project.links.source,
        issues_url: project.links.issues,
        wiki_url: project.links.wiki,
        discord_url: project.links.discord,
        date_created: project.date_created.to_rfc3339(),
        date_modified: project.date_modified.to_rfc3339(),
        platform: "Modrinth".to_string(),
    })
}

async fn get_curseforge_modpack_details(id: &str) -> Result<ModpackDetails, String> {
    let client = CurseForgeClient::new();
    
    if !client.has_api_key() {
        return Err("CurseForge API key not configured".to_string());
    }
    
    let mod_id: u32 = id.parse()
        .map_err(|_| "Invalid CurseForge modpack ID".to_string())?;
    
    let project = client.get_mod(mod_id)
        .await
        .map_err(|e| format!("Failed to get modpack details: {}", e))?;
    
    Ok(ModpackDetails {
        id: project.id,
        slug: project.slug,
        name: project.title,
        description: project.description,
        body: project.body,
        author: project.author,
        icon_url: project.icon_url,
        downloads: project.downloads,
        followers: project.followers,
        categories: project.categories,
        versions: project.versions,
        loaders: project.loaders,
        website_url: project.links.website,
        source_url: project.links.source,
        issues_url: project.links.issues,
        wiki_url: project.links.wiki,
        discord_url: project.links.discord,
        date_created: project.date_created.to_rfc3339(),
        date_modified: project.date_modified.to_rfc3339(),
        platform: "CurseForge".to_string(),
    })
}

/// Get modpack versions
#[tauri::command]
pub async fn get_modpack_versions(
    modpack_id: String,
    platform: String,
    minecraft_version: Option<String>,
    mod_loader: Option<String>,
) -> Result<Vec<ModpackVersion>, String> {
    match platform.to_lowercase().as_str() {
        "curseforge" => get_curseforge_modpack_versions(&modpack_id, minecraft_version.as_deref(), mod_loader.as_deref()).await,
        "modrinth" => get_modrinth_modpack_versions(&modpack_id, minecraft_version.as_deref(), mod_loader.as_deref()).await,
        _ => Err(format!("Unsupported platform for versions: {}", platform)),
    }
}

async fn get_modrinth_modpack_versions(
    id: &str,
    minecraft_version: Option<&str>,
    mod_loader: Option<&str>,
) -> Result<Vec<ModpackVersion>, String> {
    let client = ModrinthClient::new();
    
    let game_versions = minecraft_version.map(|v| vec![v.to_string()]);
    let loaders = mod_loader
        .filter(|l| *l != "Vanilla" && !l.is_empty())
        .map(|l| vec![l.to_lowercase()]);
    
    let versions = client.get_versions(
        id,
        game_versions.as_deref(),
        loaders.as_deref(),
    ).await.map_err(|e| format!("Failed to get modpack versions: {}", e))?;
    
    Ok(versions.into_iter().map(|v| {
        let primary_file = v.files.iter().find(|f| f.primary).or(v.files.first());
        
        ModpackVersion {
            id: v.id,
            project_id: v.project_id,
            name: v.name,
            version_number: v.version_number,
            changelog: v.changelog,
            game_versions: v.game_versions,
            loaders: v.loaders,
            download_url: primary_file.map(|f| f.url.clone()),
            filename: primary_file.map(|f| f.filename.clone()).unwrap_or_default(),
            size: primary_file.map(|f| f.size).unwrap_or(0),
            downloads: v.downloads,
            date_published: v.date_published.to_rfc3339(),
            version_type: format!("{:?}", v.version_type),
            platform: "Modrinth".to_string(),
        }
    }).collect())
}

async fn get_curseforge_modpack_versions(
    id: &str,
    minecraft_version: Option<&str>,
    mod_loader: Option<&str>,
) -> Result<Vec<ModpackVersion>, String> {
    let client = CurseForgeClient::new();
    
    if !client.has_api_key() {
        return Err("CurseForge API key not configured".to_string());
    }
    
    let mod_id: u32 = id.parse()
        .map_err(|_| "Invalid CurseForge modpack ID".to_string())?;
    
    let loader = mod_loader.filter(|l| *l != "Vanilla" && !l.is_empty());
    
    let versions = client.get_files(mod_id, minecraft_version, loader)
        .await
        .map_err(|e| format!("Failed to get modpack versions: {}", e))?;
    
    Ok(versions.into_iter().map(|v| {
        let primary_file = v.files.iter().find(|f| f.primary).or(v.files.first());
        
        ModpackVersion {
            id: v.id,
            project_id: v.project_id,
            name: v.name,
            version_number: v.version_number,
            changelog: v.changelog,
            game_versions: v.game_versions,
            loaders: v.loaders,
            download_url: primary_file.map(|f| f.url.clone()),
            filename: primary_file.map(|f| f.filename.clone()).unwrap_or_default(),
            size: primary_file.map(|f| f.size).unwrap_or(0),
            downloads: v.downloads,
            date_published: v.date_published.to_rfc3339(),
            version_type: format!("{:?}", v.version_type),
            platform: "CurseForge".to_string(),
        }
    }).collect())
}

// API response types

#[derive(Debug, serde::Deserialize)]
struct ATLauncherPack {
    id: u32,
    #[serde(default)]
    position: u32,
    name: String,
    #[serde(rename = "type", default)]
    pack_type: String,
    #[serde(default)]
    system: bool,
    #[serde(default)]
    description: String,
    #[serde(rename = "safeName", default)]
    safe_name: String,
    #[serde(default)]
    versions: Vec<ATLauncherPackVersion>,
}

#[derive(Debug, serde::Deserialize)]
struct ATLauncherPackVersion {
    version: String,
    minecraft: String,
}

#[derive(Debug, serde::Deserialize)]
struct TechnicSearchResponse {
    modpacks: Option<Vec<TechnicModpack>>,
}

#[derive(Debug, serde::Deserialize)]
struct TechnicModpack {
    slug: String,
    name: String,
    #[serde(rename = "displayName")]
    display_name: Option<String>,
    description: Option<String>,
    #[serde(rename = "iconUrl")]
    icon_url: Option<String>,
    #[serde(rename = "logoUrl")]
    logo_url: Option<String>,
    author: Option<String>,
    #[serde(rename = "websiteUrl")]
    website_url: Option<String>,
    #[serde(rename = "minecraftVersion")]
    minecraft_version: Option<String>,
}

#[derive(Debug, Default)]
struct FTBLegacyPack {
    name: String,
    description: String,
    author: String,
    logo: String,
    dir: String,
    mc_version: String,
    current_version: String,
}

/// Category info for UI
#[derive(Debug, Clone, Serialize)]
pub struct CategoryInfo {
    pub name: String,
    pub icon: String,
    pub project_type: String,
}

/// Get available categories for a project type from Modrinth
#[tauri::command]
pub async fn get_modpack_categories() -> Result<Vec<CategoryInfo>, String> {
    let client = ModrinthClient::new();
    
    let categories = client.get_categories()
        .await
        .map_err(|e| format!("Failed to get categories: {}", e))?;
    
    Ok(categories.into_iter()
        .filter(|c| c.project_type == "modpack" || c.project_type == "mod")
        .map(|c| CategoryInfo {
            name: c.name,
            icon: c.icon,
            project_type: c.project_type,
        })
        .collect())
}
