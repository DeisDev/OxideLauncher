//! Instance settings management commands

use super::InstanceSettingsUpdate;
use crate::commands::state::AppState;
use std::path::PathBuf;
use tauri::State;

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
