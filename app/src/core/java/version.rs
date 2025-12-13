//! Java version parsing and comparison
//!
//! Similar to Prism Launcher's JavaVersion.cpp

use std::cmp::Ordering;
use std::fmt;
use serde::{Deserialize, Serialize};
use regex::Regex;
use once_cell::sync::Lazy;

/// Represents a parsed Java version with comparison capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JavaVersion {
    /// The original version string
    pub string: String,
    /// Major version number (e.g., 8, 11, 17, 21)
    pub major: u32,
    /// Minor version number
    pub minor: u32,
    /// Security/patch version number
    pub security: u32,
    /// Build number (if present)
    pub build: u32,
    /// Prerelease suffix (e.g., "ea", "beta")
    pub prerelease: Option<String>,
    /// Whether the version was successfully parsed
    pub parseable: bool,
}

// Regex patterns for Java version parsing
static WITH_ONE_PREFIX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"1\.(?P<major>\d+)(\.(?P<minor>\d+))?(_(?P<security>\d+))?(-(?P<prerelease>[a-zA-Z0-9]+))?")
        .expect("Invalid regex")
});

static WITHOUT_ONE_PREFIX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?P<major>\d+)(\.(?P<minor>\d+))?(\.(?P<security>\d+))?(\+(?P<build>\d+))?(-(?P<prerelease>[a-zA-Z0-9]+))?")
        .expect("Invalid regex")
});

impl JavaVersion {
    /// Create a new JavaVersion from components
    pub fn new(major: u32, minor: u32, security: u32, build: u32) -> Self {
        let mut parts = vec![major.to_string()];
        
        if minor > 0 || security > 0 || build > 0 {
            parts.push(minor.to_string());
        }
        if security > 0 || build > 0 {
            parts.push(security.to_string());
        }
        
        let mut string = parts.join(".");
        if build > 0 {
            string.push_str(&format!("+{}", build));
        }
        
        Self {
            string,
            major,
            minor,
            security,
            build,
            prerelease: None,
            parseable: true,
        }
    }
    
    /// Parse a Java version string
    pub fn parse(version_string: &str) -> Self {
        let trimmed = version_string.trim();
        
        if trimmed.is_empty() {
            return Self {
                string: trimmed.to_string(),
                parseable: false,
                ..Default::default()
            };
        }
        
        // Choose pattern based on version format
        let (pattern, captures) = if trimmed.starts_with("1.") {
            (&*WITH_ONE_PREFIX, WITH_ONE_PREFIX.captures(trimmed))
        } else {
            (&*WITHOUT_ONE_PREFIX, WITHOUT_ONE_PREFIX.captures(trimmed))
        };
        
        match captures {
            Some(caps) => {
                let get_num = |name: &str| -> u32 {
                    caps.name(name)
                        .and_then(|m| m.as_str().parse().ok())
                        .unwrap_or(0)
                };
                
                Self {
                    string: trimmed.to_string(),
                    major: get_num("major"),
                    minor: get_num("minor"),
                    security: get_num("security"),
                    build: get_num("build"),
                    prerelease: caps.name("prerelease").map(|m| m.as_str().to_string()),
                    parseable: true,
                }
            }
            None => Self {
                string: trimmed.to_string(),
                parseable: false,
                ..Default::default()
            }
        }
    }
    
    /// Check if this Java version requires PermGen space (Java < 8)
    pub fn requires_permgen(&self) -> bool {
        !self.parseable || self.major < 8
    }
    
    /// Check if this Java defaults to UTF-8 (Java >= 18)
    pub fn defaults_to_utf8(&self) -> bool {
        self.parseable && self.major >= 18
    }
    
    /// Check if this Java is modular (Java >= 9)
    pub fn is_modular(&self) -> bool {
        self.parseable && self.major >= 9
    }
    
    /// Check if this version meets a minimum major version requirement
    pub fn meets_requirement(&self, required_major: u32) -> bool {
        self.parseable && self.major >= required_major
    }
    
    /// Get the major version or 0 if not parseable
    pub fn major_version(&self) -> u32 {
        if self.parseable { self.major } else { 0 }
    }
}

impl fmt::Display for JavaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.string)
    }
}

impl From<&str> for JavaVersion {
    fn from(s: &str) -> Self {
        Self::parse(s)
    }
}

impl From<String> for JavaVersion {
    fn from(s: String) -> Self {
        Self::parse(&s)
    }
}

impl PartialEq for JavaVersion {
    fn eq(&self, other: &Self) -> bool {
        if self.parseable && other.parseable {
            self.major == other.major
                && self.minor == other.minor
                && self.security == other.security
                && self.prerelease == other.prerelease
        } else {
            self.string == other.string
        }
    }
}

impl Eq for JavaVersion {}

impl PartialOrd for JavaVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for JavaVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.parseable && other.parseable {
            // Compare major version
            match self.major.cmp(&other.major) {
                Ordering::Equal => {}
                ord => return ord,
            }
            
            // Compare minor version
            match self.minor.cmp(&other.minor) {
                Ordering::Equal => {}
                ord => return ord,
            }
            
            // Compare security version
            match self.security.cmp(&other.security) {
                Ordering::Equal => {}
                ord => return ord,
            }
            
            // Compare prerelease status
            // A prerelease is less than a release
            match (&self.prerelease, &other.prerelease) {
                (Some(_), None) => Ordering::Less,
                (None, Some(_)) => Ordering::Greater,
                (Some(a), Some(b)) => natural_compare(a, b),
                (None, None) => Ordering::Equal,
            }
        } else {
            // Fall back to string comparison
            natural_compare(&self.string, &other.string)
        }
    }
}

/// Natural string comparison (handles numbers properly)
fn natural_compare(a: &str, b: &str) -> Ordering {
    let mut a_chars = a.chars().peekable();
    let mut b_chars = b.chars().peekable();
    
    loop {
        match (a_chars.peek(), b_chars.peek()) {
            (None, None) => return Ordering::Equal,
            (None, Some(_)) => return Ordering::Less,
            (Some(_), None) => return Ordering::Greater,
            (Some(&a_c), Some(&b_c)) => {
                if a_c.is_ascii_digit() && b_c.is_ascii_digit() {
                    // Extract and compare numbers
                    let a_num: String = a_chars.by_ref().take_while(|c| c.is_ascii_digit()).collect();
                    let b_num: String = b_chars.by_ref().take_while(|c| c.is_ascii_digit()).collect();
                    
                    let a_val: u64 = a_num.parse().unwrap_or(0);
                    let b_val: u64 = b_num.parse().unwrap_or(0);
                    
                    match a_val.cmp(&b_val) {
                        Ordering::Equal => continue,
                        ord => return ord,
                    }
                } else {
                    match a_c.to_lowercase().cmp(b_c.to_lowercase()) {
                        Ordering::Equal => {
                            a_chars.next();
                            b_chars.next();
                        }
                        ord => return ord,
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_java_8() {
        let v = JavaVersion::parse("1.8.0_292");
        assert!(v.parseable);
        assert_eq!(v.major, 8);
        assert_eq!(v.minor, 0);
        assert_eq!(v.security, 292);
    }
    
    #[test]
    fn test_parse_java_17() {
        let v = JavaVersion::parse("17.0.1");
        assert!(v.parseable);
        assert_eq!(v.major, 17);
        assert_eq!(v.minor, 0);
        assert_eq!(v.security, 1);
    }
    
    #[test]
    fn test_parse_java_21_with_build() {
        let v = JavaVersion::parse("21.0.1+12");
        assert!(v.parseable);
        assert_eq!(v.major, 21);
        assert_eq!(v.minor, 0);
        assert_eq!(v.security, 1);
        assert_eq!(v.build, 12);
    }
    
    #[test]
    fn test_version_comparison() {
        let v8 = JavaVersion::parse("1.8.0_292");
        let v17 = JavaVersion::parse("17.0.1");
        let v21 = JavaVersion::parse("21.0.1");
        
        assert!(v8 < v17);
        assert!(v17 < v21);
        assert!(v21 > v8);
    }
    
    #[test]
    fn test_requirements() {
        let v8 = JavaVersion::parse("1.8.0_292");
        let v17 = JavaVersion::parse("17.0.1");
        
        assert!(v8.requires_permgen() == false);
        assert!(v17.is_modular());
        assert!(!v8.is_modular());
    }
}
