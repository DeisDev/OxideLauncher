//! NeoForge version API client

use serde::{Deserialize, Serialize};
use crate::core::error::Result;

const NEOFORGE_MAVEN_URL: &str = "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/neoforge";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeoForgeVersion {
    pub version: String,
    pub minecraft_version: String,
    pub recommended: bool,
}

#[derive(Debug, Deserialize)]
struct NeoForgeMavenResponse {
    versions: Vec<String>,
}

/// Fetch available NeoForge versions for a Minecraft version
pub async fn get_neoforge_versions(minecraft_version: &str) -> Result<Vec<NeoForgeVersion>> {
    let client = reqwest::Client::new();
    let response = client
        .get(NEOFORGE_MAVEN_URL)
        .send()
        .await?
        .json::<NeoForgeMavenResponse>()
        .await?;
    
    // NeoForge versions are formatted like "20.2.88" (for MC 1.20.2)
    // Filter by MC version prefix
    let mc_ver_prefix = minecraft_version.replace("1.", "");
    let mut versions: Vec<NeoForgeVersion> = response.versions
        .into_iter()
        .filter(|v| v.starts_with(&mc_ver_prefix))
        .enumerate()
        .map(|(idx, version)| NeoForgeVersion {
            version: version.clone(),
            minecraft_version: minecraft_version.to_string(),
            recommended: idx == 0, // First matching version is latest/recommended
        })
        .collect();
    
    versions.reverse(); // Latest first
    Ok(versions)
}

/// Get the recommended NeoForge version for a Minecraft version
pub async fn get_recommended_neoforge(minecraft_version: &str) -> Result<Option<String>> {
    let versions = get_neoforge_versions(minecraft_version).await?;
    Ok(versions.into_iter().find(|v| v.recommended).map(|v| v.version))
}
