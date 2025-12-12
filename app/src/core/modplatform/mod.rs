//! Mod platform integrations (Modrinth, CurseForge)

pub mod modrinth;
pub mod curseforge;
pub mod types;
pub mod mod_parser;

#[allow(unused_imports)] // Types will be used as features are completed
pub use types::*;
pub use mod_parser::{parse_mod_jar, ModDetails};
