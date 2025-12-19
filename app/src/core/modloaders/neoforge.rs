//! NeoForge version API client and installer
//!
//! NeoForge is similar to modern Forge in its installation process.
//! It uses the cpw.mods.bootstraplauncher.BootstrapLauncher main class.

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

const NEOFORGE_MAVEN_URL: &str = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";
const NEOFORGE_MAVEN_BASE: &str = "https://maven.neoforged.net/releases";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoForgeVersion {
    pub version: String,
    pub minecraft_version: String,
    pub recommended: bool,
}

#[derive(Debug, Deserialize)]
struct NeoForgeMavenResponse {
    versions: Vec<String>,
}

// NeoForge version JSON structures (similar to Forge)
#[derive(Debug, Deserialize)]
struct NeoForgeVersionJson {
    #[allow(dead_code)] // Parsed from JSON for debugging/logging
    id: String,
    #[serde(rename = "mainClass")]
    main_class: String,
    #[allow(dead_code)] // Reserved for inheritance chain validation
    #[serde(rename = "inheritsFrom")]
    inherits_from: Option<String>,
    libraries: Vec<NeoForgeLibrary>,
    #[serde(default)]
    arguments: Option<NeoForgeArguments>,
}

#[derive(Debug, Deserialize)]
struct NeoForgeLibrary {
    name: String,
    downloads: Option<NeoForgeDownloads>,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct NeoForgeDownloads {
    artifact: Option<NeoForgeArtifact>,
}

#[derive(Debug, Deserialize)]
struct NeoForgeArtifact {
    path: String,
    url: String,
    sha1: Option<String>,
    size: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct NeoForgeArguments {
    game: Option<Vec<serde_json::Value>>,
    jvm: Option<Vec<serde_json::Value>>,
}

// Install profile (for NeoForge with processors)
#[derive(Debug, Deserialize)]
struct NeoForgeInstallProfile {
    #[allow(dead_code)]
    version: Option<String>,
    #[allow(dead_code)]
    json: Option<String>,
    #[allow(dead_code)]
    path: Option<String>,
    #[serde(rename = "minecraft")]
    #[allow(dead_code)]
    minecraft_version: Option<String>,
    libraries: Option<Vec<NeoForgeLibrary>>,
    processors: Option<Vec<NeoForgeProcessor>>,
    data: Option<HashMap<String, NeoForgeData>>,
}

#[derive(Debug, Deserialize)]
struct NeoForgeProcessor {
    jar: String,
    classpath: Vec<String>,
    args: Vec<String>,
    #[serde(default)]
    sides: Vec<String>,
    outputs: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
struct NeoForgeData {
    client: String,
    #[allow(dead_code)]
    server: String,
}

/// NeoForge modloader installer
pub struct NeoForgeInstaller {
    #[allow(dead_code)] // Reserved for future authenticated requests
    client: reqwest::Client,
}

impl NeoForgeInstaller {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    /// Get the installer URL for a NeoForge version
    fn get_installer_url(&self, version: &str) -> String {
        format!(
            "{}/net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
            NEOFORGE_MAVEN_BASE, version, version
        )
    }

    /// Extract version JSON from installer JAR
    fn extract_version_json(&self, installer_path: &PathBuf) -> Result<NeoForgeVersionJson> {
        let file = std::fs::File::open(installer_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        // Find version.json
        let json_name = archive
            .file_names()
            .find(|n| n.ends_with("version.json"))
            .map(|s| s.to_string())
            .ok_or_else(|| OxideError::Modloader("No version.json found in NeoForge installer".into()))?;

        let mut json_file = archive.by_name(&json_name)?;
        let mut content = String::new();
        json_file.read_to_string(&mut content)?;

        let version_json: NeoForgeVersionJson = serde_json::from_str(&content)?;
        Ok(version_json)
    }

    /// Extract install_profile.json for NeoForge (if present)
    fn extract_install_profile(&self, installer_path: &PathBuf) -> Result<Option<NeoForgeInstallProfile>> {
        let file = std::fs::File::open(installer_path)?;
        let mut archive = zip::ZipArchive::new(file)?;

        let result = match archive.by_name("install_profile.json") {
            Ok(mut zip_file) => {
                let mut content = String::new();
                zip_file.read_to_string(&mut content)?;
                let profile: NeoForgeInstallProfile = serde_json::from_str(&content)?;
                Ok(Some(profile))
            }
            Err(_) => Ok(None), // Not all versions have install_profile.json
        };
        result
    }

    /// Build the modloader profile from NeoForge version JSON
    fn build_profile(
        &self,
        minecraft_version: &str,
        neoforge_version: &str,
        version_json: &NeoForgeVersionJson,
    ) -> ModloaderProfile {
        let mut profile = ModloaderProfile::new(
            "net.neoforged".to_string(),
            neoforge_version.to_string(),
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

impl Default for NeoForgeInstaller {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ModloaderInstaller for NeoForgeInstaller {
    fn loader_type(&self) -> ModLoaderType {
        ModLoaderType::NeoForge
    }

    async fn get_versions(&self, minecraft_version: &str) -> Result<Vec<String>> {
        let versions = get_neoforge_versions(minecraft_version).await?;
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
        let temp_dir = std::env::temp_dir().join("oxide_neoforge_install");
        std::fs::create_dir_all(&temp_dir)?;

        // Download installer JAR
        let installer_url = self.get_installer_url(loader_version);
        let installer_path = temp_dir.join("neoforge-installer.jar");

        info!("Downloading NeoForge installer from: {}", installer_url);
        debug!("Temp directory: {:?}", temp_dir);

        if let Some(ref callback) = progress {
            callback(InstallProgress::Processing("Downloading installer...".to_string()));
        }

        download_file(&installer_url, &installer_path, None).await?;
        
        if !installer_path.exists() {
            return Err(OxideError::Modloader("Failed to download NeoForge installer".to_string()));
        }
        debug!("Installer downloaded successfully");

        // Extract version JSON
        if let Some(ref callback) = progress {
            callback(InstallProgress::Processing("Extracting version data...".to_string()));
        }

        let version_json = self.extract_version_json(&installer_path)?;
        
        info!(
            "Installing NeoForge {} for Minecraft {} (main class: {})",
            loader_version, minecraft_version, version_json.main_class
        );
        
        // Check for install_profile.json (NeoForge with processors)
        let install_profile = self.extract_install_profile(&installer_path)?;

        // Build the profile
        debug!("Building modloader profile...");
        let profile = self.build_profile(minecraft_version, loader_version, &version_json);
        
        debug!(
            "Profile created: main_class='{}', libraries={}, jvm_args={}, game_args={}",
            profile.main_class,
            profile.libraries.len(),
            profile.jvm_arguments.len(),
            profile.game_arguments.len()
        );

        // Download all libraries from the main profile
        info!("Downloading {} NeoForge libraries...", profile.libraries.len());
        download_modloader_libraries(&profile, libraries_dir, progress.as_ref()).await?;
        
        // Extract libraries bundled in the installer JAR
        if let Some(ref callback) = progress {
            callback(InstallProgress::Processing("Extracting bundled libraries...".to_string()));
        }
        extract_installer_libraries(&installer_path, libraries_dir)?;
        
        // Handle NeoForge with processors
        if let Some(ref install_prof) = install_profile {
            // Download processor libraries first
            if let Some(ref proc_libs) = install_prof.libraries {
                info!("Downloading {} processor libraries...", proc_libs.len());
                
                let proc_profile = ModloaderProfile {
                    uid: "neoforge-processors".to_string(),
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
            if let Some(ref neoforge_processors) = install_prof.processors {
                if !neoforge_processors.is_empty() {
                    if let Some(ref callback) = progress {
                        callback(InstallProgress::Processing(format!(
                            "Running {} NeoForge processors...",
                            neoforge_processors.len()
                        )));
                    }
                    
                    // Convert NeoForgeProcessor to Processor
                    let processors: Vec<Processor> = neoforge_processors.iter().map(|fp| {
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
                            info!("All NeoForge processors completed successfully");
                        }
                        Err(e) => {
                            warn!("NeoForge processor execution failed: {}. The game may not launch correctly.", e);
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

        info!("NeoForge installation complete");
        Ok(profile)
    }

    fn is_installed(&self, _minecraft_version: &str, loader_version: &str, libraries_dir: &PathBuf) -> bool {
        // Check if the main NeoForge library exists
        let loader_path = format!(
            "net/neoforged/neoforge/{}/neoforge-{}.jar",
            loader_version, loader_version
        );
        libraries_dir.join(loader_path).exists()
    }
}

/// Fetch available NeoForge versions for a Minecraft version
pub async fn get_neoforge_versions(minecraft_version: &str) -> Result<Vec<NeoForgeVersion>> {
    let client = reqwest::Client::new();
    let response = client
        .get(NEOFORGE_MAVEN_URL)
        .header("User-Agent", format!("OxideLauncher/{}", env!("CARGO_PKG_VERSION")))
        .send()
        .await?;

    if !response.status().is_success() {
        return Err(OxideError::Modloader(format!(
            "Failed to fetch NeoForge versions: HTTP {}",
            response.status()
        )));
    }

    let maven_response: NeoForgeMavenResponse = response.json().await?;
    
    // NeoForge versions are formatted like "21.4.88" (for MC 1.21.4)
    // The first two parts map to MC version: 21.4.x -> MC 1.21.4
    // For MC 1.21 (no minor), look for "21.0.x"
    let mc_parts: Vec<&str> = minecraft_version.split('.').collect();
    let (major, minor) = if mc_parts.len() >= 3 {
        // 1.21.4 -> (21, 4)
        (mc_parts[1], mc_parts[2])
    } else if mc_parts.len() == 2 {
        // 1.21 -> (21, 0)
        (mc_parts[1], "0")
    } else {
        return Ok(Vec::new());
    };
    
    let prefix = format!("{}.", major);
    let expected_minor = format!("{}.{}", major, minor);
    
    let mut versions: Vec<NeoForgeVersion> = maven_response.versions
        .into_iter()
        .filter(|v| {
            // Skip beta/special versions
            if v.contains("craftmine") {
                return false;
            }
            
            // Check if version matches our MC version
            if !v.starts_with(&prefix) {
                return false;
            }
            
            // Parse the version to check major.minor match
            let parts: Vec<&str> = v.split(|c| c == '.' || c == '-').collect();
            if parts.len() >= 2 {
                let ver_prefix = format!("{}.{}", parts[0], parts[1]);
                ver_prefix == expected_minor
            } else {
                false
            }
        })
        .enumerate()
        .map(|(idx, version)| NeoForgeVersion {
            version: version.clone(),
            minecraft_version: minecraft_version.to_string(),
            recommended: idx == 0,
        })
        .collect();
    
    // Sort by version number descending (latest first)
    versions.sort_by(|a, b| {
        let a_parts: Vec<u32> = a.version
            .split(|c| c == '.' || c == '-')
            .filter_map(|s| s.parse().ok())
            .collect();
        let b_parts: Vec<u32> = b.version
            .split(|c| c == '.' || c == '-')
            .filter_map(|s| s.parse().ok())
            .collect();
        b_parts.cmp(&a_parts)
    });
    
    // Mark the first (latest) as recommended
    if let Some(first) = versions.first_mut() {
        first.recommended = true;
    }
    for v in versions.iter_mut().skip(1) {
        v.recommended = false;
    }
    
    Ok(versions)
}

/// Get the recommended NeoForge version for a Minecraft version
#[allow(dead_code)] // Utility function for future auto-select feature
pub async fn get_recommended_neoforge(minecraft_version: &str) -> Result<Option<String>> {
    let versions = get_neoforge_versions(minecraft_version).await?;
    Ok(versions.into_iter().find(|v| v.recommended).map(|v| v.version))
}
