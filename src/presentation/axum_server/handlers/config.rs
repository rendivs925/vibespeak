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
            steps: wf
                .steps
                .iter()
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

    Json(ConfigResponse {
        commands,
        workflows,
        scripts,
        settings,
    })
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

// ============= COMMAND CRUD =============

#[derive(Debug, Deserialize)]
pub struct CreateCommandRequest {
    pub text: String,
    pub action: Value,
    pub category: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCommandRequest {
    pub text: Option<String>,
    pub action: Option<Value>,
    pub category: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn create_command(
    State(state): State<AppState>,
    Json(request): Json<CreateCommandRequest>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    // Parse the action from JSON
    let action: crate::domain::entities::CommandAction =
        serde_json::from_value(request.action.clone()).map_err(|_| StatusCode::BAD_REQUEST)?;

    let command = crate::domain::entities::VoiceCommand {
        id: uuid::Uuid::new_v4().to_string(),
        text: request.text,
        action,
        category: request.category,
        enabled: true,
        confidence: 0.0,
        metadata: crate::domain::entities::CommandMetadata::default(),
    };

    let id = command.id.clone();
    config.commands.push(command);

    // Save to file
    let _ = config.save_to_file("config/system.json");

    Ok(Json(json!({
        "status": "ok",
        "id": id,
        "message": "Command created successfully"
    })))
}

pub async fn update_command(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(request): Json<UpdateCommandRequest>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    let command = config
        .commands
        .iter_mut()
        .find(|c| c.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(text) = request.text {
        command.text = text;
    }
    if let Some(action) = request.action {
        command.action = serde_json::from_value(action).map_err(|_| StatusCode::BAD_REQUEST)?;
    }
    if let Some(category) = request.category {
        command.category = category;
    }
    if let Some(enabled) = request.enabled {
        command.enabled = enabled;
    }

    // Save to file
    let _ = config.save_to_file("config/system.json");

    Ok(Json(json!({
        "status": "ok",
        "message": "Command updated successfully"
    })))
}

pub async fn delete_command(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    let initial_len = config.commands.len();
    config.commands.retain(|c| c.id != id);

    if config.commands.len() == initial_len {
        return Err(StatusCode::NOT_FOUND);
    }

    // Save to file
    let _ = config.save_to_file("config/system.json");

    Ok(Json(json!({
        "status": "ok",
        "message": "Command deleted successfully"
    })))
}

pub async fn list_commands(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let config = state.config.read().await;

    Ok(Json(json!({
        "status": "ok",
        "commands": config.commands
    })))
}

pub async fn get_command(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let config = state.config.read().await;

    let command = config
        .commands
        .iter()
        .find(|c| c.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(json!({
        "status": "ok",
        "command": command
    })))
}

// ============= WORKFLOW CRUD =============

#[derive(Debug, Deserialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: String,
    pub trigger: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWorkflowRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn create_workflow(
    State(state): State<AppState>,
    Json(request): Json<CreateWorkflowRequest>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    let workflow = crate::domain::entities::Workflow {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        description: request.description,
        trigger: crate::domain::entities::WorkflowTrigger::Manual,
        steps: Vec::new(),
        variables: std::collections::HashMap::new(),
        error_handling: crate::domain::entities::ErrorStrategy::Stop,
        enabled: true,
    };

    let id = workflow.id.clone();
    config.workflows.push(workflow);

    // Save to file
    let _ = config.save_to_file("config/system.json");

    Ok(Json(json!({
        "status": "ok",
        "id": id,
        "message": "Workflow created successfully"
    })))
}

pub async fn update_workflow(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(request): Json<UpdateWorkflowRequest>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    let workflow = config
        .workflows
        .iter_mut()
        .find(|w| w.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(name) = request.name {
        workflow.name = name;
    }
    if let Some(description) = request.description {
        workflow.description = description;
    }
    if let Some(enabled) = request.enabled {
        workflow.enabled = enabled;
    }

    // Save to file
    let _ = config.save_to_file("config/system.json");

    Ok(Json(json!({
        "status": "ok",
        "message": "Workflow updated successfully"
    })))
}

pub async fn delete_workflow(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    let initial_len = config.workflows.len();
    config.workflows.retain(|w| w.id != id);

    if config.workflows.len() == initial_len {
        return Err(StatusCode::NOT_FOUND);
    }

    // Save to file
    let _ = config.save_to_file("config/system.json");

    Ok(Json(json!({
        "status": "ok",
        "message": "Workflow deleted successfully"
    })))
}

pub async fn list_workflows(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let config = state.config.read().await;

    Ok(Json(json!({
        "status": "ok",
        "workflows": config.workflows
    })))
}

pub async fn get_workflow(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let config = state.config.read().await;

    let workflow = config
        .workflows
        .iter()
        .find(|w| w.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(json!({
        "status": "ok",
        "workflow": workflow
    })))
}

// ============= SCRIPT CRUD =============

#[derive(Debug, Deserialize)]
pub struct CreateScriptRequest {
    pub name: String,
    pub language: String,
    pub content: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateScriptRequest {
    pub name: Option<String>,
    pub content: Option<String>,
    pub enabled: Option<bool>,
}

pub async fn create_script(
    State(state): State<AppState>,
    Json(request): Json<CreateScriptRequest>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    let script_type = match request.language.to_lowercase().as_str() {
        "bash" | "shell" => crate::shared::ScriptType::Bash,
        "python" | "py" => crate::shared::ScriptType::Python,
        "javascript" | "js" => crate::shared::ScriptType::JavaScript,
        "ruby" | "rb" => crate::shared::ScriptType::Ruby,
        "powershell" | "ps" => crate::shared::ScriptType::PowerShell,
        _ => crate::shared::ScriptType::Bash,
    };

    let script = crate::infrastructure::config::ScriptConfig {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        script_type,
        content: request.content,
        description: request.description.unwrap_or_default(),
        enabled: true,
    };

    let id = script.id.clone();
    config.scripts.push(script);

    // Save to file
    let _ = config.save_to_file("config/system.json");

    Ok(Json(json!({
        "status": "ok",
        "id": id,
        "message": "Script created successfully"
    })))
}

pub async fn update_script(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(request): Json<UpdateScriptRequest>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    let script = config
        .scripts
        .iter_mut()
        .find(|s| s.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    if let Some(name) = request.name {
        script.name = name;
    }
    if let Some(content) = request.content {
        script.content = content;
    }
    if let Some(enabled) = request.enabled {
        script.enabled = enabled;
    }

    // Save to file
    let _ = config.save_to_file("config/system.json");

    Ok(Json(json!({
        "status": "ok",
        "message": "Script updated successfully"
    })))
}

pub async fn delete_script(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let mut config = state.config.write().await;

    let initial_len = config.scripts.len();
    config.scripts.retain(|s| s.id != id);

    if config.scripts.len() == initial_len {
        return Err(StatusCode::NOT_FOUND);
    }

    // Save to file
    let _ = config.save_to_file("config/system.json");

    Ok(Json(json!({
        "status": "ok",
        "message": "Script deleted successfully"
    })))
}

pub async fn list_scripts(State(state): State<AppState>) -> Result<Json<Value>, StatusCode> {
    let config = state.config.read().await;

    Ok(Json(json!({
        "status": "ok",
        "scripts": config.scripts
    })))
}

pub async fn get_script(
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let config = state.config.read().await;

    let script = config
        .scripts
        .iter()
        .find(|s| s.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(json!({
        "status": "ok",
        "script": script
    })))
}
