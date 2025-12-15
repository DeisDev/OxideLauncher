//! Launch game step - actually launches the Minecraft process

use async_trait::async_trait;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

use crate::core::launch::{LaunchContext, LaunchStep, LaunchStepResult};
use crate::core::minecraft::version::{fetch_version_manifest, fetch_version_data, ArgumentValue, ArgumentValueInner, evaluate_rules_with_features, VersionData};
use crate::core::minecraft::libraries::build_classpath;
use crate::core::modloaders::ModloaderProfile;

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

    /// Load the modloader profile if it exists
    fn load_modloader_profile(&self, context: &LaunchContext) -> Option<ModloaderProfile> {
        let profile_path = context.instance.path.join("modloader_profile.json");
        
        tracing::debug!("Looking for modloader profile at: {:?}", profile_path);
        
        if profile_path.exists() {
            tracing::debug!("Modloader profile file found, attempting to load...");
            match ModloaderProfile::load(&profile_path) {
                Ok(profile) => {
                    tracing::info!(
                        "Loaded modloader profile: uid='{}', version='{}', main_class='{}', libraries={}",
                        profile.uid,
                        profile.version,
                        profile.main_class,
                        profile.libraries.len()
                    );
                    
                    // Log JVM and game arguments
                    if !profile.jvm_arguments.is_empty() {
                        tracing::debug!("JVM arguments: {:?}", profile.jvm_arguments);
                    }
                    if !profile.game_arguments.is_empty() {
                        tracing::debug!("Game arguments: {:?}", profile.game_arguments);
                    }
                    if !profile.tweakers.is_empty() {
                        tracing::debug!("Tweakers: {:?}", profile.tweakers);
                    }
                    
                    Some(profile)
                }
                Err(e) => {
                    tracing::error!("Failed to load modloader profile: {}", e);
                    tracing::error!("Falling back to vanilla launch");
                    None
                }
            }
        } else {
            tracing::debug!("No modloader profile found at {:?}", profile_path);
            
            // Also check if the instance has a mod_loader configured but profile is missing
            if context.instance.mod_loader.is_some() {
                tracing::warn!(
                    "Instance '{}' has modloader configured but profile file is missing!",
                    context.instance.name
                );
                tracing::warn!("The game will launch in vanilla mode. Re-setup the instance to fix this.");
            }
            
            None
        }
    }

    /// Build the classpath including modloader libraries
    fn build_full_classpath(
        &self,
        context: &LaunchContext,
        version_data: &VersionData,
        modloader_profile: Option<&ModloaderProfile>,
    ) -> String {
        let separator = if cfg!(target_os = "windows") { ";" } else { ":" };
        let mut paths: Vec<String> = Vec::new();

        // First add modloader libraries (they need to be before vanilla libs)
        if let Some(profile) = modloader_profile {
            tracing::debug!("Adding {} modloader libraries to classpath", profile.libraries.len());
            let mut found_count = 0;
            let mut missing_count = 0;
            
            for lib in &profile.libraries {
                if !lib.applies_to_current_os() {
                    tracing::trace!("Skipping library {} (not for current OS)", lib.name);
                    continue;
                }
                let lib_path = context.libraries_dir.join(lib.get_path());
                if lib_path.exists() {
                    paths.push(lib_path.to_string_lossy().to_string());
                    found_count += 1;
                } else {
                    tracing::warn!("Modloader library not found: {} (expected at {:?})", lib.name, lib_path);
                    missing_count += 1;
                }
            }
            
            tracing::debug!("Modloader libraries: {} found, {} missing", found_count, missing_count);
            
            if missing_count > 0 {
                tracing::warn!("Some modloader libraries are missing! The game may not launch correctly.");
            }
        }

        // Then add vanilla libraries and client jar
        let client_jar = context.config.meta_dir()
            .join("versions")
            .join(&context.instance.minecraft_version)
            .join(format!("{}.jar", &context.instance.minecraft_version));

        let vanilla_classpath = build_classpath(version_data, &context.libraries_dir, &client_jar);
        
        // Append vanilla classpath entries (avoiding duplicates)
        for path in vanilla_classpath.split(separator) {
            if !paths.contains(&path.to_string()) {
                paths.push(path.to_string());
            }
        }
        
        tracing::debug!("Total classpath entries: {}", paths.len());

        paths.join(separator)
    }

    /// Get the main class (modloader overrides vanilla if present)
    fn get_main_class(&self, version_data: &VersionData, modloader_profile: Option<&ModloaderProfile>) -> String {
        if let Some(profile) = modloader_profile {
            if !profile.main_class.is_empty() {
                tracing::info!("Using modloader main class: {}", profile.main_class);
                return profile.main_class.clone();
            } else {
                tracing::warn!("Modloader profile has empty main_class, falling back to vanilla");
            }
        }
        
        tracing::info!("Using vanilla main class: {}", version_data.main_class);
        version_data.main_class.clone()
    }
    
    /// Build JVM arguments
    fn build_jvm_args(
        &self,
        context: &LaunchContext,
        version_data: &VersionData,
        modloader_profile: Option<&ModloaderProfile>,
    ) -> Vec<String> {
        let mut args = Vec::new();
        
        // Memory settings
        let min_mem = context.instance.settings.min_memory.unwrap_or(context.config.memory.min_memory);
        let max_mem = context.instance.settings.max_memory.unwrap_or(context.config.memory.max_memory);
        
        args.push(format!("-Xms{}M", min_mem));
        args.push(format!("-Xmx{}M", max_mem));

        // Build classpath
        let classpath = self.build_full_classpath(context, version_data, modloader_profile);
        
        // Process JVM arguments from version data
        if let Some(ref arguments) = version_data.arguments {
            for arg in &arguments.jvm {
                match arg {
                    ArgumentValue::Simple(s) => {
                        args.push(self.substitute_jvm_variable(s, context, &classpath));
                    }
                    ArgumentValue::Conditional { rules, value } => {
                        // Use features-aware rule evaluation
                        if evaluate_rules_with_features(rules, &context.features) {
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
        let _game_dir = instance.game_dir(); // Used in substitute_game_variable
        let _assets_dir = &context.assets_dir; // Used in substitute_game_variable
        
        // Process game arguments
        if let Some(ref arguments) = version_data.arguments {
            for arg in &arguments.game {
                match arg {
                    ArgumentValue::Simple(s) => {
                        args.push(self.substitute_game_variable(s, context, version_data));
                    }
                    ArgumentValue::Conditional { rules, value } => {
                        // Use features-aware rule evaluation
                        if evaluate_rules_with_features(rules, &context.features) {
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
        
        // Window size - use instance settings if set, otherwise global config
        let width = instance.settings.window_width.unwrap_or(context.config.minecraft.window_width);
        let height = instance.settings.window_height.unwrap_or(context.config.minecraft.window_height);
        
        args.push("--width".to_string());
        args.push(width.to_string());
        args.push("--height".to_string());
        args.push(height.to_string());
        
        // Fullscreen/Launch maximized - instance setting takes priority
        if instance.settings.fullscreen || context.config.minecraft.launch_maximized {
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
        
        // For offline accounts, provide placeholder values instead of empty strings
        // This prevents argument parsing issues where --argName followed by empty value
        // causes the next argument to be interpreted as the value
        let access_token = if context.auth_session.access_token.is_empty() { 
            "0".to_string() 
        } else { 
            context.auth_session.access_token.clone() 
        };
        let client_id = if context.auth_session.client_id.is_empty() { 
            "0".to_string() 
        } else { 
            context.auth_session.client_id.clone() 
        };
        let xuid = if context.auth_session.xuid.is_empty() { 
            "0".to_string() 
        } else { 
            context.auth_session.xuid.clone() 
        };
        
        template
            .replace("${auth_player_name}", &context.auth_session.username)
            .replace("${auth_uuid}", &context.auth_session.uuid)
            .replace("${auth_access_token}", &access_token)
            .replace("${user_type}", &context.auth_session.user_type)
            .replace("${version_name}", &instance.minecraft_version)
            .replace("${game_directory}", &game_dir.to_string_lossy())
            .replace("${assets_root}", &context.assets_dir.to_string_lossy())
            .replace("${assets_index_name}", &version_data.assets)
            .replace("${version_type}", &format!("{:?}", version_data.version_type))
            .replace("${user_properties}", "{}")
            // Microsoft/Xbox authentication variables (required for 1.16.4+)
            .replace("${clientid}", &client_id)
            .replace("${auth_xuid}", &xuid)
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
        
        // Load modloader profile if present
        let modloader_profile = self.load_modloader_profile(context);
        
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
        
        // Build JVM arguments with modloader support
        let mut args = self.build_jvm_args(context, &version_data, modloader_profile.as_ref());
        
        // Add modloader-specific JVM arguments
        if let Some(ref profile) = modloader_profile {
            for arg in &profile.jvm_arguments {
                let substituted = self.substitute_jvm_variable(
                    arg, 
                    context, 
                    &self.build_full_classpath(context, &version_data, Some(profile))
                );
                args.push(substituted);
            }
        }
        
        // Add main class (modloader overrides vanilla)
        let main_class = self.get_main_class(&version_data, modloader_profile.as_ref());
        args.push(main_class);
        
        // Add game arguments
        args.extend(self.build_game_args(context, &version_data));
        
        // Add modloader-specific game arguments
        if let Some(ref profile) = modloader_profile {
            for arg in &profile.game_arguments {
                args.push(self.substitute_game_variable(arg, context, &version_data));
            }
            
            // Add tweaker classes (for legacy modloaders like old Forge/LiteLoader)
            for tweaker in &profile.tweakers {
                args.push("--tweakClass".to_string());
                args.push(tweaker.clone());
            }
        }
        
        self.progress = 0.5;
        
        // Handle wrapper command - use instance-specific if set, otherwise global config
        let wrapper_command = context.instance.settings.wrapper_command.clone()
            .or_else(|| context.config.commands.wrapper_command.clone());
            
        let (program, final_args) = if let Some(ref wrapper) = wrapper_command {
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
        
        // On Windows, prefer javaw.exe over java.exe to avoid console window
        #[cfg(target_os = "windows")]
        let program = {
            let program_path = std::path::Path::new(&program);
            if program_path.file_name().map(|f| f.to_string_lossy().to_lowercase()) == Some("java.exe".into()) {
                // Try to use javaw.exe instead
                let javaw_path = program_path.with_file_name("javaw.exe");
                if javaw_path.exists() {
                    info!("Using javaw.exe instead of java.exe to avoid console window");
                    javaw_path.to_string_lossy().to_string()
                } else {
                    program
                }
            } else {
                program
            }
        };
        
        #[cfg(not(target_os = "windows"))]
        let program = program;
        
        // Log launch command (debug)
        debug!("Launch command: {} {}", program, final_args.join(" "));
        
        // Launch the game
        self.status = Some("Starting Minecraft...".to_string());
        self.progress = 0.7;
        
        let game_dir = context.instance.game_dir();
        
        info!("Launching Minecraft from: {:?}", game_dir);
        
        let mut command = Command::new(&program);
        command
            .args(&final_args)
            .current_dir(&game_dir)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        // On Windows, use CREATE_NO_WINDOW to prevent console window.
        // Note: This works with javaw.exe. If using java.exe, a console may still appear.
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            command.creation_flags(CREATE_NO_WINDOW);
        }
        
        let child = match command.spawn() {
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
    
    fn get_game_process(&self) -> Option<Arc<Mutex<Child>>> {
        self.process.clone()
    }
}

impl Default for LaunchGameStep {
    fn default() -> Self {
        Self::new()
    }
}
