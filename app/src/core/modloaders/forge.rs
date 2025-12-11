//! Forge version API client

#![allow(dead_code)] // Recommended version will be used as features are completed

use serde::{Deserialize, Serialize};
use crate::core::error::Result;

const FORGE_MAVEN_METADATA: &str = "https://files.minecraftforge.net/net/minecraftforge/forge/maven-metadata.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeVersion {
    pub version: String,
    pub minecraft_version: String,
    pub recommended: bool,
    pub latest: bool,
}

#[derive(Debug, Deserialize)]
struct ForgeMavenMetadata {
    #[serde(flatten)]
    versions: std::collections::HashMap<String, Vec<String>>,
}

/// Fetch available Forge versions for a Minecraft version
pub async fn get_forge_versions(minecraft_version: &str) -> Result<Vec<ForgeVersion>> {
    let client = reqwest::Client::new();
    let response = client
        .get(FORGE_MAVEN_METADATA)
        .send()
        .await?
        .json::<ForgeMavenMetadata>()
        .await?;
    
    let mut versions = Vec::new();
    
    if let Some(mc_versions) = response.versions.get(minecraft_version) {
        for (idx, forge_ver) in mc_versions.iter().enumerate() {
            versions.push(ForgeVersion {
                version: forge_ver.clone(),
                minecraft_version: minecraft_version.to_string(),
                recommended: idx == 0, // First is usually recommended
                latest: idx == 0,
            });
        }
    }
    
    Ok(versions)
}

/// Get the recommended Forge version for a Minecraft version
pub async fn get_recommended_forge(minecraft_version: &str) -> Result<Option<String>> {
    let versions = get_forge_versions(minecraft_version).await?;
    Ok(versions.into_iter().find(|v| v.recommended).map(|v| v.version))
}
