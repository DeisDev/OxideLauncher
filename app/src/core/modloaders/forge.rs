//! Forge version API client and installer.
//!
//! Oxide Launcher — A Rust-based Minecraft launcher
//! Copyright (C) 2025 Oxide Launcher contributors
//!
//! This file is part of Oxide Launcher.
//!
//! Oxide Launcher is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License, or
//! (at your option) any later version.
//!
//! Oxide Launcher is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License
//! along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::path::PathBuf;
use std::io::Read;
use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn};

use crate::core::error::{OxideError, Result};
use crate::core::download::download_file;
use crate::core::instance::ModLoaderType;
use super::profile::{ModloaderProfile, ModloaderLibrary, maven_to_path};
use super::installer::{ModloaderInstaller, InstallProgress, ProgressCallback, download_modloader_libraries};
use super::processor::{run_processors, Processor, ProcessorContext, ProcessorData, extract_installer_libraries, extract_forge_universal_jar};

const FORGE_MAVEN_METADATA: &str = "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json";
const FORGE_MAVEN_URL: &str = "https://maven.minecraftforge.net";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeVersion {
    pub version: String,
    pub minecraft_version: String,
    pub recommended: bool,
    pub latest: bool,
}

// Forge version JSON structures (from installer JAR)
#[derive(Debug, Deserialize)]
struct ForgeVersionJson {
    #[allow(dead_code)] // Parsed from JSON, needed for version validation
    id: String,
    #[serde(rename = "mainClass")]
    main_class: String,
    #[allow(dead_code)] // Reserved for inheritance chain validation
    #[serde(rename = "inheritsFrom")]
    inherits_from: Option<String>,
    libraries: Vec<ForgeLibrary>,
    #[serde(default)]
    arguments: Option<ForgeArguments>,
    #[serde(rename = "minecraftArguments")]
    minecraft_arguments: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ForgeLibrary {
    name: String,
    downloads: Option<ForgeDownloads>,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ForgeDownloads {
    artifact: Option<ForgeArtifact>,
}

#[derive(Debug, Deserialize)]
struct ForgeArtifact {
    path: String,
    url: String,
    sha1: Option<String>,
    size: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ForgeArguments {
    game: Option<Vec<serde_json::Value>>,
    jvm: Option<Vec<serde_json::Value>>,
}

// Install profile (for modern Forge with processors)
#[derive(Debug, Deserialize)]
struct ForgeInstallProfile {
    #[allow(dead_code)] // Parsed from JSON for debugging/logging
    version: String,
    #[allow(dead_code)] // Reserved for JSON path reference
    json: Option<String>,
    #[allow(dead_code)] // Reserved for artifact path reference
    path: Option<String>,
    #[allow(dead_code)] // Reserved for version validation
    #[serde(rename = "minecraft")]
    minecraft_version: Option<String>,
    libraries: Option<Vec<ForgeLibrary>>,
    processors: Option<Vec<ForgeProcessor>>,
    data: Option<std::collections::HashMap<String, ForgeData>>,
}

// Legacy install profile format (Forge <1.13)
// Has "install" and "versionInfo" at root level
#[derive(Debug, Deserialize)]
struct LegacyForgeInstallProfile {
    #[allow(dead_code)] // Used to validate this is a legacy profile
    install: LegacyInstallInfo,
    #[serde(rename = "versionInfo")]
    version_info: ForgeVersionJson,
}

#[derive(Debug, Deserialize)]
struct LegacyInstallInfo {
    #[allow(dead_code)]
    #[serde(rename = "profileName")]
    profile_name: Option<String>,
    #[allow(dead_code)]
    target: Option<String>,
    #[allow(dead_code)]
    path: Option<String>,
    #[allow(dead_code)]
    minecraft: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ForgeProcessor {
    jar: String,
    classpath: Vec<String>,
    args: Vec<String>,
    #[serde(default)]
    sides: Vec<String>,
    outputs: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct ForgeData {
    client: String,
    server: String,
}

/// Forge modloader installer
pub struct ForgeInstaller {
    #[allow(dead_code)] // Reserved for future authenticated requests
    client: reqwest::Client,
}

impl ForgeInstaller {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Get the installer URL for a Forge version
    fn get_installer_url(&self, minecraft_version: &str, forge_version: &str) -> String {
        // Forge version format can be:
        // - Just the forge version: "47.2.0"
        // - MC-Forge format: "1.20.1-47.2.0"
        let full_version = if forge_version.contains('-') {
            forge_version.to_string()
        } else {
            format!("{}-{}", minecraft_version, forge_version)
        };

        format!(
            "{}/net/minecraftforge/forge/{}/forge-{}-installer.jar",
            FORGE_MAVEN_URL, full_version, full_version
        )
    }

    /// Extract version JSON from installer JAR
    /// Modern Forge (1.13+): Has a separate version.json file
    /// Legacy Forge (<1.13): Has versionInfo nested inside install_profile.json
    fn extract_version_json(&self, installer_path: &PathBuf) -> Result<ForgeVersionJson> {
        let file = std::fs::File::open(installer_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // First try to find version.json (modern Forge)
        let json_name = archive
            .file_names()
            .find(|n| n.ends_with("version.json"))
            .map(|s| s.to_string());

        if let Some(name) = json_name {
            debug!("Found version.json in installer (modern Forge format)");
            let mut json_file = archive.by_name(&name)?;
            let mut content = String::new();
            json_file.read_to_string(&mut content)?;
            let version_json: ForgeVersionJson = serde_json::from_str(&content)?;
            return Ok(version_json);
        }

        // Fall back to legacy format: extract versionInfo from install_profile.json
        debug!("No version.json found, trying legacy format (versionInfo in install_profile.json)");
        drop(archive);
        
        let file = std::fs::File::open(installer_path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        
        let mut profile_file = archive.by_name("install_profile.json")
            .map_err(|_| OxideError::Modloader("No version.json or install_profile.json found in installer".into()))?;
        
        let mut content = String::new();
        profile_file.read_to_string(&mut content)?;
        
        // Try parsing as legacy format first
        if let Ok(legacy_profile) = serde_json::from_str::<LegacyForgeInstallProfile>(&content) {
            info!("Parsed legacy Forge install profile (versionInfo format)");
            return Ok(legacy_profile.version_info);
        }
        
        Err(OxideError::Modloader("Failed to parse installer: neither modern version.json nor legacy versionInfo found".into()))
    }

    /// Extract install_profile.json for modern Forge
    fn extract_install_profile(&self, installer_path: &PathBuf) -> Result<Option<ForgeInstallProfile>> {
        let file = std::fs::File::open(installer_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        let result = match archive.by_name("install_profile.json") {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                
                // Try modern format first
                if let Ok(profile) = serde_json::from_str::<ForgeInstallProfile>(&content) {
                    // Verify it's modern format by checking for processors
                    if profile.processors.is_some() {
                        return Ok(Some(profile));
                    }
                }
                
                // Legacy format doesn't have a separate install profile structure we need
                Ok(None)
            }
            Err(_) => Ok(None), // Not all Forge versions have install_profile.json
        };
        result
    }

    /// Build the modloader profile from Forge version JSON
    fn build_profile(
        &self,
        minecraft_version: &str,
        forge_version: &str,
        version_json: &ForgeVersionJson,
    ) -> ModloaderProfile {
        let mut profile = ModloaderProfile::new(
            "net.minecraftforge".to_string(),
            forge_version.to_string(),
            minecraft_version.to_string(),
        );

        profile.main_class = version_json.main_class.clone();
        
        // Detect launcher type based on main class
        profile.detect_launcher_type();

        // Parse arguments
        if let Some(ref args) = version_json.arguments {
            if let Some(ref jvm_args) = args.jvm {
                for arg in jvm_args {
                    if let Some(s) = arg.as_str() {
                        profile.jvm_arguments.push(s.to_string());
                    }
                }
            }
            if let Some(ref game_args) = args.game {
                for arg in game_args {
                    if let Some(s) = arg.as_str() {
                        profile.game_arguments.push(s.to_string());
                    }
                }
            }
        }

        // Parse legacy tweakers from minecraft_arguments
        // NOTE: For legacy Forge, minecraft_arguments contains the COMPLETE set of game arguments
        // (like --username, --version, etc.) which are already handled by vanilla MC version data.
        // We only need to extract the --tweakClass arguments here.
        if let Some(ref mc_args) = version_json.minecraft_arguments {
            let parts: Vec<&str> = mc_args.split_whitespace().collect();
            let mut i = 0;
            while i < parts.len() {
                if parts[i] == "--tweakClass" && i + 1 < parts.len() {
                    profile.tweakers.push(parts[i + 1].to_string());
                    i += 2;
                } else {
                    // Skip other arguments - they're duplicates of vanilla MC arguments
                    i += 1;
                }
            }
        }

        // Add libraries
        for lib in &version_json.libraries {
            let mut modloader_lib = ModloaderLibrary {
                name: lib.name.clone(),
                url: None,
                sha1: None,
                size: None,
                path: None,
                natives: None,
                rules: Vec::new(),
            };

            if let Some(ref downloads) = lib.downloads {
                if let Some(ref artifact) = downloads.artifact {
                    modloader_lib.url = Some(artifact.url.clone());
                    modloader_lib.sha1 = artifact.sha1.clone();
                    modloader_lib.size = artifact.size;
                    modloader_lib.path = Some(artifact.path.clone());
                }
            } else if let Some(ref url) = lib.url {
                modloader_lib.url = Some(format!("{}{}", url, maven_to_path(&lib.name)));
            }

            profile.libraries.push(modloader_lib);
        }

        profile
    }
}

impl Default for ForgeInstaller {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ModloaderInstaller for ForgeInstaller {
    fn loader_type(&self) -> ModLoaderType {
        ModLoaderType::Forge
    }

    async fn get_versions(&self, minecraft_version: &str) -> Result<Vec<String>> {
        let versions = get_forge_versions(minecraft_version).await?;
        Ok(versions.into_iter().map(|v| v.version).collect())
    }

    async fn install(
        &self,
        minecraft_version: &str,
        loader_version: &str,
        libraries_dir: &PathBuf,
        progress: Option<ProgressCallback>,
    ) -> Result<ModloaderProfile> {
        if let Some(ref callback) = progress {
            callback(InstallProgress::FetchingMetadata);
        }

        // Create temp directory for installer
        let temp_dir = std::env::temp_dir().join("oxide_forge_install");
        std::fs::create_dir_all(&temp_dir)?;

        // Download installer JAR
        let installer_url = self.get_installer_url(minecraft_version, loader_version);
        let installer_path = temp_dir.join("forge-installer.jar");

        info!("Downloading Forge installer from: {}", installer_url);
        debug!("Temp directory: {:?}", temp_dir);

        if let Some(ref callback) = progress {
            callback(InstallProgress::Processing("Downloading installer...".to_string()));
        }

        download_file(&installer_url, &installer_path, None).await?;
        
        if !installer_path.exists() {
            return Err(OxideError::Modloader("Failed to download Forge installer".to_string()));
        }
        debug!("Installer downloaded successfully");

        // Extract universal JAR from legacy Forge installers
        // This must be done BEFORE downloading libraries, as the forge JAR is bundled in the installer
        // and not available for direct download from Maven for legacy versions
        let forge_version_clean = if loader_version.contains('-') {
            loader_version.split('-').last().unwrap_or(loader_version).to_string()
        } else {
            loader_version.to_string()
        };
        
        match extract_forge_universal_jar(&installer_path, minecraft_version, &forge_version_clean, libraries_dir) {
            Ok(true) => info!("Extracted Forge universal JAR from installer"),
            Ok(false) => debug!("No universal JAR in installer (normal for modern Forge)"),
            Err(e) => warn!("Failed to extract universal JAR: {} (continuing anyway)", e),
        }

        // Extract version JSON
        if let Some(ref callback) = progress {
            callback(InstallProgress::Processing("Extracting version data...".to_string()));
        }

        let version_json = self.extract_version_json(&installer_path)?;
        
        info!(
            "Installing Forge {} for Minecraft {} (main class: {})",
            loader_version, minecraft_version, version_json.main_class
        );
        
        // Check for install_profile.json (modern Forge with processors)
        let install_profile = self.extract_install_profile(&installer_path)?;
        
        // Build the profile
        debug!("Building modloader profile...");
        let profile = self.build_profile(minecraft_version, loader_version, &version_json);
        
        debug!(
            "Profile created: main_class='{}', libraries={}, jvm_args={}, game_args={}, tweakers={}",
            profile.main_class,
            profile.libraries.len(),
            profile.jvm_arguments.len(),
            profile.game_arguments.len(),
            profile.tweakers.len()
        );

        // Download all libraries from the main profile
        info!("Downloading {} Forge libraries...", profile.libraries.len());
        download_modloader_libraries(&profile, libraries_dir, progress.as_ref()).await?;
        
        // Extract libraries bundled in the installer JAR
        if let Some(ref callback) = progress {
            callback(InstallProgress::Processing("Extracting bundled libraries...".to_string()));
        }
        extract_installer_libraries(&installer_path, libraries_dir)?;
        
        // Handle modern Forge with processors
        if let Some(ref install_prof) = install_profile {
            // Download processor libraries first
            if let Some(ref proc_libs) = install_prof.libraries {
                info!("Downloading {} processor libraries...", proc_libs.len());
                
                let proc_profile = ModloaderProfile {
                    uid: "forge-processors".to_string(),
                    version: loader_version.to_string(),
                    minecraft_version: minecraft_version.to_string(),
                    main_class: String::new(),
                    launcher_type: super::profile::LauncherType::Standard,
                    libraries: proc_libs.iter().map(|lib| {
                        let mut modloader_lib = ModloaderLibrary {
                            name: lib.name.clone(),
                            url: None,
                            sha1: None,
                            size: None,
                            path: None,
                            natives: None,
                            rules: Vec::new(),
                        };
                        
                        if let Some(ref downloads) = lib.downloads {
                            if let Some(ref artifact) = downloads.artifact {
                                modloader_lib.url = Some(artifact.url.clone());
                                modloader_lib.sha1 = artifact.sha1.clone();
                                modloader_lib.size = artifact.size;
                                modloader_lib.path = Some(artifact.path.clone());
                            }
                        } else if let Some(ref url) = lib.url {
                            modloader_lib.url = Some(format!("{}{}", url, maven_to_path(&lib.name)));
                        }
                        
                        modloader_lib
                    }).collect(),
                    jvm_arguments: Vec::new(),
                    game_arguments: Vec::new(),
                    tweakers: Vec::new(),
                    min_java_version: None,
                    recommended_java_version: None,
                };
                
                download_modloader_libraries(&proc_profile, libraries_dir, progress.as_ref()).await?;
            }
            
            // Run processors if present
            if let Some(ref forge_processors) = install_prof.processors {
                if !forge_processors.is_empty() {
                    if let Some(ref callback) = progress {
                        callback(InstallProgress::Processing(format!(
                            "Running {} Forge processors...",
                            forge_processors.len()
                        )));
                    }
                    
                    // Convert ForgeProcessor to Processor
                    let processors: Vec<Processor> = forge_processors.iter().map(|fp| {
                        Processor {
                            jar: fp.jar.clone(),
                            classpath: fp.classpath.clone(),
                            args: fp.args.clone(),
                            sides: fp.sides.clone(),
                            outputs: fp.outputs.clone().unwrap_or_default(),
                        }
                    }).collect();
                    
                    // Build data map
                    let mut data_map: HashMap<String, ProcessorData> = HashMap::new();
                    if let Some(ref data) = install_prof.data {
                        for (key, value) in data {
                            data_map.insert(key.clone(), ProcessorData {
                                client: value.client.clone(),
                                server: value.server.clone(),
                            });
                        }
                    }
                    
                    // Find the Minecraft client JAR
                    // The client JAR is stored at {data_dir}/meta/versions/{mc_version}/{mc_version}.jar
                    let client_jar = libraries_dir
                        .parent()
                        .unwrap_or(libraries_dir)
                        .join("meta")
                        .join("versions")
                        .join(minecraft_version)
                        .join(format!("{}.jar", minecraft_version));
                    
                    let context = ProcessorContext {
                        libraries_dir: libraries_dir.clone(),
                        client_jar,
                        minecraft_version: minecraft_version.to_string(),
                        loader_version: loader_version.to_string(),
                        data: data_map,
                        installer_jar: installer_path.clone(),
                    };
                    
                    // Run the processors
                    match run_processors(&processors, &context) {
                        Ok(()) => {
                            info!("All Forge processors completed successfully");
                        }
                        Err(e) => {
                            warn!("Forge processor execution failed: {}. The game may not launch correctly.", e);
                            // Don't fail the entire installation - let user try to launch anyway
                        }
                    }
                }
            }
        }

        // Clean up installer
        let _ = std::fs::remove_file(&installer_path);
        let _ = std::fs::remove_dir(&temp_dir);

        if let Some(ref callback) = progress {
            callback(InstallProgress::Complete);
        }

        info!("Forge installation complete");
        Ok(profile)
    }

    fn is_installed(&self, minecraft_version: &str, loader_version: &str, libraries_dir: &PathBuf) -> bool {
        // Check if the main Forge library exists
        let full_version = if loader_version.contains('-') {
            loader_version.to_string()
        } else {
            format!("{}-{}", minecraft_version, loader_version)
        };
        
        let loader_path = format!(
            "net/minecraftforge/forge/{}/forge-{}.jar",
            full_version, full_version
        );
        libraries_dir.join(loader_path).exists()
    }
}

/// Fetch available Forge versions for a Minecraft version
pub async fn get_forge_versions(minecraft_version: &str) -> Result<Vec<ForgeVersion>> {
    let client = reqwest::Client::new();
    let response = client
        .get(FORGE_MAVEN_METADATA)
        .header("User-Agent", format!("OxideLauncher/{}", env!("CARGO_PKG_VERSION")))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(OxideError::Modloader(format!(
            "Failed to fetch Forge versions: HTTP {}",
            response.status()
        )));
    }

    let metadata: std::collections::HashMap<String, Vec<String>> = response.json().await?;
    
    let mut versions = Vec::new();
    
    if let Some(mc_versions) = metadata.get(minecraft_version) {
        for (idx, forge_ver) in mc_versions.iter().enumerate() {
            versions.push(ForgeVersion {
                version: forge_ver.clone(),
                minecraft_version: minecraft_version.to_string(),
                recommended: idx == 0,
                latest: idx == 0,
            });
        }
    }
    
    Ok(versions)
}

/// Get the recommended Forge version for a Minecraft version
#[allow(dead_code)] // Utility function for future auto-select feature
pub async fn get_recommended_forge(minecraft_version: &str) -> Result<Option<String>> {
    let versions = get_forge_versions(minecraft_version).await?;
    Ok(versions.into_iter().find(|v| v.recommended).map(|v| v.version))
}
