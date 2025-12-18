//! RustWiz - Packwiz-compatible mod metadata for OxideLauncher
//!
//! RustWiz is a native Rust implementation of the packwiz format, providing:
//! - Mod metadata tracking with update sources (Modrinth, CurseForge)
//! - Modpack import/export (.mrpack, CurseForge zip, raw packwiz)
//! - Mod update checking via platform APIs
//!
//! Compatible with packwiz tools: https://packwiz.infra.link/reference/pack-format/

pub mod types;
pub mod parser;
pub mod update_check;
pub mod export;

// Re-export commonly used types
pub use types::{
    ModToml, ModTomlExtended, OxideMetadata,
    HashFormat, Side,
    BatchUpdateResult,
};

pub use parser::{
    read_pack_toml, write_pack_toml,
    write_index_toml,
    write_mod_toml, delete_mod_toml,
    rebuild_index,
    mod_toml_filename,
    compute_file_hash,
    index_dir,
};

// Renamed functions for RustWiz branding
pub use parser::initialize_packwiz as initialize_pack;
pub use parser::has_packwiz as has_pack;

#[allow(unused_imports)] // check_instance_updates kept for backwards compatibility
pub use update_check::{check_instance_updates, check_instance_updates_with_info};

pub use export::{
    ExportOptions,
    export_modrinth, export_curseforge, export_packwiz,
};
