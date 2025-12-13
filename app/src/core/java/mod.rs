//! Java detection, management, and downloading
//!
//! This module handles all Java-related functionality:
//! - Detecting installed Java runtimes
//! - Validating Java installations via checker JAR
//! - Downloading Java from Adoptium/Azul APIs
//! - Managing launcher-installed Java runtimes

pub mod version;
pub mod metadata;
pub mod install;
pub mod detection;
pub mod checker;
pub mod download;

pub use version::JavaVersion;
pub use metadata::{JavaMetadata, DownloadType};
pub use install::{JavaInstallation, JavaArch};
pub use detection::{detect_java_installations, find_java_for_version, get_required_java_version};
pub use checker::{JavaChecker, JavaCheckResult};
pub use download::{download_java, fetch_adoptium_versions, AvailableJavaVersion, JavaDownloadProgress};
