//! Forge/NeoForge processor execution
//!
//! Modern Forge and NeoForge versions require running "processors" that patch
//! the vanilla Minecraft client JAR to work with the modloader. These processors
//! are Java programs bundled in the installer JAR.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::collections::HashMap;
use tracing::{info, debug, warn, error};

use crate::core::error::{OxideError, Result};
use crate::core::java::detect_java_installations;
use super::profile::maven_to_path;

/// Data for processor variable substitution
#[derive(Debug, Clone)]
pub struct ProcessorData {
    pub client: String,
    pub server: String,
}

/// A processor to run
#[derive(Debug, Clone)]
pub struct Processor {
    /// Maven coordinate of the processor JAR
    pub jar: String,
    /// Maven coordinates of classpath entries
    pub classpath: Vec<String>,
    /// Arguments (may contain placeholders)
    pub args: Vec<String>,
    /// Which side to run on (empty means both)
    pub sides: Vec<String>,
    /// Expected outputs (path -> sha1)
    pub outputs: HashMap<String, String>,
}

/// Context for running processors
pub struct ProcessorContext {
    /// Path to libraries directory
    pub libraries_dir: PathBuf,
    /// Path to the Minecraft client JAR
    pub client_jar: PathBuf,
    /// Minecraft version
    pub minecraft_version: String,
    /// Forge/NeoForge version
    pub loader_version: String,
    /// Data variables for substitution
    pub data: HashMap<String, ProcessorData>,
    /// Path to the installer JAR (for extracting files)
    pub installer_jar: PathBuf,
}

impl ProcessorContext {
    /// Substitute placeholders in an argument
    fn substitute_arg(&self, arg: &str) -> Result<String> {
        let mut result = arg.to_string();
        
        // Handle [artifact] references - these refer to library paths
        if result.starts_with('[') && result.ends_with(']') {
            let artifact = &result[1..result.len()-1];
            let path = self.libraries_dir.join(maven_to_path(artifact));
            return Ok(path.to_string_lossy().to_string());
        }
        
        // Handle {data} references
        if result.starts_with('{') && result.ends_with('}') {
            let key = &result[1..result.len()-1];
            
            // Special built-in variables
            match key {
                "MINECRAFT_JAR" => return Ok(self.client_jar.to_string_lossy().to_string()),
                "SIDE" => return Ok("client".to_string()),
                "MINECRAFT_VERSION" => return Ok(self.minecraft_version.clone()),
                _ => {}
            }
            
            // Look up in data map
            if let Some(data) = self.data.get(key) {
                // Use client-side value
                let value = &data.client;
                
                // The value might be a path reference starting with /
                if value.starts_with('/') {
                    // Extract from installer JAR - needs special handling
                    return Ok(format!("EXTRACT:{}", value));
                } else if value.starts_with('[') && value.ends_with(']') {
                    // It's an artifact reference
                    let artifact = &value[1..value.len()-1];
                    let path = self.libraries_dir.join(maven_to_path(artifact));
                    return Ok(path.to_string_lossy().to_string());
                }
                
                return Ok(value.clone());
            }
            
            warn!("Unknown data reference: {}", key);
        }
        
        Ok(result)
    }
}

/// Find a Java executable suitable for running processors
pub fn find_processor_java() -> Result<PathBuf> {
    // Try to find any Java 17+ installation
    let installations = detect_java_installations();
    
    // Prefer Java 17 or newer
    if let Some(java) = installations.iter().find(|j| j.version.major >= 17) {
        return Ok(java.path.clone());
    }
    
    // Fall back to any Java 8+
    if let Some(java) = installations.iter().find(|j| j.version.major >= 8) {
        return Ok(java.path.clone());
    }
    
    Err(OxideError::Modloader(
        "No suitable Java installation found for running Forge processors. \
        Please install Java 17 or newer.".to_string()
    ))
}

/// Run a single processor
pub fn run_processor(
    processor: &Processor,
    context: &ProcessorContext,
    java_path: &Path,
) -> Result<()> {
    // Check if this processor applies to client side
    if !processor.sides.is_empty() && !processor.sides.contains(&"client".to_string()) {
        debug!("Skipping processor {} (server-only)", processor.jar);
        return Ok(());
    }
    
    // Build classpath
    let mut classpath = Vec::new();
    
    // Add the processor JAR itself
    let processor_jar = context.libraries_dir.join(maven_to_path(&processor.jar));
    if !processor_jar.exists() {
        return Err(OxideError::Modloader(format!(
            "Processor JAR not found: {:?}", processor_jar
        )));
    }
    classpath.push(processor_jar.to_string_lossy().to_string());
    
    // Add classpath entries
    for entry in &processor.classpath {
        let path = context.libraries_dir.join(maven_to_path(entry));
        if path.exists() {
            classpath.push(path.to_string_lossy().to_string());
        } else {
            warn!("Classpath entry not found: {:?}", path);
        }
    }
    
    let classpath_str = classpath.join(if cfg!(windows) { ";" } else { ":" });
    
    // Build arguments
    let mut args: Vec<String> = Vec::new();
    args.push("-cp".to_string());
    args.push(classpath_str);
    
    // Get main class from JAR manifest
    let main_class = get_jar_main_class(&processor_jar)?;
    args.push(main_class);
    
    // Process processor arguments
    for arg in &processor.args {
        match context.substitute_arg(arg) {
            Ok(substituted) => {
                // Handle EXTRACT: prefix (file needs to be extracted from installer)
                if substituted.starts_with("EXTRACT:") {
                    let extract_path = &substituted[8..];
                    let extracted = extract_from_installer(&context.installer_jar, extract_path, &context.libraries_dir)?;
                    args.push(extracted.to_string_lossy().to_string());
                } else {
                    args.push(substituted);
                }
            }
            Err(e) => {
                warn!("Failed to substitute argument '{}': {}", arg, e);
                args.push(arg.clone());
            }
        }
    }
    
    debug!("Running processor: {:?} {:?}", java_path, args);
    
    // Run the processor
    let output = Command::new(java_path)
        .args(&args)
        .output()
        .map_err(|e| OxideError::Modloader(format!("Failed to run processor: {}", e)))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        error!("Processor failed with status {:?}", output.status);
        error!("stdout: {}", stdout);
        error!("stderr: {}", stderr);
        return Err(OxideError::Modloader(format!(
            "Processor failed: {}", stderr
        )));
    }
    
    Ok(())
}

/// Get the main class from a JAR file's manifest
fn get_jar_main_class(jar_path: &Path) -> Result<String> {
    let file = std::fs::File::open(jar_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    
    let mut manifest = archive.by_name("META-INF/MANIFEST.MF")
        .map_err(|_| OxideError::Modloader("No MANIFEST.MF in JAR".to_string()))?;
    
    let mut content = String::new();
    std::io::Read::read_to_string(&mut manifest, &mut content)?;
    
    // Parse manifest to find Main-Class
    for line in content.lines() {
        if line.starts_with("Main-Class:") {
            let main_class = line.trim_start_matches("Main-Class:").trim();
            return Ok(main_class.to_string());
        }
    }
    
    Err(OxideError::Modloader("No Main-Class in JAR manifest".to_string()))
}

/// Extract a file from the installer JAR
fn extract_from_installer(installer_jar: &Path, internal_path: &str, dest_dir: &Path) -> Result<PathBuf> {
    let file = std::fs::File::open(installer_jar)?;
    let mut archive = zip::ZipArchive::new(file)?;
    
    // Remove leading slash if present
    let clean_path = internal_path.trim_start_matches('/');
    
    let mut zip_file = archive.by_name(clean_path)
        .map_err(|_| OxideError::Modloader(format!(
            "File not found in installer: {}", clean_path
        )))?;
    
    // Determine destination path
    let dest_path = dest_dir.join("forge_extracted").join(clean_path);
    
    if let Some(parent) = dest_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    let mut output = std::fs::File::create(&dest_path)?;
    std::io::copy(&mut zip_file, &mut output)?;
    
    debug!("Extracted {} to {:?}", clean_path, dest_path);
    
    Ok(dest_path)
}

/// Run all processors for a Forge/NeoForge installation
pub fn run_processors(
    processors: &[Processor],
    context: &ProcessorContext,
) -> Result<()> {
    if processors.is_empty() {
        debug!("No processors to run");
        return Ok(());
    }
    
    info!("Running {} Forge processors...", processors.len());
    
    // Find Java for running processors
    let java_path = find_processor_java()?;
    info!("Using Java at {:?} for processors", java_path);
    
    for (i, processor) in processors.iter().enumerate() {
        info!("Running processor {}/{}: {}", i + 1, processors.len(), processor.jar);
        
        run_processor(processor, context, &java_path)?;
    }
    
    info!("All processors completed successfully");
    Ok(())
}

/// Extract libraries from installer JAR to libraries directory
pub fn extract_installer_libraries(
    installer_jar: &Path,
    libraries_dir: &Path,
) -> Result<()> {
    let file = std::fs::File::open(installer_jar)?;
    let mut archive = zip::ZipArchive::new(file)?;
    
    // Look for maven/ directory in the installer
    let maven_prefix = "maven/";
    let mut extracted_count = 0;
    
    for i in 0..archive.len() {
        let mut zip_file = archive.by_index(i)?;
        let name = zip_file.name().to_string();
        
        if name.starts_with(maven_prefix) && !zip_file.is_dir() {
            // Extract to libraries directory
            let relative_path = &name[maven_prefix.len()..];
            let dest_path = libraries_dir.join(relative_path);
            
            if let Some(parent) = dest_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            
            // Only extract if doesn't exist
            if !dest_path.exists() {
                let mut output = std::fs::File::create(&dest_path)?;
                std::io::copy(&mut zip_file, &mut output)?;
                extracted_count += 1;
            }
        }
    }
    
    info!("Extracted {} libraries from installer", extracted_count);
    Ok(())
}
