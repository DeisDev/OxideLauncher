//! Print instance info step - logs instance information before launch

use async_trait::async_trait;
use tracing::info;

use crate::core::launch::{LaunchContext, LaunchStep, LaunchStepResult};

/// Step that prints instance information
pub struct PrintInstanceInfoStep {
    status: Option<String>,
    progress: f32,
}

impl PrintInstanceInfoStep {
    pub fn new() -> Self {
        Self {
            status: None,
            progress: 0.0,
        }
    }
}

#[async_trait]
impl LaunchStep for PrintInstanceInfoStep {
    fn name(&self) -> &'static str {
        "Print Instance Info"
    }
    
    fn description(&self) -> &'static str {
        "Logs instance information"
    }
    
    async fn execute(&mut self, context: &mut LaunchContext) -> LaunchStepResult {
        self.status = Some("Logging instance info...".to_string());
        self.progress = 0.0;
        
        let instance = &context.instance;
        
        info!("=== Instance Information ===");
        info!("Name: {}", instance.name);
        info!("ID: {}", instance.id);
        info!("Minecraft Version: {}", instance.minecraft_version);
        
        if let Some(ref loader) = instance.mod_loader {
            info!("Mod Loader: {} {}", loader.loader_type.name(), loader.version);
        } else {
            info!("Mod Loader: Vanilla");
        }
        
        info!("Instance Path: {:?}", instance.path);
        info!("Game Directory: {:?}", instance.game_dir());
        
        self.progress = 0.5;
        
        // Log settings
        info!("=== Instance Settings ===");
        
        if let Some(ref java_path) = instance.settings.java_path {
            info!("Custom Java Path: {:?}", java_path);
        }
        
        if let Some(min) = instance.settings.min_memory {
            info!("Min Memory: {} MB", min);
        }
        if let Some(max) = instance.settings.max_memory {
            info!("Max Memory: {} MB", max);
        }
        
        if let Some(w) = instance.settings.window_width {
            if let Some(h) = instance.settings.window_height {
                info!("Window Size: {}x{}", w, h);
            }
        }
        
        if instance.settings.fullscreen {
            info!("Fullscreen: enabled");
        }
        
        if instance.settings.skip_java_compatibility_check {
            info!("Java Compatibility Check: SKIPPED");
        }
        
        // Log system info
        info!("=== System Information ===");
        info!("OS: {}", std::env::consts::OS);
        info!("Architecture: {}", std::env::consts::ARCH);
        
        // Log auth info (censored)
        info!("=== Authentication ===");
        info!("Username: {}", context.auth_session.username);
        info!("User Type: {}", context.auth_session.user_type);
        
        info!("===========================");
        
        self.status = Some("Instance info logged".to_string());
        self.progress = 1.0;
        
        LaunchStepResult::Success
    }
    
    fn progress(&self) -> f32 {
        self.progress
    }
    
    fn status(&self) -> Option<String> {
        self.status.clone()
    }
}

impl Default for PrintInstanceInfoStep {
    fn default() -> Self {
        Self::new()
    }
}
