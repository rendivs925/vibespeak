//! Configuration handlers

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::presentation::axum_server::state::AppState;

#[derive(Debug, Serialize)]
pub struct ConfigResponse {
    pub commands: Vec<CommandInfo>,
    pub workflows: Vec<WorkflowInfo>,
    pub scripts: Vec<ScriptInfo>,
    pub settings: SettingsInfo,
}

#[derive(Debug, Serialize)]
pub struct CommandInfo {
    pub id: String,
    pub text: String,
    pub action: Value,
    pub category: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct WorkflowInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<Value>,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct ScriptInfo {
    pub id: String,
    pub name: String,
    pub language: String,
    pub content: String,
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct SettingsInfo {
    pub vosk_model_path: String,
    pub sample_rate: f32,
    #[serde(default)]
    pub audio_device: Option<String>,
    pub web_server_port: u16,
    pub enable_tts: bool,
    #[serde(default)]
    pub enable_webrtc: bool,
    pub tailscale_enabled: bool,
}

pub async fn get_config(State(state): State<AppState>) -> Json<ConfigResponse> {
    let config = state.config.read().await;

    let commands: Vec<CommandInfo> = config
        .commands
        .iter()
        .map(|cmd| CommandInfo {
            id: cmd.id.clone(),
            text: cmd.text.clone(),
            action: serde_json::to_value(&cmd.action).unwrap_or(Value::Null),
            category: cmd.category.clone(),
            enabled: cmd.enabled,
        })
        .collect();

    // Map workflows from config
    let workflows: Vec<WorkflowInfo> = config
        .workflows
        .iter()
        .map(|wf| WorkflowInfo {
            id: wf.id.clone(),
            name: wf.name.clone(),
            description: wf.description.clone(),
            steps: wf.steps.iter()
                .map(|s| serde_json::to_value(s).unwrap_or(Value::Null))
                .collect(),
            enabled: wf.enabled,
        })
        .collect();

    // Map scripts from config
    let scripts: Vec<ScriptInfo> = config
        .scripts
        .iter()
        .map(|s| ScriptInfo {
            id: s.id.clone(),
            name: s.name.clone(),
            language: format!("{:?}", s.script_type),
            content: s.content.clone(),
            enabled: s.enabled,
        })
        .collect();

    let settings = SettingsInfo {
        vosk_model_path: config.settings.vosk_model_path.clone(),
        sample_rate: config.settings.sample_rate,
        audio_device: config.settings.audio_device.clone(),
        web_server_port: config.settings.web_server_port,
        enable_tts: config.settings.enable_tts,
        enable_webrtc: config.settings.enable_webrtc,
        tailscale_enabled: config.settings.tailscale_enabled,
    };

    Json(ConfigResponse { commands, workflows, scripts, settings })
}

#[derive(Debug, Deserialize)]
pub struct UpdateConfigRequest {
    pub settings: Option<UpdateSettingsRequest>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub vosk_model_path: Option<String>,
    pub sample_rate: Option<f32>,
    pub enable_tts: Option<bool>,
}

pub async fn update_config(
    State(state): State<AppState>,
    Json(request): Json<UpdateConfigRequest>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    if let Some(settings) = request.settings {
        if let Some(path) = settings.vosk_model_path {
            config.settings.vosk_model_path = path;
        }
        if let Some(rate) = settings.sample_rate {
            config.settings.sample_rate = rate;
        }
        if let Some(tts) = settings.enable_tts {
            config.settings.enable_tts = tts;
        }
    }

    Ok(Json(json!({
        "status": "ok",
        "message": "Configuration updated successfully"
    })))
}

#[derive(Debug, Serialize)]
pub struct TailscaleStatus {
    pub enabled: bool,
    pub connected: bool,
    pub hostname: Option<String>,
    pub port: u16,
    pub error: Option<String>,
}

pub async fn get_tailscale_status(State(state): State<AppState>) -> Json<TailscaleStatus> {
    let config = state.config.read().await;

    Json(TailscaleStatus {
        enabled: config.settings.tailscale_enabled,
        connected: config.settings.tailscale_enabled,
        hostname: config.settings.tailscale_hostname.clone(),
        port: config.settings.web_server_port,
        error: None,
    })
}

#[derive(Debug, Deserialize)]
pub struct UpdateTailscaleRequest {
    pub enabled: bool,
    pub hostname: Option<String>,
    pub port: Option<u16>,
}

pub async fn update_tailscale_config(
    State(state): State<AppState>,
    Json(request): Json<UpdateTailscaleRequest>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    config.settings.tailscale_enabled = request.enabled;
    if let Some(hostname) = request.hostname {
        config.settings.tailscale_hostname = Some(hostname);
    }
    if let Some(port) = request.port {
        config.settings.web_server_port = port;
    }

    Ok(Json(json!({
        "status": "ok",
        "message": "Tailscale configuration updated"
    })))
}
