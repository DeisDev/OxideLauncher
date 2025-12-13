//! Application configuration management

use serde::{Deserialize, Serialize};
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

    /// Theme name
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

    /// Launcher-wide memory settings
    #[serde(default)]
    pub memory: MemoryConfig,

    /// API keys and secrets (should be handled securely)
    #[serde(default)]
    pub api_keys: ApiKeys,
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
            memory: MemoryConfig::default(),
            api_keys: ApiKeys::default(),
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

    /// Get the themes directory
    pub fn themes_dir(&self) -> PathBuf {
        self.data_dir.join("themes")
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

    /// Maximum concurrent downloads
    #[serde(default = "default_max_downloads")]
    pub max_concurrent_downloads: usize,

    /// Download timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// User agent string
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            proxy: None,
            max_concurrent_downloads: default_max_downloads(),
            timeout_seconds: default_timeout(),
            user_agent: default_user_agent(),
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

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Show news panel
    #[serde(default = "default_true")]
    pub show_news: bool,

    /// Instance view mode
    #[serde(default)]
    pub instance_view: InstanceViewMode,

    /// Window width
    #[serde(default = "default_window_width")]
    pub window_width: u32,

    /// Window height
    #[serde(default = "default_window_height")]
    pub window_height: u32,

    /// Last selected instance
    #[serde(default)]
    pub last_instance: Option<String>,

    /// Cat mode (easter egg like Prism)
    #[serde(default)]
    pub cat_mode: bool,
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            show_news: true,
            instance_view: InstanceViewMode::default(),
            window_width: default_window_width(),
            window_height: default_window_height(),
            last_instance: None,
            cat_mode: false,
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
    "dark".to_string()
}

fn default_true() -> bool {
    true
}

fn default_max_downloads() -> usize {
    4
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

fn default_permgen() -> u32 {
    256
}
