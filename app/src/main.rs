//! Oxide Launcher - A Rust-based Minecraft Launcher
//! 
//! This launcher is inspired by Prism Launcher and aims to provide
//! a modern, fast, and feature-rich experience for managing Minecraft instances.

// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod core;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

fn main() {
    // Initialize logging
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Oxide Launcher");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(commands::AppState::new())
        .invoke_handler(tauri::generate_handler![
            // Instance commands
            commands::instances::get_instances,
            commands::instances::get_instance_details,
            commands::instances::create_instance,
            commands::instances::delete_instance,
            commands::instances::launch_instance,
            commands::instances::get_instance_logs,
            commands::instances::rename_instance,
            commands::instances::change_instance_icon,
            commands::instances::copy_instance,
            commands::instances::change_instance_group,
            commands::instances::open_instance_folder,
            commands::instances::export_instance,
            commands::instances::kill_instance,
            commands::instances::update_instance_settings,
            // Account commands
            commands::accounts::get_accounts,
            commands::accounts::add_offline_account,
            commands::accounts::set_active_account,
            commands::accounts::remove_account,
            // Config commands
            commands::config::get_config,
            commands::config::update_config,
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
            // Resource pack commands
            commands::resources::list_resource_packs,
            commands::resources::delete_resource_pack,
            // Shader pack commands
            commands::resources::list_shader_packs,
            commands::resources::delete_shader_pack,
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
