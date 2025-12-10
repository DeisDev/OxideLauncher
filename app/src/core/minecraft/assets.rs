//! Minecraft assets management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use crate::core::error::Result;

/// Asset index containing all game assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndexData {
    pub objects: HashMap<String, AssetObject>,
    #[serde(default)]
    pub map_to_resources: bool,
    #[serde(default)]
    pub r#virtual: bool,
}

/// Individual asset object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: u64,
}

impl AssetObject {
    /// Get the path where this asset should be stored
    pub fn get_path(&self) -> String {
        format!("{}/{}", &self.hash[..2], &self.hash)
    }

    /// Get the download URL for this asset
    pub fn get_url(&self) -> String {
        format!(
            "https://resources.download.minecraft.net/{}/{}",
            &self.hash[..2],
            &self.hash
        )
    }
}

/// Fetch and parse an asset index
pub async fn fetch_asset_index(url: &str) -> Result<AssetIndexData> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .send()
        .await?
        .json::<AssetIndexData>()
        .await?;
    
    Ok(response)
}

/// Get the list of assets that need to be downloaded
pub fn get_missing_assets<'a>(
    index: &'a AssetIndexData,
    assets_dir: &'a PathBuf,
) -> Vec<(&'a str, &'a AssetObject)> {
    let objects_dir = assets_dir.join("objects");
    
    index.objects
        .iter()
        .filter(|(_, asset)| {
            let asset_path = objects_dir.join(asset.get_path());
            !asset_path.exists()
        })
        .map(|(name, asset)| (name.as_str(), asset))
        .collect()
}
