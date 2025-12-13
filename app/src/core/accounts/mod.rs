//! Account management
//! 
//! Handles Microsoft account authentication and offline accounts.
//! 
//! Authentication Flow (Microsoft):
//! 1. Start device code flow - returns code for user to enter
//! 2. Poll for authentication completion
//! 3. Exchange tokens through Xbox Live -> XSTS -> Minecraft
//! 4. Verify entitlements and fetch profile
//! 
//! Similar to Prism Launcher's implementation.

mod types;
mod list;
mod microsoft;
mod offline;

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

