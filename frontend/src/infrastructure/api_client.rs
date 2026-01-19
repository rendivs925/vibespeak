//! API client infrastructure - External API communication

use crate::domain::entities::*;
use gloo_net::http::Request;
use serde::{de::DeserializeOwned, Serialize};

/// HTTP API client for communicating with the Vibespeak backend
#[derive(Clone)]
pub struct ApiClient {
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
        }
    }

    /// Create a new API client with default base URL
    pub fn new_default() -> Self {
        Self::new("/api")
    }

    /// Generic GET request
    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T, String> {
        let url = format!("{}{}", self.base_url, endpoint);

        Request::get(&url)
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?
            .json::<T>()
            .await
            .map_err(|e| format!("Parse error: {}", e))
    }

    /// Generic POST request
    pub async fn post<T: Serialize, R: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<R, String> {
        let url = format!("{}{}", self.base_url, endpoint);

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

    /// Get application configuration
    pub async fn get_config(&self) -> Result<AppConfig, String> {
        self.get("/config").await
    }

    /// Update application configuration
    pub async fn update_config(&self, config: &AppConfig) -> Result<(), String> {
        let _: serde_json::Value = self.post("/config", config).await?;
        Ok(())
    }

    /// Get Tailscale status
    pub async fn get_tailscale_status(&self) -> Result<TailscaleStatus, String> {
        self.get("/tailscale/status").await
    }

    /// Update Tailscale configuration
    pub async fn update_tailscale_config(&self, config: &serde_json::Value) -> Result<(), String> {
        let _: serde_json::Value = self.post("/tailscale/config", config).await?;
        Ok(())
    }
}

/// Voice synthesis operations
impl ApiClient {
    /// Speak text using TTS
    pub async fn speak_text(&self, text: &str) -> Result<Vec<u8>, String> {
        let url = format!("{}/tts/speak", self.base_url);

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

    /// Test voice synthesis
    pub async fn test_voice(&self) -> Result<(), String> {
        let _: serde_json::Value = self.post("/voice/test", &serde_json::json!({})).await?;
        Ok(())
    }
}

/// Remote control operations
impl ApiClient {
    /// Execute remote command
    pub async fn execute_command(&self, command: &str) -> Result<RemoteCommandResponse, String> {
        self.post(
            "/remote/command",
            &RemoteCommandRequest {
                command: command.to_string(),
                parameters: None,
            },
        )
        .await
    }

    /// Handle mouse event
    pub async fn handle_mouse_event(&self, event: &MouseEvent) -> Result<(), String> {
        let _: serde_json::Value = self.post("/remote/mouse", event).await?;
        Ok(())
    }
}

/// Dictation operations
impl ApiClient {
    /// Start dictation session
    pub async fn start_dictation(&self) -> Result<(), String> {
        let _: serde_json::Value = self
            .post("/dictation/start", &serde_json::json!({}))
            .await?;
        Ok(())
    }

    /// Stop dictation session
    pub async fn stop_dictation(&self) -> Result<(), String> {
        let _: serde_json::Value = self.post("/dictation/stop", &serde_json::json!({})).await?;
        Ok(())
    }

    /// Insert dictation text
    pub async fn insert_dictation(&self, text: &str) -> Result<(), String> {
        let _: serde_json::Value = self
            .post(
                "/dictation/insert",
                &DictationInsertRequest {
                    text: text.to_string(),
                },
            )
            .await?;
        Ok(())
    }

    /// Type dictation text
    pub async fn type_dictation(&self, text: &str) -> Result<DictationTypeResponse, String> {
        self.post(
            "/dictation/type",
            &DictationTypeRequest {
                text: text.to_string(),
                simulate_keyboard: true,
            },
        )
        .await
    }
}

/// Health check operations
impl ApiClient {
    /// Health check
    pub async fn health_check(&self) -> Result<HealthStatus, String> {
        self.get("/health").await
    }

    /// Readiness check
    pub async fn readiness_check(&self) -> Result<HealthStatus, String> {
        self.get("/ready").await
    }
}

/// Request/Response DTOs for API operations
#[derive(Serialize)]
pub struct SpeakRequest {
    pub text: String,
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

#[derive(Serialize)]
pub struct MouseEvent {
    pub x: i32,
    pub y: i32,
    pub button: String,
    pub action: String,
}

#[derive(Serialize)]
pub struct DictationInsertRequest {
    pub text: String,
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

#[derive(serde::Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: String,
    pub version: Option<String>,
}
