//! RustWiz commands - packwiz-compatible mod metadata and pack management
//! 
//! RustWiz is OxideLauncher's native implementation of the packwiz format,
//! providing mod update checking, metadata tracking, and modpack export.

use std::path::PathBuf;
use tauri::State;

use crate::commands::state::AppState;
use crate::core::rustwiz::{
    self, ModToml, ModTomlExtended, OxideMetadata,
    HashFormat, Side,
    BatchUpdateResult, ExportOptions,
};

// =============================================================================
// RustWiz Initialization
// =============================================================================

/// Initialize rustwiz files for an instance (pack.toml, index.toml)
#[tauri::command]
pub async fn init_rustwiz(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let mod_loader = instance.mod_loader.as_ref()
        .map(|ml| (ml.loader_type.name().to_lowercase(), ml.version.clone()));
    
    let instance_path = instance.path.clone();
    let instance_name = instance.name.clone();
    let mc_version = instance.minecraft_version.clone();
    drop(instances);
    
    rustwiz::initialize_pack(
        &instance_path,
        &instance_name,
        &mc_version,
        mod_loader.as_ref().map(|(t, v)| (t.as_str(), v.as_str())),
    ).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// Check if an instance has rustwiz/packwiz metadata files
#[tauri::command]
pub async fn has_rustwiz(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<bool, String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let instance_path = instance.path.clone();
    drop(instances);
    
    Ok(rustwiz::has_pack(&instance_path))
}

// =============================================================================
// Mod Metadata Management
// =============================================================================

/// Create or update mod metadata (pw.toml file)
/// 
/// Stores metadata in mods/.index/<slug>.pw.toml following Prism Launcher's approach.
/// This allows tracking mods without requiring pack.toml to be initialized.
#[tauri::command]
pub async fn create_mod_metadata(
    state: State<'_, AppState>,
    instance_id: String,
    filename: String,
    name: String,
    download_url: String,
    hash: String,
    hash_format: String,
    platform: Option<String>,
    project_id: Option<String>,
    version_id: Option<String>,
    side: Option<String>,
    icon_url: Option<String>,
    description: Option<String>,
    mc_versions: Option<Vec<String>>,
    loaders: Option<Vec<String>>,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let instance_path = instance.path.clone();
    drop(instances);
    
    // Parse hash format
    let hash_fmt = match hash_format.to_lowercase().as_str() {
        "sha256" => HashFormat::Sha256,
        "sha512" => HashFormat::Sha512,
        "sha1" => HashFormat::Sha1,
        "md5" => HashFormat::Md5,
        "murmur2" => HashFormat::Murmur2,
        _ => HashFormat::Sha256,
    };
    
    // Parse side
    let mod_side = match side.as_deref() {
        Some("client") => Side::Client,
        Some("server") => Side::Server,
        _ => Side::Both,
    };
    
    // Create base mod toml
    let mut mod_toml = ModToml::new(
        name.clone(),
        format!("mods/{}", filename),
        download_url,
        hash,
        hash_fmt,
    ).with_side(mod_side);
    
    // Add update source based on platform
    if let (Some(platform_name), Some(proj_id), Some(ver_id)) = (platform.as_deref(), project_id, version_id) {
        match platform_name.to_lowercase().as_str() {
            "modrinth" => {
                mod_toml = mod_toml.with_modrinth_update(proj_id, ver_id);
            }
            "curseforge" => {
                if let (Ok(pid), Ok(fid)) = (proj_id.parse::<u32>(), ver_id.parse::<u32>()) {
                    mod_toml = mod_toml.with_curseforge_update(pid, fid);
                }
            }
            _ => {}
        }
    }
    
    // Create extended toml with OxideLauncher metadata
    let mut extended = ModTomlExtended::from_packwiz(mod_toml);
    
    // Add oxide metadata with all available info
    let has_oxide_data = icon_url.is_some() 
        || description.is_some() 
        || mc_versions.as_ref().map_or(false, |v| !v.is_empty())
        || loaders.as_ref().map_or(false, |v| !v.is_empty());
    
    if has_oxide_data {
        extended.oxide = Some(OxideMetadata {
            icon_url,
            description,
            mc_versions: mc_versions.unwrap_or_default(),
            loaders: loaders.unwrap_or_default(),
            ..Default::default()
        });
    }
    
    // Generate toml filename and write to .index folder
    let toml_filename = rustwiz::mod_toml_filename(&filename);
    let mods_dir = instance_path.join("mods");
    let index_dir = rustwiz::index_dir(&mods_dir);
    let toml_path = index_dir.join(&toml_filename);
    
    rustwiz::write_mod_toml(&toml_path, &extended)
        .map_err(|e| e.to_string())?;
    
    // Update index if rustwiz is initialized (backwards compatibility)
    if rustwiz::has_pack(&instance_path) {
        update_instance_index(&instance_path)?;
    }
    
    Ok(())
}

/// Delete mod metadata file
/// 
/// Checks both .index folder (new location) and root folder (legacy location).
#[tauri::command]
pub async fn delete_mod_metadata(
    state: State<'_, AppState>,
    instance_id: String,
    filename: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let instance_path = instance.path.clone();
    drop(instances);
    
    let toml_filename = rustwiz::mod_toml_filename(&filename);
    let mods_dir = instance_path.join("mods");
    
    // Try .index folder first (new location)
    let index_dir = rustwiz::index_dir(&mods_dir);
    let index_toml_path = index_dir.join(&toml_filename);
    if index_toml_path.exists() {
        rustwiz::delete_mod_toml(&index_toml_path)
            .map_err(|e| e.to_string())?;
    }
    
    // Also check root folder (legacy location)
    let root_toml_path = mods_dir.join(&toml_filename);
    if root_toml_path.exists() {
        rustwiz::delete_mod_toml(&root_toml_path)
            .map_err(|e| e.to_string())?;
    }
    
    // Update index if rustwiz is initialized
    if rustwiz::has_pack(&instance_path) {
        update_instance_index(&instance_path)?;
    }
    
    Ok(())
}

// =============================================================================
// Update Checking
// =============================================================================

/// Check all mods in an instance for updates
#[tauri::command]
pub async fn check_mod_updates(
    state: State<'_, AppState>,
    instance_id: String,
) -> Result<BatchUpdateResult, String> {
    let (instance_path, mc_version, loader_name) = {
        let instances = state.instances.lock().unwrap();
        let instance = instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?;
        
        let loader = instance.mod_loader.as_ref()
            .map(|ml| format!("{:?}", ml.loader_type).to_lowercase());
        
        (instance.path.clone(), instance.minecraft_version.clone(), loader)
    }; // Lock is released here when scope ends
    
    rustwiz::check_instance_updates_with_info(
        &instance_path,
        Some(&mc_version),
        loader_name.as_deref(),
    )
        .await
        .map_err(|e| e.to_string())
}

// =============================================================================
// Export
// =============================================================================

/// Export options for frontend
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ExportOptionsJs {
    pub include_configs: Option<bool>,
    pub include_resourcepacks: Option<bool>,
    pub include_shaderpacks: Option<bool>,
    pub include_saves: Option<bool>,
    pub version: Option<String>,
    pub author: Option<String>,
}

impl From<ExportOptionsJs> for ExportOptions {
    fn from(opts: ExportOptionsJs) -> Self {
        Self {
            include_configs: opts.include_configs.unwrap_or(true),
            include_resourcepacks: opts.include_resourcepacks.unwrap_or(true),
            include_shaderpacks: opts.include_shaderpacks.unwrap_or(true),
            include_saves: opts.include_saves.unwrap_or(false),
            version: opts.version,
            author: opts.author,
        }
    }
}

/// Export instance to Modrinth format (.mrpack)
#[tauri::command]
pub async fn export_modrinth_pack(
    state: State<'_, AppState>,
    instance_id: String,
    output_path: String,
    options: ExportOptionsJs,
) -> Result<(), String> {
    let instance_path = {
        let instances = state.instances.lock().unwrap();
        let instance = instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?;
        instance.path.clone()
    };
    
    rustwiz::export_modrinth(
        &instance_path,
        &PathBuf::from(output_path),
        &options.into(),
        None,
    )
    .await
    .map_err(|e| e.to_string())
}

/// Export instance to CurseForge format
#[tauri::command]
pub async fn export_curseforge_pack(
    state: State<'_, AppState>,
    instance_id: String,
    output_path: String,
    options: ExportOptionsJs,
) -> Result<(), String> {
    let instance_path = {
        let instances = state.instances.lock().unwrap();
        let instance = instances.iter()
            .find(|i| i.id == instance_id)
            .ok_or_else(|| "Instance not found".to_string())?;
        instance.path.clone()
    };
    
    rustwiz::export_curseforge(
        &instance_path,
        &PathBuf::from(output_path),
        &options.into(),
        None,
    )
    .await
    .map_err(|e| e.to_string())
}

/// Export instance as rustwiz/packwiz format (for git/hosting)
#[tauri::command]
pub fn export_rustwiz_format(
    state: State<'_, AppState>,
    instance_id: String,
    output_dir: String,
) -> Result<(), String> {
    let instances = state.instances.lock().unwrap();
    let instance = instances.iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| "Instance not found".to_string())?;
    
    let instance_path = instance.path.clone();
    drop(instances);
    
    rustwiz::export_packwiz(
        &instance_path,
        &PathBuf::from(output_dir),
        None,
    )
    .map_err(|e| e.to_string())
}

// =============================================================================
// Helpers
// =============================================================================

/// Update the index.toml for an instance
fn update_instance_index(instance_path: &PathBuf) -> Result<(), String> {
    let mut pack = rustwiz::read_pack_toml(instance_path)
        .map_err(|e| e.to_string())?;
    
    let index = rustwiz::rebuild_index(instance_path, HashFormat::Sha256)
        .map_err(|e| e.to_string())?;
    
    rustwiz::write_index_toml(instance_path, &index, &mut pack)
        .map_err(|e| e.to_string())?;
    
    rustwiz::write_pack_toml(instance_path, &pack)
        .map_err(|e| e.to_string())?;
    
    Ok(())
}
