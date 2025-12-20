//! Application configuration management.
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

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::core::error::Result;

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Base data directory for the launcher
    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,

    /// Directory where instances are stored
    #[serde(default)]
    pub instances_dir: Option<PathBuf>,

    /// Theme name (dark, light, system)
    #[serde(default = "default_theme")]
    pub theme: String,

    /// Java settings
    #[serde(default)]
    pub java: JavaConfig,

    /// Network settings
    #[serde(default)]
    pub network: NetworkConfig,

    /// UI settings
    #[serde(default)]
    pub ui: UiConfig,

    /// Minecraft game settings
    #[serde(default)]
    pub minecraft: MinecraftConfig,

    /// Custom commands
    #[serde(default)]
    pub commands: CustomCommands,

    /// Launcher-wide memory settings
    #[serde(default)]
    pub memory: MemoryConfig,

    /// Logging settings
    #[serde(default)]
    pub logging: LoggingConfig,

    /// API keys and secrets (should be handled securely)
    #[serde(default)]
    pub api_keys: ApiKeys,
    
    /// Debug settings for troubleshooting
    #[serde(default)]
    pub debug: DebugConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            data_dir: default_data_dir(),
            instances_dir: None,
            theme: default_theme(),
            java: JavaConfig::default(),
            network: NetworkConfig::default(),
            ui: UiConfig::default(),
            minecraft: MinecraftConfig::default(),
            commands: CustomCommands::default(),
            memory: MemoryConfig::default(),
            logging: LoggingConfig::default(),
            api_keys: ApiKeys::default(),
            debug: DebugConfig::default(),
        }
    }
}

#[allow(dead_code)] // Helper methods will be used as features are completed
impl Config {
    /// Load configuration from disk
    pub fn load() -> Result<Self> {
        let config_path = config_file_path();
        
        if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            let config: Config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            // Create default config and save it
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<()> {
        let config_path = config_file_path();
        
        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&config_path, content)?;
        
        Ok(())
    }

    /// Get the instances directory
    pub fn instances_dir(&self) -> PathBuf {
        self.instances_dir
            .clone()
            .unwrap_or_else(|| self.data_dir.join("instances"))
    }

    /// Get the accounts file path
    pub fn accounts_file(&self) -> PathBuf {
        self.data_dir.join("accounts.json")
    }

    /// Get the data directory
    pub fn data_dir(&self) -> PathBuf {
        self.data_dir.clone()
    }

    /// Get the cache directory
    pub fn cache_dir(&self) -> PathBuf {
        self.data_dir.join("cache")
    }

    /// Get the assets directory
    pub fn assets_dir(&self) -> PathBuf {
        self.data_dir.join("assets")
    }

    /// Get the libraries directory
    pub fn libraries_dir(&self) -> PathBuf {
        self.data_dir.join("libraries")
    }

    /// Get the meta directory (for version manifests, etc.)
    pub fn meta_dir(&self) -> PathBuf {
        self.data_dir.join("meta")
    }

    /// Get the java directory
    pub fn java_dir(&self) -> PathBuf {
        self.data_dir.join("java")
    }

    /// Get the icons directory
    pub fn icons_dir(&self) -> PathBuf {
        self.data_dir.join("icons")
    }

    /// Get the skins directory
    pub fn skins_dir(&self) -> PathBuf {
        self.data_dir.join("skins")
    }

    /// Get the themes directory
    pub fn themes_dir(&self) -> PathBuf {
        self.data_dir.join("themes")
    }

    /// Get the logs directory
    pub fn logs_dir(&self) -> PathBuf {
        self.data_dir.join("logs")
    }

    /// Get the downloads directory for blocked mods
    /// Falls back to system downloads folder if not configured
    pub fn downloads_dir(&self) -> PathBuf {
        self.network.downloads_dir
            .clone()
            .unwrap_or_else(|| {
                dirs::download_dir().unwrap_or_else(|| self.data_dir.join("downloads"))
            })
    }

    /// Set the theme name
    pub fn set_theme(&mut self, theme: &str) {
        self.theme = theme.to_string();
    }
}

/// Java runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaConfig {
    /// Custom Java path (if not auto-detected)
    pub custom_path: Option<PathBuf>,

    /// Use bundled Java runtime
    #[serde(default = "default_true")]
    pub use_bundled: bool,

    /// Auto-detect Java installations
    #[serde(default = "default_true")]
    pub auto_detect: bool,

    /// Additional JVM arguments
    #[serde(default)]
    pub extra_args: Vec<String>,
    
    /// Skip Java version compatibility check globally
    #[serde(default)]
    pub skip_compatibility_check: bool,
    
    /// Automatically download Java when needed
    #[serde(default = "default_true")]
    pub auto_download: bool,
}

impl Default for JavaConfig {
    fn default() -> Self {
        Self {
            custom_path: None,
            use_bundled: true,
            auto_detect: true,
            extra_args: Vec::new(),
            skip_compatibility_check: false,
            auto_download: true,
        }
    }
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Proxy settings
    #[serde(default)]
    pub proxy: Option<ProxyConfig>,

    /// Maximum concurrent downloads (1-50)
    #[serde(default = "default_max_downloads")]
    pub max_concurrent_downloads: usize,

    /// Number of retry attempts for failed downloads (0-10)
    #[serde(default = "default_download_retries")]
    pub download_retries: u32,

    /// Download timeout in seconds (5-300)
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// User agent string
    #[serde(default = "default_user_agent")]
    pub user_agent: String,

    /// Directory to watch for manually downloaded blocked mods
    #[serde(default)]
    pub downloads_dir: Option<PathBuf>,

    /// Whether to watch downloads directory recursively
    #[serde(default)]
    pub downloads_dir_watch_recursive: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            proxy: None,
            max_concurrent_downloads: default_max_downloads(),
            download_retries: default_download_retries(),
            timeout_seconds: default_timeout(),
            user_agent: default_user_agent(),
            downloads_dir: None,
            downloads_dir_watch_recursive: false,
        }
    }
}

/// Proxy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub proxy_type: ProxyType,
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ProxyType {
    Http,
    Socks5,
}

/// Window position and size state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WindowState {
    /// X position on screen
    #[serde(default)]
    pub x: Option<i32>,
    /// Y position on screen
    #[serde(default)]
    pub y: Option<i32>,
    /// Window width
    #[serde(default)]
    pub width: Option<u32>,
    /// Window height
    #[serde(default)]
    pub height: Option<u32>,
    /// Whether window was maximized
    #[serde(default)]
    pub maximized: bool,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Show news panel
    #[serde(default = "default_true")]
    pub show_news: bool,

    /// Instance view mode
    #[serde(default)]
    pub instance_view: InstanceViewMode,

    /// Instance sort field
    #[serde(default = "default_sort_by")]
    pub instance_sort_by: String,

    /// Instance sort direction (true = ascending)
    #[serde(default = "default_true")]
    pub instance_sort_asc: bool,

    /// Instance grid size
    #[serde(default = "default_grid_size")]
    pub instance_grid_size: String,

    /// Color scheme name
    #[serde(default = "default_color_scheme")]
    pub color_scheme: String,

    /// Window width
    #[serde(default = "default_window_width")]
    pub window_width: u32,

    /// Window height
    #[serde(default = "default_window_height")]
    pub window_height: u32,

    /// Last selected instance
    #[serde(default)]
    pub last_instance: Option<String>,

    /// Rust mode - obnoxiously Rust-themed color scheme 🦀
    #[serde(default)]
    pub rust_mode: bool,

    /// Remember main window position and size
    #[serde(default)]
    pub remember_main_window_position: bool,

    /// Remember dialog window positions and sizes
    #[serde(default)]
    pub remember_dialog_window_positions: bool,

    /// Stored main window state
    #[serde(default)]
    pub main_window_state: WindowState,

    /// Stored dialog window states (keyed by dialog label)
    #[serde(default)]
    pub dialog_window_states: HashMap<String, WindowState>,

    /// Open instance details view after installing a modpack
    #[serde(default)]
    pub open_instance_after_install: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_news: true,
            instance_view: InstanceViewMode::default(),
            instance_sort_by: default_sort_by(),
            instance_sort_asc: true,
            instance_grid_size: default_grid_size(),
            color_scheme: default_color_scheme(),
            window_width: default_window_width(),
            window_height: default_window_height(),
            last_instance: None,
            rust_mode: false,
            remember_main_window_position: false,
            remember_dialog_window_positions: false,
            main_window_state: WindowState::default(),
            dialog_window_states: HashMap::new(),
            open_instance_after_install: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum InstanceViewMode {
    #[default]
    Grid,
    List,
}

/// Memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryConfig {
    /// Minimum memory allocation (MB)
    #[serde(default = "default_min_memory")]
    pub min_memory: u32,

    /// Maximum memory allocation (MB)
    #[serde(default = "default_max_memory")]
    pub max_memory: u32,

    /// PermGen size (MB) - for older Java versions
    #[serde(default = "default_permgen")]
    pub permgen: u32,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            min_memory: default_min_memory(),
            max_memory: default_max_memory(),
            permgen: default_permgen(),
        }
    }
}

/// Minecraft game settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinecraftConfig {
    /// Game window width
    #[serde(default = "default_game_width")]
    pub window_width: u32,

    /// Game window height
    #[serde(default = "default_game_height")]
    pub window_height: u32,

    /// Launch game maximized
    #[serde(default)]
    pub launch_maximized: bool,

    /// Close launcher after game starts
    #[serde(default)]
    pub close_after_launch: bool,

    /// Show game console window
    #[serde(default = "default_true")]
    pub show_console: bool,

    /// Auto-close console when game exits normally
    #[serde(default)]
    pub auto_close_console: bool,

    /// Show console window on crash/error
    #[serde(default = "default_true")]
    pub show_console_on_error: bool,

    /// Record game time
    #[serde(default = "default_true")]
    pub record_game_time: bool,

    /// Show game time in instance list
    #[serde(default = "default_true")]
    pub show_game_time: bool,
}

impl Default for MinecraftConfig {
    fn default() -> Self {
        Self {
            window_width: default_game_width(),
            window_height: default_game_height(),
            launch_maximized: false,
            close_after_launch: false,
            show_console: true,
            auto_close_console: false,
            show_console_on_error: true,
            record_game_time: true,
            show_game_time: true,
        }
    }
}

/// Custom commands configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CustomCommands {
    /// Command to run before launching the game
    #[serde(default)]
    pub pre_launch: Option<String>,

    /// Command to run after the game exits
    #[serde(default)]
    pub post_exit: Option<String>,

    /// Wrapper command (game command is appended)
    #[serde(default)]
    pub wrapper_command: Option<String>,
}

/// API keys configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ApiKeys {
    /// Microsoft Azure Client ID for MSA authentication
    pub msa_client_id: Option<String>,
    
    /// CurseForge API key
    pub curseforge_api_key: Option<String>,
    
    /// Modrinth API token (optional, for authenticated requests)
    pub modrinth_api_token: Option<String>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Enable debug logging to file
    #[serde(default)]
    pub debug_to_file: bool,

    /// Maximum log file size in MB before rotation
    #[serde(default = "default_log_size")]
    pub max_file_size_mb: u32,

    /// Number of rotated log files to keep
    #[serde(default = "default_log_files")]
    pub max_files: u32,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            debug_to_file: false,
            max_file_size_mb: default_log_size(),
            max_files: default_log_files(),
        }
    }
}

/// Debug configuration for troubleshooting launch and runtime issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugConfig {
    /// Force use of java.exe instead of javaw.exe globally (shows console output on Windows)
    #[serde(default)]
    pub force_java_console: bool,
    
    /// Disable CREATE_NO_WINDOW flag globally (allows console window to appear)
    #[serde(default)]
    pub disable_create_no_window: bool,
    
    /// Log the full launch command to a file in the instance directory
    #[serde(default)]
    pub log_launch_commands: bool,
    
    /// Enable verbose logging throughout the application
    #[serde(default)]
    pub verbose_logging: bool,
    
    /// Keep natives directory after launch (don't clean up for debugging)
    #[serde(default)]
    pub keep_natives_after_launch: bool,
    
    /// Pause before launch (for attaching debuggers)
    #[serde(default)]
    pub pause_before_launch: bool,
}

impl Default for DebugConfig {
    fn default() -> Self {
        Self {
            force_java_console: false,
            disable_create_no_window: false,
            log_launch_commands: false,
            verbose_logging: false,
            keep_natives_after_launch: false,
            pause_before_launch: false,
        }
    }
}

// Default value functions

fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("OxideLauncher")
}

fn config_file_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("OxideLauncher")
        .join("config.json")
}

fn default_theme() -> String {
    "system".to_string()
}

fn default_true() -> bool {
    true
}

fn default_max_downloads() -> usize {
    6
}

fn default_timeout() -> u64 {
    30
}

fn default_user_agent() -> String {
    format!("OxideLauncher/{}", env!("CARGO_PKG_VERSION"))
}

fn default_window_width() -> u32 {
    1200
}

fn default_window_height() -> u32 {
    800
}

fn default_min_memory() -> u32 {
    512
}

fn default_max_memory() -> u32 {
    4096
}

fn default_log_size() -> u32 {
    10
}

fn default_log_files() -> u32 {
    5
}

fn default_permgen() -> u32 {
    256
}

fn default_download_retries() -> u32 {
    2
}

fn default_sort_by() -> String {
    "name".to_string()
}

fn default_grid_size() -> String {
    "medium".to_string()
}

fn default_color_scheme() -> String {
    "ocean".to_string()
}

fn default_game_width() -> u32 {
    854
}

fn default_game_height() -> u32 {
    480
}
