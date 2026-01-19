use crate::domain::entities::VoiceCommand;
use crate::infrastructure::repositories::CommandRepository;
use crate::shared::{CommandId, Error, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::RwLock;

/// In-memory implementation for quick access
pub struct InMemoryCommandRepository {
    commands: RwLock<HashMap<CommandId, VoiceCommand>>,
}

impl InMemoryCommandRepository {
    pub fn new() -> Self {
        Self {
            commands: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize with some default commands
    pub fn with_defaults(self) -> Self {
        let mut commands = HashMap::new();

        // Add some built-in commands
        let hello_cmd = VoiceCommand::new(
            "hello".to_string(),
            crate::domain::entities::CommandAction::ShellCommand(
                "echo 'Hello! How can I help you?'".to_string(),
            ),
        );
        commands.insert(hello_cmd.id.clone(), hello_cmd);

        let status_cmd = VoiceCommand::new(
            "status".to_string(),
            crate::domain::entities::CommandAction::ShellCommand(
                "echo 'System is running'".to_string(),
            ),
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

/// JSON file-based persistent storage implementation
pub struct JsonFileCommandRepository {
    file_path: PathBuf,
    cache: RwLock<HashMap<CommandId, VoiceCommand>>,
}

impl JsonFileCommandRepository {
    pub fn new(file_path: PathBuf) -> Result<Self> {
        let repo = Self {
            file_path,
            cache: RwLock::new(HashMap::new()),
        };
        repo.load_from_file()?;
        Ok(repo)
    }

    /// Create with default data directory
    pub fn with_default_path() -> Result<Self> {
        let path = PathBuf::from("data/commands.json");
        Self::new(path)
    }

    fn load_from_file(&self) -> Result<()> {
        if self.file_path.exists() {
            let content = std::fs::read_to_string(&self.file_path).map_err(|e| {
                Error::Infrastructure(format!("Failed to read commands file: {}", e))
            })?;
            let commands: Vec<VoiceCommand> = serde_json::from_str(&content).map_err(|e| {
                Error::Infrastructure(format!("Failed to parse commands file: {}", e))
            })?;

            let mut cache = self.cache.write().unwrap();
            for cmd in commands {
                cache.insert(cmd.id.clone(), cmd);
            }
            tracing::info!(
                "Loaded {} commands from {}",
                cache.len(),
                self.file_path.display()
            );
        } else {
            tracing::info!("Commands file does not exist, starting with empty repository");
        }
        Ok(())
    }

    fn save_to_file(&self) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                Error::Infrastructure(format!("Failed to create data directory: {}", e))
            })?;
        }

        let commands = self.cache.read().unwrap();
        let commands_vec: Vec<&VoiceCommand> = commands.values().collect();
        let content = serde_json::to_string_pretty(&commands_vec)
            .map_err(|e| Error::Infrastructure(format!("Failed to serialize commands: {}", e)))?;

        std::fs::write(&self.file_path, content)
            .map_err(|e| Error::Infrastructure(format!("Failed to write commands file: {}", e)))?;

        tracing::debug!(
            "Saved {} commands to {}",
            commands.len(),
            self.file_path.display()
        );
        Ok(())
    }

    /// Initialize with default commands if empty
    pub fn with_defaults(self) -> Result<Self> {
        let is_empty = {
            let cache = self.cache.read().unwrap();
            cache.is_empty()
        };

        if !is_empty {
            return Ok(self);
        }

        // Add default commands
        let hello_cmd = VoiceCommand::new(
            "hello".to_string(),
            crate::domain::entities::CommandAction::ShellCommand(
                "echo 'Hello! How can I help you?'".to_string(),
            ),
        );

        let status_cmd = VoiceCommand::new(
            "status".to_string(),
            crate::domain::entities::CommandAction::ShellCommand(
                "echo 'System is running'".to_string(),
            ),
        );

        let help_cmd = VoiceCommand::new(
            "help".to_string(),
            crate::domain::entities::CommandAction::ShellCommand(
                "echo 'Available commands: hello, status, help, time, date'".to_string(),
            ),
        );

        let time_cmd = VoiceCommand::new(
            "time".to_string(),
            crate::domain::entities::CommandAction::ShellCommand("date +%H:%M:%S".to_string()),
        );

        let date_cmd = VoiceCommand::new(
            "date".to_string(),
            crate::domain::entities::CommandAction::ShellCommand("date +%Y-%m-%d".to_string()),
        );

        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(hello_cmd.id.clone(), hello_cmd);
            cache.insert(status_cmd.id.clone(), status_cmd);
            cache.insert(help_cmd.id.clone(), help_cmd);
            cache.insert(time_cmd.id.clone(), time_cmd);
            cache.insert(date_cmd.id.clone(), date_cmd);
        }

        self.save_to_file()?;
        Ok(self)
    }
}

#[async_trait]
impl CommandRepository for JsonFileCommandRepository {
    async fn save(&self, command: &VoiceCommand) -> Result<()> {
        {
            let mut cache = self.cache.write().unwrap();
            cache.insert(command.id.clone(), command.clone());
        }
        self.save_to_file()?;
        tracing::info!("Saved command: {} (persistent)", command.text);
        Ok(())
    }

    async fn find_by_id(&self, id: &CommandId) -> Result<Option<VoiceCommand>> {
        let cache = self.cache.read().unwrap();
        Ok(cache.get(id).cloned())
    }

    async fn find_all(&self) -> Result<Vec<VoiceCommand>> {
        let cache = self.cache.read().unwrap();
        Ok(cache.values().cloned().collect())
    }

    async fn delete(&self, id: &CommandId) -> Result<()> {
        {
            let mut cache = self.cache.write().unwrap();
            if cache.remove(id).is_none() {
                return Err(Error::Infrastructure(format!("Command {} not found", id)));
            }
        }
        self.save_to_file()?;
        tracing::info!("Deleted command: {} (persistent)", id);
        Ok(())
    }
}
