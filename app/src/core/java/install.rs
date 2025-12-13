//! Java installation representation
//!
//! Similar to Prism Launcher's JavaInstall.cpp

use std::cmp::Ordering;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::core::java::version::JavaVersion;

/// CPU Architecture for Java
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JavaArch {
    X64,
    X86,
    Aarch64,
    Arm32,
    Unknown,
}

impl Default for JavaArch {
    fn default() -> Self {
        Self::Unknown
    }
}

impl std::fmt::Display for JavaArch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::X64 => write!(f, "x64"),
            Self::X86 => write!(f, "x86"),
            Self::Aarch64 => write!(f, "aarch64"),
            Self::Arm32 => write!(f, "arm32"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

impl From<&str> for JavaArch {
    fn from(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("x86_64") || lower.contains("amd64") || lower == "x64" {
            Self::X64
        } else if lower.contains("x86") || lower == "i386" || lower == "i686" {
            Self::X86
        } else if lower.contains("aarch64") || lower.contains("arm64") {
            Self::Aarch64
        } else if lower.contains("arm") {
            Self::Arm32
        } else {
            Self::Unknown
        }
    }
}

impl JavaArch {
    /// Get the current system architecture
    pub fn current() -> Self {
        if cfg!(target_arch = "x86_64") {
            Self::X64
        } else if cfg!(target_arch = "x86") {
            Self::X86
        } else if cfg!(target_arch = "aarch64") {
            Self::Aarch64
        } else if cfg!(target_arch = "arm") {
            Self::Arm32
        } else {
            Self::Unknown
        }
    }
    
    /// Check if this is a 64-bit architecture
    pub fn is_64bit(&self) -> bool {
        matches!(self, Self::X64 | Self::Aarch64)
    }
    
    /// Get Mojang's platform string
    pub fn mojang_platform(&self) -> &'static str {
        if self.is_64bit() { "64" } else { "32" }
    }
}

/// Represents a detected or installed Java runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaInstallation {
    /// Path to the Java executable (java or java.exe)
    pub path: PathBuf,
    /// Parsed version information
    pub version: JavaVersion,
    /// CPU architecture
    pub arch: JavaArch,
    /// Vendor name (e.g., "Eclipse Adoptium", "Azul Zulu", "Oracle")
    pub vendor: String,
    /// Whether this installation is recommended
    #[serde(default)]
    pub recommended: bool,
    /// Whether this Java was installed by the launcher
    #[serde(default)]
    pub is_managed: bool,
    /// Unique identifier for this installation
    #[serde(default)]
    pub id: String,
}

impl Default for JavaInstallation {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            version: JavaVersion::default(),
            arch: JavaArch::Unknown,
            vendor: "Unknown".to_string(),
            recommended: false,
            is_managed: false,
            id: String::new(),
        }
    }
}

impl JavaInstallation {
    /// Create a new JavaInstallation with basic info
    pub fn new(path: PathBuf, version: JavaVersion, arch: JavaArch, vendor: String) -> Self {
        let id = format!("{}-{}-{}", 
            vendor.to_lowercase().replace(' ', "-"),
            version.major,
            path.to_string_lossy().chars().filter(|c| c.is_alphanumeric()).take(8).collect::<String>()
        );
        
        Self {
            path,
            version,
            arch,
            vendor,
            recommended: false,
            is_managed: false,
            id,
        }
    }
    
    /// Get a human-readable descriptor
    pub fn descriptor(&self) -> String {
        format!("Java {} ({}, {})", self.version.major, self.vendor, self.arch)
    }
    
    /// Get the Java home directory (parent of bin/)
    pub fn java_home(&self) -> Option<PathBuf> {
        self.path.parent()?.parent().map(|p| p.to_path_buf())
    }
    
    /// Check if this Java meets a version requirement
    pub fn meets_requirement(&self, required_major: u32) -> bool {
        self.version.meets_requirement(required_major)
    }
    
    /// Check if the Java executable exists
    pub fn exists(&self) -> bool {
        self.path.exists()
    }
    
    /// Validate that this Java installation is still valid
    pub fn validate(&self) -> bool {
        self.path.exists() && self.path.is_file()
    }
}

impl PartialEq for JavaInstallation {
    fn eq(&self, other: &Self) -> bool {
        self.arch == other.arch && self.version == other.version && self.path == other.path
    }
}

impl Eq for JavaInstallation {}

impl PartialOrd for JavaInstallation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JavaInstallation {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by architecture (prefer native arch)
        let self_native = self.arch == JavaArch::current();
        let other_native = other.arch == JavaArch::current();
        
        match (self_native, other_native) {
            (true, false) => return Ordering::Greater,
            (false, true) => return Ordering::Less,
            _ => {}
        }
        
        // Then compare by version (higher is better)
        match self.version.cmp(&other.version) {
            Ordering::Equal => {}
            ord => return ord,
        }
        
        // Finally compare by path (for determinism)
        self.path.cmp(&other.path)
    }
}

/// Result of Java installation validation/check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaValidationResult {
    /// Whether the Java is valid
    pub valid: bool,
    /// The installation being validated
    pub installation: JavaInstallation,
    /// Error message if invalid
    pub error: Option<String>,
    /// Stdout from the check process
    pub stdout: String,
    /// Stderr from the check process
    pub stderr: String,
}

impl JavaValidationResult {
    pub fn success(installation: JavaInstallation) -> Self {
        Self {
            valid: true,
            installation,
            error: None,
            stdout: String::new(),
            stderr: String::new(),
        }
    }
    
    pub fn failure(installation: JavaInstallation, error: String) -> Self {
        Self {
            valid: false,
            installation,
            error: Some(error),
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}

/// Persistent storage for managed Java installations
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManagedJavaList {
    /// List of launcher-managed Java installations
    pub installations: Vec<JavaInstallation>,
}

impl ManagedJavaList {
    /// Add a new managed installation
    pub fn add(&mut self, mut installation: JavaInstallation) {
        installation.is_managed = true;
        
        // Remove any existing installation with the same path
        self.installations.retain(|i| i.path != installation.path);
        
        self.installations.push(installation);
    }
    
    /// Remove an installation by path
    pub fn remove(&mut self, path: &PathBuf) -> bool {
        let initial_len = self.installations.len();
        self.installations.retain(|i| &i.path != path);
        self.installations.len() < initial_len
    }
    
    /// Get an installation by path
    pub fn get(&self, path: &PathBuf) -> Option<&JavaInstallation> {
        self.installations.iter().find(|i| &i.path == path)
    }
    
    /// Get all valid installations (that still exist on disk)
    pub fn get_valid(&self) -> Vec<&JavaInstallation> {
        self.installations.iter().filter(|i| i.validate()).collect()
    }
    
    /// Clean up installations that no longer exist
    pub fn cleanup(&mut self) -> usize {
        let initial_len = self.installations.len();
        self.installations.retain(|i| i.validate());
        initial_len - self.installations.len()
    }
}
