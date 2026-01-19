//! Dictation handlers

use axum::{http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::process::Command;

use crate::shared::Error;

#[derive(Debug, Deserialize)]
pub struct DictationStartRequest {
    #[serde(rename = "inputType")]
    pub input_type: String,
    pub url: Option<String>,
    pub element_info: Option<Value>,
}

pub async fn start_dictation(Json(request): Json<DictationStartRequest>) -> Json<Value> {
    tracing::info!(
        "Dictation started for input type: {} on URL: {}",
        request.input_type,
        request.url.as_deref().unwrap_or("unknown")
    );

    Json(json!({
        "status": "ok",
        "message": "Dictation started",
        "input_type": request.input_type,
        "session_id": format!("dictation_{}", chrono::Utc::now().timestamp())
    }))
}

pub async fn stop_dictation() -> Json<Value> {
    tracing::info!("Dictation stopped");

    Json(json!({
        "status": "ok",
        "message": "Dictation stopped"
    }))
}

#[derive(Debug, Deserialize)]
pub struct DictationInsertRequest {
    pub text: String,
    #[serde(rename = "inputType")]
    pub input_type: Option<String>,
}

pub async fn insert_dictation(Json(request): Json<DictationInsertRequest>) -> Json<Value> {
    tracing::info!("Inserting dictation text: {} chars", request.text.len());

    Json(json!({
        "status": "ok",
        "message": "Text insertion request received",
        "text_length": request.text.len(),
        "input_type": request.input_type.unwrap_or_else(|| "text".to_string())
    }))
}

#[derive(Debug, Deserialize)]
pub struct DictationTypeRequest {
    pub text: String,
    #[serde(rename = "simulateKeyboard")]
    pub simulate_keyboard: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct DictationBackspaceRequest {
    pub count: usize,
    #[serde(rename = "simulateKeyboard")]
    pub simulate_keyboard: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct DictationTypeResponse {
    pub success: bool,
    pub characters_typed: usize,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// Test keyboard simulation with a simple string
pub async fn test_keyboard() -> Json<serde_json::Value> {
    tracing::info!("Testing keyboard simulation");

    let test_text = "Hello from Vibespeak!";
    match simulate_keyboard_input(test_text).await {
        Ok(()) => {
            tracing::info!("Keyboard test successful");
            Json(serde_json::json!({
                "success": true,
                "message": "Keyboard simulation test passed",
                "test_text": test_text
            }))
        }
        Err(e) => {
            tracing::error!("Keyboard test failed: {}", e);
            Json(serde_json::json!({
                "success": false,
                "message": "Keyboard simulation test failed",
                "error": e.to_string(),
                "test_text": test_text
            }))
        }
    }
}

pub async fn type_dictation(
    Json(request): Json<DictationTypeRequest>,
) -> Result<Json<DictationTypeResponse>, StatusCode> {
    tracing::info!(
        "Typing dictation text: {} chars, simulate: {:?}",
        request.text.len(),
        request.simulate_keyboard
    );

    // Validate input
    if !request.simulate_keyboard.unwrap_or(true) {
        tracing::warn!("Keyboard simulation disabled in request");
        return Ok(Json(DictationTypeResponse {
            success: false,
            characters_typed: 0,
            message: "Keyboard simulation disabled".to_string(),
            error: Some("simulate_keyboard was set to false".to_string()),
        }));
    }

    if request.text.trim().is_empty() {
        tracing::warn!("Received empty text for dictation typing");
        return Ok(Json(DictationTypeResponse {
            success: false,
            characters_typed: 0,
            message: "Cannot type empty text".to_string(),
            error: Some("Text was empty or contained only whitespace".to_string()),
        }));
    }

    match simulate_keyboard_input(&request.text).await {
        Ok(()) => {
            tracing::info!(
                "Successfully typed {} characters via dictation",
                request.text.len()
            );
            Ok(Json(DictationTypeResponse {
                success: true,
                characters_typed: request.text.len(),
                message: "Text typed successfully on desktop".to_string(),
                error: None,
            }))
        }
        Err(e) => {
            tracing::error!("Failed to simulate keyboard input for dictation: {}", e);
            Ok(Json(DictationTypeResponse {
                success: false,
                characters_typed: 0,
                message: "Failed to type text on desktop".to_string(),
                error: Some(e.to_string()),
            }))
        }
    }
}

pub async fn backspace_dictation(
    Json(request): Json<DictationBackspaceRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    tracing::info!("Sending {} backspace keystrokes", request.count);

    // Validate input
    if !request.simulate_keyboard.unwrap_or(true) {
        tracing::warn!("Keyboard simulation disabled in backspace request");
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Keyboard simulation disabled".to_string(),
            "error": "simulate_keyboard was set to false".to_string()
        })));
    }

    if request.count == 0 {
        tracing::warn!("Received backspace request with count 0");
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "Cannot backspace 0 characters".to_string(),
            "error": "count was 0".to_string()
        })));
    }

    match simulate_backspace_input(request.count).await {
        Ok(()) => {
            tracing::info!("Successfully sent {} backspace keystrokes", request.count);
            Ok(Json(serde_json::json!({
                "success": true,
                "characters_backspaced": request.count,
                "message": format!("Backspaced {} characters successfully", request.count)
            })))
        }
        Err(e) => {
            tracing::error!("Failed to simulate backspace input: {}", e);
            Ok(Json(serde_json::json!({
                "success": false,
                "characters_backspaced": 0,
                "message": "Failed to backspace characters".to_string(),
                "error": e.to_string()
            })))
        }
    }
}

async fn simulate_keyboard_input(text: &str) -> Result<(), Error> {
    tracing::info!(
        "Starting keyboard simulation for text: '{}' (length: {})",
        text,
        text.len()
    );

    if text.trim().is_empty() {
        tracing::warn!("Received empty text for keyboard simulation");
        return Ok(());
    }

    // Try uinput first (real kernel-level key events)
    let text_owned = text.to_string();
    let uinput_result = tokio::task::spawn_blocking(move || {
        crate::infrastructure::adapters::keyboard_simulator::type_text_uinput(&text_owned)
    })
    .await;

    match uinput_result {
        Ok(Ok(())) => {
            tracing::info!("Successfully typed {} characters via uinput", text.len());
            return Ok(());
        }
        Ok(Err(e)) => {
            tracing::warn!("uinput failed, falling back to xdotool: {}", e);
        }
        Err(e) => {
            tracing::warn!("uinput task failed, falling back to xdotool: {}", e);
        }
    }

    // Fallback to xdotool
    tracing::info!("Using xdotool fallback for keyboard simulation");

    let escaped_text = text
        .replace("\\", "\\\\")
        .replace("\"", "\\\"")
        .replace("'", "\\'")
        .replace("$", "\\$")
        .replace("`", "\\`");

    let command = format!("DISPLAY=:0 xdotool type \"{}\" && sleep 0.05", escaped_text);

    let output = Command::new("bash")
        .arg("-c")
        .arg(&command)
        .output()
        .await
        .map_err(|e| Error::CommandExecution(format!("Failed to execute xdotool: {}", e)))?;

    if output.status.success() {
        tracing::info!(
            "Successfully typed {} characters on desktop (xdotool)",
            text.len()
        );
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::error!(
            "xdotool failed - status: {}, stdout: '{}', stderr: '{}'",
            output.status,
            stdout.trim(),
            stderr.trim()
        );
        Err(Error::CommandExecution(format!(
            "xdotool failed: {} (stdout: {})",
            stderr, stdout
        )))
    }
}

async fn simulate_backspace_input(count: usize) -> Result<(), Error> {
    tracing::info!("Starting backspace simulation for {} characters", count);

    // Try uinput first (real kernel-level key events)
    let count_owned = count;
    let uinput_result = tokio::task::spawn_blocking(move || {
        crate::infrastructure::adapters::keyboard_simulator::send_backspaces_uinput(count_owned)
    })
    .await;

    match uinput_result {
        Ok(Ok(())) => {
            tracing::info!(
                "Successfully sent {} backspace keystrokes via uinput",
                count
            );
            return Ok(());
        }
        Ok(Err(e)) => {
            tracing::warn!("uinput backspace failed, falling back to xdotool: {}", e);
        }
        Err(e) => {
            tracing::warn!(
                "uinput backspace task failed, falling back to xdotool: {}",
                e
            );
        }
    }

    // Fallback to xdotool
    tracing::info!("Using xdotool fallback for backspace simulation");

    let command = format!("DISPLAY=:0 xdotool key BackSpace");
    for _ in 0..count {
        let output = Command::new("bash")
            .arg("-c")
            .arg(&command)
            .output()
            .await
            .map_err(|e| {
                Error::CommandExecution(format!("Failed to execute xdotool backspace: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            tracing::error!(
                "xdotool backspace failed - status: {}, stdout: '{}', stderr: '{}'",
                output.status,
                stdout.trim(),
                stderr.trim()
            );
            return Err(Error::CommandExecution(format!(
                "xdotool backspace failed: {} (stdout: {})",
                stderr, stdout
            )));
        }

        // Small delay between backspaces
        tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;
    }

    tracing::info!(
        "Successfully sent {} backspace keystrokes on desktop (xdotool)",
        count
    );
    Ok(())
}
