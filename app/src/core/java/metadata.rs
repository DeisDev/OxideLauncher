//! Java metadata for downloadable Java runtimes
//!
//! Similar to Prism Launcher's JavaMetadata.cpp

use std::cmp::Ordering;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::core::java::version::JavaVersion;

/// Type of Java download
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DownloadType {
    /// Download as archive (zip/tar.gz) and extract
    Archive,
    /// Download via manifest (individual files)
    Manifest,
    /// Unknown download type
    Unknown,
}

impl Default for DownloadType {
    fn default() -> Self {
        Self::Unknown
    }
}

impl From<&str> for DownloadType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "archive" => Self::Archive,
            "manifest" => Self::Manifest,
            _ => Self::Unknown,
        }
    }
}

impl std::fmt::Display for DownloadType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Archive => write!(f, "archive"),
            Self::Manifest => write!(f, "manifest"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Checksum information for verification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Checksum {
    /// Hash algorithm (sha1, sha256)
    pub hash_type: String,
    /// Hash value
    pub hash: String,
}

impl Checksum {
    pub fn sha1(hash: &str) -> Self {
        Self {
            hash_type: "sha1".to_string(),
            hash: hash.to_string(),
        }
    }
    
    pub fn sha256(hash: &str) -> Self {
        Self {
            hash_type: "sha256".to_string(),
            hash: hash.to_string(),
        }
    }
    
    pub fn is_empty(&self) -> bool {
        self.hash.is_empty()
    }
}

/// Java vendor information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum JavaVendor {
    Adoptium,
    Azul,
    Microsoft,
    Oracle,
    Amazon,
    Other(String),
}

impl Default for JavaVendor {
    fn default() -> Self {
        Self::Other("Unknown".to_string())
    }
}

impl std::fmt::Display for JavaVendor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Adoptium => write!(f, "Eclipse Adoptium"),
            Self::Azul => write!(f, "Azul Zulu"),
            Self::Microsoft => write!(f, "Microsoft"),
            Self::Oracle => write!(f, "Oracle"),
            Self::Amazon => write!(f, "Amazon Corretto"),
            Self::Other(s) => write!(f, "{}", s),
        }
    }
}

impl From<&str> for JavaVendor {
    fn from(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("adoptium") || lower.contains("temurin") {
            Self::Adoptium
        } else if lower.contains("azul") || lower.contains("zulu") {
            Self::Azul
        } else if lower.contains("microsoft") {
            Self::Microsoft
        } else if lower.contains("oracle") {
            Self::Oracle
        } else if lower.contains("amazon") || lower.contains("corretto") {
            Self::Amazon
        } else {
            Self::Other(s.to_string())
        }
    }
}

/// Metadata for a downloadable Java runtime
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JavaMetadata {
    /// Display name
    pub name: String,
    /// Vendor
    pub vendor: JavaVendor,
    /// Download URL
    pub url: String,
    /// Release date/time
    #[serde(default)]
    pub release_time: Option<DateTime<Utc>>,
    /// Type of download (archive or manifest)
    pub download_type: DownloadType,
    /// Package type (jdk, jre)
    pub package_type: String,
    /// Target OS identifier (e.g., "windows-x64", "linux-x64", "mac-aarch64")
    pub runtime_os: String,
    /// Checksum for verification
    #[serde(default)]
    pub checksum: Checksum,
    /// Java version
    pub version: JavaVersion,
    /// File size in bytes
    #[serde(default)]
    pub size: Option<u64>,
}

impl JavaMetadata {
    /// Get a display-friendly identifier
    pub fn descriptor(&self) -> String {
        format!("{} {} ({})", self.vendor, self.version, self.package_type)
    }
    
    /// Check if this metadata matches the current OS
    pub fn matches_current_os(&self) -> bool {
        let current = get_current_runtime_os();
        self.runtime_os == current
    }
}

impl PartialEq for JavaMetadata {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version && self.name == other.name
    }
}

impl Eq for JavaMetadata {}

impl PartialOrd for JavaMetadata {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JavaMetadata {
    fn cmp(&self, other: &Self) -> Ordering {
        // First compare by version
        match self.version.cmp(&other.version) {
            Ordering::Equal => {}
            ord => return ord,
        }
        
        // Then by release time
        match (&self.release_time, &other.release_time) {
            (Some(a), Some(b)) => match a.cmp(b) {
                Ordering::Equal => {}
                ord => return ord,
            },
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            (None, None) => {}
        }
        
        // Finally by name
        self.name.cmp(&other.name)
    }
}

/// Get the current runtime OS identifier
pub fn get_current_runtime_os() -> String {
    let os = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    };
    
    let arch = if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "arm") {
        "arm"
    } else {
        "unknown"
    };
    
    format!("{}-{}", os, arch)
}

/// Get the architecture string for the current system
pub fn get_current_arch() -> &'static str {
    if cfg!(target_arch = "x86_64") {
        "x64"
    } else if cfg!(target_arch = "x86") {
        "x86"
    } else if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "arm") {
        "arm32"
    } else {
        "unknown"
    }
}

/// Get the OS string for the current system
pub fn get_current_os() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "mac"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    }
}

/// Parse Java metadata from Adoptium API response
pub fn parse_adoptium_metadata(json: &serde_json::Value) -> Option<JavaMetadata> {
    let binary = json.get("binary")?;
    let package = binary.get("package")?;
    let version_data = json.get("version")?;
    
    let version = JavaVersion::new(
        version_data.get("major")?.as_u64()? as u32,
        version_data.get("minor")?.as_u64().unwrap_or(0) as u32,
        version_data.get("security")?.as_u64().unwrap_or(0) as u32,
        version_data.get("build")?.as_u64().unwrap_or(0) as u32,
    );
    
    let checksum = package.get("checksum")
        .and_then(|c| c.as_str())
        .map(Checksum::sha256)
        .unwrap_or_default();
    
    Some(JavaMetadata {
        name: format!("Temurin {}", version),
        vendor: JavaVendor::Adoptium,
        url: package.get("link")?.as_str()?.to_string(),
        release_time: json.get("release_date")
            .and_then(|d| d.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc)),
        download_type: DownloadType::Archive,
        package_type: binary.get("image_type")?.as_str()?.to_string(),
        runtime_os: format!("{}-{}", 
            binary.get("os")?.as_str()?,
            binary.get("architecture")?.as_str()?
        ),
        checksum,
        version,
        size: package.get("size").and_then(|s| s.as_u64()),
    })
}

/// Parse Java metadata from Azul API response
pub fn parse_azul_metadata(json: &serde_json::Value) -> Option<JavaMetadata> {
    let version_str = json.get("java_version")?.as_array()?
        .iter()
        .filter_map(|v| v.as_u64())
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(".");
    
    let version = JavaVersion::parse(&version_str);
    
    let checksum = json.get("sha256_hash")
        .and_then(|c| c.as_str())
        .map(Checksum::sha256)
        .unwrap_or_default();
    
    Some(JavaMetadata {
        name: json.get("name")?.as_str()?.to_string(),
        vendor: JavaVendor::Azul,
        url: json.get("download_url")?.as_str()?.to_string(),
        release_time: json.get("release_date")
            .and_then(|d| d.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc)),
        download_type: DownloadType::Archive,
        package_type: json.get("bundle_type")?.as_str()?.to_string(),
        runtime_os: format!("{}-{}", 
            json.get("os")?.as_str()?,
            json.get("arch")?.as_str()?
        ),
        checksum,
        version,
        size: json.get("file_size").and_then(|s| s.as_u64()),
    })
}
