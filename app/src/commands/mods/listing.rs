//! Mod listing and management commands

use crate::commands::state::AppState;
use crate::core::rustwiz::{self, parser::read_mod_toml};
use super::types::*;
use tauri::State;

#[tauri::command]
pub async fn get_installed_mods(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<Vec<InstalledMod>, String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let mods_dir = instance.mods_dir();
    
    tracing::debug!("Loading installed mods from: {:?}", mods_dir);
    
    if !mods_dir.exists() {
        tracing::debug!("Mods directory does not exist");
        return Ok(Vec::new());
    }
    
    let mut mods = Vec::new();
    
    let entries: Vec<_> = std::fs::read_dir(&mods_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .collect();
    
    tracing::info!("Found {} files in mods directory", entries.len());
    
    for entry in entries {
        let path = entry.path();
        
        if path.is_file() {
            let filename = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            
            tracing::debug!("Processing file: {}", filename);
            
            // Skip metadata files
            if filename.ends_with(".metadata.json") {
                tracing::debug!("Skipping metadata file: {}", filename);
                continue;
            }
            
            // Only process .jar files (enabled or disabled)
            if !filename.ends_with(".jar") && !filename.ends_with(".jar.disabled") {
                continue;
            }
            
            let enabled = !filename.ends_with(".disabled");
            let base_filename = filename.trim_end_matches(".disabled").to_string();
            
            let file_meta = entry.metadata().ok();
            let size = file_meta.as_ref().map(|m| m.len()).unwrap_or(0);
            let modified = file_meta.and_then(|m| m.modified().ok()).map(|t| {
                let datetime: chrono::DateTime<chrono::Utc> = t.into();
                datetime.format("%Y-%m-%d %H:%M").to_string()
            });
            
            // Try to load metadata from RustWiz .pw.toml file first (in .index folder)
            let toml_filename = rustwiz::mod_toml_filename(&base_filename);
            let index_dir = rustwiz::index_dir(&mods_dir);
            let toml_path = index_dir.join(&toml_filename);
            
            let pw_toml_metadata = if toml_path.exists() {
                read_mod_toml(&toml_path).ok()
            } else {
                None
            };
            
            // Try to load metadata from .metadata.json file (legacy)
            let metadata_path = mods_dir.join(format!("{}.metadata.json", base_filename));
            let metadata: Option<ModMetadata> = if metadata_path.exists() {
                std::fs::read_to_string(&metadata_path)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
            } else {
                None
            };
            
            // Priority: pw.toml > .metadata.json > JAR parsing
            let (name, version, provider, icon_url, homepage, issues_url, source_url) = if let Some(ref pw_meta) = pw_toml_metadata {
                // Extract provider from update section
                let provider = if pw_meta.packwiz.update.as_ref().and_then(|u| u.modrinth.as_ref()).is_some() {
                    Some("modrinth".to_string())
                } else if pw_meta.packwiz.update.as_ref().and_then(|u| u.curseforge.as_ref()).is_some() {
                    Some("curseforge".to_string())
                } else {
                    None
                };
                
                // Get icon_url from oxide metadata
                let icon_url = pw_meta.oxide.as_ref().and_then(|o| o.icon_url.clone());
                
                // Get name and version from pw.toml
                let name = pw_meta.packwiz.name.clone();
                let version = pw_meta.oxide.as_ref()
                    .and_then(|o| o.mc_versions.first().cloned())
                    .or_else(|| Some("".to_string()));
                
                (name, version, provider, icon_url, None, None, None)
            } else if let Some(meta) = metadata {
                (meta.name, Some(meta.version), Some(meta.provider), meta.icon_url, None, None, None)
            } else {
                // Try to parse mod metadata from JAR file
                use crate::core::modplatform::mod_parser::{parse_mod_jar, extract_mod_icon};
                
                let jar_path = if enabled {
                    mods_dir.join(&base_filename)
                } else {
                    mods_dir.join(format!("{}.disabled", base_filename))
                };
                
                if let Some(jar_details) = parse_mod_jar(&jar_path) {
                    tracing::debug!(
                        "Parsed mod '{}': name='{}', version='{}', homepage={:?}, issues={:?}, source={:?}, icon_path={:?}",
                        base_filename,
                        jar_details.name,
                        jar_details.version,
                        jar_details.homepage,
                        jar_details.issues_url,
                        jar_details.source_url,
                        jar_details.icon_path
                    );
                    
                    let name = if !jar_details.name.is_empty() {
                        jar_details.name
                    } else {
                        base_filename.trim_end_matches(".jar").to_string()
                    };
                    let version = if !jar_details.version.is_empty() && jar_details.version != "unknown" {
                        Some(jar_details.version)
                    } else {
                        None
                    };
                    
                    // Extract icon from JAR if available
                    let icon_url = extract_mod_icon(&jar_path);
                    if icon_url.is_some() {
                        tracing::info!("Successfully extracted icon for mod '{}'", base_filename);
                    } else {
                        tracing::info!("No icon found for mod '{}' (icon_path was: {:?})", base_filename, jar_details.icon_path);
                    }
                    
                    // Log URL extraction results
                    if jar_details.homepage.is_some() || jar_details.issues_url.is_some() || jar_details.source_url.is_some() {
                        tracing::info!(
                            "Mod '{}' URLs: homepage={:?}, issues={:?}, source={:?}",
                            base_filename,
                            jar_details.homepage,
                            jar_details.issues_url,
                            jar_details.source_url
                        );
                    } else {
                        tracing::info!("No URLs found in mod '{}'", base_filename);
                    }
                    
                    // Provider is None when parsed from JAR - we don't know if it came from Modrinth/CurseForge
                    // loader_type (Fabric/Forge/etc.) is different from provider (Modrinth/CurseForge)
                    (name, version, None, icon_url, jar_details.homepage, jar_details.issues_url, jar_details.source_url)
                } else {
                    tracing::info!("Could not parse mod metadata from JAR: {}", base_filename);
                    let name = base_filename.trim_end_matches(".jar").to_string();
                    (name, None, None, None, None, None, None)
                }
            };
            
            tracing::info!(
                "Returning mod '{}': icon={}, homepage={}, issues={}, source={}",
                name,
                icon_url.is_some(),
                homepage.is_some(),
                issues_url.is_some(),
                source_url.is_some()
            );
            
            mods.push(InstalledMod {
                filename: base_filename,
                name,
                version,
                enabled,
                size,
                modified,
                provider,
                icon_url,
                homepage,
                issues_url,
                source_url,
            });
        }
    }
    
    mods.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    
    Ok(mods)
}

#[tauri::command]
pub async fn toggle_mod(
    state: State<'_, AppState>,
    instance_id: String,
    filename: String,
    enabled: bool,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let mods_dir = instance.mods_dir();
    let current_path = if enabled {
        mods_dir.join(format!("{}.disabled", filename))
    } else {
        mods_dir.join(&filename)
    };
    
    let new_path = if enabled {
        mods_dir.join(&filename)
    } else {
        mods_dir.join(format!("{}.disabled", filename))
    };
    
    std::fs::rename(current_path, new_path)
        .map_err(|e| format!("Failed to toggle mod: {}", e))?;
    
    Ok(())
}

#[tauri::command]
pub async fn delete_mod(
    state: State<'_, AppState>,
    instance_id: String,
    filename: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let mods_dir = instance.mods_dir();
    let mod_path = mods_dir.join(&filename);
    let disabled_path = mods_dir.join(format!("{}.disabled", filename));
    let metadata_path = mods_dir.join(format!("{}.metadata.json", filename));
    
    let _ = std::fs::remove_file(mod_path);
    let _ = std::fs::remove_file(disabled_path);
    let _ = std::fs::remove_file(metadata_path);
    
    Ok(())
}

#[tauri::command]
pub async fn delete_mods(
    state: State<'_, AppState>,
    instance_id: String,
    filenames: Vec<String>,
) -> Result<(), String> {
    for filename in filenames {
        delete_mod(state.clone(), instance_id.clone(), filename).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn enable_mods(
    state: State<'_, AppState>,
    instance_id: String,
    filenames: Vec<String>,
) -> Result<(), String> {
    for filename in filenames {
        toggle_mod(state.clone(), instance_id.clone(), filename, true).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn disable_mods(
    state: State<'_, AppState>,
    instance_id: String,
    filenames: Vec<String>,
) -> Result<(), String> {
    for filename in filenames {
        toggle_mod(state.clone(), instance_id.clone(), filename, false).await?;
    }
    Ok(())
}

#[tauri::command]
pub async fn open_mods_folder(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let mods_dir = instance.mods_dir();
    std::fs::create_dir_all(&mods_dir).map_err(|e| e.to_string())?;
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&mods_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&mods_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&mods_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn open_configs_folder(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let config_dir = instance.game_dir().join("config");
    std::fs::create_dir_all(&config_dir).map_err(|e| e.to_string())?;
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&config_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&config_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&config_dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(())
}
