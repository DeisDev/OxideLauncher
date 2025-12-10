//! Fabric version API client

use serde::{Deserialize, Serialize};
use crate::core::error::Result;

const FABRIC_META_URL: &str = "https://meta.fabricmc.net/v2/versions/loader";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FabricVersion {
    pub version: String,
    pub stable: bool,
}

#[derive(Debug, Deserialize)]
struct FabricLoaderVersion {
    version: String,
    stable: bool,
}

#[derive(Debug, Deserialize)]
struct FabricVersionResponse {
    loader: FabricLoaderVersion,
}

/// Fetch available Fabric versions for a Minecraft version
pub async fn get_fabric_versions(minecraft_version: &str) -> Result<Vec<FabricVersion>> {
    let client = reqwest::Client::new();
    let url = format!("{}/{}", FABRIC_META_URL, minecraft_version);
    
    let response = client
        .get(&url)
        .send()
        .await?
        .json::<Vec<FabricVersionResponse>>()
        .await?;
    
    let versions: Vec<FabricVersion> = response.into_iter()
        .map(|v| FabricVersion {
            version: v.loader.version,
            stable: v.loader.stable,
        })
        .collect();
    
    Ok(versions)
}

/// Get the latest stable Fabric version for a Minecraft version
pub async fn get_recommended_fabric(minecraft_version: &str) -> Result<Option<String>> {
    let versions = get_fabric_versions(minecraft_version).await?;
    Ok(versions.into_iter().find(|v| v.stable).map(|v| v.version))
}
