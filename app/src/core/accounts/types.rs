//! Account data types

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// A Minecraft account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    /// Unique internal ID
    pub id: String,
    
    /// Account type
    pub account_type: AccountType,
    
    /// Minecraft username/profile name
    pub username: String,
    
    /// Minecraft UUID
    pub uuid: String,
    
    /// Access token for authentication
    #[serde(skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
    
    /// Refresh token for MSA accounts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    
    /// When the access token expires
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_expires_at: Option<DateTime<Utc>>,
    
    /// Whether this is the active account
    #[serde(default)]
    pub is_active: bool,
    
    /// Skin data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skin: Option<SkinData>,
    
    /// When the account was added
    pub added_at: DateTime<Utc>,
    
    /// When the account was last used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_used: Option<DateTime<Utc>>,
}

impl Account {
    /// Create a new Microsoft account
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
            added_at: Utc::now(),
            last_used: None,
        }
    }

    /// Check if the access token needs refreshing
    pub fn needs_refresh(&self) -> bool {
        match self.account_type {
            AccountType::Microsoft => {
                if let Some(expires_at) = self.token_expires_at {
                    // Refresh if token expires in less than 5 minutes
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
                if let Some(expires_at) = self.token_expires_at {
                    Utc::now() < expires_at
                } else {
                    self.access_token.is_some()
                }
            }
        }
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
}

impl AuthSession {
    pub fn from_account(account: &Account) -> Self {
        Self {
            username: account.username.clone(),
            uuid: account.uuid.clone(),
            access_token: account.access_token.clone().unwrap_or_default(),
            user_type: match account.account_type {
                AccountType::Microsoft => "msa".to_string(),
                AccountType::Offline => "legacy".to_string(),
            },
        }
    }

    pub fn offline(username: &str) -> Self {
        let account = Account::new_offline(username.to_string());
        Self::from_account(&account)
    }
}
