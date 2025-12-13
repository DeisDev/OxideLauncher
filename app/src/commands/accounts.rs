//! Account management commands

use super::state::AppState;
use crate::core::accounts::{
    complete_authentication, create_offline_account, poll_device_code, refresh_microsoft_account,
    start_device_code_flow, validate_offline_username, Account, AccountList, AuthProgressEvent,
    DeviceCodeInfo, PollResult, MSA_CLIENT_ID,
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
    Pending(DeviceCodeInfo),
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
    app: AppHandle,
) -> Result<DeviceCodeInfoResponse, String> {
    // Check if client ID is configured
    if MSA_CLIENT_ID == "YOUR_AZURE_CLIENT_ID_HERE" {
        return Err("Microsoft Client ID not configured. Please add your Azure Client ID in the code.".to_string());
    }

    // Start device code flow
    let device_code = start_device_code_flow(MSA_CLIENT_ID)
        .await
        .map_err(|e| e.to_string())?;

    // Store for polling
    let code_key = device_code.device_code.clone();
    {
        let mut pending = PENDING_DEVICE_CODES.lock().unwrap();
        pending.insert(code_key.clone(), DeviceCodeState::Pending(device_code.clone()));
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
    let device_code_info = match device_code_state {
        DeviceCodeState::Pending(info) => info,
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

    // Poll for result
    let result = poll_device_code(MSA_CLIENT_ID, &device_code_info)
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
pub async fn is_microsoft_configured() -> Result<bool, String> {
    Ok(MSA_CLIENT_ID != "YOUR_AZURE_CLIENT_ID_HERE")
}

