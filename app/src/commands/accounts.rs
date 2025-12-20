//! Account management Tauri commands.
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

use super::state::AppState;
use crate::core::accounts::{
    complete_authentication, create_offline_account, poll_device_code, refresh_microsoft_account,
    start_device_code_flow, validate_offline_username, Account, AccountList, AuthProgressEvent,
    DeviceCodeInfo, PollResult, MSA_CLIENT_ID, skins, SkinVariant,
};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::mpsc;

/// Serializable account information for the frontend
#[derive(Debug, Clone, Serialize)]
pub struct AccountInfo {
    pub id: String,
    pub username: String,
    pub uuid: String,
    pub account_type: String,
    pub is_active: bool,
    pub is_valid: bool,
    pub needs_refresh: bool,
    pub skin_url: Option<String>,
    pub added_at: String,
    pub last_used: Option<String>,
}

impl From<&Account> for AccountInfo {
    fn from(account: &Account) -> Self {
        let skin_url = account.data.as_ref()
            .and_then(|d| d.minecraft_profile.skin.as_ref())
            .map(|s| s.url.clone())
            .or_else(|| account.skin.as_ref().and_then(|s| s.texture_url.clone()));
        
        Self {
            id: account.id.clone(),
            username: account.username.clone(),
            uuid: account.uuid.clone(),
            account_type: format!("{:?}", account.account_type),
            is_active: account.is_active,
            is_valid: account.is_valid(),
            needs_refresh: account.needs_refresh(),
            skin_url,
            added_at: account.added_at.to_rfc3339(),
            last_used: account.last_used.map(|t| t.to_rfc3339()),
        }
    }
}

/// Device code info for the frontend
#[derive(Debug, Clone, Serialize)]
pub struct DeviceCodeInfoResponse {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    pub expires_in: u32,
    pub interval: u32,
}

/// State of a pending device code authentication
#[derive(Debug, Clone)]
enum DeviceCodeState {
    /// Waiting for user to complete authentication in browser
    /// (device_code_info, client_id)
    Pending(DeviceCodeInfo, String),
    /// User authenticated, now completing the full auth flow
    Completing,
}

// Global state for pending device code authentications
lazy_static::lazy_static! {
    static ref PENDING_DEVICE_CODES: Mutex<HashMap<String, DeviceCodeState>> = Mutex::new(HashMap::new());
}

/// Get all accounts
#[tauri::command]
pub async fn get_accounts(state: State<'_, AppState>) -> Result<Vec<AccountInfo>, String> {
    let config = state.config.lock().unwrap();
    let accounts_file = config.accounts_file();
    drop(config);

    // Load accounts from file
    let account_list = AccountList::load(&accounts_file).unwrap_or_default();

    // Update state
    {
        let mut accounts = state.accounts.lock().unwrap();
        *accounts = account_list.accounts.clone();
    }

    let info: Vec<AccountInfo> = account_list
        .accounts
        .iter()
        .map(AccountInfo::from)
        .collect();

    Ok(info)
}

/// Add an offline account
#[tauri::command]
pub async fn add_offline_account(
    state: State<'_, AppState>,
    username: String,
) -> Result<AccountInfo, String> {
    // Validate username
    validate_offline_username(&username).map_err(|e| e.to_string())?;

    let config = state.config.lock().unwrap();
    let accounts_file = config.accounts_file();
    drop(config);

    // Load existing accounts
    let mut account_list = AccountList::load(&accounts_file).unwrap_or_default();

    // Check for duplicate username
    if account_list.has_username(&username) {
        return Err(format!("An account with username '{}' already exists", username));
    }

    // Create the account
    let account = create_offline_account(&username);
    let info = AccountInfo::from(&account);

    // Add and save
    account_list.add(account);
    account_list
        .save(&accounts_file)
        .map_err(|e| e.to_string())?;

    // Update state
    {
        let mut accounts = state.accounts.lock().unwrap();
        *accounts = account_list.accounts;
    }

    Ok(info)
}

/// Start Microsoft account login (device code flow)
#[tauri::command]
pub async fn start_microsoft_login(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<DeviceCodeInfoResponse, String> {
    // Get client ID from config override or use default
    let client_id = {
        let config = state.config.lock().unwrap();
        config
            .api_keys
            .msa_client_id
            .clone()
            .unwrap_or_else(|| MSA_CLIENT_ID.to_string())
    };

    // Check if client ID is configured
    if client_id == "YOUR_AZURE_CLIENT_ID_HERE" || client_id.is_empty() {
        return Err("Microsoft Client ID not configured. Please add your Azure Client ID in Settings > Advanced.".to_string());
    }

    // Start device code flow
    let device_code = start_device_code_flow(&client_id)
        .await
        .map_err(|e| e.to_string())?;

    // Store for polling (include client_id)
    let code_key = device_code.device_code.clone();
    {
        let mut pending = PENDING_DEVICE_CODES.lock().unwrap();
        pending.insert(code_key.clone(), DeviceCodeState::Pending(device_code.clone(), client_id));
        tracing::info!("Stored device code - key: {} (len: {}), total pending: {}", 
            &code_key[..8.min(code_key.len())], 
            code_key.len(),
            pending.len()
        );
    }

    // Try to open the browser
    let _ = webbrowser::open(&device_code.verification_uri);

    // Emit event for frontend
    let _ = app.emit(
        "auth_device_code",
        serde_json::json!({
            "user_code": device_code.user_code,
            "verification_uri": device_code.verification_uri,
        }),
    );

    Ok(DeviceCodeInfoResponse {
        device_code: code_key,
        user_code: device_code.user_code,
        verification_uri: device_code.verification_uri,
        expires_in: device_code.expires_in,
        interval: device_code.interval,
    })
}

/// Poll for Microsoft authentication completion
#[tauri::command(rename_all = "camelCase")]
pub async fn poll_microsoft_login(
    state: State<'_, AppState>,
    app: AppHandle,
    device_code: String,
) -> Result<Option<AccountInfo>, String> {
    tracing::info!("poll_microsoft_login called with device_code length: {}, first 8 chars: {}", 
        device_code.len(),
        &device_code[..8.min(device_code.len())]
    );
    
    let pending_count = PENDING_DEVICE_CODES.lock().unwrap().len();
    tracing::info!("Total pending device codes: {}", pending_count);
    
    // Get the stored device code state
    let device_code_state = {
        let pending = PENDING_DEVICE_CODES.lock().unwrap();
        let result = pending.get(&device_code).cloned();
        if result.is_none() {
            let available_keys: Vec<String> = pending.keys()
                .map(|k| format!("{}... (len: {})", &k[..8.min(k.len())], k.len()))
                .collect();
            tracing::error!("Device code not found! Looking for: {} (len: {}), Available keys: {:?}", 
                &device_code[..8.min(device_code.len())],
                device_code.len(),
                available_keys
            );
        }
        result
    };

    let device_code_state =
        device_code_state.ok_or("Device code not found. Please start login again.")?;

    // Check if we're already completing authentication
    let (device_code_info, client_id) = match device_code_state {
        DeviceCodeState::Pending(info, client_id) => (info, client_id),
        DeviceCodeState::Completing => {
            // Authentication is already in progress, just wait
            tracing::debug!("Device code {} is already completing authentication", &device_code[..8.min(device_code.len())]);
            let _ = app.emit(
                "auth_progress",
                AuthProgressEvent::StepStarted {
                    step: "completing".to_string(),
                    description: "Completing authentication...".to_string(),
                },
            );
            return Ok(None);
        }
    };

    // Poll for result using the stored client_id
    let result = poll_device_code(&client_id, &device_code_info)
        .await
        .map_err(|e| e.to_string())?;

    match result {
        PollResult::Success(msa_token) => {
            // Mark as completing (instead of removing)
            {
                let mut pending = PENDING_DEVICE_CODES.lock().unwrap();
                pending.insert(device_code.clone(), DeviceCodeState::Completing);
            }

            // Emit progress events
            let _ = app.emit(
                "auth_progress",
                AuthProgressEvent::StepStarted {
                    step: "completing".to_string(),
                    description: "Completing authentication...".to_string(),
                },
            );

            // Create channel for progress updates
            let (tx, mut rx) = mpsc::channel::<AuthProgressEvent>(16);
            let app_clone = app.clone();

            // Spawn task to forward progress events
            tokio::spawn(async move {
                while let Some(event) = rx.recv().await {
                    let _ = app_clone.emit("auth_progress", &event);
                }
            });

            // Complete authentication
            let complete_result = complete_authentication(msa_token, Some(tx)).await;
            
            // Always remove from pending when done (success or failure)
            {
                let mut pending = PENDING_DEVICE_CODES.lock().unwrap();
                pending.remove(&device_code);
            }
            
            // Handle the result
            let account_data = complete_result.map_err(|e| e.to_string())?;

            // Create account
            let account = Account::new_microsoft_from_data(account_data);
            let info = AccountInfo::from(&account);

            // Save account
            let accounts_file = {
                let config = state.config.lock().unwrap();
                config.accounts_file()
            };

            let mut account_list = AccountList::load(&accounts_file).unwrap_or_default();

            // Check for duplicate UUID
            let existing_idx = account_list
                .accounts
                .iter()
                .position(|a| a.uuid == account.uuid);
            if let Some(idx) = existing_idx {
                // Update existing account
                account_list.accounts[idx] = account;
            } else {
                account_list.add(account);
            }

            account_list
                .save(&accounts_file)
                .map_err(|e| e.to_string())?;

            // Update state
            {
                let mut accounts = state.accounts.lock().unwrap();
                *accounts = account_list.accounts;
            }

            let _ = app.emit(
                "auth_progress",
                AuthProgressEvent::Completed {
                    username: info.username.clone(),
                },
            );

            Ok(Some(info))
        }
        PollResult::Pending => {
            let _ = app.emit(
                "auth_progress",
                AuthProgressEvent::PollingForAuth {
                    message: "Waiting for you to complete login...".to_string(),
                },
            );
            Ok(None)
        }
        PollResult::SlowDown => {
            // Need to slow down, but don't error
            Ok(None)
        }
        PollResult::Declined => {
            // Remove from pending
            {
                let mut pending = PENDING_DEVICE_CODES.lock().unwrap();
                pending.remove(&device_code);
            }
            Err("Authentication was declined".to_string())
        }
        PollResult::Expired => {
            // Remove from pending
            {
                let mut pending = PENDING_DEVICE_CODES.lock().unwrap();
                pending.remove(&device_code);
            }
            Err("Device code expired. Please try again.".to_string())
        }
    }
}

/// Cancel a pending Microsoft login
#[tauri::command(rename_all = "camelCase")]
pub async fn cancel_microsoft_login(device_code: String) -> Result<(), String> {
    tracing::info!("Cancelling device code: {}", &device_code[..8.min(device_code.len())]);
    let mut pending = PENDING_DEVICE_CODES.lock().unwrap();
    pending.remove(&device_code);
    Ok(())
}

/// Refresh an existing Microsoft account
#[tauri::command]
pub async fn refresh_account(
    state: State<'_, AppState>,
    app: AppHandle,
    account_id: String,
) -> Result<AccountInfo, String> {
    // Get accounts file path without holding the lock across await
    let accounts_file = {
        let config = state.config.lock().unwrap();
        config.accounts_file()
    };

    let mut account_list = AccountList::load(&accounts_file).unwrap_or_default();

    let account = account_list
        .get(&account_id)
        .ok_or("Account not found")?
        .clone();

    if !account.is_online() {
        return Err("Offline accounts don't need to be refreshed".to_string());
    }

    // Create channel for progress updates
    let (tx, mut rx) = mpsc::channel::<AuthProgressEvent>(16);
    let app_clone = app.clone();

    // Spawn task to forward progress events
    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            let _ = app_clone.emit("auth_progress", &event);
        }
    });

    // Refresh the account
    let account_data = refresh_microsoft_account(&account, Some(tx))
        .await
        .map_err(|e| e.to_string())?;

    // Update the account
    let updated_account = account_list
        .get_mut(&account_id)
        .ok_or("Account not found")?;
    updated_account.update_data(account_data);

    let info = AccountInfo::from(&*updated_account);

    // Save
    account_list
        .save(&accounts_file)
        .map_err(|e| e.to_string())?;

    // Update state
    {
        let mut accounts = state.accounts.lock().unwrap();
        *accounts = account_list.accounts;
    }

    Ok(info)
}

/// Set the active account
#[tauri::command]
pub async fn set_active_account(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<(), String> {
    let config = state.config.lock().unwrap();
    let accounts_file = config.accounts_file();
    drop(config);

    let mut account_list = AccountList::load(&accounts_file).unwrap_or_default();
    account_list.set_active(&account_id);
    account_list
        .save(&accounts_file)
        .map_err(|e| e.to_string())?;

    // Update state
    {
        let mut accounts = state.accounts.lock().unwrap();
        for account in accounts.iter_mut() {
            account.is_active = account.id == account_id;
        }
    }

    Ok(())
}

/// Remove an account
#[tauri::command]
pub async fn remove_account(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<(), String> {
    let config = state.config.lock().unwrap();
    let accounts_file = config.accounts_file();
    drop(config);

    let mut account_list = AccountList::load(&accounts_file).unwrap_or_default();
    account_list.remove(&account_id);
    account_list
        .save(&accounts_file)
        .map_err(|e| e.to_string())?;

    // Update state
    {
        let mut accounts = state.accounts.lock().unwrap();
        accounts.retain(|a| a.id != account_id);
    }

    Ok(())
}

/// Get account for launching (refresh if needed)
#[tauri::command]
pub async fn get_account_for_launch(
    state: State<'_, AppState>,
    app: AppHandle,
    account_id: Option<String>,
) -> Result<AccountInfo, String> {
    // Get accounts file path without holding the lock across await
    let accounts_file = {
        let config = state.config.lock().unwrap();
        config.accounts_file()
    };

    let mut account_list = AccountList::load(&accounts_file).unwrap_or_default();

    // Get the requested account or active account
    let account = if let Some(id) = account_id {
        account_list.get(&id).cloned()
    } else {
        account_list.get_active().cloned()
    };

    let account = account.ok_or("No account available. Please add an account first.")?;

    // If it's a Microsoft account and needs refresh, refresh it
    if account.is_online() && account.needs_refresh() {
        // Create channel for progress updates
        let (tx, mut rx) = mpsc::channel::<AuthProgressEvent>(16);
        let app_clone = app.clone();

        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                let _ = app_clone.emit("auth_progress", &event);
            }
        });

        let account_data = refresh_microsoft_account(&account, Some(tx))
            .await
            .map_err(|e| format!("Failed to refresh account: {}", e))?;

        // Update the account
        if let Some(updated) = account_list.get_mut(&account.id) {
            updated.update_data(account_data);
            updated.update_last_used();
        }

        account_list
            .save(&accounts_file)
            .map_err(|e| e.to_string())?;
    }

    // Get the possibly updated account
    let final_account = account_list
        .get(&account.id)
        .ok_or("Account not found")?;

    Ok(AccountInfo::from(final_account))
}

/// Check if a Microsoft Client ID is configured
#[tauri::command]
pub async fn is_microsoft_configured(state: State<'_, AppState>) -> Result<bool, String> {
    // Check config override first
    let config = state.config.lock().unwrap();
    if let Some(client_id) = &config.api_keys.msa_client_id {
        if !client_id.is_empty() && client_id != "YOUR_AZURE_CLIENT_ID_HERE" {
            return Ok(true);
        }
    }
    
    // Fall back to hardcoded ID
    Ok(MSA_CLIENT_ID != "YOUR_AZURE_CLIENT_ID_HERE")
}

// =============================================================================
// Skin Management Commands
// =============================================================================

/// Skin info for frontend
#[derive(Debug, Clone, Serialize)]
pub struct SkinInfoResponse {
    pub id: String,
    pub url: String,
    pub variant: String,
    pub is_active: bool,
}

/// Cape info for frontend
#[derive(Debug, Clone, Serialize)]
pub struct CapeInfoResponse {
    pub id: String,
    pub url: String,
    pub alias: Option<String>,
    pub is_active: bool,
}

/// Player profile for frontend
#[derive(Debug, Clone, Serialize)]
pub struct PlayerProfileResponse {
    pub id: String,
    pub name: String,
    pub skins: Vec<SkinInfoResponse>,
    pub capes: Vec<CapeInfoResponse>,
    pub active_skin: Option<SkinInfoResponse>,
    pub active_cape: Option<CapeInfoResponse>,
}

/// Fetched skin info for import
#[derive(Debug, Clone, Serialize)]
pub struct FetchedSkinResponse {
    pub uuid: String,
    pub username: String,
    pub skin_url: Option<String>,
    pub skin_variant: String,
    pub cape_url: Option<String>,
}

/// Helper to get access token for an account
fn get_account_access_token(accounts_file: &std::path::PathBuf, account_id: &str) -> Result<String, String> {
    let account_list = AccountList::load(accounts_file).unwrap_or_default();
    let account = account_list
        .get(account_id)
        .ok_or("Account not found")?;

    if !account.is_online() {
        return Err("Skin management is only available for Microsoft accounts".to_string());
    }

    let access_token = account.get_access_token();
    if access_token.is_empty() {
        return Err("Account has no valid access token. Please refresh the account.".to_string());
    }

    Ok(access_token)
}

/// Get the full player profile including skins and capes
#[tauri::command(rename_all = "camelCase")]
pub async fn get_player_profile(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<PlayerProfileResponse, String> {
    let accounts_file = {
        let config = state.config.lock().unwrap();
        config.accounts_file()
    };

    let access_token = get_account_access_token(&accounts_file, &account_id)?;

    let profile = skins::get_player_profile(&access_token)
        .await
        .map_err(|e| e.to_string())?;

    Ok(PlayerProfileResponse {
        id: profile.id,
        name: profile.name,
        skins: profile.skins.iter().map(|s| SkinInfoResponse {
            id: s.id.clone(),
            url: s.url.clone(),
            variant: match s.variant {
                SkinVariant::Slim => "slim".to_string(),
                SkinVariant::Classic => "classic".to_string(),
            },
            is_active: profile.active_skin.as_ref().map(|a| a.id == s.id).unwrap_or(false),
        }).collect(),
        capes: profile.capes.iter().map(|c| CapeInfoResponse {
            id: c.id.clone(),
            url: c.url.clone(),
            alias: c.alias.clone(),
            is_active: profile.active_cape.as_ref().map(|a| a.id == c.id).unwrap_or(false),
        }).collect(),
        active_skin: profile.active_skin.map(|s| SkinInfoResponse {
            id: s.id.clone(),
            url: s.url.clone(),
            variant: match s.variant {
                SkinVariant::Slim => "slim".to_string(),
                SkinVariant::Classic => "classic".to_string(),
            },
            is_active: true,
        }),
        active_cape: profile.active_cape.map(|c| CapeInfoResponse {
            id: c.id.clone(),
            url: c.url.clone(),
            alias: c.alias.clone(),
            is_active: true,
        }),
    })
}

/// Change skin using a URL
#[tauri::command(rename_all = "camelCase")]
pub async fn change_skin_url(
    state: State<'_, AppState>,
    account_id: String,
    skin_url: String,
    variant: String,
) -> Result<(), String> {
    let accounts_file = {
        let config = state.config.lock().unwrap();
        config.accounts_file()
    };

    let access_token = get_account_access_token(&accounts_file, &account_id)?;

    let skin_variant = match variant.to_lowercase().as_str() {
        "slim" => SkinVariant::Slim,
        _ => SkinVariant::Classic,
    };

    skins::change_skin_url(&access_token, &skin_url, skin_variant)
        .await
        .map_err(|e| e.to_string())
}

/// Upload a skin from file
#[tauri::command(rename_all = "camelCase")]
pub async fn upload_skin(
    state: State<'_, AppState>,
    account_id: String,
    image_data: Vec<u8>,
    variant: String,
) -> Result<(), String> {
    // Validate skin image
    skins::validate_skin_image(&image_data)
        .map_err(|e| e.to_string())?;

    let accounts_file = {
        let config = state.config.lock().unwrap();
        config.accounts_file()
    };

    let access_token = get_account_access_token(&accounts_file, &account_id)?;

    let skin_variant = match variant.to_lowercase().as_str() {
        "slim" => SkinVariant::Slim,
        _ => SkinVariant::Classic,
    };

    skins::upload_skin(&access_token, &image_data, skin_variant)
        .await
        .map_err(|e| e.to_string())
}

/// Reset skin to default
#[tauri::command(rename_all = "camelCase")]
pub async fn reset_skin(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<(), String> {
    let accounts_file = {
        let config = state.config.lock().unwrap();
        config.accounts_file()
    };

    let access_token = get_account_access_token(&accounts_file, &account_id)?;

    skins::reset_skin(&access_token)
        .await
        .map_err(|e| e.to_string())
}

/// Set active cape
#[tauri::command(rename_all = "camelCase")]
pub async fn set_cape(
    state: State<'_, AppState>,
    account_id: String,
    cape_id: String,
) -> Result<(), String> {
    let accounts_file = {
        let config = state.config.lock().unwrap();
        config.accounts_file()
    };

    let access_token = get_account_access_token(&accounts_file, &account_id)?;

    skins::set_cape(&access_token, &cape_id)
        .await
        .map_err(|e| e.to_string())
}

/// Hide cape (remove active cape)
#[tauri::command(rename_all = "camelCase")]
pub async fn hide_cape(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<(), String> {
    let accounts_file = {
        let config = state.config.lock().unwrap();
        config.accounts_file()
    };

    let access_token = get_account_access_token(&accounts_file, &account_id)?;

    skins::hide_cape(&access_token)
        .await
        .map_err(|e| e.to_string())
}

/// Fetch skin from a username (for importing someone else's skin)
#[tauri::command(rename_all = "camelCase")]
pub async fn fetch_skin_from_username(
    username: String,
) -> Result<FetchedSkinResponse, String> {
    let fetched = skins::fetch_skin_from_username(&username)
        .await
        .map_err(|e| e.to_string())?;

    Ok(FetchedSkinResponse {
        uuid: fetched.uuid,
        username: fetched.username,
        skin_url: fetched.skin_url,
        skin_variant: match fetched.skin_variant {
            SkinVariant::Slim => "slim".to_string(),
            SkinVariant::Classic => "classic".to_string(),
        },
        cape_url: fetched.cape_url,
    })
}

/// Import skin from another player (by username)
#[tauri::command(rename_all = "camelCase")]
pub async fn import_skin_from_username(
    state: State<'_, AppState>,
    account_id: String,
    username: String,
    use_original_variant: bool,
    override_variant: Option<String>,
) -> Result<(), String> {
    // Fetch the skin from the username first
    let fetched = skins::fetch_skin_from_username(&username)
        .await
        .map_err(|e| e.to_string())?;

    let skin_url = fetched.skin_url
        .ok_or("Player has no custom skin to import")?;

    let variant = if use_original_variant {
        fetched.skin_variant
    } else if let Some(v) = override_variant {
        match v.to_lowercase().as_str() {
            "slim" => SkinVariant::Slim,
            _ => SkinVariant::Classic,
        }
    } else {
        fetched.skin_variant
    };

    // Get account access token
    let accounts_file = {
        let config = state.config.lock().unwrap();
        config.accounts_file()
    };

    let access_token = get_account_access_token(&accounts_file, &account_id)?;

    // Set the skin using the URL
    skins::change_skin_url(&access_token, &skin_url, variant)
        .await
        .map_err(|e| e.to_string())
}

/// Open the skins folder
#[tauri::command]
pub async fn open_skins_folder(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let data_dir = {
        let config = state.config.lock().unwrap();
        config.data_dir()
    };

    let skins_folder = skins::get_skins_folder(&data_dir);
    
    // Create folder if it doesn't exist
    std::fs::create_dir_all(&skins_folder)
        .map_err(|e| format!("Failed to create skins folder: {}", e))?;

    // Open in file explorer
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(&skins_folder)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&skins_folder)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&skins_folder)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }

    Ok(())
}

/// Set an account as the default account
#[tauri::command(rename_all = "camelCase")]
pub async fn set_default_account(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<(), String> {
    let accounts_file = {
        let config = state.config.lock().unwrap();
        config.accounts_file()
    };

    let mut account_list = AccountList::load(&accounts_file).unwrap_or_default();
    
    // Verify account exists
    if !account_list.accounts.iter().any(|a| a.id == account_id) {
        return Err("Account not found".to_string());
    }
    
    account_list.set_active(&account_id);
    account_list
        .save(&accounts_file)
        .map_err(|e| e.to_string())?;

    // Update state
    {
        let mut accounts = state.accounts.lock().unwrap();
        for account in accounts.iter_mut() {
            account.is_active = account.id == account_id;
        }
    }

    Ok(())
}

/// Download a skin image and return as base64
#[tauri::command(rename_all = "camelCase")]
pub async fn download_skin_image(
    skin_url: String,
) -> Result<String, String> {
    let image_data = skins::download_skin_image(&skin_url)
        .await
        .map_err(|e| e.to_string())?;

    use base64::Engine;
    Ok(base64::engine::general_purpose::STANDARD.encode(&image_data))
}

/// Cache a skin image to the skins folder and return the path
#[tauri::command(rename_all = "camelCase")]
pub async fn cache_skin_image(
    state: State<'_, AppState>,
    uuid: String,
    skin_url: String,
) -> Result<String, String> {
    let data_dir = {
        let config = state.config.lock().unwrap();
        config.data_dir()
    };

    let skins_folder = skins::get_skins_folder(&data_dir);
    tokio::fs::create_dir_all(&skins_folder)
        .await
        .map_err(|e| format!("Failed to create skins folder: {}", e))?;

    let skin_path = skins_folder.join(format!("{}.png", uuid));

    // Check if already cached
    if skin_path.exists() {
        return Ok(skin_path.to_string_lossy().to_string());
    }

    // Download and cache
    let image_data = skins::download_skin_image(&skin_url)
        .await
        .map_err(|e| e.to_string())?;

    tokio::fs::write(&skin_path, &image_data)
        .await
        .map_err(|e| format!("Failed to cache skin: {}", e))?;

    Ok(skin_path.to_string_lossy().to_string())
}

/// Get cached skin path if it exists
#[tauri::command(rename_all = "camelCase")]
pub async fn get_cached_skin_path(
    state: State<'_, AppState>,
    uuid: String,
) -> Result<Option<String>, String> {
    let data_dir = {
        let config = state.config.lock().unwrap();
        config.data_dir()
    };

    let skins_folder = skins::get_skins_folder(&data_dir);
    let skin_path = skins_folder.join(format!("{}.png", uuid));

    if skin_path.exists() {
        Ok(Some(skin_path.to_string_lossy().to_string()))
    } else {
        Ok(None)
    }
}

/// Read a file as bytes (for file upload without CSP issues)
#[tauri::command(rename_all = "camelCase")]
pub async fn read_file_bytes(
    file_path: String,
) -> Result<Vec<u8>, String> {
    tokio::fs::read(&file_path)
        .await
        .map_err(|e| format!("Failed to read file: {}", e))
}

