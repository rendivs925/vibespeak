use crate::shared::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CommandConfig {
    pub commands: HashMap<String, String>,
}

impl CommandConfig {
    pub fn load_from_file(path: &str) -> Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: CommandConfig = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let contents = toml::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }
}

// New configuration structure for the enhanced system
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemConfig {
    pub commands: Vec<crate::domain::entities::VoiceCommand>,
    pub workflows: Vec<crate::domain::entities::Workflow>,
    pub scripts: Vec<ScriptConfig>,
    pub settings: SystemSettings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemSettings {
    pub vosk_model_path: String,
    pub sample_rate: f32,
    pub audio_device: Option<String>,
    pub web_server_port: u16,
    pub enable_tts: bool,
    pub enable_webrtc: bool,
    pub security_level: crate::shared::SecurityLevel,
    pub tailscale_enabled: bool,
    pub web_server_bind: Option<String>,
    pub tailscale_hostname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScriptConfig {
    pub id: String,
    pub name: String,
    pub script_type: crate::shared::ScriptType,
    pub content: String,
    pub description: String,
    pub enabled: bool,
}

impl Default for SystemSettings {
    fn default() -> Self {
        Self {
            vosk_model_path: "model/vosk-model-small-en-us-0.15".to_string(),
            sample_rate: 16000.0,
            audio_device: None,
            web_server_port: 8080,
            enable_tts: true,
            enable_webrtc: false, // Start with local audio
            security_level: crate::shared::SecurityLevel::Trusted,
            tailscale_enabled: false,
            web_server_bind: Some("127.0.0.1:8080".to_string()),
            tailscale_hostname: Some("vibespeak".to_string()),
        }
    }
}

impl SystemConfig {
    pub fn load_from_file(path: &str) -> Result<Self> {
        if std::path::Path::new(path).exists() {
            let contents = fs::read_to_string(path)?;
            let config: SystemConfig = serde_json::from_str(&contents)?;
            Ok(config)
        } else {
            // Return default config if file doesn't exist
            Ok(Self::default())
        }
    }

    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let contents = serde_json::to_string_pretty(self)?;
        fs::write(path, contents)?;
        Ok(())
    }

    pub fn migrate_from_toml(toml_config: &CommandConfig) -> Self {
        let mut config = Self::default();

        // Convert TOML commands to new format
        config.commands = toml_config
            .commands
            .iter()
            .map(|(text, action)| {
                let mut command = crate::domain::entities::VoiceCommand::new(
                    text.clone(),
                    crate::domain::entities::CommandAction::ShellCommand(action.clone()),
                );
                command.category = "legacy".to_string();
                command
            })
            .collect();

        config
    }
}

impl Default for SystemConfig {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            workflows: Vec::new(),
            scripts: Vec::new(),
            settings: SystemSettings::default(),
        }
    }
}
