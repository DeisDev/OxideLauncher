//! Meta server client for fetching version metadata.
//!
//! Oxide Launcher — A Rust-based Minecraft launcher
//! Copyright (C) 2025 Oxide Launcher contributors
//!
//! This file is part of Oxide Launcher.
//!
//! Oxide Launcher is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License, or
//! (at your option) any later version.
//!
//! Oxide Launcher is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License
//! along with this program. If not, see <https://www.gnu.org/licenses/>.

use crate::core::error::{OxideError, Result};
use tracing::{debug, info};

use super::types::{MetaIndex, PackageIndex, VersionEntry, uids};

/// Default meta server URL (will point to self-hosted server when ready).
const DEFAULT_META_URL: &str = "https://meta.oxidelauncher.org";

/// Client for interacting with the PrismLauncher-format meta server.
#[derive(Debug, Clone)]
pub struct MetaClient {
    base_url: String,
    client: reqwest::Client,
}

impl Default for MetaClient {
    fn default() -> Self {
        Self::new(DEFAULT_META_URL)
    }
}

impl MetaClient {
    /// Create a new MetaClient with a custom base URL.
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        }
    }
    
    /// Get the base URL of this client.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
    
    /// Fetch the main index containing all available packages.
    pub async fn fetch_index(&self) -> Result<MetaIndex> {
        let url = format!("{}/index.json", self.base_url);
        debug!("Fetching meta index from: {}", url);
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(OxideError::Download(format!(
                "Meta server returned error {}: {}",
                response.status(),
                url
            )));
        }
        
        let index: MetaIndex = response.json().await
            .map_err(|e| OxideError::Download(format!("Failed to parse meta index: {}", e)))?;
        
        info!("Fetched meta index with {} packages", index.packages.len());
        Ok(index)
    }
    
    /// Fetch the index for a specific package (all versions).
    pub async fn fetch_package_index(&self, uid: &str) -> Result<PackageIndex> {
        let url = format!("{}/{}/index.json", self.base_url, uid);
        debug!("Fetching package index from: {}", url);
        
        let response = self.client.get(&url).send().await?;
        
        if !response.status().is_success() {
            return Err(OxideError::Download(format!(
                "Meta server returned error {} for package {}: {}",
                response.status(),
                uid,
                url
            )));
        }
        
        let index: PackageIndex = response.json().await
            .map_err(|e| OxideError::Download(format!("Failed to parse package {} index: {}", uid, e)))?;
        
        info!("Fetched {} versions for package {}", index.versions.len(), uid);
        Ok(index)
    }
    
    /// Get all Minecraft versions.
    pub async fn get_minecraft_versions(&self) -> Result<Vec<VersionEntry>> {
        let index = self.fetch_package_index(uids::MINECRAFT).await?;
        Ok(index.versions)
    }
    
    /// Get Minecraft versions filtered by type.
    pub async fn get_minecraft_versions_filtered(
        &self,
        show_releases: bool,
        show_snapshots: bool,
        show_old: bool,
    ) -> Result<Vec<VersionEntry>> {
        let versions = self.get_minecraft_versions().await?;
        
        Ok(versions
            .into_iter()
            .filter(|v| {
                let version_type = v.version_type.as_deref().unwrap_or("release");
                match version_type {
                    "release" => show_releases,
                    "snapshot" => show_snapshots,
                    "old_alpha" | "old_beta" => show_old,
                    "experiment" => show_snapshots, // Treat experiments like snapshots
                    _ => show_releases, // Unknown types default to release behavior
                }
            })
            .collect())
    }
    
    /// Get all versions for a specific modloader.
    pub async fn get_loader_versions(&self, loader_uid: &str) -> Result<Vec<VersionEntry>> {
        let index = self.fetch_package_index(loader_uid).await?;
        Ok(index.versions)
    }
    
    /// Get modloader versions compatible with a specific Minecraft version.
    pub async fn get_loader_versions_for_minecraft(
        &self,
        loader_uid: &str,
        minecraft_version: &str,
    ) -> Result<Vec<VersionEntry>> {
        let index = self.fetch_package_index(loader_uid).await?;
        
        let compatible: Vec<VersionEntry> = index
            .versions
            .into_iter()
            .filter(|v| v.is_compatible_with(minecraft_version))
            .collect();
        
        debug!(
            "Found {} {} versions compatible with MC {}",
            compatible.len(),
            loader_uid,
            minecraft_version
        );
        
        Ok(compatible)
    }
    
    // Convenience methods for specific loaders
    
    /// Get Forge versions for a Minecraft version.
    pub async fn get_forge_versions(&self, minecraft_version: &str) -> Result<Vec<VersionEntry>> {
        self.get_loader_versions_for_minecraft(uids::FORGE, minecraft_version).await
    }
    
    /// Get Fabric Loader versions for a Minecraft version.
    pub async fn get_fabric_versions(&self, minecraft_version: &str) -> Result<Vec<VersionEntry>> {
        self.get_loader_versions_for_minecraft(uids::FABRIC_LOADER, minecraft_version).await
    }
    
    /// Get NeoForge versions for a Minecraft version.
    pub async fn get_neoforge_versions(&self, minecraft_version: &str) -> Result<Vec<VersionEntry>> {
        self.get_loader_versions_for_minecraft(uids::NEOFORGE, minecraft_version).await
    }
    
    /// Get Quilt Loader versions for a Minecraft version.
    pub async fn get_quilt_versions(&self, minecraft_version: &str) -> Result<Vec<VersionEntry>> {
        self.get_loader_versions_for_minecraft(uids::QUILT_LOADER, minecraft_version).await
    }
    
    /// Get LiteLoader versions for a Minecraft version.
    pub async fn get_liteloader_versions(&self, minecraft_version: &str) -> Result<Vec<VersionEntry>> {
        self.get_loader_versions_for_minecraft(uids::LITELOADER, minecraft_version).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = MetaClient::default();
        assert_eq!(client.base_url(), "https://meta.oxidelauncher.org");
        
        let custom = MetaClient::new("https://custom.example.com/v1/");
        assert_eq!(custom.base_url(), "https://custom.example.com/v1");
    }
}
