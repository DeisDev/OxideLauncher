//! Instance management commands

use super::state::AppState;
use crate::core::instance::{setup_instance, Instance, ModLoader, ModLoaderType};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::State;

/// Serializable instance information for the frontend
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

impl From<&Instance> for InstanceInfo {
    fn from(inst: &Instance) -> Self {
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
    }
}

/// Request payload for creating a new instance
#[derive(Debug, Clone, Deserialize)]
pub struct CreateInstanceRequest {
    pub name: String,
    pub minecraft_version: String,
    pub mod_loader_type: String,
    pub loader_version: Option<String>,
}

/// Instance settings update payload
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

#[tauri::command]
pub async fn get_instances(state: State<'_, AppState>) -> Result<Vec<InstanceInfo>, String> {
    let instances = state.instances.lock().unwrap();
    Ok(instances.iter().map(InstanceInfo::from).collect())
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
    
    Ok(InstanceInfo::from(instance))
}

#[tauri::command]
pub async fn create_instance(
    state: State<'_, AppState>,
    request: CreateInstanceRequest,
) -> Result<String, String> {
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
    
    // Find the instance to get its path before removing
    if let Some(instance) = instances.iter().find(|i| i.id == instance_id) {
        // Delete the instance directory from disk
        let instance_path = instance.path.clone();
        
        if instance_path.exists() {
            std::fs::remove_dir_all(&instance_path)
                .map_err(|e| format!("Failed to delete instance directory: {}", e))?;
            tracing::info!("Deleted instance directory: {:?}", instance_path);
        }
    }
    
    // Remove from memory
    instances.retain(|i| i.id != instance_id);
    
    Ok(())
}

#[tauri::command]
pub async fn launch_instance(
    state: State<'_, AppState>,
    instance_id: String,
    launch_mode: Option<String>,
) -> Result<(), String> {
    use crate::core::{
        accounts::{AccountList, AuthSession},
        config::Config,
        launch::{LaunchContext, steps::create_default_launch_task},
        minecraft::version::LaunchFeatures,
    };
    
    let mode = launch_mode.as_deref().unwrap_or("normal");
    
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
    
    // Determine launch features based on mode and instance settings
    let mut features = LaunchFeatures::normal();
    
    // Check for custom resolution
    if instance.settings.window_width.is_some() || instance.settings.window_height.is_some() {
        features.has_custom_resolution = true;
    }
    
    // Set demo mode feature if launching in demo mode
    if mode == "demo" {
        features.is_demo_user = true;
    }
    
    // Get the active account for authentication based on mode
    let auth_session = {
        if mode == "offline" {
            tracing::info!("Launching in offline mode");
            // Use the active account name but as offline
            let accounts_file = config.accounts_file();
            let account_list = AccountList::load(&accounts_file).unwrap_or_default();
            if let Some(active_account) = account_list.get_active() {
                AuthSession::offline(&active_account.username)
            } else {
                AuthSession::offline("Player")
            }
        } else if mode == "demo" {
            tracing::info!("Launching in demo mode");
            AuthSession::demo()
        } else {
            let accounts_file = config.accounts_file();
            let account_list = AccountList::load(&accounts_file).unwrap_or_default();
            
            if let Some(active_account) = account_list.get_active() {
                tracing::info!("Using account: {} ({})", active_account.username, active_account.account_type.name());
                AuthSession::from_account(active_account)
            } else {
                tracing::warn!("No active account found, using default offline account");
                AuthSession::offline("Player")
            }
        }
    };
    
    // Create launch context with features
    let context = LaunchContext::with_features(instance.clone(), auth_session, config, features);
    
    // Create and execute launch task
    let mut launch_task = create_default_launch_task(context);
    
    // Take log receiver for monitoring
    let _log_receiver = launch_task.take_log_receiver();
    
    // Execute launch task
    match launch_task.execute().await {
        Ok(_) => {
            tracing::info!("Launch task completed successfully");
        }
        Err(e) => {
            return Err(format!("Launch failed: {}", e));
        }
    }
    
    // Get the game process from the launch task
    if let Some(process_arc) = launch_task.take_game_process() {
        let logs = Arc::new(Mutex::new(Vec::new()));
        
        // Try to get stdout/stderr from the child process
        {
            let mut process = process_arc.lock().unwrap();
            
            // Take stdout and stderr from the child process
            let stdout = process.stdout.take();
            let stderr = process.stderr.take();
            
            // Spawn a task to read stdout
            if let Some(stdout) = stdout {
                let logs_clone = logs.clone();
                std::thread::spawn(move || {
                    use std::io::{BufRead, BufReader};
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            if let Ok(mut logs) = logs_clone.lock() {
                                logs.push(format!("[GAME] {}", line));
                            }
                        }
                    }
                });
            }
            
            // Spawn a task to read stderr
            if let Some(stderr) = stderr {
                let logs_clone = logs.clone();
                std::thread::spawn(move || {
                    use std::io::{BufRead, BufReader};
                    let reader = BufReader::new(stderr);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            if let Ok(mut logs) = logs_clone.lock() {
                                logs.push(format!("[GAME/ERR] {}", line));
                            }
                        }
                    }
                });
            }
        }
        
        // Store the process in running_processes (keeping the Arc<Mutex<Child>>)
        let running_process = super::state::RunningProcess {
            child: process_arc,
            logs,
        };
        
        let mut processes = state.running_processes.lock().unwrap();
        processes.insert(instance_id.clone(), Arc::new(Mutex::new(running_process)));
        tracing::info!("Stored running process for instance {}", instance_id);
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
        if let Ok(process) = process_arc.lock() {
            if let Ok(mut child) = process.child.lock() {
                let _ = child.kill();
            }
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn is_instance_running(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<bool, String> {
    let mut processes = state.running_processes.lock().unwrap();
    
    // First, check if the process exists and whether it's still running
    let should_remove = if let Some(process_arc) = processes.get(&instance_id) {
        let process = process_arc.lock().unwrap();
        let mut child = process.child.lock().unwrap();
        
        // Check if process is still running
        match child.try_wait() {
            Ok(Some(_exit_status)) => {
                // Process has exited
                Some(false)
            }
            Ok(None) => {
                // Process is still running
                return Ok(true);
            }
            Err(_) => {
                // Error checking status, assume not running
                Some(false)
            }
        }
    } else {
        None
    };
    
    // If we need to remove the process entry, do it now
    if should_remove.is_some() {
        processes.remove(&instance_id);
    }
    
    Ok(false)
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

// =============================================================================
// Component Management Commands
// =============================================================================

/// Component information for the frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub uid: String,
    pub name: String,
    pub version: String,
    pub component_type: String,
    pub enabled: bool,
    pub removable: bool,
    pub version_changeable: bool,
    pub customizable: bool,
    pub revertible: bool,
    pub custom: bool,
    pub order: i32,
    pub problems: Vec<ComponentProblemInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentProblemInfo {
    pub severity: String,
    pub description: String,
}

#[tauri::command]
pub async fn get_instance_components(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<ComponentInfo>, String> {
    use crate::core::instance::{build_component_list, ComponentType, ProblemSeverity};
    
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Build component list from instance data
    let components = build_component_list(
        &instance.minecraft_version,
        instance.mod_loader.as_ref(),
    );
    
    let component_infos: Vec<ComponentInfo> = components.components.iter().map(|c| {
        ComponentInfo {
            uid: c.uid.clone(),
            name: c.name.clone(),
            version: c.version.clone(),
            component_type: match c.component_type {
                ComponentType::Minecraft => "minecraft".to_string(),
                ComponentType::ModLoader => "mod_loader".to_string(),
                ComponentType::Library => "library".to_string(),
                ComponentType::Agent => "agent".to_string(),
                ComponentType::JarMod => "jar_mod".to_string(),
                ComponentType::Mappings => "mappings".to_string(),
                ComponentType::Other => "other".to_string(),
            },
            enabled: c.enabled,
            removable: c.removable,
            version_changeable: c.version_changeable,
            customizable: c.customizable,
            revertible: c.revertible,
            custom: c.custom,
            order: c.order,
            problems: c.problems.iter().map(|p| ComponentProblemInfo {
                severity: match p.severity {
                    ProblemSeverity::None => "none".to_string(),
                    ProblemSeverity::Warning => "warning".to_string(),
                    ProblemSeverity::Error => "error".to_string(),
                },
                description: p.description.clone(),
            }).collect(),
        }
    }).collect();
    
    Ok(component_infos)
}

#[tauri::command]
pub async fn remove_instance_component(
    state: State<'_, AppState>,
    instance_id: String,
    component_uid: String,
) -> Result<(), String> {
    let mut instances = state.instances.lock().unwrap();
    let instance = instances.iter_mut()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Handle mod loader removal
    match component_uid.as_str() {
        "net.minecraftforge" | "net.neoforged" | "net.fabricmc.fabric-loader" | 
        "org.quiltmc.quilt-loader" | "com.mumfrey.liteloader" => {
            instance.mod_loader = None;
            instance.save().map_err(|e| format!("Failed to save instance: {}", e))?;
            Ok(())
        }
        "net.minecraft" => {
            Err("Cannot remove the Minecraft component".to_string())
        }
        "net.fabricmc.intermediary" => {
            Err("Cannot remove intermediary mappings while mod loader is installed".to_string())
        }
        _ => {
            Err(format!("Unknown component: {}", component_uid))
        }
    }
}

#[tauri::command]
pub async fn change_component_version(
    state: State<'_, AppState>,
    instance_id: String,
    component_uid: String,
    new_version: String,
) -> Result<(), String> {
    let mut instances = state.instances.lock().unwrap();
    let instance = instances.iter_mut()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    match component_uid.as_str() {
        "net.minecraft" => {
            instance.minecraft_version = new_version;
            instance.save().map_err(|e| format!("Failed to save instance: {}", e))?;
            Ok(())
        }
        "net.minecraftforge" | "net.neoforged" | "net.fabricmc.fabric-loader" | 
        "org.quiltmc.quilt-loader" | "com.mumfrey.liteloader" => {
            if let Some(ref mut loader) = instance.mod_loader {
                loader.version = new_version;
                instance.save().map_err(|e| format!("Failed to save instance: {}", e))?;
                Ok(())
            } else {
                Err("No mod loader installed".to_string())
            }
        }
        _ => {
            Err(format!("Cannot change version of component: {}", component_uid))
        }
    }
}

#[tauri::command]
pub async fn install_mod_loader(
    state: State<'_, AppState>,
    instance_id: String,
    loader_type: String,
    loader_version: String,
) -> Result<(), String> {
    use crate::core::instance::{ModLoader, ModLoaderType};
    
    let mut instances = state.instances.lock().unwrap();
    let instance = instances.iter_mut()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let loader_type = match loader_type.to_lowercase().as_str() {
        "forge" => ModLoaderType::Forge,
        "neoforge" => ModLoaderType::NeoForge,
        "fabric" => ModLoaderType::Fabric,
        "quilt" => ModLoaderType::Quilt,
        "liteloader" => ModLoaderType::LiteLoader,
        _ => return Err(format!("Unknown loader type: {}", loader_type)),
    };
    
    instance.mod_loader = Some(ModLoader {
        loader_type,
        version: loader_version,
    });
    
    instance.save().map_err(|e| format!("Failed to save instance: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn open_minecraft_folder(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let minecraft_dir = instance.game_dir();
    
    // Ensure directory exists
    std::fs::create_dir_all(&minecraft_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;
    
    // Open in file explorer
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&minecraft_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&minecraft_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&minecraft_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn open_libraries_folder(
    state: State<'_, AppState>,
    _instance_id: String,
) -> Result<(), String> {
    // Libraries are stored globally, not per-instance
    let config = state.config.lock().unwrap();
    let libraries_dir = config.data_dir().join("libraries");
    
    // Ensure directory exists
    std::fs::create_dir_all(&libraries_dir)
        .map_err(|e| format!("Failed to create directory: {}", e))?;
    
    // Open in file explorer
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&libraries_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&libraries_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&libraries_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn add_jar_mod(
    state: State<'_, AppState>,
    instance_id: String,
    jar_path: String,
) -> Result<(), String> {
    use std::fs;
    
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Create jar mods directory if it doesn't exist
    let jar_mods_dir = instance.path.join("jarmods");
    fs::create_dir_all(&jar_mods_dir)
        .map_err(|e| format!("Failed to create jarmods directory: {}", e))?;
    
    // Copy the jar file to the jarmods directory
    let source_path = std::path::PathBuf::from(&jar_path);
    let file_name = source_path.file_name()
        .ok_or_else(|| "Invalid jar path".to_string())?;
    let dest_path = jar_mods_dir.join(file_name);
    
    fs::copy(&source_path, &dest_path)
        .map_err(|e| format!("Failed to copy jar mod: {}", e))?;
    
    tracing::info!("Added jar mod: {:?}", dest_path);
    
    Ok(())
}

#[tauri::command]
pub async fn get_jar_mods(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<String>, String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let jar_mods_dir = instance.path.join("jarmods");
    
    if !jar_mods_dir.exists() {
        return Ok(Vec::new());
    }
    
    let mut jar_mods = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&jar_mods_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".jar") || name.ends_with(".zip") {
                    jar_mods.push(name.to_string());
                }
            }
        }
    }
    
    Ok(jar_mods)
}

#[tauri::command]
pub async fn remove_jar_mod(
    state: State<'_, AppState>,
    instance_id: String,
    jar_name: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let jar_path = instance.path.join("jarmods").join(&jar_name);
    
    if jar_path.exists() {
        std::fs::remove_file(&jar_path)
            .map_err(|e| format!("Failed to remove jar mod: {}", e))?;
        tracing::info!("Removed jar mod: {:?}", jar_path);
    }
    
    Ok(())
}

#[tauri::command]
pub async fn add_java_agent(
    state: State<'_, AppState>,
    instance_id: String,
    agent_path: String,
    agent_args: Option<String>,
) -> Result<(), String> {
    use std::fs;
    
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Create agents directory if it doesn't exist  
    let agents_dir = instance.path.join("agents");
    fs::create_dir_all(&agents_dir)
        .map_err(|e| format!("Failed to create agents directory: {}", e))?;
    
    // Copy the agent jar to the agents directory
    let source_path = std::path::PathBuf::from(&agent_path);
    let file_name = source_path.file_name()
        .ok_or_else(|| "Invalid agent path".to_string())?;
    let dest_path = agents_dir.join(file_name);
    
    fs::copy(&source_path, &dest_path)
        .map_err(|e| format!("Failed to copy agent: {}", e))?;
    
    // Store agent configuration
    let agents_config_path = instance.path.join("agents.json");
    let mut agents_config: Vec<AgentConfig> = if agents_config_path.exists() {
        let content = fs::read_to_string(&agents_config_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };
    
    agents_config.push(AgentConfig {
        file: file_name.to_string_lossy().to_string(),
        args: agent_args,
    });
    
    let config_content = serde_json::to_string_pretty(&agents_config)
        .map_err(|e| format!("Failed to serialize agents config: {}", e))?;
    fs::write(&agents_config_path, config_content)
        .map_err(|e| format!("Failed to write agents config: {}", e))?;
    
    tracing::info!("Added Java agent: {:?}", dest_path);
    
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub file: String,
    pub args: Option<String>,
}

#[tauri::command]
pub async fn get_java_agents(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<AgentConfig>, String> {
    use std::fs;
    
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let agents_config_path = instance.path.join("agents.json");
    
    if !agents_config_path.exists() {
        return Ok(Vec::new());
    }
    
    let content = fs::read_to_string(&agents_config_path)
        .map_err(|e| format!("Failed to read agents config: {}", e))?;
    let agents: Vec<AgentConfig> = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse agents config: {}", e))?;
    
    Ok(agents)
}

#[tauri::command]
pub async fn remove_java_agent(
    state: State<'_, AppState>,
    instance_id: String,
    agent_file: String,
) -> Result<(), String> {
    use std::fs;
    
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Remove agent file
    let agent_path = instance.path.join("agents").join(&agent_file);
    if agent_path.exists() {
        fs::remove_file(&agent_path)
            .map_err(|e| format!("Failed to remove agent file: {}", e))?;
    }
    
    // Update agents config
    let agents_config_path = instance.path.join("agents.json");
    if agents_config_path.exists() {
        let content = fs::read_to_string(&agents_config_path).unwrap_or_default();
        let mut agents: Vec<AgentConfig> = serde_json::from_str(&content).unwrap_or_default();
        agents.retain(|a| a.file != agent_file);
        
        let config_content = serde_json::to_string_pretty(&agents)
            .map_err(|e| format!("Failed to serialize agents config: {}", e))?;
        fs::write(&agents_config_path, config_content)
            .map_err(|e| format!("Failed to write agents config: {}", e))?;
    }
    
    tracing::info!("Removed Java agent: {}", agent_file);
    
    Ok(())
}

#[tauri::command]
pub async fn replace_minecraft_jar(
    state: State<'_, AppState>,
    instance_id: String,
    jar_path: String,
) -> Result<(), String> {
    use std::fs;
    
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Create patches directory if it doesn't exist
    let patches_dir = instance.path.join("patches");
    fs::create_dir_all(&patches_dir)
        .map_err(|e| format!("Failed to create patches directory: {}", e))?;
    
    // Copy the custom jar
    let source_path = std::path::PathBuf::from(&jar_path);
    let dest_path = patches_dir.join("custom.jar");
    
    fs::copy(&source_path, &dest_path)
        .map_err(|e| format!("Failed to copy custom jar: {}", e))?;
    
    // Create a marker file to indicate custom jar is in use
    let marker_path = patches_dir.join("custom_jar.json");
    let marker = serde_json::json!({
        "type": "custom_jar",
        "file": "custom.jar",
        "original_name": source_path.file_name().map(|n| n.to_string_lossy().to_string())
    });
    
    fs::write(&marker_path, marker.to_string())
        .map_err(|e| format!("Failed to write marker file: {}", e))?;
    
    tracing::info!("Replaced Minecraft jar for instance {}", instance_id);
    
    Ok(())
}

#[tauri::command]
pub async fn revert_minecraft_jar(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    use std::fs;
    
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let patches_dir = instance.path.join("patches");
    
    // Remove custom jar and marker
    let custom_jar = patches_dir.join("custom.jar");
    let marker_path = patches_dir.join("custom_jar.json");
    
    if custom_jar.exists() {
        fs::remove_file(&custom_jar)
            .map_err(|e| format!("Failed to remove custom jar: {}", e))?;
    }
    
    if marker_path.exists() {
        fs::remove_file(&marker_path)
            .map_err(|e| format!("Failed to remove marker file: {}", e))?;
    }
    
    tracing::info!("Reverted Minecraft jar for instance {}", instance_id);
    
    Ok(())
}

#[tauri::command]
pub async fn has_custom_minecraft_jar(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<bool, String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let marker_path = instance.path.join("patches").join("custom_jar.json");
    Ok(marker_path.exists())
}

#[tauri::command]
pub async fn move_component_up(
    state: State<'_, AppState>,
    instance_id: String,
    component_uid: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Load component list
    let mut component_list = crate::core::instance::ComponentList::load(&instance.path)
        .map_err(|e| format!("Failed to load components: {}", e))?;
    
    // Move the component up
    if component_list.move_up(&component_uid) {
        // Save the updated list
        component_list.save(&instance.path)
            .map_err(|e| format!("Failed to save components: {}", e))?;
        tracing::info!("Moved component {} up in instance {}", component_uid, instance_id);
        Ok(())
    } else {
        Err("Component cannot be moved up (not found or already at top)".to_string())
    }
}

#[tauri::command]
pub async fn move_component_down(
    state: State<'_, AppState>,
    instance_id: String,
    component_uid: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Load component list
    let mut component_list = crate::core::instance::ComponentList::load(&instance.path)
        .map_err(|e| format!("Failed to load components: {}", e))?;
    
    // Move the component down
    if component_list.move_down(&component_uid) {
        // Save the updated list
        component_list.save(&instance.path)
            .map_err(|e| format!("Failed to save components: {}", e))?;
        tracing::info!("Moved component {} down in instance {}", component_uid, instance_id);
        Ok(())
    } else {
        Err("Component cannot be moved down (not found or already at bottom)".to_string())
    }
}

#[tauri::command]
pub async fn add_empty_component(
    state: State<'_, AppState>,
    instance_id: String,
    name: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Load component list
    let mut component_list = crate::core::instance::ComponentList::load(&instance.path)
        .map_err(|e| format!("Failed to load components: {}", e))?;
    
    // Create a unique ID for the custom component
    let uid = format!("custom.{}", uuid::Uuid::new_v4());
    
    // Add the empty custom component
    let component = crate::core::instance::InstanceComponent {
        uid,
        name,
        version: "1.0".to_string(),
        component_type: crate::core::instance::ComponentType::Other,
        enabled: true,
        removable: true,
        version_changeable: true,
        customizable: true,
        revertible: false,
        custom: true,
        order: component_list.components.len() as i32,
        problems: Vec::new(),
    };
    
    component_list.add(component);
    
    // Save the updated list
    component_list.save(&instance.path)
        .map_err(|e| format!("Failed to save components: {}", e))?;
    
    tracing::info!("Added empty component to instance {}", instance_id);
    
    Ok(())
}

#[tauri::command]
pub async fn customize_component(
    state: State<'_, AppState>,
    instance_id: String,
    component_uid: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Load component list
    let mut component_list = crate::core::instance::ComponentList::load(&instance.path)
        .map_err(|e| format!("Failed to load components: {}", e))?;
    
    // Find and mark the component as custom
    if let Some(component) = component_list.get_mut(&component_uid) {
        if !component.customizable {
            return Err("This component cannot be customized".to_string());
        }
        component.custom = true;
        component.revertible = true;
        
        // Save the updated list
        component_list.save(&instance.path)
            .map_err(|e| format!("Failed to save components: {}", e))?;
        
        tracing::info!("Customized component {} in instance {}", component_uid, instance_id);
        Ok(())
    } else {
        Err("Component not found".to_string())
    }
}

#[tauri::command]
pub async fn revert_component(
    state: State<'_, AppState>,
    instance_id: String,
    component_uid: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    // Load component list
    let mut component_list = crate::core::instance::ComponentList::load(&instance.path)
        .map_err(|e| format!("Failed to load components: {}", e))?;
    
    // Find and revert the component
    if let Some(component) = component_list.get_mut(&component_uid) {
        if !component.revertible {
            return Err("This component cannot be reverted".to_string());
        }
        component.custom = false;
        component.revertible = false;
        
        // Save the updated list
        component_list.save(&instance.path)
            .map_err(|e| format!("Failed to save components: {}", e))?;
        
        tracing::info!("Reverted component {} in instance {}", component_uid, instance_id);
        Ok(())
    } else {
        Err("Component not found".to_string())
    }
}
