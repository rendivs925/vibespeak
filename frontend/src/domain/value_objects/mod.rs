//! Domain value objects - Immutable domain concepts

use serde::{Deserialize, Serialize};

/// Value object for command text with validation
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct CommandText(String);

impl CommandText {
    pub fn new(text: String) -> Result<Self, String> {
        let trimmed = text.trim();
        if trimmed.is_empty() {
            return Err("Command text cannot be empty".to_string());
        }
        if trimmed.len() > 500 {
            return Err("Command text too long (max 500 characters)".to_string());
        }
        Ok(Self(trimmed.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for CommandText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Value object for entity IDs
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EntityId(String);

impl EntityId {
    pub fn new(id: String) -> Result<Self, String> {
        if id.is_empty() {
            return Err("Entity ID cannot be empty".to_string());
        }
        if id.len() > 100 {
            return Err("Entity ID too long (max 100 characters)".to_string());
        }
        // Basic validation for UUID format or custom IDs
        if !id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err("Entity ID contains invalid characters".to_string());
        }
        Ok(Self(id))
    }

    pub fn generate() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl std::fmt::Display for EntityId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Value object for port numbers
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Port(u16);

impl Port {
    pub fn new(port: u16) -> Result<Self, String> {
        if port < 1024 {
            return Err("Port numbers below 1024 are reserved".to_string());
        }
        if port > 65535 {
            return Err("Port number too high".to_string());
        }
        Ok(Self(port))
    }

    pub fn value(&self) -> u16 {
        self.0
    }
}

impl std::fmt::Display for Port {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Value object for confidence scores
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Confidence(f32);

impl Confidence {
    pub fn new(confidence: f32) -> Result<Self, String> {
        if !(0.0..=1.0).contains(&confidence) {
            return Err("Confidence must be between 0.0 and 1.0".to_string());
        }
        Ok(Self(confidence))
    }

    pub fn value(&self) -> f32 {
        self.0
    }

    pub fn as_percentage(&self) -> String {
        format!("{:.1}%", self.0 * 100.0)
    }
}

impl std::fmt::Display for Confidence {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3}", self.0)
    }
}
