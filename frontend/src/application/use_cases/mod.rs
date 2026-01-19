//! Use cases - Application-specific business logic

use crate::application::services::*;
use crate::domain::entities::*;

/// Use case for managing voice commands
pub struct ManageCommandsUseCase {
    command_service: CommandService,
}

impl ManageCommandsUseCase {
    pub fn new(command_service: CommandService) -> Self {
        Self { command_service }
    }

    pub async fn get_all_commands(&self) -> Result<Vec<Command>, String> {
        self.command_service.load_commands().await
    }

    pub async fn create_new_command(
        &self,
        text: String,
        action: serde_json::Value,
        category: String,
    ) -> Result<Command, String> {
        // Validate input
        if text.trim().is_empty() {
            return Err("Command text cannot be empty".to_string());
        }

        self.command_service
            .create_command(text, action, category)
            .await
    }

    pub async fn update_existing_command(
        &self,
        mut command: Command,
        new_text: Option<String>,
        new_action: Option<serde_json::Value>,
    ) -> Result<Command, String> {
        if let Some(text) = new_text {
            if text.trim().is_empty() {
                return Err("Command text cannot be empty".to_string());
            }
            command.text = text;
        }

        if let Some(action) = new_action {
            command.action = action;
        }

        self.command_service.update_command(command.clone()).await?;
        Ok(command)
    }

    pub async fn remove_command(&self, command_id: String) -> Result<(), String> {
        if command_id.is_empty() {
            return Err("Command ID cannot be empty".to_string());
        }

        self.command_service.delete_command(command_id).await
    }
}

/// Use case for executing voice commands
pub struct ExecuteCommandUseCase {
    command_service: CommandService,
    workflow_service: WorkflowService,
    script_service: ScriptService,
}

impl ExecuteCommandUseCase {
    pub fn new(
        command_service: CommandService,
        workflow_service: WorkflowService,
        script_service: ScriptService,
    ) -> Self {
        Self {
            command_service,
            workflow_service,
            script_service,
        }
    }

    pub async fn execute_by_text(&self, command_text: String) -> Result<String, String> {
        let commands = self.command_service.load_commands().await?;

        // Find command by text (in a real implementation, this would use fuzzy matching)
        let command = commands
            .into_iter()
            .find(|c| c.text.to_lowercase() == command_text.to_lowercase())
            .ok_or_else(|| format!("Command not found: {}", command_text))?;

        self.execute_command(command).await
    }

    pub async fn execute_by_id(&self, command_id: String) -> Result<String, String> {
        let commands = self.command_service.load_commands().await?;

        let command = commands
            .into_iter()
            .find(|c| c.id == command_id)
            .ok_or_else(|| format!("Command not found: {}", command_id))?;

        self.execute_command(command).await
    }

    async fn execute_command(&self, command: Command) -> Result<String, String> {
        // Handle command action based on JSON structure
        if let Some(shell_cmd) = command.action.get("ShellCommand").and_then(|v| v.as_str()) {
            return Ok(format!("Executed shell command: {}", shell_cmd));
        }
        if let Some(workflow_id) = command.action.get("Workflow").and_then(|v| v.as_str()) {
            self.workflow_service.execute_workflow(workflow_id.to_string()).await?;
            return Ok("Workflow executed successfully".to_string());
        }
        if let Some(script_id) = command.action.get("Script").and_then(|v| v.as_str()) {
            self.script_service.execute_script(script_id.to_string()).await?;
            return Ok("Script executed successfully".to_string());
        }

        Ok(format!("Executed command: {}", command.action))
    }
}

/// Use case for managing system settings
pub struct ManageSettingsUseCase {
    settings_service: SettingsService,
}

impl ManageSettingsUseCase {
    pub fn new(settings_service: SettingsService) -> Self {
        Self { settings_service }
    }

    pub async fn get_current_settings(&self) -> Result<SystemSettings, String> {
        self.settings_service.load_settings().await
    }

    pub async fn update_vosk_model(&self, model_path: String) -> Result<SystemSettings, String> {
        let mut settings = self.settings_service.load_settings().await?;
        settings.vosk_model_path = model_path;
        self.settings_service
            .update_settings(settings.clone())
            .await?;
        Ok(settings)
    }

    pub async fn toggle_tts(&self, enabled: bool) -> Result<SystemSettings, String> {
        let mut settings = self.settings_service.load_settings().await?;
        settings.enable_tts = enabled;
        self.settings_service
            .update_settings(settings.clone())
            .await?;
        Ok(settings)
    }

    pub async fn update_web_port(&self, port: u16) -> Result<SystemSettings, String> {
        if port < 1024 {
            return Err("Port numbers below 1024 are reserved".to_string());
        }

        let mut settings = self.settings_service.load_settings().await?;
        settings.web_server_port = port;
        self.settings_service
            .update_settings(settings.clone())
            .await?;
        Ok(settings)
    }
}

/// Use case for remote control operations
pub struct RemoteControlUseCase {
    remote_control_service: RemoteControlService,
}

impl RemoteControlUseCase {
    pub fn new(remote_control_service: RemoteControlService) -> Self {
        Self {
            remote_control_service,
        }
    }

    pub async fn execute_system_command(&self, command: String) -> Result<String, String> {
        if command.trim().is_empty() {
            return Err("Command cannot be empty".to_string());
        }

        self.remote_control_service.execute_command(command).await
    }

    pub async fn type_text_input(&self, text: String) -> Result<(), String> {
        if text.is_empty() {
            return Err("Text cannot be empty".to_string());
        }

        self.remote_control_service.type_text(text).await
    }

    pub async fn speak_message(&self, message: String) -> Result<(), String> {
        if message.trim().is_empty() {
            return Err("Message cannot be empty".to_string());
        }

        self.remote_control_service.speak_text(message).await
    }
}
