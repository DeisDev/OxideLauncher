//! Tauri command handlers for the frontend

use crate::core::{
    accounts::Account,
    config::Config,
    instance::{setup_instance, Instance},
    minecraft::version::{fetch_version_manifest, VersionType},
    modloaders,
};
use tauri::Emitter;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::process::Child;
use tauri::State;

// Running process information
pub struct RunningProcess {
    pub child: Child,
    pub logs: Arc<Mutex<Vec<String>>>,
}

// Application state
pub struct AppState {
    pub instances: Mutex<Vec<Instance>>,
    pub accounts: Mutex<Vec<Account>>,
    pub config: Mutex<Config>,
    pub data_dir: PathBuf,
    pub running_processes: Mutex<HashMap<String, Arc<Mutex<RunningProcess>>>>,
}

impl AppState {
    pub fn new() -> Self {
        let data_dir = dirs::data_dir()
            .expect("Failed to get data directory")
            .join("OxideLauncher");
        
        // Ensure data directory exists
        std::fs::create_dir_all(&data_dir).expect("Failed to create data directory");
        
        Self {
            instances: Mutex::new(Vec::new()),
            accounts: Mutex::new(Vec::new()),
            config: Mutex::new(Config::default()),
            data_dir,
            running_processes: Mutex::new(HashMap::new()),
        }
    }
}

// Instance Commands

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub id: String,
    pub name: String,
    pub minecraft_version: String,
    pub mod_loader: String,
    pub mod_loader_version: Option<String>,
    pub icon: Option<String>,
    pub last_played: Option<String>,
    pub total_played_seconds: u64,
}

#[tauri::command]
pub async fn get_instances(state: State<'_, AppState>) -> Result<Vec<InstanceInfo>, String> {
    let instances = state.instances.lock().unwrap();
    let info: Vec<InstanceInfo> = instances.iter().map(|inst| {
        let (mod_loader, mod_loader_version) = match &inst.mod_loader {
            Some(ml) => (ml.loader_type.name().to_string(), Some(ml.version.clone())),
            None => ("Vanilla".to_string(), None),
        };
        InstanceInfo {
            id: inst.id.clone(),
            name: inst.name.clone(),
            minecraft_version: inst.minecraft_version.clone(),
            mod_loader,
            mod_loader_version,
            icon: Some(inst.icon.clone()),
            last_played: inst.last_played.map(|dt| dt.to_string()),
            total_played_seconds: inst.total_played_seconds,
        }
    }).collect();
    
    Ok(info)
}

#[tauri::command]
pub async fn get_instance_details(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<InstanceInfo, String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let (mod_loader, mod_loader_version) = match &instance.mod_loader {
        Some(ml) => (ml.loader_type.name().to_string(), Some(ml.version.clone())),
        None => ("Vanilla".to_string(), None),
    };
    
    Ok(InstanceInfo {
        id: instance.id.clone(),
        name: instance.name.clone(),
        minecraft_version: instance.minecraft_version.clone(),
        mod_loader,
        mod_loader_version,
        icon: Some(instance.icon.clone()),
        last_played: instance.last_played.map(|dt| dt.to_string()),
        total_played_seconds: instance.total_played_seconds,
    })
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateInstanceRequest {
    pub name: String,
    pub minecraft_version: String,
    pub mod_loader_type: String,
    pub loader_version: Option<String>,
}

#[tauri::command]
pub async fn create_instance(
    state: State<'_, AppState>,
    request: CreateInstanceRequest,
) -> Result<String, String> {
    use crate::core::instance::{ModLoader, ModLoaderType};
    
    let mod_loader = match request.mod_loader_type.as_str() {
        "Forge" => Some(ModLoader {
            loader_type: ModLoaderType::Forge,
            version: request.loader_version.clone().unwrap_or_else(|| "latest".to_string()),
        }),
        "NeoForge" => Some(ModLoader {
            loader_type: ModLoaderType::NeoForge,
            version: request.loader_version.clone().unwrap_or_else(|| "latest".to_string()),
        }),
        "Fabric" => Some(ModLoader {
            loader_type: ModLoaderType::Fabric,
            version: request.loader_version.clone().unwrap_or_else(|| "latest".to_string()),
        }),
        "Quilt" => Some(ModLoader {
            loader_type: ModLoaderType::Quilt,
            version: request.loader_version.clone().unwrap_or_else(|| "latest".to_string()),
        }),
        "LiteLoader" => Some(ModLoader {
            loader_type: ModLoaderType::LiteLoader,
            version: request.loader_version.clone().unwrap_or_else(|| "latest".to_string()),
        }),
        _ => None,
    };
    
    let instance_id = uuid::Uuid::new_v4().to_string();
    let instance_path = state.data_dir.join("instances").join(&instance_id);
    
    // Create instance directory structure
    std::fs::create_dir_all(&instance_path)
        .map_err(|e| format!("Failed to create instance directory: {}", e))?;
    
    let game_dir = instance_path.join(".minecraft");
    std::fs::create_dir_all(&game_dir)
        .map_err(|e| format!("Failed to create game directory: {}", e))?;
    std::fs::create_dir_all(game_dir.join("mods"))
        .map_err(|e| format!("Failed to create mods directory: {}", e))?;
    std::fs::create_dir_all(game_dir.join("resourcepacks"))
        .map_err(|e| format!("Failed to create resourcepacks directory: {}", e))?;
    std::fs::create_dir_all(game_dir.join("saves"))
        .map_err(|e| format!("Failed to create saves directory: {}", e))?;
    std::fs::create_dir_all(game_dir.join("screenshots"))
        .map_err(|e| format!("Failed to create screenshots directory: {}", e))?;
    
    let mut instance = Instance::new(
        request.name,
        instance_path,
        request.minecraft_version.clone(),
    );
    
    instance.mod_loader = mod_loader;
    
    // Save instance to file
    instance.save()
        .map_err(|e| format!("Failed to save instance: {}", e))?;
    
    // Clone values before moving instance
    let instance_clone = instance.clone();
    let data_dir_clone = state.data_dir.clone();
    let id_clone = instance_id.clone();
    
    // Add to state
    {
        let mut instances = state.instances.lock().unwrap();
        instances.push(instance);
    }
    
    // Setup instance (download files) in background
    tokio::spawn(async move {
        match setup_instance(&instance_clone, &data_dir_clone, None).await {
            Ok(_) => println!("Instance {} setup complete", id_clone),
            Err(e) => eprintln!("Failed to setup instance {}: {}", id_clone, e),
        }
    });
    
    Ok(instance_id)
}

#[tauri::command]
pub async fn delete_instance(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let mut instances = state.instances.lock().unwrap();
    instances.retain(|i| i.id != instance_id);
    Ok(())
}

#[tauri::command]
pub async fn launch_instance(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    use std::io::{BufRead, BufReader};
    use std::process::Stdio;
    use crate::core::{
        accounts::AuthSession,
        config::Config,
        launch::{LaunchContext, steps::create_default_launch_task},
    };
    
    // Find instance
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    // Load config
    let config = Config::load().unwrap_or_default();
    
    // Create auth session (offline for now until auth is implemented)
    let auth_session = AuthSession::offline("Player");
    
    // Create launch context
    let context = LaunchContext::new(instance.clone(), auth_session, config);
    
    // Create and execute launch task
    let mut launch_task = create_default_launch_task(context);
    
    // Take log receiver for monitoring
    let log_receiver = launch_task.take_log_receiver();
    
    // Execute launch task
    match launch_task.execute().await {
        Ok(_) => {
            tracing::info!("Launch task completed successfully");
        }
        Err(e) => {
            return Err(format!("Launch failed: {}", e));
        }
    }
    
    // Get the game process from the launch step
    // For now, we'll create a placeholder process tracking
    let logs = Arc::new(Mutex::new(Vec::new()));
    
    // Monitor logs in background
    if let Some(mut receiver) = log_receiver {
        let logs_clone = logs.clone();
        tokio::spawn(async move {
            while let Some(msg) = receiver.recv().await {
                if let Ok(mut logs) = logs_clone.lock() {
                    logs.push(format!("[{}] {}", msg.level, msg.message));
                }
            }
        });
    }
    
    Ok(())
}

#[tauri::command]
pub async fn rename_instance(
    state: State<'_, AppState>,
    instance_id: String,
    new_name: String,
) -> Result<(), String> {
    let mut instances = state.instances.lock().unwrap();
    if let Some(instance) = instances.iter_mut().find(|i| i.id == instance_id) {
        instance.name = new_name;
        instance.save().map_err(|e| format!("Failed to save instance: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn change_instance_icon(
    state: State<'_, AppState>,
    instance_id: String,
    icon: String,
) -> Result<(), String> {
    let mut instances = state.instances.lock().unwrap();
    if let Some(instance) = instances.iter_mut().find(|i| i.id == instance_id) {
        instance.icon = icon;
        instance.save().map_err(|e| format!("Failed to save instance: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn copy_instance(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<String, String> {
    
    let instances = state.instances.lock().unwrap();
    let original = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let new_id = uuid::Uuid::new_v4().to_string();
    let new_path = state.data_dir.join("instances").join(&new_id);
    
    // Copy directory
    let copy_options = fs_extra::dir::CopyOptions::new();
    fs_extra::dir::copy(&original.path, &new_path, &copy_options)
        .map_err(|e| format!("Failed to copy instance: {}", e))?;
    
    let mut new_instance = original.clone();
    new_instance.id = new_id.clone();
    new_instance.name = format!("{} (Copy)", original.name);
    new_instance.path = new_path;
    new_instance.created_at = chrono::Utc::now();
    
    new_instance.save().map_err(|e| format!("Failed to save instance: {}", e))?;
    
    drop(instances);
    let mut instances = state.instances.lock().unwrap();
    instances.push(new_instance);
    
    Ok(new_id)
}

#[tauri::command]
pub async fn change_instance_group(
    state: State<'_, AppState>,
    instance_id: String,
    group: Option<String>,
) -> Result<(), String> {
    let mut instances = state.instances.lock().unwrap();
    if let Some(instance) = instances.iter_mut().find(|i| i.id == instance_id) {
        instance.group = group;
        instance.save().map_err(|e| format!("Failed to save instance: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
pub async fn open_instance_folder(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(instance.path.to_str().unwrap())
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&instance.path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&instance.path)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(())
}

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

#[tauri::command]
pub async fn kill_instance(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let mut processes = state.running_processes.lock().unwrap();
    if let Some(process_arc) = processes.remove(&instance_id) {
        if let Ok(mut process) = process_arc.lock() {
            let _ = process.child.kill();
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn get_instance_logs(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<String>, String> {
    let processes = state.running_processes.lock().unwrap();
    if let Some(process_arc) = processes.get(&instance_id) {
        if let Ok(process) = process_arc.lock() {
            if let Ok(logs) = process.logs.lock() {
                return Ok(logs.clone());
            }
        }
    }
    Ok(Vec::new())
}

// Account Commands

#[derive(Debug, Clone, Serialize)]
pub struct AccountInfo {
    pub id: String,
    pub username: String,
    pub account_type: String,
    pub is_active: bool,
}

#[tauri::command]
pub async fn get_accounts(state: State<'_, AppState>) -> Result<Vec<AccountInfo>, String> {
    let accounts = state.accounts.lock().unwrap();
    let info: Vec<AccountInfo> = accounts.iter().map(|acc| {
        AccountInfo {
            id: acc.id.clone(),
            username: acc.username.clone(),
            account_type: format!("{:?}", acc.account_type),
            is_active: acc.is_active,
        }
    }).collect();
    
    Ok(info)
}

#[tauri::command]
pub async fn add_offline_account(
    _state: State<'_, AppState>,
    _username: String,
) -> Result<(), String> {
    // TODO: Implement offline account creation
    // This would call the actual account creation logic
    Ok(())
}

#[tauri::command]
pub async fn set_active_account(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<(), String> {
    let mut accounts = state.accounts.lock().unwrap();
    for account in accounts.iter_mut() {
        account.is_active = account.id == account_id;
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_account(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<(), String> {
    let mut accounts = state.accounts.lock().unwrap();
    accounts.retain(|a| a.id != account_id);
    Ok(())
}

// Config Commands

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<Config, String> {
    let config = state.config.lock().unwrap();
    Ok(config.clone())
}

#[tauri::command]
pub async fn update_config(
    state: State<'_, AppState>,
    config: Config,
) -> Result<(), String> {
    let mut app_config = state.config.lock().unwrap();
    *app_config = config;
    Ok(())
}

// Version Commands

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftVersionInfo {
    pub id: String,
    pub version_type: String,
    pub release_time: String,
}

#[tauri::command]
pub async fn get_minecraft_versions(
    show_releases: bool,
    show_snapshots: bool,
    show_old: bool,
) -> Result<Vec<MinecraftVersionInfo>, String> {
    let manifest = fetch_version_manifest()
        .await
        .map_err(|e| format!("Failed to fetch version manifest: {}", e))?;
    
    let versions: Vec<MinecraftVersionInfo> = manifest.versions
        .into_iter()
        .filter(|v| {
            match v.version_type {
                VersionType::Release => show_releases,
                VersionType::Snapshot => show_snapshots,
                VersionType::OldAlpha | VersionType::OldBeta => show_old,
            }
        })
        .map(|v| MinecraftVersionInfo {
            id: v.id,
            version_type: format!("{:?}", v.version_type),
            release_time: v.release_time.to_rfc3339(),
        })
        .collect();
    
    Ok(versions)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderVersionInfo {
    pub version: String,
    pub recommended: bool,
}

#[tauri::command]
pub async fn get_forge_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    let versions = modloaders::get_forge_versions(&minecraft_version)
        .await
        .map_err(|e| format!("Failed to fetch Forge versions: {}", e))?;
    
    Ok(versions.into_iter().map(|v| LoaderVersionInfo {
        version: v.version,
        recommended: v.recommended,
    }).collect())
}

#[tauri::command]
pub async fn get_fabric_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    let versions = modloaders::get_fabric_versions(&minecraft_version)
        .await
        .map_err(|e| format!("Failed to fetch Fabric versions: {}", e))?;
    
    Ok(versions.into_iter().map(|v| LoaderVersionInfo {
        version: v.version,
        recommended: v.stable,
    }).collect())
}

#[tauri::command]
pub async fn get_quilt_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    let versions = modloaders::get_quilt_versions(&minecraft_version)
        .await
        .map_err(|e| format!("Failed to fetch Quilt versions: {}", e))?;
    
    Ok(versions.into_iter().enumerate().map(|(idx, v)| LoaderVersionInfo {
        version: v.version,
        recommended: idx == 0,
    }).collect())
}

#[tauri::command]
pub async fn get_neoforge_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    let versions = modloaders::get_neoforge_versions(&minecraft_version)
        .await
        .map_err(|e| format!("Failed to fetch NeoForge versions: {}", e))?;
    
    Ok(versions.into_iter().map(|v| LoaderVersionInfo {
        version: v.version,
        recommended: v.recommended,
    }).collect())
}

#[tauri::command]
pub async fn get_liteloader_versions(minecraft_version: String) -> Result<Vec<LoaderVersionInfo>, String> {
    let versions = modloaders::get_liteloader_versions(&minecraft_version)
        .await
        .map_err(|e| format!("Failed to fetch LiteLoader versions: {}", e))?;
    
    Ok(versions.into_iter().enumerate().map(|(idx, v)| LoaderVersionInfo {
        version: v.version,
        recommended: idx == 0,
    }).collect())
}

// Mod Management Commands

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

#[tauri::command]
pub async fn search_mods(
    query: String,
    minecraft_version: String,
    mod_loader: String,
    platform: Option<String>,
) -> Result<Vec<ModSearchResult>, String> {
    use crate::core::modplatform::{modrinth::ModrinthClient, curseforge::CurseForgeClient, types::*};
    
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
            // Default to Modrinth
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
    use crate::core::modplatform::{modrinth::ModrinthClient, curseforge::CurseForgeClient};
    use crate::core::download::download_file;
    
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
            
            // Get mod files compatible with instance
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
            
            // CurseForge might not include download URL in file list
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
            // Default to Modrinth
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

/// Metadata stored for downloaded mods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModMetadata {
    pub mod_id: String,
    pub name: String,
    pub version: String,
    pub provider: String,
    pub icon_url: Option<String>,
}

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
    
    if !mods_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut mods = Vec::new();
    
    for entry in std::fs::read_dir(&mods_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        
        if path.is_file() {
            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            
            // Skip metadata files
            if filename.ends_with(".metadata.json") {
                continue;
            }
            
            // Only process .jar files (enabled or disabled)
            if !filename.ends_with(".jar") && !filename.ends_with(".jar.disabled") {
                continue;
            }
            
            // Check if mod is disabled (.disabled extension)
            let enabled = !filename.ends_with(".disabled");
            let base_filename = filename.trim_end_matches(".disabled").to_string();
            
            // Get file metadata
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
            
            let (name, version, provider, icon_url) = if let Some(meta) = metadata {
                (meta.name, Some(meta.version), Some(meta.provider), meta.icon_url)
            } else {
                // Try to parse mod metadata from JAR file
                use crate::core::modplatform::mod_parser::parse_mod_jar;
                
                let jar_path = if enabled {
                    mods_dir.join(&base_filename)
                } else {
                    mods_dir.join(format!("{}.disabled", base_filename))
                };
                
                if let Some(jar_details) = parse_mod_jar(&jar_path) {
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
                    (name, version, jar_details.loader_type, None)
                } else {
                    // Fallback: parse name from filename
                    let name = base_filename.trim_end_matches(".jar").to_string();
                    (name, None, None, None)
                }
            };
            
            mods.push(InstalledMod {
                filename: base_filename,
                name,
                version,
                enabled,
                size,
                modified,
                provider,
                icon_url,
            });
        }
    }
    
    // Sort by name
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
    
    // Try to delete the mod file, disabled version, and metadata
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
    
    // Create the directory if it doesn't exist
    std::fs::create_dir_all(&mods_dir).map_err(|e| e.to_string())?;
    
    // Open in system file manager
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
    
    // Create the directory if it doesn't exist
    std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    
    // Open in system file manager
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

// ============================================================================
// Java Management Commands
// ============================================================================

/// Serializable Java installation info for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaInstallationInfo {
    pub id: String,
    pub path: String,
    pub version: String,
    pub major_version: u32,
    pub arch: String,
    pub vendor: String,
    pub is_64bit: bool,
    pub is_managed: bool,
    pub recommended: bool,
}

impl From<crate::core::java::JavaInstallation> for JavaInstallationInfo {
    fn from(install: crate::core::java::JavaInstallation) -> Self {
        Self {
            id: install.id.clone(),
            path: install.path.to_string_lossy().to_string(),
            version: install.version.to_string(),
            major_version: install.version.major,
            arch: install.arch.to_string(),
            vendor: install.vendor.clone(),
            is_64bit: install.arch.is_64bit(),
            is_managed: install.is_managed,
            recommended: install.recommended,
        }
    }
}

/// Available Java version for download
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableJavaInfo {
    pub major: u32,
    pub name: String,
    pub is_lts: bool,
}

/// Detect all Java installations on the system
#[tauri::command]
pub async fn detect_java() -> Result<Vec<JavaInstallationInfo>, String> {
    use crate::core::java::detection::detect_java_installations;
    
    let installations = detect_java_installations();
    
    Ok(installations.into_iter().map(JavaInstallationInfo::from).collect())
}

/// Find Java that meets a version requirement
#[tauri::command]
pub async fn find_java_for_minecraft(minecraft_version: String) -> Result<Option<JavaInstallationInfo>, String> {
    use crate::core::java::detection::{find_java_for_minecraft, get_required_java_version};
    
    let required = get_required_java_version(&minecraft_version);
    
    if let Some(java) = find_java_for_minecraft(&minecraft_version) {
        Ok(Some(JavaInstallationInfo::from(java)))
    } else {
        Ok(None)
    }
}

/// Get required Java version for a Minecraft version
#[tauri::command]
pub fn get_required_java(minecraft_version: String) -> u32 {
    crate::core::java::detection::get_required_java_version(&minecraft_version)
}

/// Validate a Java installation
#[tauri::command]
pub async fn validate_java(java_path: String) -> Result<JavaInstallationInfo, String> {
    use crate::core::java::checker::JavaChecker;
    use std::path::PathBuf;
    
    let path = PathBuf::from(&java_path);
    
    if !path.exists() {
        return Err("Java executable does not exist".to_string());
    }
    
    let checker = JavaChecker::new(path);
    let result = checker.check().await;
    
    if result.valid {
        if let Some(installation) = result.to_installation() {
            return Ok(JavaInstallationInfo::from(installation));
        }
    }
    
    Err(result.error.unwrap_or_else(|| "Java validation failed".to_string()))
}

/// Fetch available Java versions for download
#[tauri::command]
pub async fn fetch_available_java_versions() -> Result<Vec<AvailableJavaInfo>, String> {
    use crate::core::java::download::fetch_adoptium_versions;
    
    let versions = fetch_adoptium_versions()
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(versions.into_iter().map(|v| AvailableJavaInfo {
        major: v.major,
        name: v.name,
        is_lts: v.is_lts,
    }).collect())
}

/// Download and install Java
#[tauri::command]
pub async fn download_java(
    major_version: u32,
    app: tauri::AppHandle,
) -> Result<JavaInstallationInfo, String> {
    use crate::core::java::download::{fetch_adoptium_download, download_java as do_download};
    use tokio::sync::mpsc;
    
    // Fetch download metadata
    let metadata = fetch_adoptium_download(major_version)
        .await
        .map_err(|e| e.to_string())?;
    
    // Create progress channel
    let (tx, mut rx) = mpsc::channel(100);
    
    // Spawn progress event emitter
    let app_clone = app.clone();
    tokio::spawn(async move {
        while let Some(progress) = rx.recv().await {
            let _ = app_clone.emit("java-download-progress", &progress);
        }
    });
    
    // Download Java
    let installation = do_download(&metadata, Some(tx))
        .await
        .map_err(|e| e.to_string())?;
    
    Ok(JavaInstallationInfo::from(installation))
}

/// Get the path to the managed Java directory
#[tauri::command]
pub fn get_java_install_dir() -> Result<String, String> {
    use crate::core::java::download::get_java_install_dir;
    
    let dir = get_java_install_dir().map_err(|e| e.to_string())?;
    Ok(dir.to_string_lossy().to_string())
}

/// Delete a managed Java installation
#[tauri::command]
pub async fn delete_java(java_path: String) -> Result<(), String> {
    use crate::core::java::download::delete_java_installation;
    use crate::core::java::install::JavaInstallation;
    use std::path::PathBuf;
    
    // Create a minimal installation struct for deletion
    let mut installation = JavaInstallation::default();
    installation.path = PathBuf::from(&java_path);
    installation.is_managed = true;
    
    delete_java_installation(&installation)
        .await
        .map_err(|e| e.to_string())
}

// =============================================================================
// World Management Commands
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldInfo {
    pub folder_name: String,
    pub name: String,
    pub seed: Option<i64>,
    pub game_type: String,
    pub hardcore: bool,
    pub last_played: Option<String>,
    pub size: String,
    pub has_icon: bool,
}

/// List worlds for an instance
#[tauri::command]
pub async fn list_worlds(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<WorldInfo>, String> {
    use crate::core::minecraft::world;
    
    // Get instance
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let saves_dir = instance.game_dir().join("saves");
    let worlds = world::list_worlds(&saves_dir);
    
    let world_infos: Vec<WorldInfo> = worlds.into_iter().map(|w| {
        let last_played = w.formatted_last_played();
        let size = w.formatted_size();
        WorldInfo {
            folder_name: w.folder_name,
            name: w.name,
            seed: w.seed,
            game_type: w.game_type.to_string(),
            hardcore: w.hardcore,
            last_played,
            size,
            has_icon: w.has_icon,
        }
    }).collect();
    
    Ok(world_infos)
}

/// Delete a world
#[tauri::command]
pub async fn delete_world(
    state: State<'_, AppState>,
    instance_id: String,
    folder_name: String,
) -> Result<(), String> {
    use crate::core::minecraft::world;
    
    // Get instance
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let saves_dir = instance.game_dir().join("saves");
    world::delete_world(&saves_dir, &folder_name)
        .map_err(|e| e.to_string())
}

/// Export a world to a ZIP file
#[tauri::command]
pub async fn export_world(
    state: State<'_, AppState>,
    instance_id: String,
    folder_name: String,
    output_path: String,
) -> Result<(), String> {
    use crate::core::minecraft::world;
    use std::path::PathBuf;
    
    // Get instance
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let saves_dir = instance.game_dir().join("saves");
    let output = PathBuf::from(output_path);
    
    world::export_world(&saves_dir, &folder_name, &output)
        .map_err(|e| e.to_string())
}

/// Copy/duplicate a world
#[tauri::command]
pub async fn copy_world(
    state: State<'_, AppState>,
    instance_id: String,
    folder_name: String,
    new_name: String,
) -> Result<(), String> {
    use crate::core::minecraft::world;
    
    // Get instance
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let saves_dir = instance.game_dir().join("saves");
    world::copy_world(&saves_dir, &folder_name, &new_name)
        .map_err(|e| e.to_string())
}

/// Get world icon as base64
#[tauri::command]
pub async fn get_world_icon(
    state: State<'_, AppState>,
    instance_id: String,
    folder_name: String,
) -> Result<Option<String>, String> {
    use crate::core::minecraft::world;
    
    // Get instance
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let saves_dir = instance.game_dir().join("saves");
    Ok(world::get_world_icon(&saves_dir, &folder_name))
}

// =============================================================================
// Resource Pack Management Commands
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePackInfo {
    pub filename: String,
    pub name: String,
    pub description: Option<String>,
    pub size: String,
    pub enabled: bool,
}

/// List resource packs for an instance
#[tauri::command]
pub async fn list_resource_packs(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<ResourcePackInfo>, String> {
    // Get instance
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
                    0 // Skip size calculation for folders for performance
                };
                
                packs.push(ResourcePackInfo {
                    filename: filename.clone(),
                    name: filename.trim_end_matches(".zip").to_string(),
                    description: None, // Could parse pack.mcmeta but skip for now
                    size: format_file_size(size),
                    enabled: true, // All packs in folder are "available"
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
    // Get instance
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

// =============================================================================
// Shader Pack Management Commands
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderPackInfo {
    pub filename: String,
    pub name: String,
    pub size: String,
}

/// List shader packs for an instance
#[tauri::command]
pub async fn list_shader_packs(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<ShaderPackInfo>, String> {
    // Get instance
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
    // Get instance
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

// =============================================================================
// Screenshots Management Commands
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenshotInfo {
    pub filename: String,
    pub path: String,
    pub timestamp: Option<String>,
    pub size: String,
}

/// List screenshots for an instance
#[tauri::command]
pub async fn list_screenshots(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<ScreenshotInfo>, String> {
    // Get instance
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let screenshots_dir = instance.game_dir().join("screenshots");
    if !screenshots_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut screenshots = Vec::new();
    
    if let Ok(entries) = std::fs::read_dir(&screenshots_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let filename = entry.file_name().to_string_lossy().to_string();
            
            // Only include PNG files
            if !filename.to_lowercase().ends_with(".png") {
                continue;
            }
            
            let metadata = entry.metadata().ok();
            let size = metadata.as_ref().map(|m| m.len()).unwrap_or(0);
            let timestamp = metadata.and_then(|m| m.modified().ok())
                .map(|t| {
                    let datetime: chrono::DateTime<chrono::Local> = t.into();
                    datetime.format("%Y-%m-%d %H:%M").to_string()
                });
            
            screenshots.push(ScreenshotInfo {
                filename: filename.clone(),
                path: path.to_string_lossy().to_string(),
                timestamp,
                size: format_file_size(size),
            });
        }
    }
    
    // Sort by filename (which includes timestamp for Minecraft screenshots)
    screenshots.sort_by(|a, b| b.filename.cmp(&a.filename));
    Ok(screenshots)
}

/// Delete a screenshot
#[tauri::command]
pub async fn delete_screenshot(
    state: State<'_, AppState>,
    instance_id: String,
    filename: String,
) -> Result<(), String> {
    // Get instance
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let screenshot_path = instance.game_dir().join("screenshots").join(&filename);
    
    if !screenshot_path.exists() {
        return Err(format!("Screenshot '{}' not found", filename));
    }
    
    std::fs::remove_file(&screenshot_path).map_err(|e| e.to_string())
}

/// Open screenshots folder in file explorer
#[tauri::command]
pub async fn open_screenshots_folder(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    // Get instance
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let screenshots_dir = instance.game_dir().join("screenshots");
    
    // Create directory if it doesn't exist
    if !screenshots_dir.exists() {
        std::fs::create_dir_all(&screenshots_dir).map_err(|e| e.to_string())?;
    }
    
    open::that(&screenshots_dir).map_err(|e| e.to_string())
}

// =============================================================================
// Shortcut Creation Commands
// =============================================================================

/// Create a desktop shortcut for an instance
#[tauri::command]
pub async fn create_instance_shortcut(
    state: State<'_, AppState>,
    instance_id: String,
    location: String, // "desktop" or "start_menu"
) -> Result<(), String> {
    // Get instance
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let exe_path = std::env::current_exe().map_err(|e| e.to_string())?;
    
    #[cfg(target_os = "windows")]
    {
        create_windows_shortcut(&instance, &exe_path, &location)
    }
    
    #[cfg(target_os = "linux")]
    {
        create_linux_shortcut(&instance, &exe_path, &location)
    }
    
    #[cfg(target_os = "macos")]
    {
        // macOS shortcut creation is more complex (requires creating .app bundle)
        Err("Shortcut creation on macOS is not yet implemented".to_string())
    }
    
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err("Shortcut creation is not supported on this platform".to_string())
    }
}

#[cfg(target_os = "windows")]
fn create_windows_shortcut(
    instance: &Instance,
    exe_path: &std::path::Path,
    location: &str,
) -> Result<(), String> {
    use std::process::Command;
    
    let shortcut_dir = match location {
        "desktop" => dirs::desktop_dir().ok_or("Could not find desktop directory")?,
        "start_menu" => {
            let app_data = dirs::data_dir().ok_or("Could not find app data directory")?;
            app_data.join("Microsoft").join("Windows").join("Start Menu").join("Programs")
        }
        _ => return Err(format!("Unknown location: {}", location)),
    };
    
    let shortcut_path = shortcut_dir.join(format!("{}.lnk", instance.name));
    let args = format!("--launch {}", instance.id);
    
    // Use PowerShell to create the shortcut
    let script = format!(
        r#"
        $WshShell = New-Object -comObject WScript.Shell
        $Shortcut = $WshShell.CreateShortcut("{}")
        $Shortcut.TargetPath = "{}"
        $Shortcut.Arguments = "{}"
        $Shortcut.WorkingDirectory = "{}"
        $Shortcut.Description = "Launch {} in OxideLauncher"
        $Shortcut.Save()
        "#,
        shortcut_path.to_string_lossy().replace("\\", "\\\\"),
        exe_path.to_string_lossy().replace("\\", "\\\\"),
        args,
        exe_path.parent().unwrap_or(exe_path).to_string_lossy().replace("\\", "\\\\"),
        instance.name
    );
    
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .map_err(|e| format!("Failed to run PowerShell: {}", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Failed to create shortcut: {}", stderr));
    }
    
    Ok(())
}

#[cfg(target_os = "linux")]
fn create_linux_shortcut(
    instance: &Instance,
    exe_path: &std::path::Path,
    location: &str,
) -> Result<(), String> {
    let shortcut_dir = match location {
        "desktop" => dirs::desktop_dir().ok_or("Could not find desktop directory")?,
        "start_menu" => {
            let data_dir = dirs::data_dir().ok_or("Could not find data directory")?;
            data_dir.join("applications")
        }
        _ => return Err(format!("Unknown location: {}", location)),
    };
    
    std::fs::create_dir_all(&shortcut_dir).map_err(|e| e.to_string())?;
    
    let desktop_file = shortcut_dir.join(format!("oxide-launcher-{}.desktop", instance.id));
    
    let content = format!(
        r#"[Desktop Entry]
Type=Application
Name={}
Comment=Launch {} in OxideLauncher
Exec="{}" --launch {}
Icon=minecraft
Terminal=false
Categories=Game;
"#,
        instance.name,
        instance.name,
        exe_path.to_string_lossy(),
        instance.id
    );
    
    std::fs::write(&desktop_file, content).map_err(|e| e.to_string())?;
    
    // Make it executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&desktop_file)
            .map_err(|e| e.to_string())?
            .permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&desktop_file, perms).map_err(|e| e.to_string())?;
    }
    
    Ok(())
}

// =============================================================================
// Instance Settings Persistence
// =============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceSettingsUpdate {
    pub name: Option<String>,
    pub java_path: Option<String>,
    pub java_args: Option<String>,
    pub min_memory: Option<u32>,
    pub max_memory: Option<u32>,
    pub window_width: Option<u32>,
    pub window_height: Option<u32>,
    pub skip_java_compatibility_check: Option<bool>,
    pub close_launcher_on_launch: Option<bool>,
    pub quit_launcher_on_exit: Option<bool>,
    pub prelaunch_command: Option<String>,
    pub postexit_command: Option<String>,
}

/// Update instance settings
#[tauri::command]
pub async fn update_instance_settings(
    state: State<'_, AppState>,
    instance_id: String,
    settings: InstanceSettingsUpdate,
) -> Result<(), String> {
    let mut instances = state.instances.lock().unwrap();
    let instance = instances.iter_mut()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Update name if provided
    if let Some(name) = settings.name {
        if !name.is_empty() {
            instance.name = name;
        }
    }
    
    // Update settings
    if let Some(java_path) = settings.java_path {
        instance.settings.java_path = if java_path.is_empty() { None } else { Some(PathBuf::from(java_path)) };
    }
    if let Some(java_args) = settings.java_args {
        instance.settings.jvm_args = if java_args.is_empty() { None } else { Some(java_args) };
    }
    if let Some(min) = settings.min_memory {
        instance.settings.min_memory = Some(min);
    }
    if let Some(max) = settings.max_memory {
        instance.settings.max_memory = Some(max);
    }
    if let Some(width) = settings.window_width {
        instance.settings.window_width = Some(width);
    }
    if let Some(height) = settings.window_height {
        instance.settings.window_height = Some(height);
    }
    if let Some(skip) = settings.skip_java_compatibility_check {
        instance.settings.skip_java_compatibility_check = skip;
    }
    if let Some(close) = settings.close_launcher_on_launch {
        instance.settings.close_launcher_on_launch = close;
    }
    if let Some(quit) = settings.quit_launcher_on_exit {
        instance.settings.quit_launcher_on_exit = quit;
    }
    if let Some(cmd) = settings.prelaunch_command {
        instance.settings.pre_launch_command = if cmd.is_empty() { None } else { Some(cmd) };
    }
    if let Some(cmd) = settings.postexit_command {
        instance.settings.post_exit_command = if cmd.is_empty() { None } else { Some(cmd) };
    }
    
    // Save instance to disk
    let instance_clone = instance.clone();
    drop(instances);
    
    instance_clone.save().map_err(|e| e.to_string())?;
    
    Ok(())
}

// Helper function for formatting file sizes
fn format_file_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}
