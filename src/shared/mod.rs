use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Domain error: {0}")]
    Domain(String),

    #[error("Infrastructure error: {0}")]
    Infrastructure(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Audio error: {0}")]
    Audio(String),

    #[error("Network error: {0}")]
    Network(String),
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error::Infrastructure(err.to_string())
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Infrastructure(format!("IO error: {}", err))
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Configuration(format!("JSON error: {}", err))
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Self {
        Error::Configuration(format!("TOML deserialization error: {}", err))
    }
}

impl From<toml::ser::Error> for Error {
    fn from(err: toml::ser::Error) -> Self {
        Error::Configuration(format!("TOML serialization error: {}", err))
    }
}

// Common types
pub type CommandId = String;
pub type SessionId = String;
pub type WorkflowId = String;
pub type PluginId = String;

// Audio types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioSample {
    pub data: Vec<i16>,
    pub sample_rate: u32,
    pub channels: u8,
}

impl AudioSample {
    pub fn new(data: Vec<i16>, sample_rate: u32, channels: u8) -> Self {
        Self {
            data,
            sample_rate,
            channels,
        }
    }
}

// Security levels for script execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityLevel {
    Sandboxed, // Restricted execution
    Trusted,   // Full access to user-approved paths
    Isolated,  // Container/VM execution
}

impl fmt::Display for SecurityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityLevel::Sandboxed => write!(f, "sandboxed"),
            SecurityLevel::Trusted => write!(f, "trusted"),
            SecurityLevel::Isolated => write!(f, "isolated"),
        }
    }
}

// Script types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScriptType {
    Bash,
    Python,
    JavaScript,
    Ruby,
    PowerShell,
    Custom(String),
}

impl fmt::Display for ScriptType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptType::Bash => write!(f, "bash"),
            ScriptType::Python => write!(f, "python"),
            ScriptType::JavaScript => write!(f, "javascript"),
            ScriptType::Ruby => write!(f, "ruby"),
            ScriptType::PowerShell => write!(f, "powershell"),
            ScriptType::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}
