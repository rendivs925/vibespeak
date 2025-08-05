use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct CommandConfig {
    pub commands: HashMap<String, String>,
}

impl CommandConfig {
    pub fn load_from(path: &str) -> anyhow::Result<Self> {
        let contents = fs::read_to_string(path)?;
        let config: CommandConfig = toml::from_str(&contents)?;
        Ok(config)
    }
}
