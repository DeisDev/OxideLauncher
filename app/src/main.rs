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
            commands::get_instances,
            commands::get_instance_details,
            commands::create_instance,
            commands::delete_instance,
            commands::launch_instance,
            commands::get_instance_logs,
            commands::rename_instance,
            commands::change_instance_icon,
            commands::copy_instance,
            commands::change_instance_group,
            commands::open_instance_folder,
            commands::export_instance,
            commands::create_instance_shortcut,
            commands::kill_instance,
            commands::get_accounts,
            commands::add_offline_account,
            commands::set_active_account,
            commands::remove_account,
            commands::get_config,
            commands::update_config,
            commands::get_minecraft_versions,
            commands::get_forge_versions,
            commands::get_neoforge_versions,
            commands::get_fabric_versions,
            commands::get_quilt_versions,
            commands::get_liteloader_versions,
            commands::search_mods,
            commands::download_mod,
            commands::get_installed_mods,
            commands::toggle_mod,
            commands::delete_mod,
            commands::delete_mods,
            commands::enable_mods,
            commands::disable_mods,
            commands::open_mods_folder,
            commands::open_configs_folder,
            commands::add_local_mod,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
