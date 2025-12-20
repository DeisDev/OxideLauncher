//! Microsoft account authentication via OAuth device code flow.
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

use chrono::{Duration, Utc};
use serde::Deserialize;
use tokio::sync::mpsc;

/// Microsoft Azure App Registration Client ID
///
/// IMPORTANT: To use Microsoft authentication, you MUST create your own Azure AD application:
/// 1. Go to https://portal.azure.com
/// 2. Azure Active Directory > App registrations > New registration
/// 3. Name: Your launcher name
/// 4. Supported account types: "Accounts in any organizational directory and personal Microsoft accounts"
/// 5. After creation, go to Authentication > Add platform > Mobile and desktop applications
/// 6. Add redirect URI: http://localhost
/// 7. Under Advanced settings, set "Allow public client flows" to YES
/// 8. Copy the Application (client) ID and replace the value below
///
/// For detailed setup instructions, see AZURE_SETUP.md
///
/// If you fork this project and use any of our custom IDs, API keys, or similar credentials,
/// you agree to the Terms of Service (TOS) of the respective platform (e.g., Microsoft/Azure).
/// Be mindful: abusing these APIs or credentials can lead to service disruption for many users.
/// Always use your own credentials where possible and respect rate limits and fair use policies.
pub const MSA_CLIENT_ID: &str = "1f8c7ebd-3140-4b03-830a-dd0e5ec3218f";

use crate::core::error::{OxideError, Result};
use super::{
    Account, AccountData, AuthProgressEvent, CapeInfo, DeviceCodeInfo, MinecraftEntitlement,
    MinecraftProfile, SkinInfo, SkinVariant, Token,
};

// =============================================================================
// API Endpoints
// =============================================================================

/// Microsoft OAuth endpoints
const MS_DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const MS_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";

/// Xbox Live endpoints
const XBL_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";

/// Minecraft services endpoints
/// /authentication/login_with_xbox is the public endpoint for third-party launchers
/// /launcher/login requires special Microsoft/Mojang approval
const MC_LOGIN_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MC_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";
const MC_ENTITLEMENTS_URL: &str = "https://api.minecraftservices.com/entitlements/license";

/// Xbox profile endpoint (for gamertag, etc.)
#[allow(dead_code)]
const XBOX_PROFILE_URL: &str = "https://profile.xboxlive.com/users/me/profile/settings";

/// OAuth scopes needed for Xbox Live authentication
/// Note: Case matters! Must be "signin" not "SignIn" and "offline_access" not "offline-access"
const OAUTH_SCOPE: &str = "XboxLive.signin offline_access";

/// Relying parties for XSTS authorization
const MINECRAFT_RELYING_PARTY: &str = "rp://api.minecraftservices.com/";
#[allow(dead_code)]
const XBOX_RELYING_PARTY: &str = "http://xboxlive.com";

// =============================================================================
// Response Types
// =============================================================================

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u32,
    interval: u32,
    #[allow(dead_code)]
    #[serde(default)]
    message: String,
}

#[derive(Debug, Deserialize)]
struct MsTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u32,
    #[allow(dead_code)]
    token_type: String,
}

#[derive(Debug, Deserialize)]
struct MsErrorResponse {
    error: String,
    error_description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct XblResponse {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: XblDisplayClaims,
    #[allow(dead_code)]
    #[serde(rename = "IssueInstant")]
    issue_instant: Option<String>,
    #[serde(rename = "NotAfter")]
    not_after: Option<String>,
}

#[derive(Debug, Deserialize)]
struct XblDisplayClaims {
    xui: Vec<XblXui>,
}

#[derive(Debug, Deserialize)]
struct XblXui {
    uhs: String,
}

#[derive(Debug, Deserialize)]
struct XstsErrorResponse {
    #[allow(dead_code)]
    #[serde(rename = "Identity")]
    identity: Option<String>,
    #[serde(rename = "XErr")]
    xerr: Option<u64>,
    #[allow(dead_code)]
    #[serde(rename = "Message")]
    message: Option<String>,
    #[allow(dead_code)]
    #[serde(rename = "Redirect")]
    redirect: Option<String>,
}

#[derive(Debug, Deserialize)]
struct McTokenResponse {
    access_token: String,
    #[allow(dead_code)]
    token_type: String,
    expires_in: u32,
}

#[derive(Debug, Deserialize)]
struct McProfileResponse {
    id: String,
    name: String,
    #[serde(default)]
    skins: Vec<McSkin>,
    #[serde(default)]
    capes: Vec<McCape>,
}

#[derive(Debug, Deserialize)]
struct McSkin {
    id: String,
    url: String,
    variant: String,
    #[serde(default)]
    state: String,
}

#[derive(Debug, Deserialize)]
struct McCape {
    id: String,
    url: String,
    #[serde(default)]
    state: String,
    #[serde(default)]
    alias: Option<String>,
}

#[derive(Debug, Deserialize)]
struct McEntitlementsResponse {
    items: Vec<McEntitlement>,
    #[allow(dead_code)]
    signature: Option<String>,
    #[allow(dead_code)]
    #[serde(rename = "keyId")]
    key_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct McEntitlement {
    name: String,
    #[allow(dead_code)]
    signature: Option<String>,
}

// =============================================================================
// Public API
// =============================================================================

/// Start the Microsoft device code authentication flow
/// Returns the device code info for the user to enter
pub async fn start_device_code_flow(client_id: &str) -> Result<DeviceCodeInfo> {
    let client = reqwest::Client::new();
    
    tracing::info!("Requesting device code from Microsoft");
    
    let response = client
        .post(MS_DEVICE_CODE_URL)
        .form(&[
            ("client_id", client_id),
            ("scope", OAUTH_SCOPE),
        ])
        .send()
        .await?;
    
    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        tracing::error!("Device code request failed: {}", error_text);
        return Err(OxideError::Auth(format!("Failed to get device code: {}", error_text)));
    }
    
    let device_code: DeviceCodeResponse = response.json().await?;
    
    tracing::info!("Got device code, user should visit: {} and enter: {}", 
        device_code.verification_uri, device_code.user_code);
    
    Ok(DeviceCodeInfo {
        device_code: device_code.device_code,
        user_code: device_code.user_code,
        verification_uri: device_code.verification_uri,
        expires_in: device_code.expires_in,
        interval: device_code.interval.max(5), // Minimum 5 seconds
        obtained_at: Utc::now(),
    })
}

/// Poll for the MSA token using the device code
/// This should be called repeatedly until it succeeds or fails
pub async fn poll_device_code(
    client_id: &str,
    device_code: &DeviceCodeInfo,
) -> Result<PollResult> {
    if device_code.is_expired() {
        return Ok(PollResult::Expired);
    }
    
    let client = reqwest::Client::new();
    
    let response = client
        .post(MS_TOKEN_URL)
        .form(&[
            ("client_id", client_id),
            ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ("device_code", &device_code.device_code),
        ])
        .send()
        .await?;
    
    if response.status().is_success() {
        let token: MsTokenResponse = response.json().await?;
        return Ok(PollResult::Success(MsaToken {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            expires_in: token.expires_in,
        }));
    }
    
    // Parse error response
    let error: MsErrorResponse = response.json().await?;
    
    match error.error.as_str() {
        "authorization_pending" => Ok(PollResult::Pending),
        "slow_down" => Ok(PollResult::SlowDown),
        "authorization_declined" => Ok(PollResult::Declined),
        "expired_token" => Ok(PollResult::Expired),
        "bad_verification_code" => {
            Err(OxideError::Auth("Invalid device code".into()))
        }
        _ => {
            let desc = error.error_description.unwrap_or_else(|| error.error.clone());
            Err(OxideError::Auth(format!("Authentication error: {}", desc)))
        }
    }
}

/// Result of polling for device code
#[derive(Debug)]
pub enum PollResult {
    /// Authentication succeeded
    Success(MsaToken),
    /// Still waiting for user to authenticate
    Pending,
    /// Need to slow down polling
    SlowDown,
    /// User declined authentication
    Declined,
    /// Device code expired
    Expired,
}

/// MSA token from successful authentication
#[derive(Debug, Clone)]
pub struct MsaToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: u32,
}

/// Complete the authentication flow after getting MSA token
pub async fn complete_authentication(
    msa_token: MsaToken,
    progress_tx: Option<mpsc::Sender<AuthProgressEvent>>,
) -> Result<AccountData> {
    let client = reqwest::Client::new();
    let mut account_data = AccountData::default();
    
    // Store MSA token
    let msa_expires = Utc::now() + Duration::seconds(msa_token.expires_in as i64);
    account_data.msa_token = Token::new(msa_token.access_token.clone())
        .with_expiry(msa_expires);
    if let Some(ref rt) = msa_token.refresh_token {
        account_data.msa_token.extra.insert("refresh_token".to_string(), rt.clone());
    }
    
    // Step 1: Xbox Live User Authentication
    send_progress(&progress_tx, AuthProgressEvent::StepStarted {
        step: "xbox_user".to_string(),
        description: "Logging in as Xbox user...".to_string(),
    }).await;
    
    let xbl_token = authenticate_xbox_user(&client, &msa_token.access_token).await?;
    account_data.user_token = xbl_token.clone();
    
    send_progress(&progress_tx, AuthProgressEvent::StepCompleted {
        step: "xbox_user".to_string(),
    }).await;
    
    // Step 2: XSTS Authorization for Minecraft
    send_progress(&progress_tx, AuthProgressEvent::StepStarted {
        step: "xsts".to_string(),
        description: "Getting authorization for Minecraft services...".to_string(),
    }).await;
    
    let xsts_token = authenticate_xsts(&client, &xbl_token, MINECRAFT_RELYING_PARTY).await?;
    account_data.xsts_token = xsts_token.clone();
    
    send_progress(&progress_tx, AuthProgressEvent::StepCompleted {
        step: "xsts".to_string(),
    }).await;
    
    // Step 3: Minecraft Launcher Login
    send_progress(&progress_tx, AuthProgressEvent::StepStarted {
        step: "minecraft_login".to_string(),
        description: "Getting Minecraft access token...".to_string(),
    }).await;
    
    let mc_token = authenticate_minecraft(&client, &xsts_token).await?;
    account_data.minecraft_token = mc_token;
    
    send_progress(&progress_tx, AuthProgressEvent::StepCompleted {
        step: "minecraft_login".to_string(),
    }).await;
    
    // Step 4: Check Entitlements
    send_progress(&progress_tx, AuthProgressEvent::StepStarted {
        step: "entitlements".to_string(),
        description: "Checking game ownership...".to_string(),
    }).await;
    
    let entitlement = check_entitlements(&client, &account_data.minecraft_token.token).await?;
    account_data.minecraft_entitlement = entitlement;
    
    if !account_data.minecraft_entitlement.owns_minecraft {
        return Err(OxideError::Auth(
            "This Microsoft account does not own Minecraft Java Edition. \
             If you believe you own the game, please visit minecraft.net to verify your purchase. \
             Note: Minecraft Bedrock Edition (Windows 10/11) is a separate game from Java Edition.".into()
        ));
    }
    
    send_progress(&progress_tx, AuthProgressEvent::StepCompleted {
        step: "entitlements".to_string(),
    }).await;
    
    // Step 5: Get Minecraft Profile
    send_progress(&progress_tx, AuthProgressEvent::StepStarted {
        step: "profile".to_string(),
        description: "Fetching Minecraft profile...".to_string(),
    }).await;
    
    let profile = get_minecraft_profile(&client, &account_data.minecraft_token.token).await?;
    account_data.minecraft_profile = profile;
    
    send_progress(&progress_tx, AuthProgressEvent::StepCompleted {
        step: "profile".to_string(),
    }).await;
    
    send_progress(&progress_tx, AuthProgressEvent::Completed {
        username: account_data.minecraft_profile.name.clone(),
    }).await;
    
    Ok(account_data)
}

/// Refresh an existing Microsoft account
pub async fn refresh_microsoft_account(
    account: &Account,
    progress_tx: Option<mpsc::Sender<AuthProgressEvent>>,
) -> Result<AccountData> {
    let refresh_token = account.get_refresh_token()
        .ok_or_else(|| OxideError::Auth("No refresh token available".into()))?;
    
    send_progress(&progress_tx, AuthProgressEvent::StepStarted {
        step: "refresh_msa".to_string(),
        description: "Refreshing Microsoft token...".to_string(),
    }).await;
    
    let client = reqwest::Client::new();
    
    // Refresh the MSA token
    let response = client
        .post(MS_TOKEN_URL)
        .form(&[
            ("client_id", MSA_CLIENT_ID),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token.as_str()),
            ("scope", OAUTH_SCOPE),
        ])
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        tracing::error!("Token refresh failed ({}): {}", status, error_text);
        
        let message = match status.as_u16() {
            400 => "Failed to refresh token: The refresh token is invalid or has expired. Please log in again.",
            401 => "Failed to refresh token: Authorization failed. Please log in again.",
            429 => "Failed to refresh token: Too many requests. Please wait and try again.",
            500..=599 => "Failed to refresh token: Microsoft servers are experiencing issues. Please try again later.",
            _ => "Failed to refresh token: Could not renew your session. Please log in again.",
        };
        return Err(OxideError::Auth(message.into()));
    }
    
    let ms_token: MsTokenResponse = response.json().await?;
    
    send_progress(&progress_tx, AuthProgressEvent::StepCompleted {
        step: "refresh_msa".to_string(),
    }).await;
    
    // Complete the rest of the authentication flow
    let msa_token = MsaToken {
        access_token: ms_token.access_token,
        refresh_token: ms_token.refresh_token.or(Some(refresh_token)),
        expires_in: ms_token.expires_in,
    };
    
    complete_authentication(msa_token, progress_tx).await
}

/// Login with Microsoft account using device code flow (legacy wrapper)
#[allow(dead_code)]
pub async fn login_microsoft() -> Result<Account> {
    // Start device code flow
    let device_code = start_device_code_flow(MSA_CLIENT_ID).await?;
    
    tracing::info!("Please open {} and enter code: {}", 
        device_code.verification_uri, 
        device_code.user_code
    );
    
    // Open browser for user
    let _ = webbrowser::open(&device_code.verification_uri);
    
    // Poll for token
    let interval = std::time::Duration::from_secs(device_code.interval as u64);
    
    let msa_token = loop {
        tokio::time::sleep(interval).await;
        
        match poll_device_code(MSA_CLIENT_ID, &device_code).await? {
            PollResult::Success(token) => break token,
            PollResult::Pending => continue,
            PollResult::SlowDown => {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
            PollResult::Declined => {
                return Err(OxideError::Auth("Authentication was declined by the user".into()));
            }
            PollResult::Expired => {
                return Err(OxideError::Auth("Device code expired. Please try again.".into()));
            }
        }
    };
    
    // Complete authentication
    let account_data = complete_authentication(msa_token, None).await?;
    
    // Create account
    let account = Account::new_microsoft_from_data(account_data);
    
    tracing::info!("Successfully authenticated as {}", account.username);
    
    Ok(account)
}

// =============================================================================
// Internal Authentication Steps
// =============================================================================

/// Helper to send progress events
async fn send_progress(tx: &Option<mpsc::Sender<AuthProgressEvent>>, event: AuthProgressEvent) {
    if let Some(ref tx) = tx {
        let _ = tx.send(event).await;
    }
}

/// Authenticate with Xbox Live user endpoint
async fn authenticate_xbox_user(client: &reqwest::Client, ms_token: &str) -> Result<Token> {
    let body = serde_json::json!({
        "Properties": {
            "AuthMethod": "RPS",
            "SiteName": "user.auth.xboxlive.com",
            "RpsTicket": format!("d={}", ms_token)
        },
        "RelyingParty": "http://auth.xboxlive.com",
        "TokenType": "JWT"
    });
    
    let response = client
        .post(XBL_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .header("x-xbl-contract-version", "1")
        .json(&body)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        tracing::error!("Xbox user auth failed ({}): {}", status, error_text);
        
        let message = match status.as_u16() {
            401 => "Xbox Live authentication failed: Invalid or expired Microsoft token. Please try logging in again.",
            403 => "Xbox Live authentication failed: Access forbidden. Your Microsoft account may have restrictions.",
            429 => "Xbox Live authentication failed: Too many requests. Please wait a moment and try again.",
            500..=599 => "Xbox Live authentication failed: Xbox servers are experiencing issues. Please try again later.",
            _ => "Xbox Live authentication failed: Could not authenticate with Xbox Live services.",
        };
        return Err(OxideError::Auth(message.into()));
    }
    
    let xbl: XblResponse = response.json().await?;
    
    let user_hash = xbl.display_claims.xui.first()
        .map(|x| x.uhs.clone())
        .ok_or_else(|| OxideError::Auth("No user hash in Xbox response".into()))?;
    
    let mut token = Token::new(xbl.token);
    token.extra.insert("uhs".to_string(), user_hash);
    
    // Parse expiry time if present
    if let Some(ref not_after) = xbl.not_after {
        if let Ok(expires) = chrono::DateTime::parse_from_rfc3339(not_after) {
            token.expires_at = Some(expires.with_timezone(&Utc));
        }
    }
    
    Ok(token)
}

/// Authenticate with XSTS for a specific relying party
async fn authenticate_xsts(
    client: &reqwest::Client,
    user_token: &Token,
    relying_party: &str,
) -> Result<Token> {
    let body = serde_json::json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [user_token.token]
        },
        "RelyingParty": relying_party,
        "TokenType": "JWT"
    });
    
    let response = client
        .post(XSTS_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await?;
    
    if !response.status().is_success() {
        // Try to parse XSTS error
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        if let Ok(error) = serde_json::from_str::<XstsErrorResponse>(&error_text) {
            let message = match error.xerr {
                Some(2148916233) => "Xbox authorization failed: This Microsoft account does not have an Xbox account. Please create one at xbox.com/xbox-live",
                Some(2148916234) => "Xbox authorization failed: This account is banned from Xbox Live services.",
                Some(2148916235) => "Xbox authorization failed: Xbox Live is not available in your country/region. You may need a VPN or a different account.",
                Some(2148916236) => "Xbox authorization failed: This account requires adult verification. Please sign in at account.xbox.com and complete verification.",
                Some(2148916237) => "Xbox authorization failed: This account requires adult verification (parental consent). Please visit account.xbox.com",
                Some(2148916238) => "Xbox authorization failed: This is a child account and must be added to a Family group. Please visit account.xbox.com/family",
                Some(2148916239) => "Xbox authorization failed: Microsoft account sign-in required. Please complete sign-in at account.microsoft.com",
                _ => "Xbox authorization failed: Could not get authorization for Minecraft services.",
            };
            return Err(OxideError::Auth(message.into()));
        }
        
        let message = match status.as_u16() {
            401 => "Xbox authorization failed: Your Xbox session has expired. Please try logging in again.",
            403 => "Xbox authorization failed: Access to Minecraft services was denied.",
            429 => "Xbox authorization failed: Rate limited. Please wait a moment and try again.",
            500..=599 => "Xbox authorization failed: Xbox servers are experiencing issues. Please try again later.",
            _ => &format!("Xbox authorization failed: Server returned error {}", status),
        };
        return Err(OxideError::Auth(message.to_string()));
    }
    
    let xbl: XblResponse = response.json().await?;
    
    let user_hash = xbl.display_claims.xui.first()
        .map(|x| x.uhs.clone())
        .ok_or_else(|| OxideError::Auth("No user hash in XSTS response".into()))?;
    
    // Verify user hash matches
    if let Some(expected_uhs) = user_token.extra.get("uhs") {
        if &user_hash != expected_uhs {
            tracing::warn!("XSTS user hash doesn't match user token hash");
        }
    }
    
    let mut token = Token::new(xbl.token);
    token.extra.insert("uhs".to_string(), user_hash);
    
    if let Some(ref not_after) = xbl.not_after {
        if let Ok(expires) = chrono::DateTime::parse_from_rfc3339(not_after) {
            token.expires_at = Some(expires.with_timezone(&Utc));
        }
    }
    
    Ok(token)
}

/// Authenticate with Minecraft services
async fn authenticate_minecraft(client: &reqwest::Client, xsts_token: &Token) -> Result<Token> {
    let user_hash = xsts_token.extra.get("uhs")
        .ok_or_else(|| OxideError::Auth("Missing user hash for Minecraft auth".into()))?;
    
    // Use the public authentication endpoint (login_with_xbox)
    // This is the correct endpoint for third-party launchers
    let body = serde_json::json!({
        "identityToken": format!("XBL3.0 x={};{}", user_hash, xsts_token.token)
    });
    
    tracing::debug!("Authenticating with Minecraft services...");
    
    let response = client
        .post(MC_LOGIN_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await?;
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        tracing::error!("Minecraft login failed ({}): {}", status, error_text);
        
        let message = match status.as_u16() {
            401 => "Minecraft authentication failed: Invalid Xbox credentials. Please try logging in again.",
            403 => "Minecraft authentication failed: Access denied. Your account may not have permission to access Minecraft services.",
            404 => "Minecraft authentication failed: Minecraft services endpoint not found. This may be a temporary issue.",
            429 => "Minecraft authentication failed: Too many login attempts. Please wait a few minutes and try again.",
            500..=599 => "Minecraft authentication failed: Minecraft servers are experiencing issues. Please try again later.",
            _ => {
                if error_text.contains("NOT_FOUND") {
                    "Minecraft authentication failed: Could not find your Minecraft account. Please ensure you own Minecraft Java Edition."
                } else {
                    "Minecraft authentication failed: Could not obtain Minecraft access token."
                }
            }
        };
        return Err(OxideError::Auth(message.into()));
    }
    
    let mc: McTokenResponse = response.json().await?;
    
    tracing::info!("Successfully obtained Minecraft access token");
    
    let expires = Utc::now() + Duration::seconds(mc.expires_in as i64);
    let token = Token::new(mc.access_token).with_expiry(expires);
    
    Ok(token)
}

/// Check Minecraft entitlements
async fn check_entitlements(client: &reqwest::Client, mc_token: &str) -> Result<MinecraftEntitlement> {
    // Generate request ID (like Prism does)
    let request_id = uuid::Uuid::new_v4().to_string();
    
    let url = format!("{}?requestId={}", MC_ENTITLEMENTS_URL, request_id);
    
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", mc_token))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .send()
        .await?;
    
    if !response.status().is_success() {
        tracing::warn!("Entitlements check failed, assuming ownership");
        // If entitlements check fails, we'll verify ownership through profile
        return Ok(MinecraftEntitlement {
            owns_minecraft: true,
            game_pass: false,
        });
    }
    
    let entitlements: McEntitlementsResponse = response.json().await?;
    
    let owns_minecraft = entitlements.items.iter().any(|item| {
        item.name == "game_minecraft" 
            || item.name == "product_minecraft"
            || item.name == "game_minecraft_bedrock" // Some accounts have this
    });
    
    let game_pass = entitlements.items.iter().any(|item| {
        item.name.contains("game_pass")
    });
    
    Ok(MinecraftEntitlement {
        owns_minecraft,
        game_pass,
    })
}

/// Get Minecraft profile (UUID, username, skin, cape)
async fn get_minecraft_profile(client: &reqwest::Client, mc_token: &str) -> Result<MinecraftProfile> {
    let response = client
        .get(MC_PROFILE_URL)
        .header("Authorization", format!("Bearer {}", mc_token))
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .send()
        .await?;
    
    if response.status() == 404 {
        return Err(OxideError::Auth(
            "Minecraft profile not found: This Microsoft account does not have a Minecraft profile. \
             Please ensure you own Minecraft Java Edition and have set up a profile at minecraft.net. \
             If you recently purchased the game, it may take a few minutes for your profile to be created.".into()
        ));
    }
    
    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        tracing::error!("Profile fetch failed ({}): {}", status, error_text);
        
        let message = match status.as_u16() {
            401 => "Failed to fetch Minecraft profile: Your session has expired. Please log in again.",
            403 => "Failed to fetch Minecraft profile: Access denied to profile services.",
            429 => "Failed to fetch Minecraft profile: Rate limited. Please wait and try again.",
            500..=599 => "Failed to fetch Minecraft profile: Minecraft servers are experiencing issues.",
            _ => "Failed to fetch Minecraft profile: Could not retrieve your profile information.",
        };
        return Err(OxideError::Auth(message.into()));
    }
    
    let profile: McProfileResponse = response.json().await?;
    
    // Find active skin
    let skin = profile.skins.into_iter()
        .find(|s| s.state == "ACTIVE")
        .map(|s| SkinInfo {
            id: s.id,
            url: s.url,
            variant: if s.variant.to_uppercase() == "SLIM" {
                SkinVariant::Slim
            } else {
                SkinVariant::Classic
            },
            cached_data: None,
        });
    
    // Find active cape
    let cape = profile.capes.into_iter()
        .find(|c| c.state == "ACTIVE")
        .map(|c| CapeInfo {
            id: c.id,
            url: c.url,
            alias: c.alias,
            cached_data: None,
        });
    
    Ok(MinecraftProfile {
        id: profile.id,
        name: profile.name,
        skin,
        cape,
    })
}

/// Download skin texture data
#[allow(dead_code)]
pub async fn download_skin(skin_url: &str) -> Result<Vec<u8>> {
    let client = reqwest::Client::new();
    let response = client.get(skin_url).send().await?;
    
    if !response.status().is_success() {
        return Err(OxideError::Download("Failed to download skin".into()));
    }
    
    Ok(response.bytes().await?.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_expiry() {
        let token = Token::new("test".to_string());
        assert!(!token.is_expired());
        
        let expired_token = Token::new("test".to_string())
            .with_expiry(Utc::now() - Duration::hours(1));
        assert!(expired_token.is_expired());
    }
}
