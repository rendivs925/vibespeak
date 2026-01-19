//! Presentation state management - UI-specific state

use crate::application::services::AppStateService;
use crate::infrastructure::api_client::ApiClient;
use leptos::*;

/// UI state context for the presentation layer
#[derive(Clone)]
pub struct PresentationState {
    app_state_service: AppStateService,
}

impl PresentationState {
    pub fn new(api_client: ApiClient) -> Self {
        Self {
            app_state_service: AppStateService::new(api_client),
        }
    }

    /// Initialize the presentation state context
    pub fn init(api_client: ApiClient) {
        let state = Self::new(api_client);
        provide_context(state);
    }

    /// Get the presentation state from context
    pub fn get() -> Self {
        expect_context::<Self>()
    }

    /// Access the underlying application state service
    pub fn app_state_service(&self) -> &AppStateService {
        &self.app_state_service
    }

    /// Load initial application data
    pub async fn load_initial_data(&self) {
        self.app_state_service.load_initial_data().await;
    }
}

/// Reactive state hooks for common UI patterns
pub fn use_loading() -> RwSignal<bool> {
    PresentationState::get().app_state_service.loading()
}

pub fn use_error() -> RwSignal<Option<String>> {
    PresentationState::get().app_state_service.error()
}

pub fn use_status_message() -> RwSignal<String> {
    PresentationState::get().app_state_service.status_message()
}

pub fn use_app_config() -> RwSignal<Option<crate::domain::entities::AppConfig>> {
    PresentationState::get().app_state_service.config()
}

/// Hook for command operations
pub fn use_commands() -> crate::application::services::CommandService {
    PresentationState::get()
        .app_state_service
        .command_service()
        .clone()
}

/// Hook for workflow operations
pub fn use_workflows() -> crate::application::services::WorkflowService {
    PresentationState::get()
        .app_state_service
        .workflow_service()
        .clone()
}

/// Hook for script operations
pub fn use_scripts() -> crate::application::services::ScriptService {
    PresentationState::get()
        .app_state_service
        .script_service()
        .clone()
}

/// Hook for settings operations
pub fn use_settings() -> crate::application::services::SettingsService {
    PresentationState::get()
        .app_state_service
        .settings_service()
        .clone()
}

/// Hook for remote control operations
pub fn use_remote_control() -> crate::application::services::RemoteControlService {
    PresentationState::get()
        .app_state_service
        .remote_control_service()
        .clone()
}
