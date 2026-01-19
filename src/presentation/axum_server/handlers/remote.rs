//! Remote control handlers

use axum::{extract::State, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::presentation::axum_server::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RemoteCommandRequest {
    pub command: String,
    pub parameters: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct RemoteCommandResponse {
    pub status: String,
    pub command: String,
    pub result: Option<String>,
    pub error: Option<String>,
    pub processed: bool,
}

pub async fn execute_remote_command(
    State(state): State<AppState>,
    Json(request): Json<RemoteCommandRequest>,
) -> Result<Json<RemoteCommandResponse>, StatusCode> {
    match state
        .voice_service
        .execute_remote_command(&request.command, request.parameters.as_ref())
        .await
    {
        Ok(result) => Ok(Json(RemoteCommandResponse {
            status: "ok".to_string(),
            command: request.command,
            result: Some(result),
            error: None,
            processed: true,
        })),
        Err(e) => Ok(Json(RemoteCommandResponse {
            status: "error".to_string(),
            command: request.command,
            result: None,
            error: Some(e.to_string()),
            processed: false,
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct RemoteMouseRequest {
    #[serde(rename = "type")]
    pub event_type: String,
    pub x: i32,
    pub y: i32,
    pub timestamp: Option<u64>,
}

pub async fn handle_mouse_event(
    State(state): State<AppState>,
    Json(request): Json<RemoteMouseRequest>,
) -> Json<Value> {
    match state
        .voice_service
        .handle_remote_mouse(&request.event_type, request.x, request.y)
        .await
    {
        Ok(result) => Json(json!({
            "status": "ok",
            "event_type": request.event_type,
            "x": request.x,
            "y": request.y,
            "result": result,
            "message": "Mouse event processed successfully"
        })),
        Err(e) => Json(json!({
            "status": "error",
            "event_type": request.event_type,
            "x": request.x,
            "y": request.y,
            "error": e.to_string(),
            "message": "Failed to process mouse event"
        })),
    }
}

#[derive(Debug, Deserialize)]
pub struct ScreenOfferRequest {
    pub session_id: Option<String>,
}

pub async fn create_screen_offer(
    State(state): State<AppState>,
    Json(request): Json<ScreenOfferRequest>,
) -> Json<Value> {
    let session_id = request
        .session_id
        .unwrap_or_else(|| format!("session_{}", chrono::Utc::now().timestamp()));

    tracing::info!("Creating screen sharing session: {}", session_id);

    match state
        .voice_service
        .start_screen_sharing(session_id.clone())
        .await
    {
        Ok(offer) => Json(json!({
            "status": "ok",
            "session_id": session_id,
            "offer": serde_json::from_str::<Value>(&offer).unwrap_or(Value::Null),
            "message": "Screen sharing session created"
        })),
        Err(e) => {
            tracing::error!("Failed to create screen sharing session: {}", e);
            Json(json!({
                "status": "error",
                "error": e.to_string(),
                "message": "Failed to create screen sharing session"
            }))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct ScreenAnswerRequest {
    pub session_id: String,
    pub answer: Value,
}

pub async fn handle_screen_answer(Json(request): Json<ScreenAnswerRequest>) -> Json<Value> {
    tracing::info!("Screen answer received for session: {}", request.session_id);

    Json(json!({
        "status": "ok",
        "message": "Screen answer received - implementation pending"
    }))
}
