//! Health check handlers

use axum::{extract::State, Json};
use serde_json::{json, Value};

use crate::presentation::axum_server::state::AppState;

pub async fn health_check() -> Json<Value> {
    Json(json!({
        "status": "ok",
        "service": "vibespeak",
        "timestamp": chrono::Utc::now().to_rfc3339()
    }))
}

pub async fn ready_check(State(state): State<AppState>) -> Json<Value> {
    let config = state.config.read().await;
    Json(json!({
        "status": "ready",
        "voice_model_loaded": true,
        "config_loaded": true,
        "tailscale_enabled": config.settings.tailscale_enabled
    }))
}
