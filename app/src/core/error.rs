//! Error types for the launcher

use thiserror::Error;

/// Main error type for the launcher
#[derive(Error, Debug)]
pub enum OxideError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Download error: {0}")]
    Download(String),

    #[error("Authentication error: {0}")]
    Auth(String),

    #[error("Instance error: {0}")]
    Instance(String),

    #[error("Launch error: {0}")]
    Launch(String),

    #[error("Java error: {0}")]
    Java(String),

    #[error("Version error: {0}")]
    Version(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Mod platform error: {0}")]
    ModPlatform(String),

    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("{0}")]
    Other(String),
}

impl From<String> for OxideError {
    fn from(s: String) -> Self {
        OxideError::Other(s)
    }
}

impl From<&str> for OxideError {
    fn from(s: &str) -> Self {
        OxideError::Other(s.to_string())
    }
}

pub type Result<T> = std::result::Result<T, OxideError>;
