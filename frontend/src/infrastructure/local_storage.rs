//! Local storage infrastructure - Browser local storage adapter

use crate::domain::entities::*;

/// Local storage adapter for persisting data in the browser
#[derive(Clone)]
pub struct LocalStorage {
    // TODO: Implement proper storage backend
}

impl LocalStorage {
    /// Create a new local storage adapter
    pub fn new() -> Result<Self, String> {
        // For now, we'll create a mock storage since web_sys localStorage is complex
        // TODO: Implement proper localStorage when needed
        Err("Local storage not yet implemented".to_string())
    }

    /// Get a value from local storage
    pub fn get(&self, _key: &str) -> Result<Option<String>, String> {
        // TODO: Implement local storage
        Err("Local storage not implemented".to_string())
    }

    /// Set a value in local storage
    pub fn set(&self, _key: &str, _value: &str) -> Result<(), String> {
        // TODO: Implement local storage
        Err("Local storage not implemented".to_string())
    }

    /// Remove a value from local storage
    pub fn remove(&self, _key: &str) -> Result<(), String> {
        // TODO: Implement local storage
        Err("Local storage not implemented".to_string())
    }

    /// Clear all local storage
    pub fn clear(&self) -> Result<(), String> {
        // TODO: Implement local storage
        Err("Local storage not implemented".to_string())
    }
}

/// Specialized local storage for application preferences
impl LocalStorage {
    /// Get user preferences
    pub fn get_user_preferences(&self) -> Result<UserPreferences, String> {
        match self.get("user_preferences")? {
            Some(json) => serde_json::from_str(&json)
                .map_err(|e| format!("Failed to parse preferences: {}", e)),
            None => Ok(UserPreferences::default()),
        }
    }

    /// Save user preferences
    pub fn set_user_preferences(&self, preferences: &UserPreferences) -> Result<(), String> {
        let json = serde_json::to_string(preferences)
            .map_err(|e| format!("Failed to serialize preferences: {}", e))?;
        self.set("user_preferences", &json)
    }

    /// Get UI state
    pub fn get_ui_state(&self) -> Result<UiState, String> {
        match self.get("ui_state")? {
            Some(json) => {
                serde_json::from_str(&json).map_err(|e| format!("Failed to parse UI state: {}", e))
            }
            None => Ok(UiState::default()),
        }
    }

    /// Save UI state
    pub fn set_ui_state(&self, ui_state: &UiState) -> Result<(), String> {
        let json = serde_json::to_string(ui_state)
            .map_err(|e| format!("Failed to serialize UI state: {}", e))?;
        self.set("ui_state", &json)
    }

    /// Get recent commands history
    pub fn get_recent_commands(&self) -> Result<Vec<String>, String> {
        match self.get("recent_commands")? {
            Some(json) => serde_json::from_str(&json)
                .map_err(|e| format!("Failed to parse recent commands: {}", e)),
            None => Ok(Vec::new()),
        }
    }

    /// Add command to recent history
    pub fn add_recent_command(&self, command: String) -> Result<(), String> {
        let mut recent = self.get_recent_commands().unwrap_or_default();

        // Remove if already exists, then add to front
        recent.retain(|c| c != &command);
        recent.insert(0, command);

        // Keep only last 10 commands
        recent.truncate(10);

        let json = serde_json::to_string(&recent)
            .map_err(|e| format!("Failed to serialize recent commands: {}", e))?;
        self.set("recent_commands", &json)
    }
}

/// User preferences entity
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct UserPreferences {
    pub theme: Theme,
    pub language: String,
    pub enable_sound_feedback: bool,
    pub auto_save_interval: u32, // seconds
    pub show_advanced_options: bool,
}

/// UI state entity
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub struct UiState {
    pub sidebar_collapsed: bool,
    pub selected_tab: String,
    pub window_size: (i32, i32),
    pub last_viewed_page: String,
}

/// Theme variants
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, Default)]
pub enum Theme {
    #[default]
    Light,
    Dark,
    System,
}
