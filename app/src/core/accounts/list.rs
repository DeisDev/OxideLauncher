//! Account list management and persistence.
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

#![allow(dead_code)] // List management will be used as features are completed

use std::path::PathBuf;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::core::error::Result;
use super::Account;

/// Stored accounts file format (for JSON serialization)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AccountsFile {
    /// File format version for future migrations
    #[serde(default = "default_version")]
    version: u32,
    /// Whether game ownership has been verified via Microsoft account
    #[serde(default)]
    ownership_verified: bool,
    /// When ownership was first verified
    #[serde(skip_serializing_if = "Option::is_none")]
    ownership_verified_at: Option<DateTime<Utc>>,
    /// All accounts
    accounts: Vec<Account>,
}

fn default_version() -> u32 {
    1
}

impl Default for AccountsFile {
    fn default() -> Self {
        Self {
            version: 1,
            ownership_verified: false,
            ownership_verified_at: None,
            accounts: Vec::new(),
        }
    }
}

/// List of all accounts with ownership verification state
#[derive(Debug, Clone)]
pub struct AccountList {
    /// All accounts
    pub accounts: Vec<Account>,
    /// Whether game ownership has been verified via Microsoft account
    ownership_verified: bool,
    /// When ownership was first verified
    ownership_verified_at: Option<DateTime<Utc>>,
}

impl AccountList {
    /// Create a new empty account list
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
            ownership_verified: false,
            ownership_verified_at: None,
        }
    }

    /// Load accounts from file (handles both old and new format)
    pub fn load(accounts_file: &PathBuf) -> Result<Self> {
        if accounts_file.exists() {
            let content = std::fs::read_to_string(accounts_file)?;
            
            // Try to parse as new format first
            if let Ok(file) = serde_json::from_str::<AccountsFile>(&content) {
                return Ok(Self {
                    accounts: file.accounts,
                    ownership_verified: file.ownership_verified,
                    ownership_verified_at: file.ownership_verified_at,
                });
            }
            
            // Fall back to old format (just Vec<Account>)
            let accounts: Vec<Account> = serde_json::from_str(&content)?;
            Ok(Self { 
                accounts,
                ownership_verified: false,
                ownership_verified_at: None,
            })
        } else {
            Ok(Self::new())
        }
    }

    /// Save accounts to file (uses new format)
    pub fn save(&self, accounts_file: &PathBuf) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = accounts_file.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let file = AccountsFile {
            version: 1,
            ownership_verified: self.ownership_verified,
            ownership_verified_at: self.ownership_verified_at,
            accounts: self.accounts.clone(),
        };

        let content = serde_json::to_string_pretty(&file)?;
        std::fs::write(accounts_file, content)?;
        
        Ok(())
    }

    /// Check if game ownership has been verified
    pub fn is_ownership_verified(&self) -> bool {
        self.ownership_verified
    }

    /// Mark ownership as verified (called when Microsoft account verifies game ownership)
    pub fn set_ownership_verified(&mut self) {
        if !self.ownership_verified {
            self.ownership_verified = true;
            self.ownership_verified_at = Some(Utc::now());
        }
    }

    /// Get when ownership was verified
    pub fn ownership_verified_at(&self) -> Option<DateTime<Utc>> {
        self.ownership_verified_at
    }

    /// Check if there are any Microsoft accounts with valid game ownership
    pub fn has_microsoft_account(&self) -> bool {
        self.accounts
            .iter()
            .any(|a| matches!(a.account_type, super::AccountType::Microsoft))
    }

    /// Get an account by ID
    pub fn get(&self, id: &str) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == id)
    }

    /// Get a mutable account by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut Account> {
        self.accounts.iter_mut().find(|a| a.id == id)
    }

    /// Get the active account
    pub fn get_active(&self) -> Option<&Account> {
        self.accounts.iter().find(|a| a.is_active)
    }

    /// Get a mutable reference to the active account
    pub fn get_active_mut(&mut self) -> Option<&mut Account> {
        self.accounts.iter_mut().find(|a| a.is_active)
    }

    /// Set the active account
    pub fn set_active(&mut self, id: &str) {
        for account in &mut self.accounts {
            account.is_active = account.id == id;
        }
    }

    /// Add an account
    pub fn add(&mut self, mut account: Account) {
        // If this is the first account, make it active
        if self.accounts.is_empty() {
            account.is_active = true;
        }
        
        self.accounts.push(account);
    }

    /// Remove an account by ID
    pub fn remove(&mut self, id: &str) -> Option<Account> {
        if let Some(pos) = self.accounts.iter().position(|a| a.id == id) {
            let account = self.accounts.remove(pos);
            
            // If we removed the active account, activate the first remaining one
            if account.is_active && !self.accounts.is_empty() {
                self.accounts[0].is_active = true;
            }
            
            Some(account)
        } else {
            None
        }
    }

    /// Update an existing account
    pub fn update(&mut self, account: Account) {
        if let Some(existing) = self.get_mut(&account.id) {
            *existing = account;
        }
    }

    /// Get all Microsoft accounts
    pub fn microsoft_accounts(&self) -> Vec<&Account> {
        self.accounts
            .iter()
            .filter(|a| matches!(a.account_type, super::AccountType::Microsoft))
            .collect()
    }

    /// Get all offline accounts
    pub fn offline_accounts(&self) -> Vec<&Account> {
        self.accounts
            .iter()
            .filter(|a| matches!(a.account_type, super::AccountType::Offline))
            .collect()
    }

    /// Check if an account with the given username exists
    pub fn has_username(&self, username: &str) -> bool {
        self.accounts.iter().any(|a| a.username == username)
    }

    /// Get account count
    pub fn len(&self) -> usize {
        self.accounts.len()
    }

    /// Check if list is empty
    pub fn is_empty(&self) -> bool {
        self.accounts.is_empty()
    }

    /// Iterate over accounts
    pub fn iter(&self) -> std::slice::Iter<'_, Account> {
        self.accounts.iter()
    }
}

impl Default for AccountList {
    fn default() -> Self {
        Self::new()
    }
}