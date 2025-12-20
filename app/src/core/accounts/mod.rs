//! Account management module for Microsoft and offline accounts.
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

mod types;
mod list;
mod microsoft;
mod offline;
pub mod skins;

pub use types::*;
pub use list::AccountList;
pub use microsoft::{
    start_device_code_flow,
    poll_device_code,
    complete_authentication,
    refresh_microsoft_account,
    PollResult,
    MSA_CLIENT_ID,
};
pub use offline::{create_offline_account, validate_offline_username};

