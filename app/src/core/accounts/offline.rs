//! Offline account creation and validation.
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

#![allow(dead_code)] // Offline account creation will be used as features are completed

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
