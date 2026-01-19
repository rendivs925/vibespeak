//! Application state management

use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub commands: Vec<CommandInfo>,
    pub settings: SettingsInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CommandInfo {
    pub id: String,
    pub text: String,
    pub action: serde_json::Value,
    pub category: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SettingsInfo {
    pub vosk_model_path: String,
    pub sample_rate: f32,
    pub enable_tts: bool,
    pub tailscale_enabled: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TailscaleStatus {
    pub enabled: bool,
    pub connected: bool,
    pub hostname: Option<String>,
    pub port: u16,
    pub error: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AppState {
    pub config: RwSignal<Option<AppConfig>>,
    pub loading: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
    pub status_message: RwSignal<String>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            config: create_rw_signal(None),
            loading: create_rw_signal(false),
            error: create_rw_signal(None),
            status_message: create_rw_signal("Loading system status...".to_string()),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn provide_app_state() {
    provide_context(AppState::new());
}

pub fn use_app_state() -> AppState {
    expect_context::<AppState>()
}
