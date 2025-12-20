//! Java installation validation via checker JAR.
//!
//! Oxide Launcher — A Rust-based Minecraft launcher
//! Copyright (C) 2025 Oxide Launcher contributors
//!
//! This file is part of Oxide Launcher.
//!
//! Oxide Launcher is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License, or
//! (at your option) any later version.
//!
//! Oxide Launcher is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License
//! along with this program. If not, see <https://www.gnu.org/licenses/>.

use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;
use serde::{Deserialize, Serialize};
use tracing::debug;
use crate::core::java::install::{JavaInstallation, JavaArch, JavaValidationResult};
use crate::core::java::version::JavaVersion;

/// Timeout for Java checker process (15 seconds like Prism)
const CHECKER_TIMEOUT: Duration = Duration::from_secs(15);

/// Result from running the Java checker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaCheckResult {
    /// Path to the Java executable
    pub path: PathBuf,
    /// Whether the check was successful
    pub valid: bool,
    /// Java version string
    pub java_version: Option<String>,
    /// Java vendor
    pub java_vendor: Option<String>,
    /// OS architecture as reported by Java
    pub os_arch: Option<String>,
    /// Whether this is a 64-bit JVM
    pub is_64bit: bool,
    /// Platform string for Mojang ("32" or "64")
    pub mojang_platform: String,
    /// Error message if check failed
    pub error: Option<String>,
    /// Stdout from the process
    pub stdout: String,
    /// Stderr from the process
    pub stderr: String,
}

impl Default for JavaCheckResult {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            valid: false,
            java_version: None,
            java_vendor: None,
            os_arch: None,
            is_64bit: false,
            mojang_platform: "32".to_string(),
            error: None,
            stdout: String::new(),
            stderr: String::new(),
        }
    }
}

impl JavaCheckResult {
    /// Convert to a JavaInstallation if valid
    pub fn to_installation(&self) -> Option<JavaInstallation> {
        if !self.valid {
            return None;
        }
        
        let version = self.java_version.as_ref()
            .map(|v| JavaVersion::parse(v))
            .unwrap_or_default();
        
        let arch = if self.is_64bit {
            if self.os_arch.as_ref().map(|a| a.contains("aarch64") || a.contains("arm64")).unwrap_or(false) {
                JavaArch::Aarch64
            } else {
                JavaArch::X64
            }
        } else {
            JavaArch::X86
        };
        
        let vendor = self.java_vendor.clone().unwrap_or_else(|| "Unknown".to_string());
        
        Some(JavaInstallation::new(
            self.path.clone(),
            version,
            arch,
            vendor,
        ))
    }
}

/// Java checker that validates installations
pub struct JavaChecker {
    /// Path to the Java executable
    path: PathBuf,
    /// Additional JVM arguments
    args: Vec<String>,
    /// Minimum memory (MB)
    min_mem: Option<u32>,
    /// Maximum memory (MB)
    max_mem: Option<u32>,
}

impl JavaChecker {
    /// Create a new Java checker for the given path
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            args: Vec::new(),
            min_mem: None,
            max_mem: None,
        }
    }
    
    /// Set additional JVM arguments
    #[allow(dead_code)] // Part of public API for future use
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }
    
    /// Set memory limits
    #[allow(dead_code)] // Part of public API for future use
    pub fn with_memory(mut self, min_mb: u32, max_mb: u32) -> Self {
        self.min_mem = Some(min_mb);
        self.max_mem = Some(max_mb);
        self
    }
    
    /// Run the Java check
    pub async fn check(&self) -> JavaCheckResult {
        let mut result = JavaCheckResult {
            path: self.path.clone(),
            ..Default::default()
        };
        
        // Build arguments
        let mut args = Vec::new();
        
        // Add custom args
        for arg in &self.args {
            args.push(arg.clone());
        }
        
        // Add memory settings
        if let Some(min) = self.min_mem {
            args.push(format!("-Xms{}M", min));
        }
        if let Some(max) = self.max_mem {
            args.push(format!("-Xmx{}M", max));
        }
        
        // Use -XshowSettings:all to get detailed info
        args.push("-XshowSettings:all".to_string());
        args.push("-version".to_string());
        
        debug!("Running Java checker: {:?} {:?}", self.path, args);
        
        // Spawn the process
        let mut command = Command::new(&self.path);
        command
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        
        // Hide console window on Windows
        #[cfg(target_os = "windows")]
        {
            #[allow(unused_imports)]
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x08000000;
            command.creation_flags(CREATE_NO_WINDOW);
        }
        
        let process_result = command.spawn();

        let child = match process_result {
            Ok(child) => child,
            Err(e) => {
                result.error = Some(format!("Failed to start Java process: {}", e));
                return result;
            }
        };
        
        // Wait for completion with timeout
        let output = match timeout(CHECKER_TIMEOUT, child.wait_with_output()).await {
            Ok(Ok(output)) => output,
            Ok(Err(e)) => {
                result.error = Some(format!("Java process error: {}", e));
                return result;
            }
            Err(_) => {
                // Timeout occurred - process was consumed by wait_with_output
                result.error = Some("Java check timed out".to_string());
                return result;
            }
        };
        
        result.stdout = String::from_utf8_lossy(&output.stdout).to_string();
        result.stderr = String::from_utf8_lossy(&output.stderr).to_string();
        
        debug!("Java checker stdout: {}", result.stdout);
        debug!("Java checker stderr: {}", result.stderr);
        
        // Check exit status
        if !output.status.success() && output.status.code() != Some(1) {
            result.error = Some(format!("Java process exited with code: {:?}", output.status.code()));
            return result;
        }
        
        // Parse the output (Java prints version info to stderr)
        let combined_output = format!("{}\n{}", result.stdout, result.stderr);
        
        // Parse version, vendor, and architecture
        if let Some(parsed) = parse_java_output(&combined_output) {
            result.java_version = Some(parsed.version);
            result.java_vendor = Some(parsed.vendor);
            result.os_arch = Some(parsed.arch.clone());
            
            // Determine if 64-bit
            result.is_64bit = parsed.arch.contains("64") 
                || parsed.arch.contains("amd64") 
                || parsed.arch.contains("x86_64")
                || parsed.arch.contains("aarch64")
                || parsed.arch.contains("arm64");
            
            result.mojang_platform = if result.is_64bit { "64" } else { "32" }.to_string();
            result.valid = true;
        } else {
            result.error = Some("Failed to parse Java version output".to_string());
        }
        
        result
    }
    
    /// Check and convert to JavaInstallation
    #[allow(dead_code)] // Part of public API for conversion
    pub async fn check_and_install(&self) -> Option<JavaInstallation> {
        let result = self.check().await;
        result.to_installation()
    }
}

/// Parsed Java output information
struct ParsedJavaOutput {
    version: String,
    vendor: String,
    arch: String,
}

/// Parse Java -version and -XshowSettings output
fn parse_java_output(output: &str) -> Option<ParsedJavaOutput> {
    let mut version = None;
    let mut vendor = None;
    let mut arch = None;
    
    for line in output.lines() {
        let line_lower = line.to_lowercase();
        let line_trimmed = line.trim();
        
        // Parse version from java -version output
        if line_lower.contains("version") && line.contains('"') {
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    version = Some(line[start + 1..start + 1 + end].to_string());
                }
            }
        }
        
        // Parse vendor
        if vendor.is_none() {
            if line_lower.contains("temurin") || line_lower.contains("adoptium") {
                vendor = Some("Eclipse Adoptium".to_string());
            } else if line_lower.contains("zulu") || line_lower.contains("azul") {
                vendor = Some("Azul Zulu".to_string());
            } else if line_lower.contains("openjdk runtime") {
                vendor = Some("OpenJDK".to_string());
            } else if line_lower.contains("java(tm)") || line_lower.contains("oracle") {
                vendor = Some("Oracle".to_string());
            } else if line_lower.contains("corretto") {
                vendor = Some("Amazon Corretto".to_string());
            } else if line_lower.contains("microsoft") {
                vendor = Some("Microsoft".to_string());
            } else if line_lower.contains("graalvm") {
                vendor = Some("GraalVM".to_string());
            }
        }
        
        // Parse java.vendor from -XshowSettings output
        if line_trimmed.starts_with("java.vendor =") {
            let value = line_trimmed.trim_start_matches("java.vendor =").trim();
            if vendor.is_none() || vendor.as_ref().map(|v| v == "OpenJDK").unwrap_or(false) {
                vendor = Some(value.to_string());
            }
        }
        
        // Parse os.arch from -XshowSettings output
        if line_trimmed.starts_with("os.arch =") {
            let value = line_trimmed.trim_start_matches("os.arch =").trim();
            arch = Some(value.to_string());
        }
        
        // Parse architecture from other indicators
        if arch.is_none() {
            if line_lower.contains("64-bit") || line_lower.contains("amd64") {
                arch = Some("amd64".to_string());
            } else if line_lower.contains("aarch64") || line_lower.contains("arm64") {
                arch = Some("aarch64".to_string());
            } else if line_lower.contains("32-bit") || line_lower.contains("i386") {
                arch = Some("x86".to_string());
            }
        }
    }
    
    // If we got a version, consider it success
    if let Some(ver) = version {
        Some(ParsedJavaOutput {
            version: ver,
            vendor: vendor.unwrap_or_else(|| "Unknown".to_string()),
            arch: arch.unwrap_or_else(|| "unknown".to_string()),
        })
    } else {
        None
    }
}

/// Check multiple Java installations concurrently
#[allow(dead_code)] // Part of public API for batch validation
pub async fn check_multiple_java(paths: Vec<PathBuf>, max_concurrent: usize) -> Vec<JavaCheckResult> {
    use futures::stream::{self, StreamExt};
    
    let results = stream::iter(paths)
        .map(|path| {
            async move {
                let checker = JavaChecker::new(path);
                checker.check().await
            }
        })
        .buffer_unordered(max_concurrent)
        .collect::<Vec<_>>()
        .await;
    
    results
}

/// Validate a Java installation and return detailed result
#[allow(dead_code)] // Part of public API for detailed validation
pub async fn validate_java_installation(installation: &JavaInstallation) -> JavaValidationResult {
    if !installation.path.exists() {
        return JavaValidationResult::failure(
            installation.clone(),
            "Java executable does not exist".to_string(),
        );
    }
    
    let checker = JavaChecker::new(installation.path.clone());
    let result = checker.check().await;
    
    if result.valid {
        let mut validated = installation.clone();
        
        // Update with checked values
        if let Some(ver) = &result.java_version {
            validated.version = JavaVersion::parse(ver);
        }
        if let Some(vendor) = &result.java_vendor {
            validated.vendor = vendor.clone();
        }
        validated.arch = if result.is_64bit {
            if result.os_arch.as_ref().map(|a| a.contains("aarch64")).unwrap_or(false) {
                JavaArch::Aarch64
            } else {
                JavaArch::X64
            }
        } else {
            JavaArch::X86
        };
        
        JavaValidationResult {
            valid: true,
            installation: validated,
            error: None,
            stdout: result.stdout,
            stderr: result.stderr,
        }
    } else {
        JavaValidationResult {
            valid: false,
            installation: installation.clone(),
            error: result.error,
            stdout: result.stdout,
            stderr: result.stderr,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_java_output() {
        let output = r#"
openjdk version "21.0.1" 2023-10-17
OpenJDK Runtime Environment Temurin-21.0.1+12 (build 21.0.1+12)
OpenJDK 64-Bit Server VM Temurin-21.0.1+12 (build 21.0.1+12, mixed mode, sharing)
"#;
        
        let parsed = parse_java_output(output).unwrap();
        assert_eq!(parsed.version, "21.0.1");
        assert!(parsed.vendor.contains("Temurin") || parsed.vendor.contains("Adoptium"));
    }
    
    #[test]
    fn test_parse_java_8_output() {
        let output = r#"
java version "1.8.0_292"
Java(TM) SE Runtime Environment (build 1.8.0_292-b10)
Java HotSpot(TM) 64-Bit Server VM (build 25.292-b10, mixed mode)
"#;
        
        let parsed = parse_java_output(output).unwrap();
        assert_eq!(parsed.version, "1.8.0_292");
        assert_eq!(parsed.vendor, "Oracle");
    }
}
