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

// Public API re-exports - may not all be used internally but are part of the public module interface
#[allow(unused_imports)]
pub use version::JavaVersion;
#[allow(unused_imports)]
pub use metadata::{JavaMetadata, DownloadType};
#[allow(unused_imports)]
pub use install::{JavaInstallation, JavaArch};
#[allow(unused_imports)] // Functions used through commands module
pub use detection::{detect_java_installations, find_java_for_version, get_required_java_version};
#[allow(unused_imports)]
pub use checker::{JavaChecker, JavaCheckResult};
#[allow(unused_imports)]
pub use download::{download_java, fetch_adoptium_versions, AvailableJavaVersion, JavaDownloadProgress};
