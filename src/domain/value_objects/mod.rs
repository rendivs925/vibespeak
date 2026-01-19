// Domain value objects - immutable business values

use crate::shared::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CommandText(String);

impl CommandText {
    pub fn new(text: String) -> Result<Self> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err(Error::Domain("Command text cannot be empty".to_string()));
        }
        if trimmed.len() > 100 {
            return Err(Error::Domain(
                "Command text too long (max 100 characters)".to_string(),
            ));
        }
        Ok(Self(trimmed.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn contains(&self, substring: &str) -> bool {
        self.0.contains(&substring.to_lowercase())
    }
}

impl std::fmt::Display for CommandText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Confidence(f64);

impl Confidence {
    pub fn new(value: f64) -> Result<Self> {
        if !(0.0..=1.0).contains(&value) {
            return Err(Error::Domain(
                "Confidence must be between 0.0 and 1.0".to_string(),
            ));
        }
        Ok(Self(value))
    }

    pub fn as_f64(&self) -> f64 {
        self.0
    }

    pub fn is_high(&self) -> bool {
        self.0 >= 0.8
    }

    pub fn is_medium(&self) -> bool {
        self.0 >= 0.5
    }
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Category(String);

impl Category {
    pub fn new(category: String) -> Result<Self> {
        let trimmed = category.trim();
        if trimmed.is_empty() {
            return Err(Error::Domain("Category cannot be empty".to_string()));
        }
        if trimmed.len() > 50 {
            return Err(Error::Domain(
                "Category too long (max 50 characters)".to_string(),
            ));
        }
        // Only allow alphanumeric characters, spaces, and hyphens
        if !trimmed
            .chars()
            .all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-')
        {
            return Err(Error::Domain(
                "Category contains invalid characters".to_string(),
            ));
        }
        Ok(Self(trimmed.to_lowercase()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Category {
    fn default() -> Self {
        Self("general".to_string())
    }
}

impl std::fmt::Display for Category {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionTime(std::time::Duration);

impl ExecutionTime {
    pub fn new(duration: std::time::Duration) -> Self {
        Self(duration)
    }

    pub fn from_millis(millis: u64) -> Self {
        Self(std::time::Duration::from_millis(millis))
    }

    pub fn from_secs(secs: u64) -> Self {
        Self(std::time::Duration::from_secs(secs))
    }

    pub fn as_duration(&self) -> &std::time::Duration {
        &self.0
    }

    pub fn as_millis(&self) -> u128 {
        self.0.as_millis()
    }

    pub fn is_fast(&self) -> bool {
        self.0 < std::time::Duration::from_millis(100)
    }

    pub fn is_slow(&self) -> bool {
        self.0 > std::time::Duration::from_secs(10)
    }
}

impl std::fmt::Display for ExecutionTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.2}ms", self.0.as_millis())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginVersion {
    major: u32,
    minor: u32,
    patch: u32,
}

impl PluginVersion {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn parse(version: &str) -> Result<Self> {
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return Err(Error::Domain(format!(
                "Invalid version format: {}",
                version
            )));
        }

        let major = parts[0]
            .parse()
            .map_err(|_| Error::Domain("Invalid major version".to_string()))?;
        let minor = parts[1]
            .parse()
            .map_err(|_| Error::Domain("Invalid minor version".to_string()))?;
        let patch = parts[2]
            .parse()
            .map_err(|_| Error::Domain("Invalid patch version".to_string()))?;

        Ok(Self {
            major,
            minor,
            patch,
        })
    }

    pub fn is_compatible(&self, other: &Self) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}

impl std::fmt::Display for PluginVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Port(u16);

impl Port {
    pub fn new(port: u16) -> Result<Self> {
        if port < 1024 {
            return Err(Error::Domain(
                "Port numbers below 1024 are reserved".to_string(),
            ));
        }
        Ok(Self(port))
    }

    pub fn as_u16(&self) -> u16 {
        self.0
    }
}

impl std::fmt::Display for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
