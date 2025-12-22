//! Instance launch commands for starting and managing Minecraft processes.
//!
//! Oxide Launcher — A Rust-based Minecraft launcher
//! Copyright (C) 2025 Oxide Launcher contributors
//!
//! This file is part of Oxide Launcher.
//!
//! Oxide Launcher is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License, or
//! (at your option) any later version.
//!
//! Oxide Launcher is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License
//! along with this program. If not, see <https://www.gnu.org/licenses/>.

use crate::commands::state::{AppState, RunningProcess};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tauri::State;

/// Status information returned by get_instance_status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceStatus {
    pub running: bool,
    pub exit_code: Option<i32>,
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
    
    // Load config first to get accounts file path
    let config = Config::load().unwrap_or_default();
    let accounts_file = config.accounts_file();
    
    // Check ownership verification before allowing launch
    let account_list = AccountList::load(&accounts_file).unwrap_or_default();
    if !account_list.is_ownership_verified() {
        return Err("You must sign in with a Microsoft account that owns Minecraft before playing. Go to Accounts to sign in and verify game ownership.".to_string());
    }
    
    // Find instance
    let instance = {
        let instances = state.instances.lock().unwrap();
        instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?
            .clone()
    };
    
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
            if let Some(active_account) = account_list.get_active() {
                AuthSession::offline(&active_account.username)
            } else {
                AuthSession::offline("Player")
            }
        } else if mode == "demo" {
            tracing::info!("Launching in demo mode");
            AuthSession::demo()
        } else {
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
        use crate::core::logging::{LogEntry, LogLevel};
        
        let logs = Arc::new(Mutex::new(Vec::new()));
        let exit_code = Arc::new(Mutex::new(None));
        
        // Add initial launcher log entry
        {
            let mut log_vec = logs.lock().unwrap();
            log_vec.push(LogEntry::launcher_info(format!(
                "Game process started for instance '{}'", 
                instance.name
            )));
        }
        
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
                                logs.push(LogEntry::game(line));
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
                                logs.push(LogEntry::stderr(line));
                            }
                        }
                    }
                });
            }
        }
        
        // Spawn a task to monitor process exit
        {
            let process_arc_clone = process_arc.clone();
            let logs_clone = logs.clone();
            let exit_code_clone = exit_code.clone();
            let instance_name = instance.name.clone();
            
            std::thread::spawn(move || {
                // Wait for the process to exit
                loop {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    
                    if let Ok(mut child) = process_arc_clone.lock() {
                        match child.try_wait() {
                            Ok(Some(status)) => {
                                let code = status.code();
                                
                                // Store exit code
                                if let Ok(mut ec) = exit_code_clone.lock() {
                                    *ec = code;
                                }
                                
                                // Add exit log entry
                                if let Ok(mut log_vec) = logs_clone.lock() {
                                    match code {
                                        Some(0) => {
                                            log_vec.push(LogEntry::launcher_info(format!(
                                                "Game '{}' exited normally (exit code: 0)",
                                                instance_name
                                            )));
                                        }
                                        Some(code) => {
                                            log_vec.push(LogEntry::launcher(
                                                LogLevel::Error,
                                                format!(
                                                    "Game '{}' exited with error code: {}",
                                                    instance_name, code
                                                )
                                            ));
                                        }
                                        None => {
                                            log_vec.push(LogEntry::launcher_warn(format!(
                                                "Game '{}' exited (signal terminated)",
                                                instance_name
                                            )));
                                        }
                                    }
                                }
                                break;
                            }
                            Ok(None) => {
                                // Still running, continue waiting
                            }
                            Err(e) => {
                                // Error checking status
                                if let Ok(mut log_vec) = logs_clone.lock() {
                                    log_vec.push(LogEntry::launcher_error(format!(
                                        "Error checking process status: {}",
                                        e
                                    )));
                                }
                                break;
                            }
                        }
                    } else {
                        break; // Mutex poisoned, exit
                    }
                }
            });
        }
        
        // Store the process in running_processes (keeping the Arc<Mutex<Child>>)
        let running_process = RunningProcess {
            child: process_arc,
            logs,
            launch_time: std::time::Instant::now(),
            exit_code,
        };
        
        let mut processes = state.running_processes.lock().unwrap();
        processes.insert(instance_id.clone(), Arc::new(Mutex::new(running_process)));
        tracing::info!("Stored running process for instance {}", instance_id);
    }
    
    // Update last played time for the instance
    {
        let mut instances = state.instances.lock().unwrap();
        if let Some(instance) = instances.iter_mut().find(|i| i.id == instance_id) {
            instance.update_last_played();
            if let Err(e) = instance.save() {
                tracing::error!("Failed to save instance after updating last played: {}", e);
            }
        }
    }
    
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
    let status = get_instance_status(state, instance_id).await?;
    Ok(status.running)
}

#[tauri::command]
pub async fn get_instance_status(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<InstanceStatus, String> {
    let mut processes = state.running_processes.lock().unwrap();
    
    // Check if the process exists and whether it's still running
    let result = if let Some(process_arc) = processes.get(&instance_id) {
        let process = process_arc.lock().unwrap();
        let mut child = process.child.lock().unwrap();
        
        // Get launch time before checking status
        let launch_time = process.launch_time;
        
        // Check if we already have an exit code from the monitor thread
        let stored_exit_code = process.exit_code.lock().ok().and_then(|ec| *ec);
        
        // Check if process is still running
        match child.try_wait() {
            Ok(Some(exit_status)) => {
                // Process has exited - use stored exit code if available, otherwise get from status
                let exit_code = stored_exit_code.or(exit_status.code());
                
                // Calculate play time
                let play_time_secs = launch_time.elapsed().as_secs();
                
                Some((false, exit_code, Some(play_time_secs)))
            }
            Ok(None) => {
                // Process is still running
                return Ok(InstanceStatus { running: true, exit_code: None });
            }
            Err(_) => {
                // Error checking status - use stored exit code if available
                Some((false, stored_exit_code, None))
            }
        }
    } else {
        None
    };
    
    // If process has exited, update playtime and remove from running processes
    if let Some((running, exit_code, play_time)) = result {
        // Update play time before removing
        if let Some(play_time_secs) = play_time {
            // Load config to check if we should record game time
            let config = crate::core::config::Config::load().unwrap_or_default();
            if config.minecraft.record_game_time {
                let mut instances = state.instances.lock().unwrap();
                if let Some(instance) = instances.iter_mut().find(|i| i.id == instance_id) {
                    instance.add_play_time(play_time_secs);
                    if let Err(e) = instance.save() {
                        tracing::error!("Failed to save instance play time: {}", e);
                    } else {
                        tracing::info!("Recorded {} seconds of play time for instance {}", play_time_secs, instance_id);
                    }
                }
            }
        }
        
        processes.remove(&instance_id);
        return Ok(InstanceStatus { running, exit_code });
    }
    
    Ok(InstanceStatus { running: false, exit_code: None })
}

#[tauri::command]
pub async fn get_instance_logs(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<crate::core::logging::LogEntry>, String> {
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
