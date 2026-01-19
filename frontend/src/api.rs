//! API client for communicating with the Vibespeak backend

use gloo_net::http::Request;
use serde::{de::DeserializeOwned, Serialize};

use crate::state::{AppConfig, TailscaleStatus};

const API_BASE: &str = "/api";

pub async fn fetch<T: DeserializeOwned>(endpoint: &str) -> Result<T, String> {
    let url = format!("{}{}", API_BASE, endpoint);

    Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?
        .json::<T>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

pub async fn post<T: Serialize, R: DeserializeOwned>(
    endpoint: &str,
    body: &T,
) -> Result<R, String> {
    let url = format!("{}{}", API_BASE, endpoint);

    Request::post(&url)
        .header("Content-Type", "application/json")
        .json(body)
        .map_err(|e| format!("Serialization error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?
        .json::<R>()
        .await
        .map_err(|e| format!("Parse error: {}", e))
}

pub async fn get_config() -> Result<AppConfig, String> {
    fetch("/config").await
}

pub async fn get_tailscale_status() -> Result<TailscaleStatus, String> {
    fetch("/tailscale/status").await
}

#[derive(Serialize)]
pub struct SpeakRequest {
    pub text: String,
}

pub async fn speak(text: &str) -> Result<Vec<u8>, String> {
    let url = format!("{}/tts/speak", API_BASE);

    let response = Request::post(&url)
        .header("Content-Type", "application/json")
        .json(&SpeakRequest {
            text: text.to_string(),
        })
        .map_err(|e| format!("Serialization error: {}", e))?
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    let bytes = response
        .binary()
        .await
        .map_err(|e| format!("Binary read error: {}", e))?;

    Ok(bytes)
}

#[derive(Serialize)]
pub struct RemoteCommandRequest {
    pub command: String,
    pub parameters: Option<serde_json::Value>,
}

#[derive(serde::Deserialize)]
pub struct RemoteCommandResponse {
    pub status: String,
    pub command: String,
    pub result: Option<String>,
    pub error: Option<String>,
    pub processed: bool,
}

pub async fn execute_command(command: &str) -> Result<RemoteCommandResponse, String> {
    post(
        "/remote/command",
        &RemoteCommandRequest {
            command: command.to_string(),
            parameters: None,
        },
    )
    .await
}

#[derive(Serialize)]
pub struct DictationTypeRequest {
    pub text: String,
    #[serde(rename = "simulateKeyboard")]
    pub simulate_keyboard: bool,
}

#[derive(serde::Deserialize)]
pub struct DictationTypeResponse {
    pub success: bool,
    pub characters_typed: usize,
    pub message: String,
    pub error: Option<String>,
}

pub async fn type_dictation(text: &str) -> Result<DictationTypeResponse, String> {
    post(
        "/dictation/type",
        &DictationTypeRequest {
            text: text.to_string(),
            simulate_keyboard: true,
        },
    )
    .await
}
