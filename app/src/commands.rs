//! Tauri command handlers for the frontend

use crate::core::{
    accounts::Account,
    config::Config,
    instance::{setup_instance, Instance},
    minecraft::version::{fetch_version_manifest, VersionType},
    modloaders,
};
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
    pub icon: Option<String>,
    pub last_played: Option<String>,
    pub total_played_seconds: u64,
}

#[tauri::command]
pub async fn get_instances(state: State<'_, AppState>) -> Result<Vec<InstanceInfo>, String> {
    let instances = state.instances.lock().unwrap();
    let info: Vec<InstanceInfo> = instances.iter().map(|inst| {
        InstanceInfo {
            id: inst.id.clone(),
            name: inst.name.clone(),
            minecraft_version: inst.minecraft_version.clone(),
            mod_loader: inst.mod_loader.as_ref().map(|ml| format!("{:?}", ml)).unwrap_or_else(|| "Vanilla".to_string()),
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
    
    Ok(InstanceInfo {
        id: instance.id.clone(),
        name: instance.name.clone(),
        minecraft_version: instance.minecraft_version.clone(),
        mod_loader: instance.mod_loader.as_ref().map(|ml| format!("{:?}", ml)).unwrap_or_else(|| "Vanilla".to_string()),
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
    use std::process::{Command, Stdio};
    use crate::core::{
        minecraft::version::{fetch_version_manifest, fetch_version_data},
        minecraft::libraries::build_classpath,
        accounts::AuthSession,
        config::Config,
    };
    
    // Find instance
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    let _config = Config::default();
    
    // Get version data
    let manifest = fetch_version_manifest().await
        .map_err(|e| format!("Failed to fetch version manifest: {}", e))?;
    let version_info = manifest.get_version(&instance.minecraft_version)
        .ok_or_else(|| format!("Version {} not found", instance.minecraft_version))?;
    let version_data = fetch_version_data(version_info).await
        .map_err(|e| format!("Failed to fetch version data: {}", e))?;
    
    // Setup paths
    let game_dir = instance.game_dir();
    let libraries_dir = state.data_dir.join("libraries");
    let assets_dir = state.data_dir.join("assets");
    let client_jar = state.data_dir
        .join("meta")
        .join("versions")
        .join(&instance.minecraft_version)
        .join(format!("{}.jar", &instance.minecraft_version));
    
    // Build classpath
    let classpath = build_classpath(&version_data, &libraries_dir, &client_jar);
    
    // Create offline auth session (since we don't have auth yet)
    let auth_session = AuthSession::offline("Player");
    
    // Build Java arguments
    let mut args = Vec::new();
    
    // Memory settings
    args.push(format!("-Xms{}M", instance.settings.min_memory.unwrap_or(512)));
    args.push(format!("-Xmx{}M", instance.settings.max_memory.unwrap_or(2048)));
    
    // Natives directory
    let natives_dir = game_dir.join("natives");
    args.push(format!("-Djava.library.path={}", natives_dir.to_string_lossy()));
    
    // Classpath
    args.push("-cp".to_string());
    args.push(classpath);
    
    // Main class
    args.push(version_data.main_class.clone());
    
    // Game arguments
    args.push("--username".to_string());
    args.push(auth_session.username.clone());
    args.push("--version".to_string());
    args.push(instance.minecraft_version.clone());
    args.push("--gameDir".to_string());
    args.push(game_dir.to_string_lossy().to_string());
    args.push("--assetsDir".to_string());
    args.push(assets_dir.join("objects").to_string_lossy().to_string());
    args.push("--assetIndex".to_string());
    args.push(version_data.assets.clone());
    args.push("--uuid".to_string());
    args.push(auth_session.uuid.clone());
    args.push("--accessToken".to_string());
    args.push(auth_session.access_token.clone());
    args.push("--userType".to_string());
    args.push(auth_session.user_type.clone());
    
    // Window size if specified
    if let Some(width) = instance.settings.window_width {
        args.push("--width".to_string());
        args.push(width.to_string());
    }
    if let Some(height) = instance.settings.window_height {
        args.push("--height".to_string());
        args.push(height.to_string());
    }
    
    // Find Java
    let java_exe = if cfg!(target_os = "windows") { "java.exe" } else { "java" };
    let java_path = which::which(java_exe)
        .map_err(|_| "No Java installation found".to_string())?;
    
    // Launch the game
    let mut child = Command::new(&java_path)
        .args(&args)
        .current_dir(&game_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to start game: {}", e))?;
    
    // Capture stdout and stderr
    let stdout = child.stdout.take().ok_or("Failed to capture stdout")?;
    let stderr = child.stderr.take().ok_or("Failed to capture stderr")?;
    
    let logs = Arc::new(Mutex::new(Vec::new()));
    let logs_clone = logs.clone();
    
    // Read stdout in background
    let stdout_logs = logs.clone();
    tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(mut logs) = stdout_logs.lock() {
                    logs.push(line);
                }
            }
        }
    });
    
    // Read stderr in background
    let stderr_logs = logs.clone();
    tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(mut logs) = stderr_logs.lock() {
                    logs.push(format!("[ERROR] {}", line));
                }
            }
        }
    });
    
    // Store process in state
    let process = RunningProcess {
        child,
        logs: logs_clone,
    };
    
    {
        let mut processes = state.running_processes.lock().unwrap();
        processes.insert(instance_id.clone(), Arc::new(Mutex::new(process)));
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
pub async fn create_instance_shortcut(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let _instance = instances.iter().find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    #[cfg(target_os = "windows")]
    {
        // Create .lnk shortcut on Windows
        // This would require mslnk or similar crate
        return Err("Shortcut creation not yet implemented for Windows".to_string());
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        return Err("Shortcut creation not yet implemented for this platform".to_string());
    }
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
}

#[tauri::command]
pub async fn search_mods(
    query: String,
    minecraft_version: String,
    mod_loader: String,
) -> Result<Vec<ModSearchResult>, String> {
    use crate::core::modplatform::{modrinth::ModrinthClient, types::*};
    
    let client = ModrinthClient::new();
    
    let search_query = SearchQuery {
        query,
        resource_type: Some(ResourceType::Mod),
        categories: vec![],
        game_versions: vec![minecraft_version],
        loaders: if mod_loader != "Vanilla" { 
            vec![mod_loader.to_lowercase()] 
        } else { 
            vec![] 
        },
        sort: SortOrder::Relevance,
        limit: 20,
        offset: 0,
    };
    
    let results = client.search(&search_query)
        .await
        .map_err(|e| format!("Failed to search mods: {}", e))?;
    
    Ok(results.hits.into_iter().map(|hit| ModSearchResult {
        id: hit.id,
        name: hit.title,
        description: hit.description,
        author: hit.author,
        downloads: hit.downloads as u64,
        icon_url: hit.icon_url,
        project_type: format!("{:?}", hit.resource_type),
    }).collect())
}

#[tauri::command]
pub async fn download_mod(
    state: State<'_, AppState>,
    instance_id: String,
    mod_id: String,
) -> Result<(), String> {
    use crate::core::modplatform::modrinth::ModrinthClient;
    use crate::core::download::download_file;
    
    let client = ModrinthClient::new();
    
    // Get instance
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
    // Get mod versions compatible with instance
    let loader_name = instance.mod_loader.as_ref()
        .map(|ml| format!("{:?}", ml.loader_type).to_lowercase())
        .unwrap_or_else(|| "vanilla".to_string());
    
    let versions = client.get_versions(
        &mod_id,
        Some(&[instance.minecraft_version.clone()]),
        Some(&[loader_name]),
    ).await.map_err(|e| format!("Failed to get mod versions: {}", e))?;
    
    if versions.is_empty() {
        return Err("No compatible mod version found".to_string());
    }
    
    // Use the first (latest) compatible version
    let version = &versions[0];
    if version.files.is_empty() {
        return Err("No files available for this mod".to_string());
    }
    
    // Download the primary file
    let file = &version.files[0];
    let mods_dir = instance.mods_dir();
    let file_path = mods_dir.join(&file.filename);
    
    download_file(&file.url, &file_path, None)
        .await
        .map_err(|e| format!("Failed to download mod: {}", e))?;
    
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct InstalledMod {
    pub filename: String,
    pub enabled: bool,
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
    
    for entry in std::fs::read_dir(mods_dir).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        
        if path.is_file() {
            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            
            // Check if mod is disabled (.disabled extension)
            let enabled = !filename.ends_with(".disabled");
            
            mods.push(InstalledMod {
                filename: filename.trim_end_matches(".disabled").to_string(),
                enabled,
            });
        }
    }
    
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
    
    // Try to delete both enabled and disabled versions
    let _ = std::fs::remove_file(mod_path);
    let _ = std::fs::remove_file(disabled_path);
    
    Ok(())
}
