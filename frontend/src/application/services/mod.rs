//! Application services - Business logic orchestration

use crate::domain::entities::*;
use crate::infrastructure::api_client::ApiClient;
use leptos::*;

/// Application service for managing voice commands
#[derive(Clone)]
pub struct CommandService {
    api_client: ApiClient,
}

impl CommandService {
    pub fn new(api_client: ApiClient) -> Self {
        Self { api_client }
    }

    pub async fn load_commands(&self) -> Result<Vec<Command>, String> {
        let config = self.api_client.get_config().await?;
        Ok(config.commands)
    }

    pub async fn create_command(
        &self,
        text: String,
        action: serde_json::Value,
        category: String,
    ) -> Result<Command, String> {
        let command = Command {
            id: uuid::Uuid::new_v4().to_string(),
            text,
            action,
            category,
            enabled: true,
        };

        // In a full implementation, this would call the API to persist
        // For now, we'll just return the command
        Ok(command)
    }

    pub async fn update_command(&self, _command: Command) -> Result<(), String> {
        // TODO: Implement API call to update command
        Ok(())
    }

    pub async fn delete_command(&self, _command_id: String) -> Result<(), String> {
        // TODO: Implement API call to delete command
        Ok(())
    }
}

/// Application service for managing workflows
#[derive(Clone)]
pub struct WorkflowService {
    api_client: ApiClient,
}

impl WorkflowService {
    pub fn new(api_client: ApiClient) -> Self {
        Self { api_client }
    }

    pub async fn load_workflows(&self) -> Result<Vec<Workflow>, String> {
        let config = self.api_client.get_config().await?;
        Ok(config.workflows)
    }

    pub async fn execute_workflow(&self, workflow_id: String) -> Result<(), String> {
        self.api_client
            .execute_command(&format!("workflow:{}", workflow_id))
            .await?;
        Ok(())
    }
}

/// Application service for managing scripts
#[derive(Clone)]
pub struct ScriptService {
    api_client: ApiClient,
}

impl ScriptService {
    pub fn new(api_client: ApiClient) -> Self {
        Self { api_client }
    }

    pub async fn load_scripts(&self) -> Result<Vec<Script>, String> {
        let config = self.api_client.get_config().await?;
        Ok(config.scripts)
    }

    pub async fn execute_script(&self, script_id: String) -> Result<(), String> {
        self.api_client
            .execute_command(&format!("script:{}", script_id))
            .await?;
        Ok(())
    }
}

/// Application service for managing system settings
#[derive(Clone)]
pub struct SettingsService {
    api_client: ApiClient,
}

impl SettingsService {
    pub fn new(api_client: ApiClient) -> Self {
        Self { api_client }
    }

    pub async fn load_settings(&self) -> Result<SystemSettings, String> {
        let config = self.api_client.get_config().await?;
        Ok(config.settings)
    }

    pub async fn update_settings(&self, _settings: SystemSettings) -> Result<(), String> {
        // TODO: Implement API call to update settings
        Ok(())
    }
}

/// Application service for remote control functionality
#[derive(Clone)]
pub struct RemoteControlService {
    api_client: ApiClient,
}

impl RemoteControlService {
    pub fn new(api_client: ApiClient) -> Self {
        Self { api_client }
    }

    pub async fn execute_command(&self, command: String) -> Result<String, String> {
        let response = self.api_client.execute_command(&command).await?;
        Ok(response
            .result
            .unwrap_or_else(|| "Command executed successfully".to_string()))
    }

    pub async fn type_text(&self, text: String) -> Result<(), String> {
        self.api_client.type_dictation(&text).await?;
        Ok(())
    }

    pub async fn speak_text(&self, text: String) -> Result<(), String> {
        self.api_client.speak_text(&text).await?;
        Ok(())
    }
}

/// Application service for managing application state
#[derive(Clone)]
pub struct AppStateService {
    config: RwSignal<Option<AppConfig>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    status_message: RwSignal<String>,
    command_service: CommandService,
    workflow_service: WorkflowService,
    script_service: ScriptService,
    settings_service: SettingsService,
    remote_control_service: RemoteControlService,
}

impl AppStateService {
    pub fn new(api_client: ApiClient) -> Self {
        Self {
            config: create_rw_signal(None),
            loading: create_rw_signal(false),
            error: create_rw_signal(None),
            status_message: create_rw_signal("Loading system status...".to_string()),
            command_service: CommandService::new(api_client.clone()),
            workflow_service: WorkflowService::new(api_client.clone()),
            script_service: ScriptService::new(api_client.clone()),
            settings_service: SettingsService::new(api_client.clone()),
            remote_control_service: RemoteControlService::new(api_client),
        }
    }

    // Getters for reactive signals
    pub fn config(&self) -> RwSignal<Option<AppConfig>> {
        self.config
    }

    pub fn loading(&self) -> RwSignal<bool> {
        self.loading
    }

    pub fn error(&self) -> RwSignal<Option<String>> {
        self.error
    }

    pub fn status_message(&self) -> RwSignal<String> {
        self.status_message
    }

    // Service accessors
    pub fn command_service(&self) -> &CommandService {
        &self.command_service
    }

    pub fn workflow_service(&self) -> &WorkflowService {
        &self.workflow_service
    }

    pub fn script_service(&self) -> &ScriptService {
        &self.script_service
    }

    pub fn settings_service(&self) -> &SettingsService {
        &self.settings_service
    }

    pub fn remote_control_service(&self) -> &RemoteControlService {
        &self.remote_control_service
    }

    // Business logic methods
    pub async fn load_initial_data(&self) {
        self.loading.set(true);
        self.error.set(None);
        self.status_message
            .set("Loading configuration...".to_string());

        match self.command_service.load_commands().await {
            Ok(commands) => {
                // Load other data and combine into AppConfig
                let workflows = self
                    .workflow_service
                    .load_workflows()
                    .await
                    .unwrap_or_default();
                let scripts = self.script_service.load_scripts().await.unwrap_or_default();
                let settings = self
                    .settings_service
                    .load_settings()
                    .await
                    .unwrap_or_default();

                let config = AppConfig {
                    commands,
                    workflows,
                    scripts,
                    settings,
                };

                self.config.set(Some(config));
                self.status_message.set("System ready".to_string());
            }
            Err(e) => {
                self.error
                    .set(Some(format!("Failed to load configuration: {}", e)));
                self.status_message.set("Error loading system".to_string());
            }
        }

        self.loading.set(false);
    }
}
