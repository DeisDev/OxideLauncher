//! Java runtime download from Adoptium and Azul APIs.
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

use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tokio::io::AsyncWriteExt;
use futures::StreamExt;
use tracing::{debug, info};
use serde::{Deserialize, Serialize};
use crate::core::java::metadata::{JavaMetadata, DownloadType, get_current_arch, get_current_os};
use crate::core::java::install::JavaInstallation;
use crate::core::java::detection::JAVA_EXECUTABLE;
use crate::core::error::{OxideError, Result};

/// Progress event for Java downloads
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JavaDownloadProgress {
    /// Starting download
    Started { name: String, total_size: Option<u64> },
    /// Download progress
    Downloading { name: String, downloaded: u64, total: Option<u64> },
    /// Extracting archive
    Extracting { name: String },
    /// Extraction progress
    ExtractProgress { name: String, current: u64, total: u64 },
    /// Completed successfully
    Completed { name: String, path: PathBuf },
    /// Failed with error
    Failed { name: String, error: String },
}

/// Available Java versions from online sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableJavaVersion {
    /// Major version (8, 11, 17, 21, etc.)
    pub major: u32,
    /// Display name
    pub name: String,
    /// Whether this version is LTS
    pub is_lts: bool,
}

/// Get the managed Java installation directory
pub fn get_java_install_dir() -> Result<PathBuf> {
    let data_dir = dirs::data_dir()
        .ok_or_else(|| OxideError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Could not find data directory",
        )))?;
    
    Ok(data_dir.join("OxideLauncher").join("java"))
}

/// Fetch available Java versions from Adoptium API
pub async fn fetch_adoptium_versions() -> Result<Vec<AvailableJavaVersion>> {
    let client = reqwest::Client::new();
    let url = "https://api.adoptium.net/v3/info/available_releases";
    
    let response = client.get(url)
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(OxideError::Download(format!(
            "Failed to fetch Adoptium releases: HTTP {}",
            response.status()
        )));
    }
    
    let data: serde_json::Value = response.json().await?;
    
    let mut versions = Vec::new();
    
    // Get available LTS versions
    if let Some(lts_versions) = data.get("available_lts_releases").and_then(|v| v.as_array()) {
        for v in lts_versions {
            if let Some(major) = v.as_u64() {
                versions.push(AvailableJavaVersion {
                    major: major as u32,
                    name: format!("Java {} (LTS)", major),
                    is_lts: true,
                });
            }
        }
    }
    
    // Get other available versions
    if let Some(all_versions) = data.get("available_releases").and_then(|v| v.as_array()) {
        for v in all_versions {
            if let Some(major) = v.as_u64() {
                let major = major as u32;
                // Skip if already added as LTS
                if !versions.iter().any(|av| av.major == major) {
                    versions.push(AvailableJavaVersion {
                        major,
                        name: format!("Java {}", major),
                        is_lts: false,
                    });
                }
            }
        }
    }
    
    // Sort by major version descending
    versions.sort_by(|a, b| b.major.cmp(&a.major));
    
    Ok(versions)
}

/// Fetch Java download metadata from Adoptium for a specific major version
pub async fn fetch_adoptium_download(major_version: u32) -> Result<JavaMetadata> {
    let client = reqwest::Client::new();
    
    let os = get_current_os();
    let arch = get_current_arch();
    
    // Map our arch names to Adoptium's
    let adoptium_arch = match arch {
        "x64" => "x64",
        "x86" => "x32",
        "aarch64" => "aarch64",
        "arm32" => "arm",
        _ => return Err(OxideError::Download(format!("Unsupported architecture: {}", arch))),
    };
    
    let url = format!(
        "https://api.adoptium.net/v3/assets/latest/{}/hotspot?os={}&architecture={}&image_type=jdk",
        major_version, os, adoptium_arch
    );
    
    debug!("Fetching Adoptium download: {}", url);
    
    let response = client.get(&url)
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(OxideError::Download(format!(
            "Failed to fetch Adoptium download info: HTTP {}",
            response.status()
        )));
    }
    
    let data: Vec<serde_json::Value> = response.json().await?;
    
    if data.is_empty() {
        return Err(OxideError::Download(format!(
            "No Adoptium release found for Java {} on {}-{}",
            major_version, os, arch
        )));
    }
    
    // Get the first (latest) release
    let release = &data[0];
    
    crate::core::java::metadata::parse_adoptium_metadata(release)
        .ok_or_else(|| OxideError::Download("Failed to parse Adoptium metadata".to_string()))
}

/// Download and install Java from metadata
pub async fn download_java(
    metadata: &JavaMetadata,
    progress_tx: Option<mpsc::Sender<JavaDownloadProgress>>,
) -> Result<JavaInstallation> {
    let install_dir = get_java_install_dir()?;
    std::fs::create_dir_all(&install_dir)?;
    
    // Create a unique directory name for this Java version
    let java_dir_name = format!(
        "{}-{}-{}",
        metadata.vendor.to_string().to_lowercase().replace(' ', "-"),
        metadata.version.major,
        metadata.version.minor
    );
    let java_dir = install_dir.join(&java_dir_name);
    
    info!("Downloading Java to {:?}", java_dir);
    
    // Send start progress
    if let Some(tx) = &progress_tx {
        let _ = tx.send(JavaDownloadProgress::Started {
            name: metadata.name.clone(),
            total_size: metadata.size,
        }).await;
    }
    
    match metadata.download_type {
        DownloadType::Archive => {
            download_archive(&metadata.url, &java_dir, &metadata.checksum.hash, progress_tx.clone()).await?
        }
        DownloadType::Manifest => {
            download_manifest(&metadata.url, &java_dir, progress_tx.clone()).await?
        }
        DownloadType::Unknown => {
            // Try archive download as default
            download_archive(&metadata.url, &java_dir, &metadata.checksum.hash, progress_tx.clone()).await?
        }
    }
    
    // Find the Java executable
    let java_path = find_java_in_extracted_dir(&java_dir)?;
    
    info!("Java installed successfully at {:?}", java_path);
    
    // Send completion
    if let Some(tx) = &progress_tx {
        let _ = tx.send(JavaDownloadProgress::Completed {
            name: metadata.name.clone(),
            path: java_path.clone(),
        }).await;
    }
    
    // Create and return installation
    let mut installation = JavaInstallation::new(
        java_path,
        metadata.version.clone(),
        crate::core::java::install::JavaArch::current(),
        metadata.vendor.to_string(),
    );
    installation.is_managed = true;
    
    Ok(installation)
}

/// Download and extract an archive (zip/tar.gz)
async fn download_archive(
    url: &str,
    dest_dir: &Path,
    expected_hash: &str,
    progress_tx: Option<mpsc::Sender<JavaDownloadProgress>>,
) -> Result<()> {
    let client = reqwest::Client::new();
    
    // Create temp file for download
    let temp_dir = tempfile::tempdir()?;
    let archive_name = url.split('/').last().unwrap_or("java.zip");
    let archive_path = temp_dir.path().join(archive_name);
    
    debug!("Downloading {} to {:?}", url, archive_path);
    
    // Download the file
    let response = client.get(url)
        .send()
        .await?;
    
    if !response.status().is_success() {
        return Err(OxideError::Download(format!(
            "Failed to download Java: HTTP {}",
            response.status()
        )));
    }
    
    let total_size = response.content_length();
    let mut downloaded: u64 = 0;
    
    let mut file = tokio::fs::File::create(&archive_path).await?;
    let mut stream = response.bytes_stream();
    
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
        downloaded += chunk.len() as u64;
        
        if let Some(tx) = &progress_tx {
            let _ = tx.send(JavaDownloadProgress::Downloading {
                name: "Java".to_string(),
                downloaded,
                total: total_size,
            }).await;
        }
    }
    
    drop(file);
    
    // Verify hash if provided
    if !expected_hash.is_empty() {
        debug!("Verifying hash...");
        let actual_hash = compute_sha256(&archive_path)?;
        if actual_hash.to_lowercase() != expected_hash.to_lowercase() {
            return Err(OxideError::Download(format!(
                "SHA256 mismatch: expected {}, got {}",
                expected_hash, actual_hash
            )));
        }
    }
    
    // Send extracting progress
    if let Some(tx) = &progress_tx {
        let _ = tx.send(JavaDownloadProgress::Extracting {
            name: "Java".to_string(),
        }).await;
    }
    
    // Extract the archive
    debug!("Extracting {:?} to {:?}", archive_path, dest_dir);
    
    // Create destination directory
    std::fs::create_dir_all(dest_dir)?;
    
    // Determine archive type and extract
    if archive_name.ends_with(".zip") {
        extract_zip(&archive_path, dest_dir)?;
    } else if archive_name.ends_with(".tar.gz") || archive_name.ends_with(".tgz") {
        extract_tar_gz(&archive_path, dest_dir)?;
    } else {
        // Try zip first, then tar.gz
        if extract_zip(&archive_path, dest_dir).is_err() {
            extract_tar_gz(&archive_path, dest_dir)?;
        }
    }
    
    Ok(())
}

/// Download Java via manifest (individual files)
async fn download_manifest(
    _url: &str,
    _dest_dir: &Path,
    _progress_tx: Option<mpsc::Sender<JavaDownloadProgress>>,
) -> Result<()> {
    // Manifest download is more complex and less commonly used
    // For now, return an error suggesting archive download
    Err(OxideError::Download(
        "Manifest-based Java download not yet supported. Please use archive download.".to_string()
    ))
}

/// Extract a ZIP archive
fn extract_zip(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    let file = std::fs::File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    
    // Get the root directory name from the archive (if any)
    let root_dir = archive.file_names()
        .next()
        .and_then(|name| name.split('/').next())
        .map(|s| s.to_string());
    
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => {
                // Strip the root directory if present
                let stripped = if let Some(ref root) = root_dir {
                    path.strip_prefix(root).map(|p| p.to_path_buf()).unwrap_or_else(|_| path.to_path_buf())
                } else {
                    path.to_path_buf()
                };
                dest_dir.join(stripped)
            }
            None => continue,
        };
        
        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
            
            // Set executable permissions on Unix
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
                }
            }
        }
    }
    
    Ok(())
}

/// Extract a tar.gz archive
fn extract_tar_gz(archive_path: &Path, dest_dir: &Path) -> Result<()> {
    use flate2::read::GzDecoder;
    use tar::Archive;
    
    let file = std::fs::File::open(archive_path)?;
    let gz = GzDecoder::new(file);
    let _archive = Archive::new(gz);
    
    // First pass: find root directory
    let mut root_dir: Option<String> = None;
    
    // Re-open to iterate entries
    let file = std::fs::File::open(archive_path)?;
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);
    
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        
        // Determine root directory from first entry
        if root_dir.is_none() {
            root_dir = path.components()
                .next()
                .map(|c| c.as_os_str().to_string_lossy().to_string());
        }
        
        // Strip root directory
        let stripped_path = if let Some(ref root) = root_dir {
            path.strip_prefix(root).map(|p| p.to_path_buf()).unwrap_or_else(|_| path.clone())
        } else {
            path.clone()
        };
        
        let outpath = dest_dir.join(&stripped_path);
        
        if entry.header().entry_type().is_dir() {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(parent) = outpath.parent() {
                std::fs::create_dir_all(parent)?;
            }
            entry.unpack(&outpath)?;
        }
    }
    
    Ok(())
}

/// Find Java executable in an extracted directory
fn find_java_in_extracted_dir(dir: &Path) -> Result<PathBuf> {
    // Direct bin/java
    let direct = dir.join("bin").join(JAVA_EXECUTABLE);
    if direct.exists() {
        return Ok(direct);
    }
    
    // macOS structure: Contents/Home/bin/java
    #[cfg(target_os = "macos")]
    {
        let macos = dir.join("Contents").join("Home").join("bin").join("java");
        if macos.exists() {
            return Ok(macos);
        }
    }
    
    // Search subdirectories (one level)
    if dir.exists() {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                let java_path = path.join("bin").join(JAVA_EXECUTABLE);
                if java_path.exists() {
                    return Ok(java_path);
                }
                
                // macOS structure in subdirectory
                #[cfg(target_os = "macos")]
                {
                    let macos = path.join("Contents").join("Home").join("bin").join("java");
                    if macos.exists() {
                        return Ok(macos);
                    }
                }
            }
        }
    }
    
    Err(OxideError::Download(
        "Could not find Java executable in extracted directory".to_string()
    ))
}

/// Compute SHA256 hash of a file
fn compute_sha256(path: &Path) -> Result<String> {
    use sha2::{Sha256, Digest};
    
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)?;
    
    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}

/// Delete a managed Java installation
pub async fn delete_java_installation(installation: &JavaInstallation) -> Result<()> {
    if !installation.is_managed {
        return Err(OxideError::Other(
            "Cannot delete non-managed Java installation".to_string()
        ));
    }
    
    // Get the Java home directory
    let java_home = installation.java_home()
        .ok_or_else(|| OxideError::Other("Invalid Java installation path".to_string()))?;
    
    // Verify it's in our managed directory
    let managed_dir = get_java_install_dir()?;
    if !java_home.starts_with(&managed_dir) {
        return Err(OxideError::Other(
            "Java installation is not in managed directory".to_string()
        ));
    }
    
    info!("Deleting Java installation at {:?}", java_home);
    
    // Remove the directory
    tokio::fs::remove_dir_all(&java_home).await?;
    
    Ok(())
}
