//! Mod platform integrations (Modrinth, CurseForge)

pub mod modrinth;
pub mod curseforge;
pub mod types;

#[allow(unused_imports)] // Types will be used as features are completed
pub use types::*;
