//! LiteLoader version API client

use serde::{Deserialize, Serialize};
use crate::core::error::Result;

const LITELOADER_VERSIONS_URL: &str = "http://dl.liteloader.com/versions/versions.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteLoaderVersion {
    pub version: String,
    pub minecraft_version: String,
}

#[derive(Debug, Deserialize)]
struct LiteLoaderVersionsResponse {
    versions: std::collections::HashMap<String, LiteLoaderMcVersions>,
}

#[derive(Debug, Deserialize)]
struct LiteLoaderMcVersions {
    snapshots: Option<std::collections::HashMap<String, LiteLoaderVersionInfo>>,
    releases: Option<std::collections::HashMap<String, LiteLoaderVersionInfo>>,
}

#[derive(Debug, Deserialize)]
struct LiteLoaderVersionInfo {
    version: String,
}

/// Fetch available LiteLoader versions for a Minecraft version
pub async fn get_liteloader_versions(minecraft_version: &str) -> Result<Vec<LiteLoaderVersion>> {
    let client = reqwest::Client::new();
    let response = client
        .get(LITELOADER_VERSIONS_URL)
        .send()
        .await?
        .json::<LiteLoaderVersionsResponse>()
        .await?;
    
    let mut versions = Vec::new();
    
    if let Some(mc_versions) = response.versions.get(minecraft_version) {
        // Add releases
        if let Some(releases) = &mc_versions.releases {
            for info in releases.values() {
                versions.push(LiteLoaderVersion {
                    version: info.version.clone(),
                    minecraft_version: minecraft_version.to_string(),
                });
            }
        }
        
        // Add snapshots
        if let Some(snapshots) = &mc_versions.snapshots {
            for info in snapshots.values() {
                versions.push(LiteLoaderVersion {
                    version: info.version.clone(),
                    minecraft_version: minecraft_version.to_string(),
                });
            }
        }
    }
    
    Ok(versions)
}

/// Get the latest LiteLoader version for a Minecraft version
pub async fn get_recommended_liteloader(minecraft_version: &str) -> Result<Option<String>> {
    let versions = get_liteloader_versions(minecraft_version).await?;
    Ok(versions.into_iter().next().map(|v| v.version))
}
