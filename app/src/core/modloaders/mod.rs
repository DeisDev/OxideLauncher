//! Modloader API clients and installers

pub mod forge;
pub mod neoforge;
pub mod fabric;
pub mod quilt;
pub mod liteloader;
pub mod profile;
pub mod installer;
pub mod processor;

// Re-export commonly used items
pub use forge::{get_forge_versions, get_recommended_forge, ForgeVersion, ForgeInstaller};
pub use neoforge::{get_neoforge_versions, get_recommended_neoforge, NeoForgeVersion, NeoForgeInstaller};
pub use fabric::{get_fabric_versions, get_recommended_fabric, FabricVersion, FabricInstaller};
pub use quilt::{get_quilt_versions, get_recommended_quilt, QuiltVersion, QuiltInstaller};
pub use liteloader::{get_liteloader_versions, get_recommended_liteloader, LiteLoaderVersion, LiteLoaderInstaller};
pub use profile::{ModloaderProfile, ModloaderLibrary, maven_to_path};
pub use installer::{ModloaderInstaller, InstallProgress, ProgressCallback, install_modloader, get_installer};
pub use processor::{run_processors, Processor, ProcessorContext, ProcessorData, extract_installer_libraries};
