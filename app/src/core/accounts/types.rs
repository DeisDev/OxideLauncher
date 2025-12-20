//! Account type definitions and authentication data structures.
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

#![allow(dead_code)] // Types will be used as features are completed

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A generic token with expiry and extra data
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Token {
    /// The token string
    pub token: String,
    
    /// When the token was issued
    #[serde(skip_serializing_if = "Option::is_none")]
    pub issued_at: Option<DateTime<Utc>>,
    
    /// When the token expires
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<DateTime<Utc>>,
    
    /// Extra data (e.g., user hash for Xbox tokens)
    #[serde(default, skip_serializing_if = "std::collections::HashMap::is_empty")]
    pub extra: std::collections::HashMap<String, String>,
}

impl Token {
    pub fn new(token: String) -> Self {
        Self {
            token,
            issued_at: Some(Utc::now()),
            expires_at: None,
            extra: std::collections::HashMap::new(),
        }
    }

    pub fn with_expiry(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn with_extra(mut self, key: &str, value: &str) -> Self {
        self.extra.insert(key.to_string(), value.to_string());
        self
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires) = self.expires_at {
            Utc::now() >= expires
        } else {
            false
        }
    }

    pub fn is_valid(&self) -> bool {
        !self.token.is_empty() && !self.is_expired()
    }
}

/// Minecraft profile information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MinecraftProfile {
    /// Profile UUID (without dashes)
    pub id: String,
    
    /// Player username
    pub name: String,
    
    /// Skin information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skin: Option<SkinInfo>,
    
    /// Cape information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cape: Option<CapeInfo>,
}

/// Skin texture information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinInfo {
    /// Skin ID
    pub id: String,
    
    /// Texture URL
    pub url: String,
    
    /// Skin variant (classic or slim)
    #[serde(default)]
    pub variant: SkinVariant,
    
    /// Cached texture data (base64 encoded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_data: Option<String>,
}

/// Cape texture information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapeInfo {
    /// Cape ID
    pub id: String,
    
    /// Texture URL
    pub url: String,
    
    /// Cape alias/name (e.g., "Migrator", "Vanilla")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alias: Option<String>,
    
    /// Cached texture data (base64 encoded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_data: Option<String>,
}

/// Skin variant type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SkinVariant {
    #[default]
    Classic,
    Slim,
}

/// Minecraft entitlement information
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MinecraftEntitlement {
    /// Whether the account owns Minecraft
    pub owns_minecraft: bool,
    
    /// Whether this is a Game Pass subscription
    pub game_pass: bool,
}

/// Account data containing all tokens and profile info (similar to Prism's AccountData)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccountData {
    /// Microsoft account token (from OAuth)
    #[serde(default)]
    pub msa_token: Token,
    
    /// Xbox Live user token
    #[serde(default)]
    pub user_token: Token,
    
    /// XSTS token for Minecraft services
    #[serde(default)]
    pub xsts_token: Token,
    
    /// Minecraft access token (Yggdrasil-style)
    #[serde(default)]
    pub minecraft_token: Token,
    
    /// Minecraft profile
    #[serde(default)]
    pub minecraft_profile: MinecraftProfile,
    
    /// Minecraft entitlement
    #[serde(default)]
    pub minecraft_entitlement: MinecraftEntitlement,
    
    /// Last error message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
}

impl AccountData {
    /// Check if we can use this account to play
    pub fn is_playable(&self) -> bool {
        self.minecraft_token.is_valid() 
            && !self.minecraft_profile.id.is_empty()
            && self.minecraft_entitlement.owns_minecraft
    }
    
    /// Get the access token for launching
    pub fn access_token(&self) -> &str {
        &self.minecraft_token.token
    }
    
    /// Get the profile UUID
    pub fn profile_id(&self) -> &str {
        &self.minecraft_profile.id
    }
    
    /// Get the profile name
    pub fn profile_name(&self) -> &str {
        &self.minecraft_profile.name
    }
}

/// A Minecraft account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Unique internal ID
    pub id: String,
    
    /// Account type
    pub account_type: AccountType,
    
    /// Minecraft username/profile name (cached from profile)
    pub username: String,
    
    /// Minecraft UUID (cached from profile)
    pub uuid: String,
    
    /// Full account data for Microsoft accounts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<AccountData>,
    
    /// Access token for authentication (legacy field, use data.minecraft_token instead)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    
    /// Refresh token for MSA accounts (legacy field, kept for migration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    
    /// When the access token expires (legacy field)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_expires_at: Option<DateTime<Utc>>,
    
    /// Whether this is the active account
    #[serde(default)]
    pub is_active: bool,
    
    /// Skin data (legacy field, use data.minecraft_profile.skin instead)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skin: Option<SkinData>,
    
    /// When the account was added
    pub added_at: DateTime<Utc>,
    
    /// When the account was last used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<DateTime<Utc>>,
}

impl Account {
    /// Create a new Microsoft account from account data
    pub fn new_microsoft_from_data(data: AccountData) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            account_type: AccountType::Microsoft,
            username: data.minecraft_profile.name.clone(),
            uuid: data.minecraft_profile.id.clone(),
            access_token: Some(data.minecraft_token.token.clone()),
            refresh_token: Some(data.msa_token.extra.get("refresh_token").cloned().unwrap_or_default()),
            token_expires_at: data.minecraft_token.expires_at,
            is_active: false,
            skin: data.minecraft_profile.skin.as_ref().map(|s| SkinData {
                texture_url: Some(s.url.clone()),
                cape_url: data.minecraft_profile.cape.as_ref().map(|c| c.url.clone()),
                model: match s.variant {
                    SkinVariant::Slim => SkinModel::Slim,
                    SkinVariant::Classic => SkinModel::Classic,
                },
                cached_texture: s.cached_data.clone(),
            }),
            data: Some(data),
            added_at: Utc::now(),
            last_used: None,
        }
    }

    /// Create a new Microsoft account (legacy constructor)
    pub fn new_microsoft(
        username: String,
        uuid: String,
        access_token: String,
        refresh_token: String,
        expires_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            account_type: AccountType::Microsoft,
            username,
            uuid,
            access_token: Some(access_token),
            refresh_token: Some(refresh_token),
            token_expires_at: Some(expires_at),
            is_active: false,
            skin: None,
            data: None,
            added_at: Utc::now(),
            last_used: None,
        }
    }

    /// Create a new offline account
    pub fn new_offline(username: String) -> Self {
        // Generate a deterministic UUID for offline accounts
        let uuid = generate_offline_uuid(&username);
        
        Self {
            id: Uuid::new_v4().to_string(),
            account_type: AccountType::Offline,
            username,
            uuid,
            access_token: None,
            refresh_token: None,
            token_expires_at: None,
            is_active: false,
            skin: None,
            data: None,
            added_at: Utc::now(),
            last_used: None,
        }
    }

    /// Check if the access token needs refreshing
    pub fn needs_refresh(&self) -> bool {
        match self.account_type {
            AccountType::Microsoft => {
                // Check the new data field first
                if let Some(ref data) = self.data {
                    if data.minecraft_token.is_expired() {
                        return true;
                    }
                    // Refresh if token expires in less than 5 minutes
                    if let Some(expires) = data.minecraft_token.expires_at {
                        return Utc::now() + chrono::Duration::minutes(5) > expires;
                    }
                }
                // Fall back to legacy field
                if let Some(expires_at) = self.token_expires_at {
                    Utc::now() + chrono::Duration::minutes(5) > expires_at
                } else {
                    true
                }
            }
            AccountType::Offline => false,
        }
    }

    /// Check if this is an online account
    pub fn is_online(&self) -> bool {
        matches!(self.account_type, AccountType::Microsoft)
    }

    /// Get display string
    pub fn display_string(&self) -> String {
        match self.account_type {
            AccountType::Microsoft => format!("{} (Microsoft)", self.username),
            AccountType::Offline => format!("{} (Offline)", self.username),
        }
    }

    /// Check if the account is currently valid/usable
    pub fn is_valid(&self) -> bool {
        match self.account_type {
            AccountType::Offline => true,
            AccountType::Microsoft => {
                // Check new data field first
                if let Some(ref data) = self.data {
                    return data.is_playable();
                }
                // Fall back to legacy fields
                if let Some(expires_at) = self.token_expires_at {
                    Utc::now() < expires_at
                } else {
                    self.access_token.is_some()
                }
            }
        }
    }

    /// Get the access token for launching Minecraft
    pub fn get_access_token(&self) -> String {
        if let Some(ref data) = self.data {
            return data.minecraft_token.token.clone();
        }
        self.access_token.clone().unwrap_or_default()
    }

    /// Get the MSA refresh token
    pub fn get_refresh_token(&self) -> Option<String> {
        if let Some(ref data) = self.data {
            if let Some(rt) = data.msa_token.extra.get("refresh_token") {
                return Some(rt.clone());
            }
        }
        self.refresh_token.clone()
    }

    /// Update account data after refresh
    pub fn update_data(&mut self, data: AccountData) {
        self.username = data.minecraft_profile.name.clone();
        self.uuid = data.minecraft_profile.id.clone();
        self.access_token = Some(data.minecraft_token.token.clone());
        self.token_expires_at = data.minecraft_token.expires_at;
        if let Some(rt) = data.msa_token.extra.get("refresh_token") {
            self.refresh_token = Some(rt.clone());
        }
        self.data = Some(data);
    }

    /// Update last used time
    pub fn update_last_used(&mut self) {
        self.last_used = Some(Utc::now());
    }
}

/// Account type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AccountType {
    Microsoft,
    Offline,
}

impl AccountType {
    pub fn name(&self) -> &'static str {
        match self {
            AccountType::Microsoft => "Microsoft",
            AccountType::Offline => "Offline",
        }
    }
}

/// Skin data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinData {
    /// Skin texture URL
    pub texture_url: Option<String>,
    
    /// Cape texture URL
    pub cape_url: Option<String>,
    
    /// Skin model (classic or slim)
    pub model: SkinModel,
    
    /// Cached skin image data (base64)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_texture: Option<String>,
}

/// Skin model type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default)]
pub enum SkinModel {
    #[default]
    Classic,
    Slim,
}

/// Generate a deterministic UUID for offline accounts
/// This matches how Minecraft generates UUIDs for offline players
fn generate_offline_uuid(username: &str) -> String {
    use sha2::{Sha256, Digest};
    
    let input = format!("OfflinePlayer:{}", username);
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    
    // Create UUID from first 16 bytes
    let mut bytes = [0u8; 16];
    bytes.copy_from_slice(&result[..16]);
    
    // Set version to 3 (name-based) and variant to RFC 4122
    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    
    // Format as UUID string
    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3],
        bytes[4], bytes[5],
        bytes[6], bytes[7],
        bytes[8], bytes[9],
        bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15]
    )
}

/// Authentication session for launching Minecraft
#[derive(Debug, Clone)]
pub struct AuthSession {
    pub username: String,
    pub uuid: String,
    pub access_token: String,
    pub user_type: String,
    /// Xbox User ID (for Microsoft accounts, empty for offline)
    pub xuid: String,
    /// Client ID (usually launcher's MSA client ID for Microsoft accounts)
    pub client_id: String,
}

impl AuthSession {
    pub fn from_account(account: &Account) -> Self {
        // For Microsoft accounts, try to get xuid from account data
        let xuid = account.data.as_ref()
            .and_then(|d| d.xsts_token.extra.get("xuid").cloned())
            .unwrap_or_default();
        
        Self {
            username: account.username.clone(),
            uuid: account.uuid.clone(),
            access_token: account.get_access_token(),
            user_type: match account.account_type {
                AccountType::Microsoft => "msa".to_string(),
                AccountType::Offline => "legacy".to_string(),
            },
            xuid,
            client_id: match account.account_type {
                AccountType::Microsoft => crate::core::accounts::microsoft::MSA_CLIENT_ID.to_string(),
                AccountType::Offline => "".to_string(),
            },
        }
    }

    pub fn offline(username: &str) -> Self {
        let account = Account::new_offline(username.to_string());
        Self::from_account(&account)
    }

    pub fn demo() -> Self {
        Self {
            username: "Demo".to_string(),
            uuid: "00000000-0000-0000-0000-000000000000".to_string(),
            access_token: "".to_string(),
            user_type: "legacy".to_string(),
            xuid: "".to_string(),
            client_id: "".to_string(),
        }
    }
}

/// Device code response for MSA device code flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCodeInfo {
    /// The device code to poll for completion
    pub device_code: String,
    
    /// The user code to display to the user
    pub user_code: String,
    
    /// The URL where the user should enter the code
    pub verification_uri: String,
    
    /// How many seconds until the device code expires
    pub expires_in: u32,
    
    /// How many seconds to wait between polling attempts
    pub interval: u32,
    
    /// When the device code was obtained
    pub obtained_at: DateTime<Utc>,
}

impl DeviceCodeInfo {
    pub fn is_expired(&self) -> bool {
        let elapsed = (Utc::now() - self.obtained_at).num_seconds();
        elapsed >= self.expires_in as i64
    }
}

/// Authentication task state (similar to Prism's AccountTaskState)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthTaskState {
    /// Task created but not started
    Created,
    /// Task is currently running
    Working,
    /// Task completed successfully
    Succeeded,
    /// Task failed but might work if retried (network issues, etc.)
    FailedSoft,
    /// Task failed and won't work if retried (invalid credentials, etc.)
    FailedHard,
    /// Account is gone/deleted on Microsoft side
    FailedGone,
    /// Service is offline
    Offline,
    /// Service is disabled/unavailable
    Disabled,
}

/// Authentication step progress event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum AuthProgressEvent {
    /// Starting a new authentication step
    StepStarted {
        step: String,
        description: String,
    },
    
    /// Device code ready - user needs to authenticate
    DeviceCodeReady {
        user_code: String,
        verification_uri: String,
        expires_in: u32,
    },
    
    /// Polling for device code completion
    PollingForAuth {
        message: String,
    },
    
    /// Authentication step completed
    StepCompleted {
        step: String,
    },
    
    /// Authentication failed
    Failed {
        step: String,
        error: String,
    },
    
    /// All authentication steps completed
    Completed {
        username: String,
    },
}

