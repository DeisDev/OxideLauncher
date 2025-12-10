//! Quilt version API client

use serde::{Deserialize, Serialize};
use crate::core::error::Result;

const QUILT_META_URL: &str = "https://meta.quiltmc.org/v3/versions/loader";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuiltVersion {
    pub version: String,
}

#[derive(Debug, Deserialize)]
struct QuiltLoaderVersion {
    version: String,
}

#[derive(Debug, Deserialize)]
struct QuiltVersionResponse {
    loader: QuiltLoaderVersion,
}

/// Fetch available Quilt versions for a Minecraft version
pub async fn get_quilt_versions(minecraft_version: &str) -> Result<Vec<QuiltVersion>> {
    let client = reqwest::Client::new();
    let url = format!("{}/{}", QUILT_META_URL, minecraft_version);
    
    let response = client
        .get(&url)
        .send()
        .await?
        .json::<Vec<QuiltVersionResponse>>()
        .await?;
    
    let versions: Vec<QuiltVersion> = response.into_iter()
        .map(|v| QuiltVersion {
            version: v.loader.version,
        })
        .collect();
    
    Ok(versions)
}

/// Get the latest Quilt version for a Minecraft version
pub async fn get_recommended_quilt(minecraft_version: &str) -> Result<Option<String>> {
    let versions = get_quilt_versions(minecraft_version).await?;
    Ok(versions.into_iter().next().map(|v| v.version))
}
