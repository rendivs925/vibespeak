use crate::domain::entities::VoiceCommand;
use crate::infrastructure::repositories::CommandRepository;
use crate::shared::{CommandId, Result, Error};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;

// In-memory implementation for now
// TODO: Add persistent storage (SQLite, JSON file, etc.)
pub struct InMemoryCommandRepository {
    commands: RwLock<HashMap<CommandId, VoiceCommand>>,
}

impl InMemoryCommandRepository {
    pub fn new() -> Self {
        Self {
            commands: RwLock::new(HashMap::new()),
        }
    }

    // Initialize with some default commands
    pub fn with_defaults(mut self) -> Self {
        let mut commands = HashMap::new();

        // Add some built-in commands
        let hello_cmd = VoiceCommand::new(
            "hello".to_string(),
            crate::domain::entities::CommandAction::Execute("echo 'Hello! How can I help you?'".to_string())
        );
        commands.insert(hello_cmd.id.clone(), hello_cmd);

        let status_cmd = VoiceCommand::new(
            "status".to_string(),
            crate::domain::entities::CommandAction::Execute("echo 'System is running'".to_string())
        );
        commands.insert(status_cmd.id.clone(), status_cmd);

        *self.commands.write().unwrap() = commands;
        self
    }
}

#[async_trait]
impl CommandRepository for InMemoryCommandRepository {
    async fn save(&self, command: &VoiceCommand) -> Result<()> {
        let mut commands = self.commands.write().unwrap();
        commands.insert(command.id.clone(), command.clone());
        tracing::info!("Saved command: {}", command.text);
        Ok(())
    }

    async fn find_by_id(&self, id: &CommandId) -> Result<Option<VoiceCommand>> {
        let commands = self.commands.read().unwrap();
        Ok(commands.get(id).cloned())
    }

    async fn find_all(&self) -> Result<Vec<VoiceCommand>> {
        let commands = self.commands.read().unwrap();
        Ok(commands.values().cloned().collect())
    }

    async fn delete(&self, id: &CommandId) -> Result<()> {
        let mut commands = self.commands.write().unwrap();
        if commands.remove(id).is_some() {
            tracing::info!("Deleted command: {}", id);
            Ok(())
        } else {
            Err(Error::Infrastructure(format!("Command {} not found", id)))
        }
    }
}