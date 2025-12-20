//! Structured logging types for game output capture.
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

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// Source of a log entry
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogSource {
    /// Log from the launcher itself (process events, errors, etc.)
    Launcher,
    /// Log from game stdout
    Game,
    /// Log from game stderr
    StdErr,
}

/// Log level/severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Trace/verbose debugging
    Trace,
    /// Debug information
    Debug,
    /// Normal informational messages
    Info,
    /// Warning messages
    Warning,
    /// Error messages
    Error,
    /// Fatal/crash messages
    Fatal,
}

/// A structured log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Unix timestamp in milliseconds when the log was received
    pub timestamp: u64,
    /// Source of the log (launcher, game stdout, game stderr)
    pub source: LogSource,
    /// Detected log level
    pub level: LogLevel,
    /// The log message content
    pub content: String,
}

impl LogEntry {
    /// Create a new log entry with the current timestamp
    pub fn new(source: LogSource, level: LogLevel, content: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0);
        
        Self {
            timestamp,
            source,
            level,
            content,
        }
    }
    
    /// Create a launcher log entry
    pub fn launcher(level: LogLevel, content: impl Into<String>) -> Self {
        Self::new(LogSource::Launcher, level, content.into())
    }
    
    /// Create a launcher info log entry
    pub fn launcher_info(content: impl Into<String>) -> Self {
        Self::launcher(LogLevel::Info, content)
    }
    
    /// Create a launcher warning log entry
    pub fn launcher_warn(content: impl Into<String>) -> Self {
        Self::launcher(LogLevel::Warning, content)
    }
    
    /// Create a launcher error log entry
    pub fn launcher_error(content: impl Into<String>) -> Self {
        Self::launcher(LogLevel::Error, content)
    }
    
    /// Create a game log entry from stdout, parsing the log level from content
    pub fn game(content: impl Into<String>) -> Self {
        let content = content.into();
        let level = detect_log_level(&content);
        Self::new(LogSource::Game, level, content)
    }
    
    /// Create a game log entry from stderr, parsing the log level from content
    pub fn stderr(content: impl Into<String>) -> Self {
        let content = content.into();
        // stderr defaults to Warning, but may be upgraded to Error/Fatal based on content
        let detected = detect_log_level(&content);
        let level = if detected >= LogLevel::Warning {
            detected
        } else {
            LogLevel::Warning
        };
        Self::new(LogSource::StdErr, level, content)
    }
}

/// Detect the log level from log line content
/// 
/// Matches common Minecraft/Java/Forge/NeoForge log patterns:
/// - `[timestamp] [Thread/LEVEL]` format (Minecraft Log4j)
/// - `[LEVEL]` format
/// - Keyword-based detection (Exception, Error, etc.)
pub fn detect_log_level(line: &str) -> LogLevel {
    let lower = line.to_lowercase();
    
    // Check for fatal/crash indicators first (highest priority)
    if lower.contains("fatal") 
        || lower.contains("crash") 
        || lower.contains("[main/fatal]")
        || lower.contains("/fatal]")
    {
        return LogLevel::Fatal;
    }
    
    // Check for error patterns
    if lower.contains("/error]")
        || lower.contains("[error]")
        || lower.contains(" error:")
        || lower.contains("exception")
        || lower.contains("failed to")
        || lower.contains("could not")
        || lower.contains("unable to")
        || (lower.contains("error") && !lower.contains("errors=0"))
    {
        return LogLevel::Error;
    }
    
    // Check for warning patterns
    if lower.contains("/warn]")
        || lower.contains("[warn]")
        || lower.contains("[warning]")
        || lower.contains(" warn:")
        || lower.contains(" warning:")
    {
        return LogLevel::Warning;
    }
    
    // Check for debug patterns
    if lower.contains("/debug]")
        || lower.contains("[debug]")
    {
        return LogLevel::Debug;
    }
    
    // Check for trace patterns
    if lower.contains("/trace]")
        || lower.contains("[trace]")
    {
        return LogLevel::Trace;
    }
    
    // Default to Info
    LogLevel::Info
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_log_level_minecraft_format() {
        assert_eq!(detect_log_level("[12:34:56] [main/INFO]: Loading..."), LogLevel::Info);
        assert_eq!(detect_log_level("[12:34:56] [main/WARN]: Warning message"), LogLevel::Warning);
        assert_eq!(detect_log_level("[12:34:56] [main/ERROR]: Error occurred"), LogLevel::Error);
        assert_eq!(detect_log_level("[12:34:56] [main/FATAL]: Fatal error!"), LogLevel::Fatal);
        assert_eq!(detect_log_level("[12:34:56] [main/DEBUG]: Debug info"), LogLevel::Debug);
    }
    
    #[test]
    fn test_detect_log_level_exceptions() {
        assert_eq!(detect_log_level("java.lang.NullPointerException"), LogLevel::Error);
        assert_eq!(detect_log_level("Exception in thread \"main\""), LogLevel::Error);
    }
    
    #[test]
    fn test_detect_log_level_forge_patterns() {
        assert_eq!(detect_log_level("[FML/INFO]: Forge mod loading..."), LogLevel::Info);
        assert_eq!(detect_log_level("[FML/WARN]: Mod compatibility warning"), LogLevel::Warning);
    }
}
