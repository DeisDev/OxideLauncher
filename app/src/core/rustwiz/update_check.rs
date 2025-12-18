//! RustWiz update checking - Check mods for available updates
//!
//! Queries Modrinth and CurseForge APIs to check for newer versions
//! of installed mods based on their metadata.

use std::path::Path;

use crate::core::error::Result;
use crate::core::modplatform::modrinth::ModrinthClient;
use crate::core::modplatform::curseforge::CurseForgeClient;
use super::types::*;
use super::parser::{find_mod_tomls, read_mod_toml, read_pack_toml};

// =============================================================================
// Update Checking
// =============================================================================

/// Check a single mod for updates
pub async fn check_mod_update(
    mod_toml: &ModTomlExtended,
    minecraft_version: &str,
    mod_loader: Option<&str>,
) -> Result<UpdateCheckResult> {
    let filename = mod_toml.packwiz.filename.clone();
    let current_version = extract_version_from_filename(&filename);
    
    // Get update source info
    let update = mod_toml.packwiz.update.as_ref();
    
    if let Some(update_info) = update {
        // Try Modrinth first
        if let Some(ref modrinth) = update_info.modrinth {
            return check_modrinth_update(
                &filename,
                &current_version,
                modrinth,
                minecraft_version,
                mod_loader,
            ).await;
        }
        
        // Try CurseForge
        if let Some(ref curseforge) = update_info.curseforge {
            return check_curseforge_update(
                &filename,
                &current_version,
                curseforge,
                minecraft_version,
                mod_loader,
            ).await;
        }
    }
    
    // No update source available
    Ok(UpdateCheckResult {
        filename,
        current_version,
        latest_version: None,
        latest_version_id: None,
        update_available: false,
        platform: "unknown".to_string(),
        changelog: None,
    })
}

/// Check all mods in an instance for updates
/// 
/// This function no longer requires pack.toml to exist. Instead, it accepts
/// the minecraft version and mod loader directly from the instance.
/// If not provided, it will try to fall back to pack.toml if it exists.
#[allow(dead_code)] // Kept for backwards compatibility
pub async fn check_instance_updates(
    instance_path: &Path,
) -> Result<BatchUpdateResult> {
    check_instance_updates_with_info(instance_path, None, None).await
}

/// Check all mods in an instance for updates with explicit version info
/// 
/// This allows checking for updates without requiring pack.toml by passing
/// the minecraft version and mod loader directly from the instance metadata.
pub async fn check_instance_updates_with_info(
    instance_path: &Path,
    minecraft_version: Option<&str>,
    mod_loader: Option<&str>,
) -> Result<BatchUpdateResult> {
    let mut result = BatchUpdateResult {
        updates_available: Vec::new(),
        up_to_date: Vec::new(),
        unchecked: Vec::new(),
        errors: Vec::new(),
    };
    
    // Determine minecraft version and mod loader
    // Priority: 1) Passed parameters, 2) pack.toml if exists
    let (mc_version, loader) = if let (Some(mc), loader) = (minecraft_version, mod_loader) {
        (mc.to_string(), loader.map(|l| l.to_string()))
    } else {
        // Try to read from pack.toml as fallback
        match read_pack_toml(instance_path) {
            Ok(pack) => {
                let mc = pack.versions.minecraft.clone();
                let loader = pack.get_mod_loader().map(|(l, _)| l.to_string());
                (mc, loader)
            }
            Err(_) => {
                // No pack.toml and no parameters provided - we can't check updates
                // without knowing the minecraft version
                result.errors.push(
                    "Cannot check updates: no minecraft version provided and pack.toml not found. \
                    Try checking updates from the instance details page.".to_string()
                );
                return Ok(result);
            }
        }
    };
    
    // Find all mod tomls
    let mod_tomls = find_mod_tomls(instance_path)?;
    
    for toml_path in mod_tomls {
        let mod_toml = match read_mod_toml(&toml_path) {
            Ok(m) => m,
            Err(e) => {
                result.errors.push(format!(
                    "Failed to read {:?}: {}",
                    toml_path.file_name().unwrap_or_default(),
                    e
                ));
                continue;
            }
        };
        
        // Check if we have update info
        if mod_toml.packwiz.update.is_none() {
            result.unchecked.push(mod_toml.packwiz.filename.clone());
            continue;
        }
        
        // Use stored mc_versions/loaders from metadata if available, otherwise use instance info
        let check_mc_version = if let Some(ref oxide) = mod_toml.oxide {
            if !oxide.mc_versions.is_empty() {
                // Use first stored version that matches instance version if possible
                if oxide.mc_versions.contains(&mc_version) {
                    mc_version.clone()
                } else {
                    oxide.mc_versions.first().cloned().unwrap_or_else(|| mc_version.clone())
                }
            } else {
                mc_version.clone()
            }
        } else {
            mc_version.clone()
        };
        
        let check_loader = if let Some(ref oxide) = mod_toml.oxide {
            if !oxide.loaders.is_empty() {
                oxide.loaders.first().map(|s| s.as_str())
            } else {
                loader.as_deref()
            }
        } else {
            loader.as_deref()
        };
        
        match check_mod_update(&mod_toml, &check_mc_version, check_loader).await {
            Ok(check_result) => {
                if check_result.update_available {
                    result.updates_available.push(check_result);
                } else {
                    result.up_to_date.push(check_result.filename);
                }
            }
            Err(e) => {
                result.errors.push(format!(
                    "Failed to check {}: {}",
                    mod_toml.packwiz.filename,
                    e
                ));
            }
        }
    }
    
    Ok(result)
}

// =============================================================================
// Platform-Specific Checks
// =============================================================================

async fn check_modrinth_update(
    filename: &str,
    current_version: &str,
    modrinth: &ModrinthUpdate,
    minecraft_version: &str,
    mod_loader: Option<&str>,
) -> Result<UpdateCheckResult> {
    let modrinth_client = ModrinthClient::new();
    
    // Build filter arrays
    let game_versions = vec![minecraft_version.to_string()];
    let loaders = mod_loader.map(|l| vec![l.to_string()]);
    
    // Get project versions
    let versions = modrinth_client
        .get_versions(
            &modrinth.mod_id,
            Some(&game_versions),
            loaders.as_deref(),
        )
        .await?;
    
    // Find the latest compatible version
    let latest = versions.first();
    
    if let Some(latest_version) = latest {
        let update_available = latest_version.id != modrinth.version;
        
        Ok(UpdateCheckResult {
            filename: filename.to_string(),
            current_version: current_version.to_string(),
            latest_version: Some(latest_version.version_number.clone()),
            latest_version_id: Some(latest_version.id.clone()),
            update_available,
            platform: "modrinth".to_string(),
            changelog: latest_version.changelog.clone(),
        })
    } else {
        Ok(UpdateCheckResult {
            filename: filename.to_string(),
            current_version: current_version.to_string(),
            latest_version: None,
            latest_version_id: None,
            update_available: false,
            platform: "modrinth".to_string(),
            changelog: None,
        })
    }
}

async fn check_curseforge_update(
    filename: &str,
    current_version: &str,
    curseforge: &CurseForgeUpdate,
    minecraft_version: &str,
    mod_loader: Option<&str>,
) -> Result<UpdateCheckResult> {
    let cf_client = CurseForgeClient::new();
    
    // Get mod files
    let files = cf_client
        .get_files(
            curseforge.project_id,
            Some(minecraft_version),
            mod_loader,
        )
        .await?;
    
    // Find the latest compatible version
    let latest = files.first();
    
    if let Some(latest_file) = latest {
        // Convert ID to compare
        let latest_id: u32 = latest_file.id.parse().unwrap_or(0);
        let update_available = latest_id != curseforge.file_id;
        
        Ok(UpdateCheckResult {
            filename: filename.to_string(),
            current_version: current_version.to_string(),
            latest_version: Some(latest_file.name.clone()),
            latest_version_id: Some(latest_file.id.clone()),
            update_available,
            platform: "curseforge".to_string(),
            changelog: None, // CurseForge changelog requires separate API call
        })
    } else {
        Ok(UpdateCheckResult {
            filename: filename.to_string(),
            current_version: current_version.to_string(),
            latest_version: None,
            latest_version_id: None,
            update_available: false,
            platform: "curseforge".to_string(),
            changelog: None,
        })
    }
}

// =============================================================================
// Utilities
// =============================================================================

/// Extract version string from a filename
fn extract_version_from_filename(filename: &str) -> String {
    // Strip path prefix and extension
    let name = filename
        .rsplit('/')
        .next()
        .unwrap_or(filename)
        .trim_end_matches(".jar")
        .trim_end_matches(".disabled");
    
    // Common patterns to extract version:
    // mod-name-mc1.21.1-1.0.0 -> 1.0.0
    // mod-name-1.0.0+fabric -> 1.0.0
    // mod-name_1.0.0 -> 1.0.0
    
    // Find the last version-like segment
    let parts: Vec<&str> = name.split(|c| c == '-' || c == '_' || c == '+').collect();
    
    for part in parts.iter().rev() {
        // Skip mc version indicators
        if part.starts_with("mc") || part.starts_with("MC") {
            continue;
        }
        // Skip loader indicators
        if *part == "fabric" || *part == "forge" || *part == "quilt" || *part == "neoforge" {
            continue;
        }
        // Check if it looks like a version (starts with digit)
        if part.chars().next().map_or(false, |c| c.is_ascii_digit()) {
            return part.to_string();
        }
    }
    
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_extraction() {
        assert_eq!(
            extract_version_from_filename("mods/sodium-mc1.21.1-0.6.0.jar"),
            "0.6.0"
        );
        assert_eq!(
            extract_version_from_filename("iris-mc1.21.1-1.8.0+fabric.jar"),
            "1.8.0"
        );
        assert_eq!(
            extract_version_from_filename("jei-1.21.1-forge-19.0.0.7.jar"),
            "19.0.0.7"
        );
    }
}
