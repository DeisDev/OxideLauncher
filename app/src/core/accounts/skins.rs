//! Skin and cape management via the Mojang API.
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

use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::core::error::{OxideError, Result};
use super::{CapeInfo, SkinInfo, SkinVariant};

// =============================================================================
// API Endpoints
// =============================================================================

/// Minecraft services base URL
const MC_SERVICES_URL: &str = "https://api.minecraftservices.com";

/// Session server for fetching other players' skins
const SESSION_SERVER_URL: &str = "https://sessionserver.mojang.com";

/// Mojang API for username to UUID lookup
const MOJANG_API_URL: &str = "https://api.mojang.com";

// =============================================================================
// Response Types
// =============================================================================

#[derive(Debug, Deserialize)]
struct ProfileResponse {
    id: String,
    name: String,
    #[serde(default)]
    skins: Vec<SkinResponse>,
    #[serde(default)]
    capes: Vec<CapeResponse>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SkinResponse {
    pub id: String,
    pub state: String,
    pub url: String,
    pub variant: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CapeResponse {
    pub id: String,
    pub state: String,
    pub url: String,
    #[serde(default)]
    pub alias: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UsernameResponse {
    id: String,
    #[allow(dead_code)]
    name: String,
}

#[derive(Debug, Deserialize)]
struct SessionProfileResponse {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    name: String,
    properties: Vec<ProfileProperty>,
}

#[derive(Debug, Deserialize)]
struct ProfileProperty {
    name: String,
    value: String,
}

#[derive(Debug, Deserialize)]
struct TexturesPayload {
    #[serde(rename = "profileId")]
    profile_id: String,
    #[serde(rename = "profileName")]
    profile_name: String,
    textures: TexturesData,
}

#[derive(Debug, Deserialize)]
struct TexturesData {
    #[serde(rename = "SKIN")]
    skin: Option<TextureSkin>,
    #[serde(rename = "CAPE")]
    cape: Option<TextureCape>,
}

#[derive(Debug, Deserialize)]
struct TextureSkin {
    url: String,
    #[serde(default)]
    metadata: Option<SkinMetadata>,
}

#[derive(Debug, Deserialize)]
struct SkinMetadata {
    model: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TextureCape {
    url: String,
}

// =============================================================================
// Public API Types
// =============================================================================

/// Full player profile with skins and capes
#[derive(Debug, Clone, Serialize)]
pub struct PlayerProfile {
    pub id: String,
    pub name: String,
    pub skins: Vec<SkinInfo>,
    pub capes: Vec<CapeInfo>,
    pub active_skin: Option<SkinInfo>,
    pub active_cape: Option<CapeInfo>,
}

/// Result of fetching skin from a username
#[derive(Debug, Clone, Serialize)]
pub struct FetchedSkin {
    pub uuid: String,
    pub username: String,
    pub skin_url: Option<String>,
    pub skin_variant: SkinVariant,
    pub cape_url: Option<String>,
}

// =============================================================================
// Public Functions
// =============================================================================

/// Get the player's full profile including all skins and capes
pub async fn get_player_profile(access_token: &str) -> Result<PlayerProfile> {
    let client = reqwest::Client::new();
    
    let response = client
        .get(format!("{}/minecraft/profile", MC_SERVICES_URL))
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(OxideError::Other(format!(
            "Failed to fetch profile: HTTP {} - {}",
            status, body
        )));
    }
    
    let profile: ProfileResponse = response.json().await?;
    
    // Convert to our types
    let skins: Vec<SkinInfo> = profile.skins.iter().map(|s| SkinInfo {
        id: s.id.clone(),
        url: s.url.clone(),
        variant: match s.variant.to_uppercase().as_str() {
            "SLIM" => SkinVariant::Slim,
            _ => SkinVariant::Classic,
        },
        cached_data: None,
    }).collect();
    
    let capes: Vec<CapeInfo> = profile.capes.iter().map(|c| CapeInfo {
        id: c.id.clone(),
        url: c.url.clone(),
        alias: c.alias.clone(),
        cached_data: None,
    }).collect();
    
    let active_skin = profile.skins.iter()
        .find(|s| s.state.to_uppercase() == "ACTIVE")
        .map(|s| SkinInfo {
            id: s.id.clone(),
            url: s.url.clone(),
            variant: match s.variant.to_uppercase().as_str() {
                "SLIM" => SkinVariant::Slim,
                _ => SkinVariant::Classic,
            },
            cached_data: None,
        });
    
    let active_cape = profile.capes.iter()
        .find(|c| c.state.to_uppercase() == "ACTIVE")
        .map(|c| CapeInfo {
            id: c.id.clone(),
            url: c.url.clone(),
            alias: c.alias.clone(),
            cached_data: None,
        });
    
    Ok(PlayerProfile {
        id: profile.id,
        name: profile.name,
        skins,
        capes,
        active_skin,
        active_cape,
    })
}

/// Change skin using a URL
/// 
/// The URL must point to a valid PNG skin image (64x64 or 64x32)
pub async fn change_skin_url(access_token: &str, url: &str, variant: SkinVariant) -> Result<()> {
    let client = reqwest::Client::new();
    
    let variant_str = match variant {
        SkinVariant::Slim => "slim",
        SkinVariant::Classic => "classic",
    };
    
    let payload = serde_json::json!({
        "variant": variant_str,
        "url": url
    });
    
    let response = client
        .post(format!("{}/minecraft/profile/skins", MC_SERVICES_URL))
        .header("Authorization", format!("Bearer {}", access_token))
        .json(&payload)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(OxideError::Other(format!(
            "Failed to change skin: HTTP {} - {}",
            status, body
        )));
    }
    
    Ok(())
}

/// Upload a skin from file data (PNG image)
/// Uses multipart/form-data with raw image bytes
pub async fn upload_skin(access_token: &str, image_data: &[u8], variant: SkinVariant) -> Result<()> {
    let client = reqwest::Client::new();
    
    let variant_str = match variant {
        SkinVariant::Slim => "slim",
        SkinVariant::Classic => "classic",
    };
    
    // Build the multipart boundary manually since we don't have the multipart feature
    let boundary = "----OxideLauncherBoundary";
    let mut body = Vec::new();
    
    // Add variant field
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"variant\"\r\n\r\n");
    body.extend_from_slice(variant_str.as_bytes());
    body.extend_from_slice(b"\r\n");
    
    // Add file field
    body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
    body.extend_from_slice(b"Content-Disposition: form-data; name=\"file\"; filename=\"skin.png\"\r\n");
    body.extend_from_slice(b"Content-Type: image/png\r\n\r\n");
    body.extend_from_slice(image_data);
    body.extend_from_slice(b"\r\n");
    
    // End boundary
    body.extend_from_slice(format!("--{}--\r\n", boundary).as_bytes());
    
    let response = client
        .post(format!("{}/minecraft/profile/skins", MC_SERVICES_URL))
        .header("Authorization", format!("Bearer {}", access_token))
        .header("Content-Type", format!("multipart/form-data; boundary={}", boundary))
        .body(body)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(OxideError::Other(format!(
            "Failed to upload skin: HTTP {} - {}",
            status, body
        )));
    }
    
    Ok(())
}

/// Reset skin to default (removes custom skin)
pub async fn reset_skin(access_token: &str) -> Result<()> {
    let client = reqwest::Client::new();
    
    let response = client
        .delete(format!("{}/minecraft/profile/skins/active", MC_SERVICES_URL))
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(OxideError::Other(format!(
            "Failed to reset skin: HTTP {} - {}",
            status, body
        )));
    }
    
    Ok(())
}

/// Set the active cape
pub async fn set_cape(access_token: &str, cape_id: &str) -> Result<()> {
    let client = reqwest::Client::new();
    
    let payload = serde_json::json!({
        "capeId": cape_id
    });
    
    let response = client
        .put(format!("{}/minecraft/profile/capes/active", MC_SERVICES_URL))
        .header("Authorization", format!("Bearer {}", access_token))
        .json(&payload)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(OxideError::Other(format!(
            "Failed to set cape: HTTP {} - {}",
            status, body
        )));
    }
    
    Ok(())
}

/// Hide cape (remove active cape)
pub async fn hide_cape(access_token: &str) -> Result<()> {
    let client = reqwest::Client::new();
    
    let response = client
        .delete(format!("{}/minecraft/profile/capes/active", MC_SERVICES_URL))
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(OxideError::Other(format!(
            "Failed to hide cape: HTTP {} - {}",
            status, body
        )));
    }
    
    Ok(())
}

/// Fetch skin information from a Minecraft username (for import)
/// 
/// This does NOT require authentication - it uses the public session server API
pub async fn fetch_skin_from_username(username: &str) -> Result<FetchedSkin> {
    let client = reqwest::Client::new();
    
    // First, get UUID from username
    let uuid_response = client
        .get(format!("{}/users/profiles/minecraft/{}", MOJANG_API_URL, username))
        .send()
        .await?;
    
    if !uuid_response.status().is_success() {
        if uuid_response.status() == 404 {
            return Err(OxideError::Other(format!("Player '{}' not found", username)));
        }
        let status = uuid_response.status();
        let body = uuid_response.text().await.unwrap_or_default();
        return Err(OxideError::Other(format!(
            "Failed to lookup username: HTTP {} - {}",
            status, body
        )));
    }
    
    let user_data: UsernameResponse = uuid_response.json().await?;
    
    // Now get the profile with textures
    let profile_response = client
        .get(format!("{}/session/minecraft/profile/{}", SESSION_SERVER_URL, user_data.id))
        .send()
        .await?;
    
    if !profile_response.status().is_success() {
        let status = profile_response.status();
        let body = profile_response.text().await.unwrap_or_default();
        return Err(OxideError::Other(format!(
            "Failed to fetch profile: HTTP {} - {}",
            status, body
        )));
    }
    
    let session_profile: SessionProfileResponse = profile_response.json().await?;
    
    // Find the textures property and decode it
    let textures_property = session_profile.properties.iter()
        .find(|p| p.name == "textures")
        .ok_or_else(|| OxideError::Other("No textures found for player".to_string()))?;
    
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&textures_property.value)
        .map_err(|e| OxideError::Other(format!("Failed to decode textures: {}", e)))?;
    
    let textures: TexturesPayload = serde_json::from_slice(&decoded)?;
    
    let skin_url = textures.textures.skin.as_ref().map(|s| s.url.clone());
    let skin_variant = textures.textures.skin
        .as_ref()
        .and_then(|s| s.metadata.as_ref())
        .and_then(|m| m.model.as_ref())
        .map(|m| if m == "slim" { SkinVariant::Slim } else { SkinVariant::Classic })
        .unwrap_or(SkinVariant::Classic);
    let cape_url = textures.textures.cape.as_ref().map(|c| c.url.clone());
    
    Ok(FetchedSkin {
        uuid: textures.profile_id,
        username: textures.profile_name,
        skin_url,
        skin_variant,
        cape_url,
    })
}

/// Download a skin image from URL and return as PNG bytes
pub async fn download_skin_image(url: &str) -> Result<Vec<u8>> {
    let client = reqwest::Client::new();
    
    let response = client
        .get(url)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        return Err(OxideError::Other(format!(
            "Failed to download skin: HTTP {}",
            status
        )));
    }
    
    let bytes = response.bytes().await?;
    
    Ok(bytes.to_vec())
}

/// Validate that image data is a valid Minecraft skin (PNG, 64x64 or 64x32)
pub fn validate_skin_image(image_data: &[u8]) -> Result<()> {
    // Check PNG signature
    if image_data.len() < 8 || &image_data[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(OxideError::Other("Not a valid PNG file".to_string()));
    }
    
    // Parse PNG header to get dimensions
    // IHDR chunk starts at byte 8, first 4 bytes are length, next 4 are "IHDR"
    // Then width (4 bytes), height (4 bytes)
    if image_data.len() < 24 {
        return Err(OxideError::Other("PNG file too small".to_string()));
    }
    
    let width = u32::from_be_bytes([image_data[16], image_data[17], image_data[18], image_data[19]]);
    let height = u32::from_be_bytes([image_data[20], image_data[21], image_data[22], image_data[23]]);
    
    // Valid Minecraft skins are 64x64 or 64x32 (legacy)
    if width != 64 || (height != 64 && height != 32) {
        return Err(OxideError::Other(format!(
            "Invalid skin dimensions: {}x{} (must be 64x64 or 64x32)",
            width, height
        )));
    }
    
    Ok(())
}

/// Get the skins folder path for caching downloaded skins
pub fn get_skins_folder(data_dir: &std::path::Path) -> std::path::PathBuf {
    data_dir.join("skins")
}

/// Cache a skin to the skins folder
#[allow(dead_code)] // Reserved for future skin caching feature
pub async fn cache_skin(data_dir: &std::path::Path, uuid: &str, image_data: &[u8]) -> Result<std::path::PathBuf> {
    let skins_folder = get_skins_folder(data_dir);
    tokio::fs::create_dir_all(&skins_folder).await?;
    
    let skin_path = skins_folder.join(format!("{}.png", uuid));
    tokio::fs::write(&skin_path, image_data).await?;
    
    Ok(skin_path)
}
