//! Game launch system

use std::path::PathBuf;
use std::process::{Command, Stdio};
use crate::core::error::{OxideError, Result};
use crate::core::instance::Instance;
use crate::core::accounts::{Account, AuthSession};
use crate::core::config::Config;
use crate::core::minecraft::version::{VersionData, fetch_version_data, ArgumentValue, ArgumentValueInner, current_os_name, current_arch};
use crate::core::java;

/// Launch a Minecraft instance
pub async fn launch_instance(
    instance: &Instance,
    account: Option<&Account>,
) -> Result<()> {
    let config = Config::load().unwrap_or_default();
    
    // Get version data
    let manifest = crate::core::minecraft::version::fetch_version_manifest().await?;
    let version_info = manifest.get_version(&instance.minecraft_version)
        .ok_or_else(|| OxideError::Launch(format!(
            "Version {} not found", instance.minecraft_version
        )))?;
    
    let version_data = fetch_version_data(version_info).await?;
    
    // Create auth session
    let auth_session = if let Some(acc) = account {
        AuthSession::from_account(acc)
    } else if instance.settings.offline_mode {
        AuthSession::offline("Player")
    } else {
        return Err(OxideError::Launch("No account selected".into()));
    };
    
    // Find Java
    let java_path = find_java(instance, &config, &version_data)?;
    
    // Build launch command
    let launch_command = build_launch_command(
        instance,
        &config,
        &version_data,
        &auth_session,
        &java_path,
    )?;
    
    tracing::info!("Launching with command: {:?}", launch_command);
    
    // Run pre-launch command if configured
    if let Some(pre_launch) = &instance.settings.pre_launch_command {
        run_command(pre_launch, &instance.game_dir())?;
    }
    
    // Launch the game
    let mut child = Command::new(&java_path)
        .args(&launch_command)
        .current_dir(&instance.game_dir())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| OxideError::Launch(format!("Failed to start game: {}", e)))?;
    
    // Wait for process to complete (in background)
    tokio::spawn(async move {
        let _ = child.wait();
    });
    
    Ok(())
}

/// Find Java executable to use
fn find_java(
    instance: &Instance,
    config: &Config,
    version_data: &VersionData,
) -> Result<PathBuf> {
    // Check instance-specific Java path
    if let Some(java_path) = &instance.settings.java_path {
        if java_path.exists() {
            return Ok(java_path.clone());
        }
    }
    
    // Check global custom Java path
    if let Some(java_path) = &config.java.custom_path {
        if java_path.exists() {
            return Ok(java_path.clone());
        }
    }
    
    // Get required Java version
    let required_version = version_data.java_version
        .as_ref()
        .map(|j| j.major_version)
        .unwrap_or(8);
    
    // Try to auto-detect Java
    if config.java.auto_detect {
        if let Some(java) = java::find_java_installation(required_version) {
            return Ok(java);
        }
    }
    
    // Fallback to PATH
    let java_exe = if cfg!(target_os = "windows") { "java.exe" } else { "java" };
    which::which(java_exe)
        .map_err(|_| OxideError::Java("No suitable Java installation found".into()))
}

/// Build the launch command arguments
fn build_launch_command(
    instance: &Instance,
    config: &Config,
    version_data: &VersionData,
    auth_session: &AuthSession,
    java_path: &PathBuf,
) -> Result<Vec<String>> {
    let mut args = Vec::new();
    
    // Memory settings
    let min_mem = instance.settings.min_memory.unwrap_or(config.memory.min_memory);
    let max_mem = instance.settings.max_memory.unwrap_or(config.memory.max_memory);
    
    args.push(format!("-Xms{}M", min_mem));
    args.push(format!("-Xmx{}M", max_mem));
    
    // Build paths
    let game_dir = instance.game_dir();
    let natives_dir = game_dir.join("natives");
    let libraries_dir = config.libraries_dir();
    let assets_dir = config.assets_dir();
    let client_jar = config.meta_dir()
        .join("versions")
        .join(&instance.minecraft_version)
        .join(format!("{}.jar", &instance.minecraft_version));
    
    // JVM arguments
    let jvm_args = build_jvm_arguments(
        version_data,
        &natives_dir,
        &libraries_dir,
        &client_jar,
    );
    args.extend(jvm_args);
    
    // Custom JVM arguments
    if let Some(custom_args) = &instance.settings.jvm_args {
        args.extend(custom_args.split_whitespace().map(String::from));
    }
    
    // Extra JVM arguments from config
    args.extend(config.java.extra_args.clone());
    
    // Main class
    args.push(version_data.main_class.clone());
    
    // Game arguments
    let game_args = build_game_arguments(
        version_data,
        instance,
        auth_session,
        &game_dir,
        &assets_dir,
    );
    args.extend(game_args);
    
    // Custom game arguments
    if let Some(custom_args) = &instance.settings.game_args {
        args.extend(custom_args.split_whitespace().map(String::from));
    }
    
    Ok(args)
}

/// Build JVM arguments from version data
fn build_jvm_arguments(
    version_data: &VersionData,
    natives_dir: &PathBuf,
    libraries_dir: &PathBuf,
    client_jar: &PathBuf,
) -> Vec<String> {
    let mut args = Vec::new();
    
    // Build classpath
    let classpath = crate::core::minecraft::libraries::build_classpath(
        version_data,
        libraries_dir,
        client_jar,
    );
    
    // Process JVM arguments
    if let Some(arguments) = &version_data.arguments {
        for arg in &arguments.jvm {
            match arg {
                ArgumentValue::Simple(s) => {
                    args.push(substitute_variables(s, &SubstituteContext {
                        natives_directory: natives_dir.to_string_lossy().to_string(),
                        classpath: classpath.clone(),
                        launcher_name: "OxideLauncher".to_string(),
                        launcher_version: env!("CARGO_PKG_VERSION").to_string(),
                        ..Default::default()
                    }));
                }
                ArgumentValue::Conditional { rules, value } => {
                    if crate::core::minecraft::version::evaluate_rules(rules) {
                        let values = match value {
                            ArgumentValueInner::Single(s) => vec![s.clone()],
                            ArgumentValueInner::Multiple(v) => v.clone(),
                        };
                        for v in values {
                            args.push(substitute_variables(&v, &SubstituteContext {
                                natives_directory: natives_dir.to_string_lossy().to_string(),
                                classpath: classpath.clone(),
                                launcher_name: "OxideLauncher".to_string(),
                                launcher_version: env!("CARGO_PKG_VERSION").to_string(),
                                ..Default::default()
                            }));
                        }
                    }
                }
            }
        }
    } else {
        // Legacy JVM arguments
        args.push(format!("-Djava.library.path={}", natives_dir.to_string_lossy()));
        args.push("-cp".to_string());
        args.push(classpath);
    }
    
    args
}

/// Build game arguments from version data
fn build_game_arguments(
    version_data: &VersionData,
    instance: &Instance,
    auth_session: &AuthSession,
    game_dir: &PathBuf,
    assets_dir: &PathBuf,
) -> Vec<String> {
    let mut args = Vec::new();
    
    let context = SubstituteContext {
        auth_player_name: auth_session.username.clone(),
        auth_uuid: auth_session.uuid.clone(),
        auth_access_token: auth_session.access_token.clone(),
        user_type: auth_session.user_type.clone(),
        version_name: instance.minecraft_version.clone(),
        game_directory: game_dir.to_string_lossy().to_string(),
        assets_root: assets_dir.to_string_lossy().to_string(),
        assets_index_name: version_data.assets.clone(),
        version_type: format!("{:?}", version_data.version_type),
        ..Default::default()
    };
    
    if let Some(arguments) = &version_data.arguments {
        for arg in &arguments.game {
            match arg {
                ArgumentValue::Simple(s) => {
                    args.push(substitute_variables(s, &context));
                }
                ArgumentValue::Conditional { rules, value } => {
                    if crate::core::minecraft::version::evaluate_rules(rules) {
                        let values = match value {
                            ArgumentValueInner::Single(s) => vec![s.clone()],
                            ArgumentValueInner::Multiple(v) => v.clone(),
                        };
                        for v in values {
                            args.push(substitute_variables(&v, &context));
                        }
                    }
                }
            }
        }
    } else if let Some(minecraft_arguments) = &version_data.minecraft_arguments {
        // Legacy game arguments
        for arg in minecraft_arguments.split_whitespace() {
            args.push(substitute_variables(arg, &context));
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
    
    args
}

/// Context for variable substitution
#[derive(Default)]
struct SubstituteContext {
    auth_player_name: String,
    auth_uuid: String,
    auth_access_token: String,
    user_type: String,
    version_name: String,
    game_directory: String,
    assets_root: String,
    assets_index_name: String,
    version_type: String,
    natives_directory: String,
    classpath: String,
    launcher_name: String,
    launcher_version: String,
}

/// Substitute variables in a string
fn substitute_variables(template: &str, ctx: &SubstituteContext) -> String {
    template
        .replace("${auth_player_name}", &ctx.auth_player_name)
        .replace("${auth_uuid}", &ctx.auth_uuid)
        .replace("${auth_access_token}", &ctx.auth_access_token)
        .replace("${user_type}", &ctx.user_type)
        .replace("${version_name}", &ctx.version_name)
        .replace("${game_directory}", &ctx.game_directory)
        .replace("${assets_root}", &ctx.assets_root)
        .replace("${assets_index_name}", &ctx.assets_index_name)
        .replace("${version_type}", &ctx.version_type)
        .replace("${natives_directory}", &ctx.natives_directory)
        .replace("${classpath}", &ctx.classpath)
        .replace("${launcher_name}", &ctx.launcher_name)
        .replace("${launcher_version}", &ctx.launcher_version)
        .replace("${classpath_separator}", if cfg!(target_os = "windows") { ";" } else { ":" })
        .replace("${library_directory}", &ctx.assets_root) // Simplified
        .replace("${user_properties}", "{}")
}

/// Run a shell command
fn run_command(command: &str, working_dir: &PathBuf) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", command])
            .current_dir(working_dir)
            .spawn()
            .map_err(|e| OxideError::Launch(format!("Failed to run command: {}", e)))?
            .wait()
            .map_err(|e| OxideError::Launch(format!("Command failed: {}", e)))?;
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("sh")
            .args(["-c", command])
            .current_dir(working_dir)
            .spawn()
            .map_err(|e| OxideError::Launch(format!("Failed to run command: {}", e)))?
            .wait()
            .map_err(|e| OxideError::Launch(format!("Command failed: {}", e)))?;
    }
    
    Ok(())
}
