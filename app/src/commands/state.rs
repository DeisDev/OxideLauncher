//! Application state management

use crate::core::{
    accounts::{Account, AccountList},
    config::Config,
    instance::{Instance, InstanceList},
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
    pub logs: Arc<Mutex<Vec<String>>>,
    /// Time when the game was launched
    pub launch_time: Instant,
    /// Exit code when process exits (None if still running or not checked yet)
    pub exit_code: Option<i32>,
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
