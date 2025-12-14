//! Forge version API client and installer
//!
//! Forge uses a more complex installation process that involves:
//! 1. Downloading the installer JAR
//! 2. Extracting the version JSON from the installer
//! 3. Running processors (for newer Forge versions)
//! 4. Downloading libraries
//!
//! Modern Forge (1.13+) uses the cpw.mods.bootstraplauncher.BootstrapLauncher main class.
//! Legacy Forge (<1.13) uses tweakers with the vanilla main class.

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
use super::processor::{run_processors, Processor, ProcessorContext, ProcessorData, extract_installer_libraries};

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
    id: String,
    #[serde(rename = "mainClass")]
    main_class: String,
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
    version: String,
    json: Option<String>,
    path: Option<String>,
    #[serde(rename = "minecraft")]
    minecraft_version: Option<String>,
    libraries: Option<Vec<ForgeLibrary>>,
    processors: Option<Vec<ForgeProcessor>>,
    data: Option<std::collections::HashMap<String, ForgeData>>,
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
    fn extract_version_json(&self, installer_path: &PathBuf) -> Result<ForgeVersionJson> {
        let file = std::fs::File::open(installer_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // Try to find version.json (modern Forge) or the version-specific JSON
        let json_name = archive
            .file_names()
            .find(|n| n.ends_with("version.json"))
            .map(|s| s.to_string())
            .ok_or_else(|| OxideError::Modloader("No version.json found in installer".into()))?;

        let mut json_file = archive.by_name(&json_name)?;
        let mut content = String::new();
        json_file.read_to_string(&mut content)?;

        let version_json: ForgeVersionJson = serde_json::from_str(&content)?;
        Ok(version_json)
    }

    /// Extract install_profile.json for modern Forge
    fn extract_install_profile(&self, installer_path: &PathBuf) -> Result<Option<ForgeInstallProfile>> {
        let file = std::fs::File::open(installer_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        let result = match archive.by_name("install_profile.json") {
            Ok(mut file) => {
                let mut content = String::new();
                file.read_to_string(&mut content)?;
                let profile: ForgeInstallProfile = serde_json::from_str(&content)?;
                Ok(Some(profile))
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
        if let Some(ref mc_args) = version_json.minecraft_arguments {
            let parts: Vec<&str> = mc_args.split_whitespace().collect();
            let mut i = 0;
            while i < parts.len() {
                if parts[i] == "--tweakClass" && i + 1 < parts.len() {
                    profile.tweakers.push(parts[i + 1].to_string());
                    i += 2;
                } else {
                    profile.game_arguments.push(parts[i].to_string());
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
pub async fn get_recommended_forge(minecraft_version: &str) -> Result<Option<String>> {
    let versions = get_forge_versions(minecraft_version).await?;
    Ok(versions.into_iter().find(|v| v.recommended).map(|v| v.version))
}
