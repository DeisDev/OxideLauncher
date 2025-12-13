//! Account management commands

use super::state::AppState;
use serde::Serialize;
use tauri::State;

/// Serializable account information for the frontend
#[derive(Debug, Clone, Serialize)]
pub struct AccountInfo {
    pub id: String,
    pub username: String,
    pub account_type: String,
    pub is_active: bool,
}

#[tauri::command]
pub async fn get_accounts(state: State<'_, AppState>) -> Result<Vec<AccountInfo>, String> {
    let accounts = state.accounts.lock().unwrap();
    let info: Vec<AccountInfo> = accounts.iter().map(|acc| {
        AccountInfo {
            id: acc.id.clone(),
            username: acc.username.clone(),
            account_type: format!("{:?}", acc.account_type),
            is_active: acc.is_active,
        }
    }).collect();
    
    Ok(info)
}

#[tauri::command]
pub async fn add_offline_account(
    _state: State<'_, AppState>,
    _username: String,
) -> Result<(), String> {
    // TODO: Implement offline account creation
    Ok(())
}

#[tauri::command]
pub async fn set_active_account(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<(), String> {
    let mut accounts = state.accounts.lock().unwrap();
    for account in accounts.iter_mut() {
        account.is_active = account.id == account_id;
    }
    Ok(())
}

#[tauri::command]
pub async fn remove_account(
    state: State<'_, AppState>,
    account_id: String,
) -> Result<(), String> {
    let mut accounts = state.accounts.lock().unwrap();
    accounts.retain(|a| a.id != account_id);
    Ok(())
}
