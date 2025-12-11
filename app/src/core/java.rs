//! Java detection and management

#![allow(dead_code)] // Java detection will be used in launch pipeline

use std::path::PathBuf;

/// Find a Java installation with the required major version
pub fn find_java_installation(required_version: u32) -> Option<PathBuf> {
    // Try common Java locations based on OS
    let candidates = get_java_candidates();
    
    for candidate in candidates {
        if candidate.exists() {
            if let Some(version) = get_java_version(&candidate) {
                if version >= required_version {
                    return Some(candidate);
                }
            }
        }
    }
    
    // Try PATH
    let java_exe = if cfg!(target_os = "windows") { "java.exe" } else { "java" };
    if let Ok(path) = which::which(java_exe) {
        if let Some(version) = get_java_version(&path) {
            if version >= required_version {
                return Some(path);
            }
        }
    }
    
    None
}

/// Get candidate Java paths based on OS
fn get_java_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    
    #[cfg(target_os = "windows")]
    {
        // Check Program Files
        if let Some(program_files) = std::env::var_os("ProgramFiles") {
            let pf = PathBuf::from(program_files);
            candidates.extend(find_java_in_dir(&pf.join("Java")));
            candidates.extend(find_java_in_dir(&pf.join("Eclipse Adoptium")));
            candidates.extend(find_java_in_dir(&pf.join("Microsoft")));
            candidates.extend(find_java_in_dir(&pf.join("Zulu")));
        }
        
        // Check Program Files (x86)
        if let Some(program_files_x86) = std::env::var_os("ProgramFiles(x86)") {
            let pf = PathBuf::from(program_files_x86);
            candidates.extend(find_java_in_dir(&pf.join("Java")));
        }
        
        // Check registry for Java installations
        candidates.extend(find_java_from_registry());
    }
    
    #[cfg(target_os = "macos")]
    {
        // Common macOS Java locations
        candidates.extend(find_java_in_dir(&PathBuf::from("/Library/Java/JavaVirtualMachines")));
        
        if let Some(home) = dirs::home_dir() {
            candidates.extend(find_java_in_dir(&home.join("Library/Java/JavaVirtualMachines")));
        }
        
        // Homebrew Java
        candidates.extend(find_java_in_dir(&PathBuf::from("/opt/homebrew/opt/openjdk")));
        candidates.extend(find_java_in_dir(&PathBuf::from("/usr/local/opt/openjdk")));
    }
    
    #[cfg(target_os = "linux")]
    {
        // Common Linux Java locations
        candidates.extend(find_java_in_dir(&PathBuf::from("/usr/lib/jvm")));
        candidates.extend(find_java_in_dir(&PathBuf::from("/usr/java")));
        candidates.extend(find_java_in_dir(&PathBuf::from("/opt/java")));
        
        if let Some(home) = dirs::home_dir() {
            candidates.extend(find_java_in_dir(&home.join(".sdkman/candidates/java")));
        }
    }
    
    candidates
}

/// Find Java executables in a directory
fn find_java_in_dir(dir: &PathBuf) -> Vec<PathBuf> {
    let mut results = Vec::new();
    
    if !dir.exists() {
        return results;
    }
    
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Look for java executable
                let java_path = if cfg!(target_os = "windows") {
                    path.join("bin").join("java.exe")
                } else if cfg!(target_os = "macos") {
                    // macOS has a different structure
                    let contents_home = path.join("Contents").join("Home").join("bin").join("java");
                    if contents_home.exists() {
                        contents_home
                    } else {
                        path.join("bin").join("java")
                    }
                } else {
                    path.join("bin").join("java")
                };
                
                if java_path.exists() {
                    results.push(java_path);
                }
            }
        }
    }
    
    results
}

/// Get Java version from executable
fn get_java_version(java_path: &PathBuf) -> Option<u32> {
    let output = std::process::Command::new(java_path)
        .arg("-version")
        .output()
        .ok()?;
    
    // Java version is printed to stderr
    let version_output = String::from_utf8_lossy(&output.stderr);
    
    // Parse version string (e.g., "1.8.0_292" or "17.0.1")
    for line in version_output.lines() {
        if line.contains("version") {
            // Extract version number from quotes
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    let version_str = &line[start + 1..start + 1 + end];
                    return parse_java_version(version_str);
                }
            }
        }
    }
    
    None
}

/// Parse Java version string to major version number
fn parse_java_version(version: &str) -> Option<u32> {
    let parts: Vec<&str> = version.split('.').collect();
    
    if parts.is_empty() {
        return None;
    }
    
    // Handle old format (1.8.0) vs new format (17.0.1)
    if parts[0] == "1" && parts.len() > 1 {
        parts[1].parse().ok()
    } else {
        parts[0].split('-').next()?.parse().ok()
    }
}

/// Find Java installations from Windows registry
#[cfg(target_os = "windows")]
fn find_java_from_registry() -> Vec<PathBuf> {
    let mut results = Vec::new();
    
    use winreg::enums::*;
    use winreg::RegKey;
    
    let paths = [
        r"SOFTWARE\JavaSoft\Java Runtime Environment",
        r"SOFTWARE\JavaSoft\Java Development Kit",
        r"SOFTWARE\JavaSoft\JDK",
        r"SOFTWARE\Eclipse Adoptium\JDK",
        r"SOFTWARE\AdoptOpenJDK\JDK",
        r"SOFTWARE\Azul Systems\Zulu",
    ];
    
    for path in paths {
        if let Ok(key) = RegKey::predef(HKEY_LOCAL_MACHINE).open_subkey(path) {
            if let Ok(subkeys) = key.enum_keys().collect::<Result<Vec<_>, _>>() {
                for subkey_name in subkeys {
                    if let Ok(subkey) = key.open_subkey(&subkey_name) {
                        if let Ok(java_home) = subkey.get_value::<String, _>("JavaHome") {
                            let java_path = PathBuf::from(&java_home).join("bin").join("java.exe");
                            if java_path.exists() {
                                results.push(java_path);
                            }
                        }
                    }
                }
            }
        }
    }
    
    results
}

#[cfg(not(target_os = "windows"))]
fn find_java_from_registry() -> Vec<PathBuf> {
    Vec::new()
}

/// Java installation info
#[derive(Debug, Clone)]
pub struct JavaInstallation {
    pub path: PathBuf,
    pub version: u32,
    pub vendor: String,
}

/// Get all Java installations
pub fn get_all_java_installations() -> Vec<JavaInstallation> {
    let candidates = get_java_candidates();
    
    candidates
        .into_iter()
        .filter_map(|path| {
            let version = get_java_version(&path)?;
            Some(JavaInstallation {
                path,
                version,
                vendor: "Unknown".to_string(),
            })
        })
        .collect()
}
