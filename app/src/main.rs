//! Oxide Launcher - A Rust-based Minecraft Launcher
//! 
//! This launcher is inspired by Prism Launcher and aims to provide
//! a modern, fast, and feature-rich experience for managing Minecraft instances.

// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod core;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Layer};
use std::sync::Arc;
use parking_lot::RwLock;
use chrono::Local;

/// Global logging state to allow runtime reconfiguration
static LOGGING_STATE: once_cell::sync::Lazy<Arc<RwLock<LoggingState>>> = 
    once_cell::sync::Lazy::new(|| Arc::new(RwLock::new(LoggingState::default())));

#[derive(Default)]
struct LoggingState {
    file_logging_enabled: bool,
}

fn initialize_logging() {
    // Load config to check if file logging is enabled
    let config = core::config::Config::load().unwrap_or_default();
    
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| {
            if config.logging.debug_to_file {
                EnvFilter::new("debug")
            } else {
                EnvFilter::new("info")
            }
        });
    
    // Console logging layer (always enabled)
    let console_layer = tracing_subscriber::fmt::layer()
        .with_filter(env_filter.clone());
    
    // File logging layer (conditional) - creates a new file per session
    if config.logging.debug_to_file {
        let logs_dir = config.logs_dir();
        std::fs::create_dir_all(&logs_dir).ok();
        
        // Create session-based log filename with timestamp
        let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
        let log_filename = format!("oxide-launcher_{}.log", timestamp);
        let log_path = logs_dir.join(&log_filename);
        
        // Create the log file
        let log_file = std::fs::File::create(&log_path)
            .expect("Failed to create log file");
        
        let (non_blocking, _guard) = tracing_appender::non_blocking(log_file);
        
        // Store guard so it doesn't drop (would stop logging)
        std::mem::forget(_guard);
        
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_filter(EnvFilter::new("debug"));
        
        tracing_subscriber::registry()
            .with(console_layer)
            .with(file_layer)
            .init();
        
        LOGGING_STATE.write().file_logging_enabled = true;
        
        // Clean up old log files (keep last 10 sessions)
        cleanup_old_logs(&logs_dir, 10);
        
        tracing::info!("Session log file created: {:?}", log_path);
    } else {
        tracing_subscriber::registry()
            .with(console_layer)
            .init();
    }
}

/// Clean up old log files, keeping only the most recent `keep_count` files
fn cleanup_old_logs(logs_dir: &std::path::Path, keep_count: usize) {
    if let Ok(entries) = std::fs::read_dir(logs_dir) {
        let mut log_files: Vec<_> = entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with("oxide-launcher_") && n.ends_with(".log"))
                    .unwrap_or(false)
            })
            .collect();
        
        // Sort by modified time (newest first)
        log_files.sort_by(|a, b| {
            let a_time = a.metadata().and_then(|m| m.modified()).ok();
            let b_time = b.metadata().and_then(|m| m.modified()).ok();
            b_time.cmp(&a_time)
        });
        
        // Delete files beyond the keep count
        for file in log_files.into_iter().skip(keep_count) {
            let _ = std::fs::remove_file(file.path());
        }
    }
}

fn main() {
    // Initialize logging
    initialize_logging();

    tracing::info!("Starting Oxide Launcher v{}", env!("CARGO_PKG_VERSION"));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(commands::AppState::new())
        .invoke_handler(tauri::generate_handler![
            // Instance commands
            commands::instances::get_instances,
            commands::instances::get_instance_details,
            commands::instances::create_instance,
            commands::instances::delete_instance,
            commands::instances::launch_instance,
            commands::instances::get_instance_logs,
            commands::instances::is_instance_running,
            commands::instances::get_instance_status,
            commands::instances::rename_instance,
            commands::instances::change_instance_icon,
            commands::instances::copy_instance,
            commands::instances::change_instance_group,
            commands::instances::open_instance_folder,
            commands::instances::open_instance_logs_folder,
            commands::instances::export_instance,
            commands::instances::kill_instance,
            commands::instances::update_instance_settings,
            // Component management commands
            commands::instances::get_instance_components,
            commands::instances::remove_instance_component,
            commands::instances::change_component_version,
            commands::instances::install_mod_loader,
            commands::instances::open_minecraft_folder,
            commands::instances::open_libraries_folder,
            // Jar mods and agents
            commands::instances::add_jar_mod,
            commands::instances::get_jar_mods,
            commands::instances::remove_jar_mod,
            commands::instances::add_java_agent,
            commands::instances::get_java_agents,
            commands::instances::remove_java_agent,
            commands::instances::replace_minecraft_jar,
            commands::instances::revert_minecraft_jar,
            commands::instances::has_custom_minecraft_jar,
            // Component ordering and customization
            commands::instances::move_component_up,
            commands::instances::move_component_down,
            commands::instances::add_empty_component,
            commands::instances::customize_component,
            commands::instances::revert_component,
            // Import/Export commands
            commands::instances::export_instance_to_file,
            commands::instances::detect_import_format,
            commands::instances::import_instance_from_file,
            // Account commands
            commands::accounts::get_accounts,
            commands::accounts::add_offline_account,
            commands::accounts::start_microsoft_login,
            commands::accounts::poll_microsoft_login,
            commands::accounts::cancel_microsoft_login,
            commands::accounts::refresh_account,
            commands::accounts::set_active_account,
            commands::accounts::remove_account,
            commands::accounts::get_account_for_launch,
            commands::accounts::is_microsoft_configured,
            // Skin management commands
            commands::accounts::get_player_profile,
            commands::accounts::change_skin_url,
            commands::accounts::upload_skin,
            commands::accounts::reset_skin,
            commands::accounts::set_cape,
            commands::accounts::hide_cape,
            commands::accounts::fetch_skin_from_username,
            commands::accounts::import_skin_from_username,
            commands::accounts::open_skins_folder,
            commands::accounts::set_default_account,
            commands::accounts::download_skin_image,
            // Config commands
            commands::config::get_config,
            commands::config::update_config,
            commands::config::get_logs_directory,
            commands::config::open_logs_directory,
            commands::config::open_data_directory,
            commands::config::open_launcher_folder,
            commands::config::open_external_url,
            // Version commands
            commands::versions::get_minecraft_versions,
            commands::versions::get_forge_versions,
            commands::versions::get_neoforge_versions,
            commands::versions::get_fabric_versions,
            commands::versions::get_quilt_versions,
            commands::versions::get_liteloader_versions,
            // Mod commands
            commands::mods::search_mods,
            commands::mods::download_mod,
            commands::mods::get_installed_mods,
            commands::mods::toggle_mod,
            commands::mods::delete_mod,
            commands::mods::delete_mods,
            commands::mods::enable_mods,
            commands::mods::disable_mods,
            commands::mods::open_mods_folder,
            commands::mods::open_configs_folder,
            commands::mods::add_local_mod,
            commands::mods::add_local_mod_from_bytes,
            // Enhanced mod commands for new download dialog
            commands::mods::search_mods_detailed,
            commands::mods::get_mod_details,
            commands::mods::get_mod_versions,
            commands::mods::download_mod_version,
            // Java commands
            commands::java::detect_java,
            commands::java::find_java_for_minecraft,
            commands::java::get_required_java,
            commands::java::validate_java,
            commands::java::fetch_available_java_versions,
            commands::java::download_java,
            commands::java::get_java_install_dir,
            commands::java::delete_java,
            // World commands
            commands::worlds::list_worlds,
            commands::worlds::delete_world,
            commands::worlds::export_world,
            commands::worlds::copy_world,
            commands::worlds::get_world_icon,
            commands::worlds::open_saves_folder,
            // Resource pack commands
            commands::resources::list_resource_packs,
            commands::resources::delete_resource_pack,
            commands::resources::open_resourcepacks_folder,
            commands::resources::search_resource_packs,
            commands::resources::get_resource_pack_details,
            commands::resources::get_resource_pack_versions,
            commands::resources::download_resource_pack_version,
            commands::resources::add_local_resource_pack,
            commands::resources::add_local_resource_pack_from_bytes,
            // Shader pack commands
            commands::resources::list_shader_packs,
            commands::resources::delete_shader_pack,
            commands::resources::open_shaderpacks_folder,
            commands::resources::search_shader_packs,
            commands::resources::get_shader_pack_details,
            commands::resources::get_shader_pack_versions,
            commands::resources::download_shader_pack_version,
            commands::resources::add_local_shader_pack,
            commands::resources::add_local_shader_pack_from_bytes,
            commands::resources::get_shader_pack_details,
            commands::resources::get_shader_pack_versions,
            commands::resources::download_shader_pack_version,
            commands::resources::add_local_shader_pack,
            // Screenshot commands
            commands::screenshots::list_screenshots,
            commands::screenshots::delete_screenshot,
            commands::screenshots::open_screenshots_folder,
            // Shortcut commands
            commands::shortcuts::create_instance_shortcut,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
