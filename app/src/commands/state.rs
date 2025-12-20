//! Application state management for Tauri commands.
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

use crate::core::{
    accounts::{Account, AccountList},
    config::Config,
    instance::{Instance, InstanceList},
    logging::LogEntry,
};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Child;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Running Minecraft process information
pub struct RunningProcess {
    /// The child process - wrapped in Arc<Mutex<>> since we got it from the launch task
    pub child: Arc<Mutex<Child>>,
    /// Structured log entries from the game process
    pub logs: Arc<Mutex<Vec<LogEntry>>>,
    /// Time when the game was launched
    pub launch_time: Instant,
    /// Exit code when process exits (None if still running or not checked yet)
    pub exit_code: Arc<Mutex<Option<i32>>>,
}

/// Application state shared across all commands
pub struct AppState {
    pub instances: Mutex<Vec<Instance>>,
    pub accounts: Mutex<Vec<Account>>,
    pub config: Mutex<Config>,
    pub data_dir: PathBuf,
    pub running_processes: Mutex<HashMap<String, Arc<Mutex<RunningProcess>>>>,
}

impl AppState {
    pub fn new() -> Self {
        // Load configuration from disk (or create default)
        let config = Config::load().unwrap_or_else(|e| {
            tracing::warn!("Failed to load config, using defaults: {}", e);
            Config::default()
        });
        
        // Get data directory from config
        let data_dir = config.data_dir();
        
        // Ensure data directory exists
        if let Err(e) = std::fs::create_dir_all(&data_dir) {
            tracing::error!("Failed to create data directory: {}", e);
        }
        
        // Load instances from disk
        let instances_dir = config.instances_dir();
        let instances = match InstanceList::load(&instances_dir) {
            Ok(list) => {
                tracing::info!("Loaded {} instances from {:?}", list.instances.len(), instances_dir);
                list.instances
            }
            Err(e) => {
                tracing::warn!("Failed to load instances: {}", e);
                Vec::new()
            }
        };
        
        // Load accounts from disk
        let accounts_file = config.accounts_file();
        let accounts = match AccountList::load(&accounts_file) {
            Ok(list) => {
                tracing::info!("Loaded {} accounts from {:?}", list.accounts.len(), accounts_file);
                list.accounts
            }
            Err(e) => {
                tracing::warn!("Failed to load accounts: {}", e);
                Vec::new()
            }
        };
        
        Self {
            instances: Mutex::new(instances),
            accounts: Mutex::new(accounts),
            config: Mutex::new(config),
            data_dir,
            running_processes: Mutex::new(HashMap::new()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
