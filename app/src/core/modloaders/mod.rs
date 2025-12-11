//! Modloader API clients

pub mod forge;
pub mod neoforge;
pub mod fabric;
pub mod quilt;
pub mod liteloader;

#[allow(unused_imports)] // Types will be used as features are completed
pub use forge::{get_forge_versions, get_recommended_forge, ForgeVersion};
#[allow(unused_imports)]
pub use neoforge::{get_neoforge_versions, get_recommended_neoforge, NeoForgeVersion};
#[allow(unused_imports)]
pub use fabric::{get_fabric_versions, get_recommended_fabric, FabricVersion};
#[allow(unused_imports)]
pub use quilt::{get_quilt_versions, get_recommended_quilt, QuiltVersion};
#[allow(unused_imports)]
pub use liteloader::{get_liteloader_versions, get_recommended_liteloader, LiteLoaderVersion};
