//! Account management
//! 
//! Handles Microsoft account authentication and offline accounts.

mod types;
mod list;
mod microsoft;
mod offline;

pub use types::*;
#[allow(unused_imports)] // Will be used when account management is fully implemented
pub use list::AccountList;
#[allow(unused_imports)]
pub use microsoft::login_microsoft;
#[allow(unused_imports)]
pub use offline::create_offline_account;
