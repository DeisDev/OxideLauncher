//! Library management for Minecraft

use std::path::PathBuf;
use crate::core::minecraft::version::{Library, VersionData};

/// Get all libraries needed for a version
pub fn get_required_libraries(version: &VersionData) -> Vec<&Library> {
    version.libraries
        .iter()
        .filter(|lib| lib.applies_to_current_os())
        .collect()
}

/// Get the classpath for launching
pub fn build_classpath(
    version: &VersionData,
    libraries_dir: &PathBuf,
    client_jar: &PathBuf,
) -> String {
    let separator = if cfg!(target_os = "windows") { ";" } else { ":" };
    
    let mut paths: Vec<String> = get_required_libraries(version)
        .iter()
        .filter_map(|lib| {
            let path = if let Some(downloads) = &lib.downloads {
                if let Some(artifact) = &downloads.artifact {
                    libraries_dir.join(&artifact.path)
                } else {
                    libraries_dir.join(lib.artifact_path())
                }
            } else {
                libraries_dir.join(lib.artifact_path())
            };
            
            if path.exists() {
                Some(path.to_string_lossy().to_string())
            } else {
                None
            }
        })
        .collect();
    
    // Add client jar at the end
    paths.push(client_jar.to_string_lossy().to_string());
    
    paths.join(separator)
}

/// Get libraries that need to be downloaded
pub fn get_missing_libraries(
    version: &VersionData,
    libraries_dir: &PathBuf,
) -> Vec<LibraryDownload> {
    get_required_libraries(version)
        .iter()
        .filter_map(|lib| {
            if let Some(downloads) = &lib.downloads {
                if let Some(artifact) = &downloads.artifact {
                    let path = libraries_dir.join(&artifact.path);
                    if !path.exists() {
                        return Some(LibraryDownload {
                            name: lib.name.clone(),
                            url: artifact.url.clone(),
                            sha1: artifact.sha1.clone(),
                            size: artifact.size,
                            path: artifact.path.clone(),
                        });
                    }
                }
            } else if let Some(url) = &lib.url {
                // Maven-style library
                let artifact_path = lib.artifact_path();
                let path = libraries_dir.join(&artifact_path);
                if !path.exists() {
                    return Some(LibraryDownload {
                        name: lib.name.clone(),
                        url: format!("{}{}", url, artifact_path),
                        sha1: String::new(), // No hash available
                        size: 0,
                        path: artifact_path,
                    });
                }
            }
            None
        })
        .collect()
}

/// Get native libraries that need to be extracted
pub fn get_native_libraries(
    version: &VersionData,
    _libraries_dir: &PathBuf,
) -> Vec<NativeLibrary> {
    get_required_libraries(version)
        .iter()
        .filter_map(|lib| {
            let classifier = lib.native_classifier()?;
            
            let (path, url, sha1, size) = if let Some(downloads) = &lib.downloads {
                if let Some(classifiers) = &downloads.classifiers {
                    if let Some(native) = classifiers.get(&classifier) {
                        (native.path.clone(), native.url.clone(), native.sha1.clone(), native.size)
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            };
            
            Some(NativeLibrary {
                name: lib.name.clone(),
                classifier,
                url,
                sha1,
                size,
                path,
                extract_exclude: lib.extract.as_ref()
                    .and_then(|e| e.exclude.clone())
                    .unwrap_or_default(),
            })
        })
        .collect()
}

/// Information about a library to download
#[allow(dead_code)] // Used in library download pipeline
#[derive(Debug, Clone)]
pub struct LibraryDownload {
    pub name: String,
    pub url: String,
    pub sha1: String,
    pub size: u64,
    pub path: String,
}

/// Information about a native library
#[allow(dead_code)] // Used in native extraction pipeline
#[derive(Debug, Clone)]
pub struct NativeLibrary {
    pub name: String,
    pub classifier: String,
    pub url: String,
    pub sha1: String,
    pub size: u64,
    pub path: String,
    pub extract_exclude: Vec<String>,
}
