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
    instances.retain(|i| i.id != instance_id);
    Ok(())
}

#[tauri::command]
pub async fn launch_instance(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
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
