//! Core backend functionality
//! 
//! This module contains all the business logic and data structures
//! for the launcher, separate from the UI.

pub mod config;
pub mod instance;
pub mod accounts;
pub mod minecraft;
pub mod modplatform;
pub mod modloaders;
pub mod download;
pub mod launch;
pub mod java;
pub mod error;

// Re-export commonly used Java types
pub use java::{
    JavaVersion, JavaInstallation, JavaArch, JavaMetadata, DownloadType,
    detect_java_installations, find_java_for_version,
};
