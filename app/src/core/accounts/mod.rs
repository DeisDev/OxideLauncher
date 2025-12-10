//! Account management
//! 
//! Handles Microsoft account authentication and offline accounts.

mod types;
mod list;
mod microsoft;
mod offline;

pub use types::*;
pub use list::AccountList;
pub use microsoft::login_microsoft;
pub use offline::create_offline_account;
