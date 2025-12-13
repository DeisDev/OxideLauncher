//! Application state management

use crate::core::{accounts::Account, config::Config, instance::Instance};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Child;
use std::sync::{Arc, Mutex};

/// Running Minecraft process information
pub struct RunningProcess {
    pub child: Child,
    pub logs: Arc<Mutex<Vec<String>>>,
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

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
