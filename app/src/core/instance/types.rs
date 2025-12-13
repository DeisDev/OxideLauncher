//! Instance data types

#![allow(dead_code)] // Types will be used as features are completed

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A Minecraft instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    /// Unique identifier
    pub id: String,
    
    /// Display name
    pub name: String,
    
    /// Path to the instance directory
    pub path: PathBuf,
    
    /// Instance icon (name or path)
    pub icon: String,
    
    /// Group this instance belongs to
    pub group: Option<String>,
    
    /// Minecraft version
    pub minecraft_version: String,
    
    /// Mod loader information
    pub mod_loader: Option<ModLoader>,
    
    /// Instance-specific settings
    pub settings: InstanceSettings,
    
    /// When the instance was created
    pub created_at: DateTime<Utc>,
    
    /// When the instance was last modified
    pub modified_at: DateTime<Utc>,
    
    /// When the instance was last played
    pub last_played: Option<DateTime<Utc>>,
    
    /// Total time played in seconds
    pub total_played_seconds: u64,
    
    /// Notes about this instance
    pub notes: String,
    
    /// Whether this instance is from a managed modpack
    pub managed_pack: Option<ManagedPack>,
    
    /// Instance status
    #[serde(default)]
    pub status: InstanceStatus,
}

#[allow(dead_code)] // Helper methods will be used as features are completed
impl Instance {
    /// Create a new instance with default settings
    pub fn new(name: String, path: PathBuf, minecraft_version: String) -> Self {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        Self {
            id,
            name,
            path,
            icon: "default".to_string(),
            group: None,
            minecraft_version,
            mod_loader: None,
            settings: InstanceSettings::default(),
            created_at: now,
            modified_at: now,
            last_played: None,
            total_played_seconds: 0,
            notes: String::new(),
            managed_pack: None,
            status: InstanceStatus::Ready,
        }
    }

    /// Get the path to the instance configuration file
    pub fn config_path(&self) -> PathBuf {
        self.path.join("instance.json")
    }

    /// Get the path to the minecraft game directory
    pub fn game_dir(&self) -> PathBuf {
        self.path.join(".minecraft")
    }

    /// Get the mods directory
    pub fn mods_dir(&self) -> PathBuf {
        self.game_dir().join("mods")
    }

    /// Get the resource packs directory
    pub fn resourcepacks_dir(&self) -> PathBuf {
        self.game_dir().join("resourcepacks")
    }

    /// Get the shader packs directory
    pub fn shaderpacks_dir(&self) -> PathBuf {
        self.game_dir().join("shaderpacks")
    }

    /// Get the saves directory
    pub fn saves_dir(&self) -> PathBuf {
        self.game_dir().join("saves")
    }

    /// Get the screenshots directory
    pub fn screenshots_dir(&self) -> PathBuf {
        self.game_dir().join("screenshots")
    }

    /// Get the logs directory
    pub fn logs_dir(&self) -> PathBuf {
        self.game_dir().join("logs")
    }

    /// Save the instance configuration
    pub fn save(&self) -> crate::core::error::Result<()> {
        let config_path = self.config_path();
        
        // Ensure instance directory exists
        std::fs::create_dir_all(&self.path)?;
        
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(config_path, content)?;
        
        Ok(())
    }

    /// Load instance from directory
    pub fn load(path: &PathBuf) -> crate::core::error::Result<Self> {
        let config_path = path.join("instance.json");
        let content = std::fs::read_to_string(config_path)?;
        let mut instance: Instance = serde_json::from_str(&content)?;
        instance.path = path.clone();
        Ok(instance)
    }

    /// Update last played time
    pub fn update_last_played(&mut self) {
        self.last_played = Some(Utc::now());
        self.modified_at = Utc::now();
    }

    /// Add play time
    pub fn add_play_time(&mut self, seconds: u64) {
        self.total_played_seconds += seconds;
        self.modified_at = Utc::now();
    }

    /// Check if instance has a mod loader
    pub fn has_mod_loader(&self) -> bool {
        self.mod_loader.is_some()
    }

    /// Get display string for mod loader
    pub fn mod_loader_display(&self) -> String {
        match &self.mod_loader {
            Some(loader) => format!("{} {}", loader.loader_type.name(), loader.version),
            None => "Vanilla".to_string(),
        }
    }
}

/// Instance status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
pub enum InstanceStatus {
    #[default]
    Ready,
    NeedsUpdate,
    Downloading,
    Running,
    Broken,
}

/// Instance type (vanilla or modded)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum InstanceType {
    #[default]
    Vanilla,
    Forge,
    NeoForge,
    Fabric,
    Quilt,
}

impl InstanceType {
    pub fn name(&self) -> &'static str {
        match self {
            InstanceType::Vanilla => "Vanilla",
            InstanceType::Forge => "Forge",
            InstanceType::NeoForge => "NeoForge",
            InstanceType::Fabric => "Fabric",
            InstanceType::Quilt => "Quilt",
        }
    }
    
    pub fn to_mod_loader_type(&self) -> Option<ModLoaderType> {
        match self {
            InstanceType::Vanilla => None,
            InstanceType::Forge => Some(ModLoaderType::Forge),
            InstanceType::NeoForge => Some(ModLoaderType::NeoForge),
            InstanceType::Fabric => Some(ModLoaderType::Fabric),
            InstanceType::Quilt => Some(ModLoaderType::Quilt),
        }
    }
}

/// Mod loader information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModLoader {
    pub loader_type: ModLoaderType,
    pub version: String,
}

/// Types of mod loaders
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModLoaderType {
    Forge,
    NeoForge,
    Fabric,
    Quilt,
    LiteLoader,
}

impl ModLoaderType {
    pub fn name(&self) -> &'static str {
        match self {
            ModLoaderType::Forge => "Forge",
            ModLoaderType::NeoForge => "NeoForge",
            ModLoaderType::Fabric => "Fabric",
            ModLoaderType::Quilt => "Quilt",
            ModLoaderType::LiteLoader => "LiteLoader",
        }
    }

    pub fn all() -> &'static [ModLoaderType] {
        &[
            ModLoaderType::Forge,
            ModLoaderType::NeoForge,
            ModLoaderType::Fabric,
            ModLoaderType::Quilt,
        ]
    }
}

/// Instance-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceSettings {
    /// Override global Java path
    pub java_path: Option<PathBuf>,
    
    /// Override JVM arguments
    pub jvm_args: Option<String>,
    
    /// Override game arguments
    pub game_args: Option<String>,
    
    /// Override minimum memory (MB)
    pub min_memory: Option<u32>,
    
    /// Override maximum memory (MB)
    pub max_memory: Option<u32>,
    
    /// Game window width
    pub window_width: Option<u32>,
    
    /// Game window height
    pub window_height: Option<u32>,
    
    /// Start in fullscreen
    #[serde(default)]
    pub fullscreen: bool,
    
    /// Pre-launch command
    pub pre_launch_command: Option<String>,
    
    /// Post-exit command
    pub post_exit_command: Option<String>,
    
    /// Wrapper command
    pub wrapper_command: Option<String>,
    
    /// Enable game time recording
    #[serde(default = "default_true")]
    pub record_play_time: bool,
    
    /// Show console when running
    #[serde(default)]
    pub show_console: bool,
    
    /// Auto-close console on game exit
    #[serde(default = "default_true")]
    pub auto_close_console: bool,
    
    /// Enable offline mode
    #[serde(default)]
    pub offline_mode: bool,
    
    /// Skip Java version compatibility check
    #[serde(default)]
    pub skip_java_compatibility_check: bool,
    
    /// Close launcher when game starts
    #[serde(default)]
    pub close_launcher_on_launch: bool,
    
    /// Quit launcher when game exits
    #[serde(default)]
    pub quit_launcher_on_exit: bool,
}

impl Default for InstanceSettings {
    fn default() -> Self {
        Self {
            java_path: None,
            jvm_args: None,
            game_args: None,
            min_memory: None,
            max_memory: None,
            window_width: None,
            window_height: None,
            fullscreen: false,
            pre_launch_command: None,
            post_exit_command: None,
            wrapper_command: None,
            record_play_time: true,
            show_console: false,
            auto_close_console: true,
            offline_mode: false,
            skip_java_compatibility_check: false,
            close_launcher_on_launch: false,
            quit_launcher_on_exit: false,
        }
    }
}

fn default_true() -> bool {
    true
}

/// Information about a managed modpack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagedPack {
    /// Platform the pack came from
    pub platform: ModpackPlatform,
    
    /// Pack ID on the platform
    pub pack_id: String,
    
    /// Pack name
    pub pack_name: String,
    
    /// Version ID
    pub version_id: String,
    
    /// Version name
    pub version_name: String,
}

/// Modpack platforms
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ModpackPlatform {
    Modrinth,
    CurseForge,
    ATLauncher,
    Technic,
    FTB,
}

/// Configuration for creating a new instance
#[derive(Debug, Clone)]
pub struct InstanceConfig {
    /// Instance name
    pub name: String,
    
    /// Group to put the instance in
    pub group: Option<String>,
    
    /// Icon name
    pub icon: String,
    
    /// Minecraft version to use
    pub minecraft_version: String,
    
    /// Mod loader to install
    pub mod_loader: Option<ModLoader>,
    
    /// Copy from existing instance
    pub copy_from: Option<PathBuf>,
    
    /// Import from modpack
    pub import_modpack: Option<ImportModpack>,
}

impl Default for InstanceConfig {
    fn default() -> Self {
        Self {
            name: "New Instance".to_string(),
            group: None,
            icon: "default".to_string(),
            minecraft_version: String::new(),
            mod_loader: None,
            copy_from: None,
            import_modpack: None,
        }
    }
}

/// Import modpack configuration
#[derive(Debug, Clone)]
pub struct ImportModpack {
    /// Source type
    pub source: ModpackSource,
    
    /// URL or file path
    pub location: String,
}

/// Modpack import source
#[derive(Debug, Clone)]
pub enum ModpackSource {
    /// Local file (mrpack, zip)
    File(PathBuf),
    
    /// Modrinth modpack
    Modrinth { project_id: String, version_id: String },
    
    /// CurseForge modpack
    CurseForge { project_id: u32, file_id: u32 },
    
    /// Direct URL
    Url(String),
}
