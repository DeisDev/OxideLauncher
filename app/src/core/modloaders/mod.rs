//! Modloader API clients

pub mod forge;
pub mod neoforge;
pub mod fabric;
pub mod quilt;
pub mod liteloader;

pub use forge::{get_forge_versions, get_recommended_forge, ForgeVersion};
pub use neoforge::{get_neoforge_versions, get_recommended_neoforge, NeoForgeVersion};
pub use fabric::{get_fabric_versions, get_recommended_fabric, FabricVersion};
pub use quilt::{get_quilt_versions, get_recommended_quilt, QuiltVersion};
pub use liteloader::{get_liteloader_versions, get_recommended_liteloader, LiteLoaderVersion};
