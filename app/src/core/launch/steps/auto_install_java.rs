//! Auto-install Java step - automatically downloads and installs Java if needed

use async_trait::async_trait;
use tracing::{info, warn};

use crate::core::launch::{LaunchContext, LaunchStep, LaunchStepResult};
use crate::core::java::{self, download::{fetch_adoptium_versions, fetch_adoptium_download, download_java}};

/// Step that automatically downloads and installs Java if needed
pub struct AutoInstallJavaStep {
    status: Option<String>,
    progress: f32,
}

impl AutoInstallJavaStep {
    pub fn new() -> Self {
        Self {
            status: None,
            progress: 0.0,
        }
    }
    
    /// Parse major version from Java version string
    fn parse_major_version(version: &str) -> Option<u32> {
        let version = version.trim();
        
        if version.starts_with("1.") {
            version.split('.').nth(1)?.parse().ok()
        } else {
            version.split('.').next()?.parse().ok()
        }
    }
    
    /// Check if the current Java is compatible
    fn is_java_compatible(java_version: &str, minecraft_version: &str) -> bool {
        let java_major = match Self::parse_major_version(java_version) {
            Some(v) => v,
            None => return false,
        };
        
        let required = java::get_required_java_version(minecraft_version);
        
        // Check if current Java meets requirement
        match required {
            8 => java_major == 8,
            16 => java_major >= 16 && java_major <= 17,
            17 => java_major >= 17,
            21 => java_major >= 21,
            _ => java_major >= required,
        }
    }
}

#[async_trait]
impl LaunchStep for AutoInstallJavaStep {
    fn name(&self) -> &'static str {
        "Auto-Install Java"
    }
    
    fn description(&self) -> &'static str {
        "Automatically downloads and installs Java if needed"
    }
    
    async fn execute(&mut self, context: &mut LaunchContext) -> LaunchStepResult {
        self.status = Some("Checking if Java needs to be installed...".to_string());
        self.progress = 0.0;
        
        // Check if auto-download is enabled
        if !context.config.java.auto_download {
            info!("Auto-download Java is disabled");
            return LaunchStepResult::Success;
        }
        
        // Check if we already have compatible Java
        if let Some(ref java_version) = context.java_version {
            if Self::is_java_compatible(java_version, &context.instance.minecraft_version) {
                info!("Current Java {} is compatible, no auto-install needed", java_version);
                return LaunchStepResult::Success;
            }
            warn!(
                "Current Java {} is not compatible with Minecraft {}, will try to auto-install",
                java_version, context.instance.minecraft_version
            );
        }
        
        // Determine required Java version
        let required_major = java::get_required_java_version(&context.instance.minecraft_version);
        info!(
            "Minecraft {} requires Java {}, checking for existing installation...",
            context.instance.minecraft_version, required_major
        );
        
        self.status = Some(format!("Looking for Java {}...", required_major));
        self.progress = 0.1;
        
        // First check if we already have the required Java installed
        if let Some(existing) = java::find_java_for_version(required_major) {
            info!("Found existing Java {} at {:?}", required_major, existing.path);
            context.java_path = Some(existing.path);
            context.java_version = Some(existing.version.to_string());
            context.java_architecture = Some(existing.arch.mojang_platform().to_string());
            self.status = Some(format!("Using existing Java {}", required_major));
            self.progress = 1.0;
            return LaunchStepResult::Success;
        }
        
        // Need to download Java
        self.status = Some(format!("Downloading Java {}...", required_major));
        self.progress = 0.2;
        
        info!("No compatible Java found, downloading Java {}", required_major);
        
        // Check available versions
        let available = match fetch_adoptium_versions().await {
            Ok(versions) => versions,
            Err(e) => {
                warn!("Failed to fetch available Java versions: {}", e);
                return LaunchStepResult::Failed(format!(
                    "Failed to check available Java versions: {}\n\n\
                     You can manually install Java {} from:\n\
                     https://adoptium.net/",
                    e, required_major
                ));
            }
        };
        
        // Check if our required version is available
        if !available.iter().any(|v| v.major == required_major) {
            return LaunchStepResult::Failed(format!(
                "Java {} is not available for automatic download.\n\n\
                 Please manually install Java {} from:\n\
                 https://adoptium.net/",
                required_major, required_major
            ));
        }
        
        self.progress = 0.3;
        
        // Fetch download metadata for the required version
        let metadata = match fetch_adoptium_download(required_major).await {
            Ok(m) => m,
            Err(e) => {
                return LaunchStepResult::Failed(format!(
                    "Failed to get Java {} download info: {}\n\n\
                     You can manually install Java from:\n\
                     https://adoptium.net/",
                    required_major, e
                ));
            }
        };
        
        self.progress = 0.4;
        
        // Download and install
        match download_java(&metadata, None).await {
            Ok(installation) => {
                info!(
                    "Successfully installed Java {} at {:?}",
                    required_major, installation.path
                );
                
                context.java_path = Some(installation.path);
                context.java_version = Some(installation.version.to_string());
                context.java_architecture = Some(installation.arch.mojang_platform().to_string());
                
                self.status = Some(format!("Installed Java {}", required_major));
                self.progress = 1.0;
                LaunchStepResult::Success
            }
            Err(e) => {
                LaunchStepResult::Failed(format!(
                    "Failed to download Java {}: {}\n\n\
                     You can manually install Java from:\n\
                     https://adoptium.net/",
                    required_major, e
                ))
            }
        }
    }
    
    fn progress(&self) -> f32 {
        self.progress
    }
    
    fn status(&self) -> Option<String> {
        self.status.clone()
    }
}

impl Default for AutoInstallJavaStep {
    fn default() -> Self {
        Self::new()
    }
}
