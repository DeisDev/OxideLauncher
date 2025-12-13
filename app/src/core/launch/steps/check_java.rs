//! Check Java step - validates that a Java installation exists and is usable

use async_trait::async_trait;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, info, warn};

use crate::core::launch::{LaunchContext, LaunchStep, LaunchStepResult};
use crate::core::java;

/// Step that checks for a valid Java installation
pub struct CheckJavaStep {
    status: Option<String>,
    progress: f32,
}

impl CheckJavaStep {
    pub fn new() -> Self {
        Self {
            status: None,
            progress: 0.0,
        }
    }
    
    /// Resolve Java executable path from a setting
    fn resolve_java_path(&self, path: &str) -> Option<PathBuf> {
        let path = PathBuf::from(path);
        
        // Check if it's already a direct path
        if path.exists() {
            return Some(path);
        }
        
        // Try to find it in PATH
        if let Ok(resolved) = which::which(&path) {
            return Some(resolved);
        }
        
        None
    }
    
    /// Validate a Java installation by running it
    async fn validate_java(&self, java_path: &PathBuf) -> Option<JavaInfo> {
        debug!("Validating Java at: {:?}", java_path);
        
        // Run java -XshowSettings:all -version
        let output = Command::new(java_path)
            .args(["-XshowSettings:all", "-version"])
            .output()
            .ok()?;
        
        // Parse output (version info is typically in stderr)
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let combined = format!("{}\n{}", stdout, stderr);
        
        // Extract version
        let version = self.parse_java_version(&combined)?;
        
        // Extract architecture
        let architecture = self.parse_java_architecture(&combined);
        
        // Extract real architecture (for checking 32-bit on 64-bit)
        let real_architecture = self.parse_real_architecture(&combined);
        
        // Extract vendor
        let vendor = self.parse_java_vendor(&combined);
        
        Some(JavaInfo {
            version,
            architecture,
            real_architecture,
            vendor,
        })
    }
    
    fn parse_java_version(&self, output: &str) -> Option<String> {
        // Look for version patterns like:
        // java version "1.8.0_292"
        // openjdk version "17.0.1"
        // java version "21.0.1"
        
        for line in output.lines() {
            let line = line.trim();
            if line.contains("version") && (line.contains("java") || line.contains("openjdk")) {
                // Extract version from quotes
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        return Some(line[start + 1..start + 1 + end].to_string());
                    }
                }
            }
        }
        
        None
    }
    
    fn parse_java_architecture(&self, output: &str) -> String {
        // Look for sun.arch.data.model in -XshowSettings output
        for line in output.lines() {
            let line = line.trim();
            if line.contains("sun.arch.data.model") {
                if line.contains("64") {
                    return "64".to_string();
                } else if line.contains("32") {
                    return "32".to_string();
                }
            }
            // Also check os.arch
            if line.contains("os.arch") {
                if line.contains("amd64") || line.contains("x86_64") || line.contains("aarch64") {
                    return "64".to_string();
                } else if line.contains("i386") || line.contains("x86") {
                    return "32".to_string();
                }
            }
        }
        
        "64".to_string() // Default assumption
    }
    
    fn parse_real_architecture(&self, output: &str) -> String {
        // This would be the actual OS architecture, not JVM
        // For now, use the same as JVM architecture
        self.parse_java_architecture(output)
    }
    
    fn parse_java_vendor(&self, output: &str) -> String {
        // Look for vendor info
        for line in output.lines() {
            let line = line.trim();
            if line.contains("java.vendor") && line.contains("=") {
                if let Some(value) = line.split('=').nth(1) {
                    return value.trim().to_string();
                }
            }
        }
        
        // Fallback: check for common vendors in output
        let output_lower = output.to_lowercase();
        if output_lower.contains("adoptium") || output_lower.contains("temurin") {
            "Eclipse Adoptium".to_string()
        } else if output_lower.contains("azul") || output_lower.contains("zulu") {
            "Azul Systems".to_string()
        } else if output_lower.contains("openjdk") {
            "OpenJDK".to_string()
        } else if output_lower.contains("oracle") {
            "Oracle".to_string()
        } else {
            "Unknown".to_string()
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)] // Fields used for debugging/logging
struct JavaInfo {
    version: String,
    architecture: String,
    real_architecture: String,
    vendor: String,
}

#[async_trait]
impl LaunchStep for CheckJavaStep {
    fn name(&self) -> &'static str {
        "Check Java"
    }
    
    fn description(&self) -> &'static str {
        "Validates that a suitable Java installation is available"
    }
    
    async fn execute(&mut self, context: &mut LaunchContext) -> LaunchStepResult {
        self.status = Some("Checking Java installation...".to_string());
        self.progress = 0.0;
        
        let instance = &context.instance;
        let config = &context.config;
        
        // Determine Java path to use
        let java_path: Option<PathBuf>;
        #[allow(unused_assignments)] // Used for debugging/logging purposes
        let mut _per_instance: bool = false;
        
        // Check instance-specific Java first
        if let Some(ref instance_java) = instance.settings.java_path {
            if let Some(resolved) = self.resolve_java_path(&instance_java.to_string_lossy()) {
                java_path = Some(resolved);
                _per_instance = true;
                info!("Using instance-specific Java: {:?}", java_path);
            } else {
                return LaunchStepResult::Failed(format!(
                    "Instance Java path not found: {:?}\n\
                     Please set up Java in the launcher's Java settings.",
                    instance_java
                ));
            }
        }
        // Check global custom Java path
        else if let Some(ref custom_path) = config.java.custom_path {
            if let Some(resolved) = self.resolve_java_path(&custom_path.to_string_lossy()) {
                java_path = Some(resolved);
                _per_instance = false;
                info!("Using global custom Java: {:?}", java_path);
            } else {
                return LaunchStepResult::Failed(format!(
                    "Global Java path not found: {:?}\n\
                     Please set up Java in the launcher's Java settings.",
                    custom_path
                ));
            }
        }
        // Auto-detect Java
        else if config.java.auto_detect {
            self.status = Some("Auto-detecting Java...".to_string());
            
            // Get required Java version for this Minecraft version
            let required = java::get_required_java_version(&instance.minecraft_version);
            
            if let Some(detected) = java::find_java_for_version(required) {
                java_path = Some(detected.path);
                _per_instance = false;
                info!("Auto-detected Java for version {}: {:?}", required, java_path);
            } else {
                // Fall back to any Java in PATH
                let java_exe = if cfg!(target_os = "windows") { "java.exe" } else { "java" };
                if let Ok(path) = which::which(java_exe) {
                    java_path = Some(path);
                    _per_instance = false;
                    warn!("Using fallback Java from PATH");
                } else {
                    return LaunchStepResult::Failed(
                        "No Java installation found.\n\
                         Please install Java or configure it in the launcher settings.\n\n\
                         You can download Java from:\n\
                         - https://adoptium.net/ (Recommended)\n\
                         - https://www.azul.com/downloads/".to_string()
                    );
                }
            }
        }
        // Fall back to PATH
        else {
            let java_exe = if cfg!(target_os = "windows") { "java.exe" } else { "java" };
            if let Ok(path) = which::which(java_exe) {
                java_path = Some(path);
                _per_instance = false;
            } else {
                return LaunchStepResult::Failed(
                    "No Java installation found in PATH.\n\
                     Please install Java or configure it in the launcher settings.".to_string()
                );
            }
        }
        
        let java_path = java_path.unwrap();
        
        // Validate the Java installation
        self.status = Some("Validating Java...".to_string());
        self.progress = 0.5;
        
        match self.validate_java(&java_path).await {
            Some(info) => {
                info!(
                    "Java validated: {} ({}) from {}",
                    info.version, info.architecture, info.vendor
                );
                
                // Store in context
                context.java_path = Some(java_path.clone());
                context.java_version = Some(info.version.clone());
                context.java_architecture = Some(info.architecture.clone());
                
                self.status = Some(format!(
                    "Using Java {} ({}-bit) from {}",
                    info.version,
                    info.architecture,
                    info.vendor
                ));
                self.progress = 1.0;
                
                LaunchStepResult::Success
            }
            None => {
                LaunchStepResult::Failed(format!(
                    "Failed to validate Java installation at {:?}\n\
                     The Java executable may be corrupted or incompatible.",
                    java_path
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

impl Default for CheckJavaStep {
    fn default() -> Self {
        Self::new()
    }
}
