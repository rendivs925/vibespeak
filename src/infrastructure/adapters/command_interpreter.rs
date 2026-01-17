use crate::domain::services::{
    CommandAction, CommandContext, CommandInterpreter, InterpretedCommand,
};
use crate::shared::{Error, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use strsim::jaro_winkler;

pub struct FuzzyCommandInterpreter {
    commands: HashMap<String, String>,
}

impl FuzzyCommandInterpreter {
    pub fn new(commands: HashMap<String, String>) -> Self {
        Self { commands }
    }

    pub fn from_toml_config(config: &crate::infrastructure::config::CommandConfig) -> Self {
        Self::new(config.commands.clone())
    }

    fn find_best_match(&self, input: &str) -> Option<(String, f64)> {
        let mut best_match: Option<(String, f64)> = None;

        for command_text in self.commands.keys() {
            let similarity = jaro_winkler(input, command_text);

            if similarity > 0.85 {
                // High confidence threshold
                match best_match {
                    Some((_, best_sim)) if similarity > best_sim => {
                        best_match = Some((command_text.clone(), similarity));
                    }
                    None => {
                        best_match = Some((command_text.clone(), similarity));
                    }
                    _ => {}
                }
            }
        }

        best_match
    }
}

#[async_trait]
impl CommandInterpreter for FuzzyCommandInterpreter {
    async fn interpret(&self, text: &str, _context: &CommandContext) -> Result<InterpretedCommand> {
        let input = text.trim().to_lowercase();

        // Special case for "type" command (existing functionality)
        if input == "type" {
            return Ok(InterpretedCommand {
                command_id: Some("type".to_string()),
                action: CommandAction::Execute("type".to_string()),
                confidence: 1.0,
                parameters: HashMap::new(),
            });
        }

        // Find best matching command
        if let Some((matched_command, confidence)) = self.find_best_match(&input) {
            if let Some(command_action) = self.commands.get(&matched_command) {
                return Ok(InterpretedCommand {
                    command_id: Some(matched_command),
                    action: CommandAction::Execute(command_action.clone()),
                    confidence,
                    parameters: HashMap::new(),
                });
            }
        }

        // No match found
        Ok(InterpretedCommand {
            command_id: None,
            action: CommandAction::Execute("".to_string()), // Empty action
            confidence: 0.0,
            parameters: HashMap::new(),
        })
    }

    async fn get_available_commands(&self) -> Result<Vec<String>> {
        Ok(self.commands.keys().cloned().collect())
    }
}
