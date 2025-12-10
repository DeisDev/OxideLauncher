//! Microsoft account authentication
//! 
//! Implements the Microsoft OAuth flow for Minecraft authentication.
//! 
//! Flow:
//! 1. User logs in with Microsoft account (OAuth)
//! 2. Exchange MS token for Xbox Live token
//! 3. Exchange XBL token for XSTS token
//! 4. Exchange XSTS token for Minecraft token
//! 5. Get Minecraft profile

use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use crate::core::error::{OxideError, Result};
use crate::core::config::Config;
use super::Account;

/// Microsoft OAuth endpoints
const MS_AUTH_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize";
const MS_TOKEN_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const MS_DEVICE_CODE_URL: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";

/// Xbox Live endpoints
const XBL_AUTH_URL: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_AUTH_URL: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";

/// Minecraft endpoints  
const MC_AUTH_URL: &str = "https://api.minecraftservices.com/authentication/login_with_xbox";
const MC_PROFILE_URL: &str = "https://api.minecraftservices.com/minecraft/profile";
const MC_ENTITLEMENTS_URL: &str = "https://api.minecraftservices.com/entitlements/mcstore";

/// OAuth scopes needed
const OAUTH_SCOPE: &str = "XboxLive.signin offline_access";

/// Login with Microsoft account using device code flow
pub async fn login_microsoft() -> Result<Account> {
    let config = Config::load().unwrap_or_default();
    
    let client_id = config.api_keys.msa_client_id
        .ok_or_else(|| OxideError::Auth("Microsoft Client ID not configured".into()))?;

    let client = reqwest::Client::new();

    // Step 1: Get device code
    tracing::info!("Starting Microsoft device code authentication flow");
    let device_code = get_device_code(&client, &client_id).await?;
    
    tracing::info!("Please open {} and enter code: {}", 
        device_code.verification_uri, 
        device_code.user_code
    );

    // Open browser for user
    let _ = webbrowser::open(&device_code.verification_uri);

    // Step 2: Poll for token
    let ms_token = poll_for_token(&client, &client_id, &device_code).await?;

    // Step 3: Authenticate with Xbox Live
    tracing::info!("Authenticating with Xbox Live");
    let xbl_token = authenticate_xbl(&client, &ms_token.access_token).await?;

    // Step 4: Authenticate with XSTS
    tracing::info!("Authenticating with XSTS");
    let xsts_token = authenticate_xsts(&client, &xbl_token.token).await?;

    // Step 5: Authenticate with Minecraft
    tracing::info!("Authenticating with Minecraft");
    let mc_token = authenticate_minecraft(&client, &xsts_token).await?;

    // Step 6: Check entitlements
    tracing::info!("Checking Minecraft entitlements");
    let owns_game = check_entitlements(&client, &mc_token.access_token).await?;
    
    if !owns_game {
        return Err(OxideError::Auth("This Microsoft account does not own Minecraft".into()));
    }

    // Step 7: Get profile
    tracing::info!("Fetching Minecraft profile");
    let profile = get_minecraft_profile(&client, &mc_token.access_token).await?;

    // Calculate token expiry
    let expires_at = Utc::now() + Duration::seconds(mc_token.expires_in as i64);

    // Create account
    let account = Account::new_microsoft(
        profile.name,
        profile.id,
        mc_token.access_token,
        ms_token.refresh_token.unwrap_or_default(),
        expires_at,
    );

    tracing::info!("Successfully authenticated as {}", account.username);

    Ok(account)
}

/// Refresh an existing Microsoft account token
pub async fn refresh_microsoft_account(account: &mut Account) -> Result<()> {
    let config = Config::load().unwrap_or_default();
    
    let client_id = config.api_keys.msa_client_id
        .ok_or_else(|| OxideError::Auth("Microsoft Client ID not configured".into()))?;

    let refresh_token = account.refresh_token.as_ref()
        .ok_or_else(|| OxideError::Auth("No refresh token available".into()))?;

    let client = reqwest::Client::new();

    // Refresh MS token
    let ms_token = refresh_ms_token(&client, &client_id, refresh_token).await?;

    // Re-authenticate through the chain
    let xbl_token = authenticate_xbl(&client, &ms_token.access_token).await?;
    let xsts_token = authenticate_xsts(&client, &xbl_token.token).await?;
    let mc_token = authenticate_minecraft(&client, &xsts_token).await?;

    // Update account
    account.access_token = Some(mc_token.access_token);
    account.refresh_token = ms_token.refresh_token.or(account.refresh_token.take());
    account.token_expires_at = Some(Utc::now() + Duration::seconds(mc_token.expires_in as i64));

    Ok(())
}

// Response types

#[derive(Debug, Deserialize)]
struct DeviceCodeResponse {
    device_code: String,
    user_code: String,
    verification_uri: String,
    expires_in: u32,
    interval: u32,
}

#[derive(Debug, Deserialize)]
struct MsTokenResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: u32,
    token_type: String,
}

#[derive(Debug, Deserialize)]
struct XblResponse {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: XblDisplayClaims,
}

#[derive(Debug, Deserialize)]
struct XblDisplayClaims {
    xui: Vec<XblXui>,
}

#[derive(Debug, Deserialize)]
struct XblXui {
    uhs: String,
}

#[derive(Debug, Clone)]
struct XstsToken {
    token: String,
    user_hash: String,
}

#[derive(Debug, Deserialize)]
struct McTokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u32,
}

#[derive(Debug, Deserialize)]
struct McProfile {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct McEntitlements {
    items: Vec<McEntitlement>,
}

#[derive(Debug, Deserialize)]
struct McEntitlement {
    name: String,
}

// Authentication steps

async fn get_device_code(client: &reqwest::Client, client_id: &str) -> Result<DeviceCodeResponse> {
    let response = client
        .post(MS_DEVICE_CODE_URL)
        .form(&[
            ("client_id", client_id),
            ("scope", OAUTH_SCOPE),
        ])
        .send()
        .await?
        .json::<DeviceCodeResponse>()
        .await?;

    Ok(response)
}

async fn poll_for_token(
    client: &reqwest::Client,
    client_id: &str,
    device_code: &DeviceCodeResponse,
) -> Result<MsTokenResponse> {
    let interval = std::time::Duration::from_secs(device_code.interval as u64);
    let timeout = std::time::Duration::from_secs(device_code.expires_in as u64);
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() > timeout {
            return Err(OxideError::Auth("Device code expired".into()));
        }

        tokio::time::sleep(interval).await;

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
            return Ok(response.json::<MsTokenResponse>().await?);
        }

        // Check for pending authorization
        let error: serde_json::Value = response.json().await?;
        let error_code = error.get("error").and_then(|e| e.as_str()).unwrap_or("");
        
        match error_code {
            "authorization_pending" => continue,
            "slow_down" => {
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
            "authorization_declined" => {
                return Err(OxideError::Auth("Authorization was declined".into()));
            }
            "expired_token" => {
                return Err(OxideError::Auth("Device code expired".into()));
            }
            _ => {
                return Err(OxideError::Auth(format!("Authentication error: {}", error_code)));
            }
        }
    }
}

async fn refresh_ms_token(
    client: &reqwest::Client,
    client_id: &str,
    refresh_token: &str,
) -> Result<MsTokenResponse> {
    let response = client
        .post(MS_TOKEN_URL)
        .form(&[
            ("client_id", client_id),
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("scope", OAUTH_SCOPE),
        ])
        .send()
        .await?
        .json::<MsTokenResponse>()
        .await?;

    Ok(response)
}

async fn authenticate_xbl(client: &reqwest::Client, ms_token: &str) -> Result<XblResponse> {
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
        .json(&body)
        .send()
        .await?
        .json::<XblResponse>()
        .await?;

    Ok(response)
}

async fn authenticate_xsts(client: &reqwest::Client, xbl_token: &str) -> Result<XstsToken> {
    let body = serde_json::json!({
        "Properties": {
            "SandboxId": "RETAIL",
            "UserTokens": [xbl_token]
        },
        "RelyingParty": "rp://api.minecraftservices.com/",
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
        let error: serde_json::Value = response.json().await?;
        let xerr = error.get("XErr").and_then(|e| e.as_u64()).unwrap_or(0);
        
        let message = match xerr {
            2148916233 => "This Microsoft account does not have an Xbox account",
            2148916235 => "Xbox Live is not available in your country",
            2148916236 | 2148916237 => "Adult verification required on Xbox.com",
            2148916238 => "This account is a child account - add it to a family on Xbox.com",
            _ => "Xbox authentication failed",
        };
        
        return Err(OxideError::Auth(message.into()));
    }

    let xbl_response: XblResponse = response.json().await?;
    let user_hash = xbl_response.display_claims.xui.first()
        .map(|x| x.uhs.clone())
        .ok_or_else(|| OxideError::Auth("No user hash in XSTS response".into()))?;

    Ok(XstsToken {
        token: xbl_response.token,
        user_hash,
    })
}

async fn authenticate_minecraft(client: &reqwest::Client, xsts: &XstsToken) -> Result<McTokenResponse> {
    let body = serde_json::json!({
        "identityToken": format!("XBL3.0 x={};{}", xsts.user_hash, xsts.token)
    });

    let response = client
        .post(MC_AUTH_URL)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json")
        .json(&body)
        .send()
        .await?
        .json::<McTokenResponse>()
        .await?;

    Ok(response)
}

async fn check_entitlements(client: &reqwest::Client, mc_token: &str) -> Result<bool> {
    let response = client
        .get(MC_ENTITLEMENTS_URL)
        .header("Authorization", format!("Bearer {}", mc_token))
        .send()
        .await?
        .json::<McEntitlements>()
        .await?;

    // Check for game_minecraft or product_minecraft
    let owns = response.items.iter().any(|item| {
        item.name == "game_minecraft" || item.name == "product_minecraft"
    });

    Ok(owns)
}

async fn get_minecraft_profile(client: &reqwest::Client, mc_token: &str) -> Result<McProfile> {
    let response = client
        .get(MC_PROFILE_URL)
        .header("Authorization", format!("Bearer {}", mc_token))
        .send()
        .await?;

    if response.status() == 404 {
        return Err(OxideError::Auth("This account does not have a Minecraft profile".into()));
    }

    let profile = response.json::<McProfile>().await?;
    Ok(profile)
}
