//! Launch game step - actually launches the Minecraft process

use async_trait::async_trait;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

use crate::core::launch::{LaunchContext, LaunchStep, LaunchStepResult, MessageLevel};
use crate::core::minecraft::version::{fetch_version_manifest, fetch_version_data, ArgumentValue, ArgumentValueInner};
use crate::core::minecraft::libraries::build_classpath;

/// Step that launches the actual game process
pub struct LaunchGameStep {
    status: Option<String>,
    progress: f32,
    process: Option<Arc<Mutex<Child>>>,
}

impl LaunchGameStep {
    pub fn new() -> Self {
        Self {
            status: None,
            progress: 0.0,
            process: None,
        }
    }
    
    /// Build JVM arguments
    fn build_jvm_args(&self, context: &LaunchContext, version_data: &crate::core::minecraft::version::VersionData) -> Vec<String> {
        let mut args = Vec::new();
        
        // Memory settings
        let min_mem = context.instance.settings.min_memory.unwrap_or(context.config.memory.min_memory);
        let max_mem = context.instance.settings.max_memory.unwrap_or(context.config.memory.max_memory);
        
        args.push(format!("-Xms{}M", min_mem));
        args.push(format!("-Xmx{}M", max_mem));
        
        // Build classpath
        let client_jar = context.config.meta_dir()
            .join("versions")
            .join(&context.instance.minecraft_version)
            .join(format!("{}.jar", &context.instance.minecraft_version));
        
        let classpath = build_classpath(version_data, &context.libraries_dir, &client_jar);
        
        // Process JVM arguments from version data
        if let Some(ref arguments) = version_data.arguments {
            for arg in &arguments.jvm {
                match arg {
                    ArgumentValue::Simple(s) => {
                        args.push(self.substitute_jvm_variable(s, context, &classpath));
                    }
                    ArgumentValue::Conditional { rules, value } => {
                        if crate::core::minecraft::version::evaluate_rules(rules) {
                            let values = match value {
                                ArgumentValueInner::Single(s) => vec![s.clone()],
                                ArgumentValueInner::Multiple(v) => v.clone(),
                            };
                            for v in values {
                                args.push(self.substitute_jvm_variable(&v, context, &classpath));
                            }
                        }
                    }
                }
            }
        } else {
            // Legacy JVM arguments
            args.push(format!("-Djava.library.path={}", context.natives_dir.to_string_lossy()));
            args.push("-cp".to_string());
            args.push(classpath);
        }
        
        // Custom JVM arguments from instance
        if let Some(ref custom_args) = context.instance.settings.jvm_args {
            args.extend(custom_args.split_whitespace().map(String::from));
        }
        
        // Extra JVM arguments from global config
        args.extend(context.config.java.extra_args.clone());
        
        args
    }
    
    /// Build game arguments
    fn build_game_args(&self, context: &LaunchContext, version_data: &crate::core::minecraft::version::VersionData) -> Vec<String> {
        let mut args = Vec::new();
        let instance = &context.instance;
        let game_dir = instance.game_dir();
        let assets_dir = &context.assets_dir;
        
        // Process game arguments
        if let Some(ref arguments) = version_data.arguments {
            for arg in &arguments.game {
                match arg {
                    ArgumentValue::Simple(s) => {
                        args.push(self.substitute_game_variable(s, context, version_data));
                    }
                    ArgumentValue::Conditional { rules, value } => {
                        if crate::core::minecraft::version::evaluate_rules(rules) {
                            let values = match value {
                                ArgumentValueInner::Single(s) => vec![s.clone()],
                                ArgumentValueInner::Multiple(v) => v.clone(),
                            };
                            for v in values {
                                args.push(self.substitute_game_variable(&v, context, version_data));
                            }
                        }
                    }
                }
            }
        } else if let Some(ref minecraft_arguments) = version_data.minecraft_arguments {
            // Legacy game arguments
            for arg in minecraft_arguments.split_whitespace() {
                args.push(self.substitute_game_variable(arg, context, version_data));
            }
        }
        
        // Window size
        if let Some(width) = instance.settings.window_width {
            args.push("--width".to_string());
            args.push(width.to_string());
        }
        if let Some(height) = instance.settings.window_height {
            args.push("--height".to_string());
            args.push(height.to_string());
        }
        
        // Fullscreen
        if instance.settings.fullscreen {
            args.push("--fullscreen".to_string());
        }
        
        // Custom game arguments
        if let Some(ref custom_args) = instance.settings.game_args {
            args.extend(custom_args.split_whitespace().map(String::from));
        }
        
        args
    }
    
    /// Substitute JVM argument variables
    fn substitute_jvm_variable(&self, template: &str, context: &LaunchContext, classpath: &str) -> String {
        template
            .replace("${natives_directory}", &context.natives_dir.to_string_lossy())
            .replace("${classpath}", classpath)
            .replace("${launcher_name}", "OxideLauncher")
            .replace("${launcher_version}", env!("CARGO_PKG_VERSION"))
            .replace("${classpath_separator}", if cfg!(target_os = "windows") { ";" } else { ":" })
            .replace("${library_directory}", &context.libraries_dir.to_string_lossy())
    }
    
    /// Substitute game argument variables
    fn substitute_game_variable(&self, template: &str, context: &LaunchContext, version_data: &crate::core::minecraft::version::VersionData) -> String {
        let instance = &context.instance;
        let game_dir = instance.game_dir();
        
        template
            .replace("${auth_player_name}", &context.auth_session.username)
            .replace("${auth_uuid}", &context.auth_session.uuid)
            .replace("${auth_access_token}", &context.auth_session.access_token)
            .replace("${user_type}", &context.auth_session.user_type)
            .replace("${version_name}", &instance.minecraft_version)
            .replace("${game_directory}", &game_dir.to_string_lossy())
            .replace("${assets_root}", &context.assets_dir.to_string_lossy())
            .replace("${assets_index_name}", &version_data.assets)
            .replace("${version_type}", &format!("{:?}", version_data.version_type))
            .replace("${user_properties}", "{}")
    }
}

#[async_trait]
impl LaunchStep for LaunchGameStep {
    fn name(&self) -> &'static str {
        "Launch Game"
    }
    
    fn description(&self) -> &'static str {
        "Launches the Minecraft process"
    }
    
    async fn execute(&mut self, context: &mut LaunchContext) -> LaunchStepResult {
        self.status = Some("Preparing to launch...".to_string());
        self.progress = 0.0;
        
        // Get Java path
        let java_path = match &context.java_path {
            Some(path) => path.clone(),
            None => {
                return LaunchStepResult::Failed("Java path not set. CheckJava step may have failed.".to_string());
            }
        };
        
        // Fetch version data
        self.status = Some("Fetching version data...".to_string());
        self.progress = 0.1;
        
        let manifest = match fetch_version_manifest().await {
            Ok(m) => m,
            Err(e) => return LaunchStepResult::Failed(format!("Failed to fetch version manifest: {}", e)),
        };
        
        let version_info = match manifest.get_version(&context.instance.minecraft_version) {
            Some(v) => v,
            None => return LaunchStepResult::Failed(format!(
                "Version {} not found", context.instance.minecraft_version
            )),
        };
        
        let version_data = match fetch_version_data(version_info).await {
            Ok(d) => d,
            Err(e) => return LaunchStepResult::Failed(format!("Failed to fetch version data: {}", e)),
        };
        
        self.progress = 0.3;
        
        // Build arguments
        self.status = Some("Building launch arguments...".to_string());
        
        let mut args = self.build_jvm_args(context, &version_data);
        
        // Add main class
        args.push(version_data.main_class.clone());
        
        // Add game arguments
        args.extend(self.build_game_args(context, &version_data));
        
        self.progress = 0.5;
        
        // Handle wrapper command
        let (program, final_args) = if let Some(ref wrapper) = context.instance.settings.wrapper_command {
            if !wrapper.trim().is_empty() {
                // Split wrapper command
                let mut wrapper_parts: Vec<String> = wrapper.split_whitespace().map(String::from).collect();
                
                // Add Java and its args
                wrapper_parts.push(java_path.to_string_lossy().to_string());
                wrapper_parts.extend(args);
                
                let program = wrapper_parts.remove(0);
                (program, wrapper_parts)
            } else {
                (java_path.to_string_lossy().to_string(), args)
            }
        } else {
            (java_path.to_string_lossy().to_string(), args)
        };
        
        // Log launch command (debug)
        debug!("Launch command: {} {}", program, final_args.join(" "));
        
        // Launch the game
        self.status = Some("Starting Minecraft...".to_string());
        self.progress = 0.7;
        
        let game_dir = context.instance.game_dir();
        
        info!("Launching Minecraft from: {:?}", game_dir);
        
        let child = match Command::new(&program)
            .args(&final_args)
            .current_dir(&game_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                return LaunchStepResult::Failed(format!(
                    "Failed to start Minecraft: {}\n\nCommand: {} {}",
                    e, program, final_args.join(" ")
                ));
            }
        };
        
        info!("Minecraft process started with PID: {}", child.id());
        
        self.process = Some(Arc::new(Mutex::new(child)));
        
        self.status = Some("Minecraft is running".to_string());
        self.progress = 1.0;
        
        LaunchStepResult::Success
    }
    
    fn can_abort(&self) -> bool {
        true
    }
    
    async fn abort(&mut self) -> bool {
        if let Some(ref process) = self.process {
            if let Ok(mut child) = process.lock() {
                info!("Killing Minecraft process");
                return child.kill().is_ok();
            }
        }
        false
    }
    
    fn progress(&self) -> f32 {
        self.progress
    }
    
    fn status(&self) -> Option<String> {
        self.status.clone()
    }
}

impl Default for LaunchGameStep {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the child process (for external monitoring)
impl LaunchGameStep {
    pub fn get_process(&self) -> Option<Arc<Mutex<Child>>> {
        self.process.clone()
    }
}
