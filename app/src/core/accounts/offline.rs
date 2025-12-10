//! Offline account creation

use super::Account;

/// Create an offline account with the given username
pub fn create_offline_account(username: &str) -> Account {
    Account::new_offline(username.to_string())
}

/// Validate an offline username
pub fn validate_offline_username(username: &str) -> Result<(), &'static str> {
    // Check length (3-16 characters like Minecraft)
    if username.len() < 3 {
        return Err("Username must be at least 3 characters");
    }
    if username.len() > 16 {
        return Err("Username must be at most 16 characters");
    }

    // Check characters (alphanumeric and underscore only)
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err("Username can only contain letters, numbers, and underscores");
    }

    Ok(())
}
