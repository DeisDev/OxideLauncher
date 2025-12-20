//! Java installation detection across different operating systems.
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
use std::collections::HashSet;
use tracing::{debug, info};
use crate::core::java::install::{JavaInstallation, JavaArch};
use crate::core::java::version::JavaVersion;

/// The Java executable name for the current platform
#[cfg(target_os = "windows")]
pub const JAVA_EXECUTABLE: &str = "javaw.exe";

#[cfg(not(target_os = "windows"))]
pub const JAVA_EXECUTABLE: &str = "java";

/// Detect all Java installations on the system
pub fn detect_java_installations() -> Vec<JavaInstallation> {
    info!("Detecting Java installations...");
    
    let mut found_paths: HashSet<PathBuf> = HashSet::new();
    let mut installations: Vec<JavaInstallation> = Vec::new();
    
    // Get all candidate paths
    let candidates = get_all_java_candidates();
    debug!("Found {} candidate Java paths", candidates.len());
    
    for path in candidates {
        // Skip duplicates
        let canonical = match path.canonicalize() {
            Ok(p) => p,
            Err(_) => path.clone(),
        };
        
        if found_paths.contains(&canonical) {
            continue;
        }
        
        if !path.exists() {
            continue;
        }
        
        debug!("Checking Java at: {:?}", path);
        
        // Try to get version info
        if let Some(installation) = probe_java(&path) {
            info!("Found Java {} at {:?}", installation.version, path);
            found_paths.insert(canonical);
            installations.push(installation);
        }
    }
    
    // Sort by version (highest first)
    installations.sort_by(|a, b| b.cmp(a));
    
    info!("Detected {} Java installations", installations.len());
    installations
}

/// Find the best Java installation that meets the version requirement
/// 
/// Selection priority:
/// 1. Exact major version match (prefer managed installations)
/// 2. Compatible higher version (within reasonable range)
/// 3. 64-bit over 32-bit
/// 4. Managed installations over system-installed
pub fn find_java_for_version(required_major: u32) -> Option<JavaInstallation> {
    let installations = detect_java_installations();
    
    if installations.is_empty() {
        return None;
    }
    
    // Get compatible version range based on Minecraft requirements
    let (min_version, max_version) = get_compatible_java_range(required_major);
    
    // Filter to compatible installations
    let mut compatible: Vec<_> = installations
        .into_iter()
        .filter(|java| {
            let major = java.version.major;
            major >= min_version && major <= max_version
        })
        .collect();
    
    if compatible.is_empty() {
        return None;
    }
    
    // Score and sort installations
    compatible.sort_by(|a, b| {
        let score_a = score_java_installation(a, required_major);
        let score_b = score_java_installation(b, required_major);
        score_b.cmp(&score_a) // Higher score first
    });
    
    compatible.into_iter().next()
}

/// Get the compatible Java version range for a required version
fn get_compatible_java_range(required_major: u32) -> (u32, u32) {
    match required_major {
        8 => (8, 8), // Minecraft < 1.17 strictly needs Java 8
        16 => (16, 17), // 1.17 snapshots can use 16 or 17
        17 => (17, 21), // 1.18-1.20.4 works with 17-21
        21 => (21, 25), // 1.21+ requires 21+
        _ => (required_major, required_major + 4), // Default: allow 4 versions higher
    }
}

/// Score a Java installation for selection priority
/// Higher score = better match
fn score_java_installation(java: &JavaInstallation, required_major: u32) -> i32 {
    let mut score = 0;
    
    // Exact version match gets highest priority
    if java.version.major == required_major {
        score += 1000;
    } else {
        // Penalty for version difference (prefer closer versions)
        let diff = (java.version.major as i32 - required_major as i32).abs();
        score += 500 - (diff * 50);
    }
    
    // Prefer 64-bit installations
    if java.arch.is_64bit() {
        score += 200;
    }
    
    // Prefer managed installations (launcher-installed Java)
    if java.is_managed {
        score += 150;
    }
    
    // Prefer native architecture
    if java.arch == JavaArch::current() {
        score += 100;
    }
    
    // Small bonus for recommended installations
    if java.recommended {
        score += 50;
    }
    
    score
}

/// Find the best Java for a Minecraft version
pub fn find_java_for_minecraft(minecraft_version: &str) -> Option<JavaInstallation> {
    // Determine required Java version based on Minecraft version
    let required = get_required_java_version(minecraft_version);
    find_java_for_version(required)
}

/// Get the required Java major version for a Minecraft version
pub fn get_required_java_version(minecraft_version: &str) -> u32 {
    // Parse the minecraft version
    let parts: Vec<&str> = minecraft_version.split('.').collect();
    
    let major: u32 = parts.first().and_then(|s| s.parse().ok()).unwrap_or(1);
    let minor: u32 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    
    // Minecraft version to Java version requirements:
    // - 1.21+ requires Java 21
    // - 1.18+ requires Java 17
    // - 1.17+ requires Java 16
    // - 1.12+ works best with Java 8
    // - Older versions need Java 8
    
    if major >= 1 && minor >= 21 {
        21
    } else if major >= 1 && minor >= 18 {
        17
    } else if major >= 1 && minor >= 17 {
        16
    } else {
        8
    }
}

/// Probe a Java executable to get version and architecture info
fn probe_java(java_path: &PathBuf) -> Option<JavaInstallation> {
    let mut command = std::process::Command::new(java_path);
    command.arg("-version");
    
    // Hide console window on Windows
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    
    let output = command.output().ok()?;
    
    // Java version is printed to stderr
    let version_output = String::from_utf8_lossy(&output.stderr);
    
    let mut version = JavaVersion::default();
    let mut vendor = "Unknown".to_string();
    let mut arch = JavaArch::Unknown;
    
    for line in version_output.lines() {
        let line_lower = line.to_lowercase();
        
        // Parse version from first line
        if line_lower.contains("version") {
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    let version_str = &line[start + 1..start + 1 + end];
                    version = JavaVersion::parse(version_str);
                }
            }
        }
        
        // Detect vendor
        if line_lower.contains("temurin") || line_lower.contains("adoptium") {
            vendor = "Eclipse Adoptium".to_string();
        } else if line_lower.contains("zulu") || line_lower.contains("azul") {
            vendor = "Azul Zulu".to_string();
        } else if line_lower.contains("openjdk") {
            if vendor == "Unknown" {
                vendor = "OpenJDK".to_string();
            }
        } else if line_lower.contains("oracle") || line_lower.contains("java(tm)") {
            vendor = "Oracle".to_string();
        } else if line_lower.contains("corretto") || line_lower.contains("amazon") {
            vendor = "Amazon Corretto".to_string();
        } else if line_lower.contains("microsoft") {
            vendor = "Microsoft".to_string();
        } else if line_lower.contains("graalvm") {
            vendor = "GraalVM".to_string();
        }
        
        // Detect architecture
        if line_lower.contains("64-bit") || line_lower.contains("amd64") || line_lower.contains("x86_64") {
            arch = JavaArch::X64;
        } else if line_lower.contains("aarch64") || line_lower.contains("arm64") {
            arch = JavaArch::Aarch64;
        } else if line_lower.contains("32-bit") || line_lower.contains("i386") || line_lower.contains("x86") {
            arch = JavaArch::X86;
        }
    }
    
    if !version.parseable {
        return None;
    }
    
    // If arch wasn't detected from output, try to infer from path
    if arch == JavaArch::Unknown {
        let path_str = java_path.to_string_lossy().to_lowercase();
        if path_str.contains("x64") || path_str.contains("amd64") {
            arch = JavaArch::X64;
        } else if path_str.contains("x86") && !path_str.contains("x86_64") {
            arch = JavaArch::X86;
        } else if path_str.contains("aarch64") || path_str.contains("arm64") {
            arch = JavaArch::Aarch64;
        } else {
            // Default to current system arch
            arch = JavaArch::current();
        }
    }
    
    Some(JavaInstallation::new(
        java_path.clone(),
        version,
        arch,
        vendor,
    ))
}

/// Get all candidate Java paths to check
fn get_all_java_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    
    // Platform-specific candidates
    #[cfg(target_os = "windows")]
    candidates.extend(get_windows_java_candidates());
    
    #[cfg(target_os = "macos")]
    candidates.extend(get_macos_java_candidates());
    
    #[cfg(target_os = "linux")]
    candidates.extend(get_linux_java_candidates());
    
    // Add Java from PATH
    candidates.extend(get_java_from_path());
    
    // Add Java from environment variables
    candidates.extend(get_java_from_env());
    
    // Add managed Java installations
    candidates.extend(get_managed_java_paths());
    
    candidates
}

/// Get Java candidates on Windows
#[cfg(target_os = "windows")]
fn get_windows_java_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    
    // Check Program Files directories
    let program_dirs: Vec<PathBuf> = [
        std::env::var_os("ProgramFiles").map(PathBuf::from),
        std::env::var_os("ProgramFiles(x86)").map(PathBuf::from),
        std::env::var_os("ProgramW6432").map(PathBuf::from),
        Some(PathBuf::from("C:\\Program Files")),
        Some(PathBuf::from("C:\\Program Files (x86)")),
    ]
    .into_iter()
    .flatten()
    .collect();
    
    let java_dirs = [
        "Java",
        "Eclipse Adoptium",
        "Eclipse Foundation",
        "AdoptOpenJDK",
        "Zulu",
        "Microsoft",
        "Amazon Corretto",
        "BellSoft",
        "Semeru",
    ];
    
    for program_dir in &program_dirs {
        for java_dir in &java_dirs {
            let dir = program_dir.join(java_dir);
            candidates.extend(find_java_in_directory(&dir));
        }
    }
    
    // Check registry for Java installations
    candidates.extend(get_java_from_windows_registry());
    
    // Check common installation paths
    let local_app_data = std::env::var_os("LOCALAPPDATA").map(PathBuf::from);
    if let Some(local) = local_app_data {
        // Scoop
        let scoop_java = local.join("scoop").join("apps");
        if scoop_java.exists() {
            for entry in std::fs::read_dir(&scoop_java).into_iter().flatten().flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains("jdk") || name.contains("java") || name.contains("temurin") || name.contains("zulu") {
                    candidates.extend(find_java_in_directory(&entry.path().join("current")));
                }
            }
        }
    }
    
    candidates
}

/// Get Java from Windows Registry
#[cfg(target_os = "windows")]
fn get_java_from_windows_registry() -> Vec<PathBuf> {
    use winreg::enums::*;
    use winreg::RegKey;
    
    let mut candidates = Vec::new();
    
    let registry_paths = [
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\JavaSoft\Java Runtime Environment"),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\JavaSoft\Java Development Kit"),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\JavaSoft\JDK"),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\Eclipse Adoptium\JDK"),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\Eclipse Adoptium\JRE"),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\AdoptOpenJDK\JDK"),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\AdoptOpenJDK\JRE"),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\Azul Systems\Zulu"),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\JDK"),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\BellSoft\Liberica"),
        (HKEY_CURRENT_USER, r"SOFTWARE\JavaSoft\Java Runtime Environment"),
        (HKEY_CURRENT_USER, r"SOFTWARE\JavaSoft\Java Development Kit"),
    ];
    
    for (hkey, path) in registry_paths {
        if let Ok(key) = RegKey::predef(hkey).open_subkey(path) {
            if let Ok(subkeys) = key.enum_keys().collect::<Result<Vec<_>, _>>() {
                for subkey_name in subkeys {
                    if let Ok(subkey) = key.open_subkey(&subkey_name) {
                        // Try different value names for Java home
                        for value_name in ["JavaHome", "InstallationPath", "Path"] {
                            if let Ok(java_home) = subkey.get_value::<String, _>(value_name) {
                                let java_path = PathBuf::from(&java_home).join("bin").join(JAVA_EXECUTABLE);
                                if java_path.exists() {
                                    candidates.push(java_path);
                                }
                            }
                        }
                    }
                }
            }
            
            // Also check for JavaHome directly on the key
            if let Ok(java_home) = key.get_value::<String, _>("JavaHome") {
                let java_path = PathBuf::from(&java_home).join("bin").join(JAVA_EXECUTABLE);
                if java_path.exists() {
                    candidates.push(java_path);
                }
            }
        }
    }
    
    candidates
}

/// Get Java candidates on macOS
#[cfg(target_os = "macos")]
fn get_macos_java_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    
    // System Java Virtual Machines
    let system_jvm = PathBuf::from("/Library/Java/JavaVirtualMachines");
    candidates.extend(find_java_in_macos_jvm_dir(&system_jvm));
    
    // User Java Virtual Machines
    if let Some(home) = dirs::home_dir() {
        let user_jvm = home.join("Library/Java/JavaVirtualMachines");
        candidates.extend(find_java_in_macos_jvm_dir(&user_jvm));
    }
    
    // Homebrew locations
    let homebrew_dirs = [
        PathBuf::from("/opt/homebrew/opt"),
        PathBuf::from("/usr/local/opt"),
        PathBuf::from("/opt/homebrew/Cellar"),
        PathBuf::from("/usr/local/Cellar"),
    ];
    
    for brew_dir in &homebrew_dirs {
        if brew_dir.exists() {
            for entry in std::fs::read_dir(brew_dir).into_iter().flatten().flatten() {
                let name = entry.file_name().to_string_lossy().to_lowercase();
                if name.contains("openjdk") || name.contains("java") || name.contains("temurin") || name.contains("zulu") {
                    candidates.extend(find_java_in_directory(&entry.path()));
                    // Also check versioned subdirectories
                    for subentry in std::fs::read_dir(entry.path()).into_iter().flatten().flatten() {
                        candidates.extend(find_java_in_directory(&subentry.path()));
                    }
                }
            }
        }
    }
    
    // SDKMAN
    if let Some(home) = dirs::home_dir() {
        let sdkman_java = home.join(".sdkman/candidates/java");
        if sdkman_java.exists() {
            for entry in std::fs::read_dir(&sdkman_java).into_iter().flatten().flatten() {
                if entry.path().is_dir() {
                    candidates.extend(find_java_in_directory(&entry.path()));
                }
            }
        }
    }
    
    candidates
}

/// Find Java in macOS JVM directory structure
#[cfg(target_os = "macos")]
fn find_java_in_macos_jvm_dir(dir: &PathBuf) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    
    if !dir.exists() {
        return candidates;
    }
    
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // macOS Java structure: .../Contents/Home/bin/java
                let java_path = path.join("Contents/Home/bin/java");
                if java_path.exists() {
                    candidates.push(java_path);
                }
                
                // Some installations may have direct bin/java
                let direct_path = path.join("bin/java");
                if direct_path.exists() {
                    candidates.push(direct_path);
                }
            }
        }
    }
    
    candidates
}

/// Get Java candidates on Linux
#[cfg(target_os = "linux")]
fn get_linux_java_candidates() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    
    // Common Linux Java directories
    let java_dirs = [
        PathBuf::from("/usr/lib/jvm"),
        PathBuf::from("/usr/lib64/jvm"),
        PathBuf::from("/usr/java"),
        PathBuf::from("/opt/java"),
        PathBuf::from("/opt/jdk"),
    ];
    
    for dir in &java_dirs {
        candidates.extend(find_java_in_directory(dir));
    }
    
    // User-local installations
    if let Some(home) = dirs::home_dir() {
        // SDKMAN
        let sdkman_java = home.join(".sdkman/candidates/java");
        if sdkman_java.exists() {
            for entry in std::fs::read_dir(&sdkman_java).into_iter().flatten().flatten() {
                if entry.path().is_dir() && entry.file_name() != "current" {
                    candidates.extend(find_java_in_directory(&entry.path()));
                }
            }
        }
        
        // ASDF
        let asdf_java = home.join(".asdf/installs/java");
        if asdf_java.exists() {
            for entry in std::fs::read_dir(&asdf_java).into_iter().flatten().flatten() {
                if entry.path().is_dir() {
                    candidates.extend(find_java_in_directory(&entry.path()));
                }
            }
        }
        
        // Jabba
        let jabba_java = home.join(".jabba/jdk");
        if jabba_java.exists() {
            for entry in std::fs::read_dir(&jabba_java).into_iter().flatten().flatten() {
                if entry.path().is_dir() {
                    candidates.extend(find_java_in_directory(&entry.path()));
                }
            }
        }
        
        // Local Java directory
        let local_java = home.join(".local/share/java");
        candidates.extend(find_java_in_directory(&local_java));
    }
    
    // Snap Java installations
    let snap_dir = PathBuf::from("/snap");
    if snap_dir.exists() {
        for entry in std::fs::read_dir(&snap_dir).into_iter().flatten().flatten() {
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if name.contains("openjdk") || name.contains("java") {
                let current = entry.path().join("current");
                candidates.extend(find_java_in_directory(&current));
            }
        }
    }
    
    // Flatpak Java (in managed directory)
    
    candidates
}

/// Find Java executables in a directory (recursively checks subdirectories)
fn find_java_in_directory(dir: &PathBuf) -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    
    if !dir.exists() {
        return candidates;
    }
    
    // Check if this directory has bin/java directly
    let direct_java = dir.join("bin").join(JAVA_EXECUTABLE);
    if direct_java.exists() {
        candidates.push(direct_java);
    }
    
    // Check subdirectories
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let java_path = path.join("bin").join(JAVA_EXECUTABLE);
                if java_path.exists() {
                    candidates.push(java_path);
                }
                
                // Check for macOS structure in case of cross-platform installs
                #[cfg(target_os = "macos")]
                {
                    let macos_java = path.join("Contents/Home/bin/java");
                    if macos_java.exists() {
                        candidates.push(macos_java);
                    }
                }
            }
        }
    }
    
    candidates
}

/// Get Java from system PATH
fn get_java_from_path() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    
    // Try to find java in PATH
    if let Ok(path) = which::which(JAVA_EXECUTABLE) {
        candidates.push(path);
    }
    
    // Also try javaw on Windows
    #[cfg(target_os = "windows")]
    {
        if let Ok(path) = which::which("java.exe") {
            candidates.push(path);
        }
    }
    
    candidates
}

/// Get Java from environment variables
fn get_java_from_env() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    
    // JAVA_HOME
    if let Some(java_home) = std::env::var_os("JAVA_HOME") {
        let java_path = PathBuf::from(&java_home).join("bin").join(JAVA_EXECUTABLE);
        if java_path.exists() {
            candidates.push(java_path);
        }
    }
    
    // JRE_HOME
    if let Some(jre_home) = std::env::var_os("JRE_HOME") {
        let java_path = PathBuf::from(&jre_home).join("bin").join(JAVA_EXECUTABLE);
        if java_path.exists() {
            candidates.push(java_path);
        }
    }
    
    candidates
}

/// Get launcher-managed Java installation paths
fn get_managed_java_paths() -> Vec<PathBuf> {
    let mut candidates = Vec::new();
    
    // Get the launcher's data directory
    if let Some(data_dir) = dirs::data_dir() {
        let managed_java_dir = data_dir.join("OxideLauncher").join("java");
        if managed_java_dir.exists() {
            candidates.extend(find_java_in_directory(&managed_java_dir));
        }
    }
    
    candidates
}

/// Stub for non-Windows platforms
#[cfg(not(target_os = "windows"))]
fn get_windows_java_candidates() -> Vec<PathBuf> {
    Vec::new()
}

/// Stub for non-macOS platforms  
#[cfg(not(target_os = "macos"))]
#[allow(dead_code)] // Platform-specific stub, needed for cross-compilation
fn get_macos_java_candidates() -> Vec<PathBuf> {
    Vec::new()
}

/// Stub for non-Linux platforms
#[cfg(not(target_os = "linux"))]
#[allow(dead_code)] // Platform-specific stub, needed for cross-compilation
fn get_linux_java_candidates() -> Vec<PathBuf> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_java_requirement_for_minecraft() {
        assert_eq!(get_required_java_version("1.21"), 21);
        assert_eq!(get_required_java_version("1.20.4"), 17);
        assert_eq!(get_required_java_version("1.18"), 17);
        assert_eq!(get_required_java_version("1.17"), 16);
        assert_eq!(get_required_java_version("1.16.5"), 8);
        assert_eq!(get_required_java_version("1.12.2"), 8);
    }
}
