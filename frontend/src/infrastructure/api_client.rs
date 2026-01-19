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

    /// Generic PUT request
    pub async fn put<T: Serialize, R: DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &T,
    ) -> Result<R, String> {
        let url = format!("{}{}", self.base_url, endpoint);

        Request::put(&url)
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

    /// Generic DELETE request
    pub async fn delete<R: DeserializeOwned>(&self, endpoint: &str) -> Result<R, String> {
        let url = format!("{}{}", self.base_url, endpoint);

        Request::delete(&url)
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

    /// Process a voice command
    pub async fn process_voice_command(
        &self,
        text: &str,
        confidence: Option<f32>,
    ) -> Result<serde_json::Value, String> {
        let request = serde_json::json!({
            "text": text,
            "confidence": confidence.unwrap_or(0.8)
        });
        self.post("/voice/process", &request).await
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

    /// Test keyboard simulation
    pub async fn test_keyboard(&self) -> Result<serde_json::Value, String> {
        self.get("/dictation/test-keyboard").await
    }
}

/// Screen sharing operations
impl ApiClient {
    /// Create screen sharing offer
    pub async fn create_screen_offer(&self) -> Result<ScreenOfferResponse, String> {
        self.post("/screen/offer", &serde_json::json!({})).await
    }

    /// Handle screen sharing answer
    pub async fn handle_screen_answer(&self, answer: &ScreenAnswerRequest) -> Result<(), String> {
        let _: serde_json::Value = self.post("/screen/answer", answer).await?;
        Ok(())
    }
}

/// Command CRUD operations
impl ApiClient {
    /// List all commands
    pub async fn list_commands(&self) -> Result<ListCommandsResponse, String> {
        self.get("/commands").await
    }

    /// Get a specific command by ID
    pub async fn get_command(&self, id: &str) -> Result<GetCommandResponse, String> {
        self.get(&format!("/commands/{}", id)).await
    }

    /// Create a new command
    pub async fn create_command(
        &self,
        request: &CreateCommandRequest,
    ) -> Result<CrudResponse, String> {
        self.post("/commands", request).await
    }

    /// Update an existing command
    pub async fn update_command(
        &self,
        id: &str,
        request: &UpdateCommandRequest,
    ) -> Result<CrudResponse, String> {
        self.put(&format!("/commands/{}", id), request).await
    }

    /// Delete a command
    pub async fn delete_command(&self, id: &str) -> Result<CrudResponse, String> {
        self.delete(&format!("/commands/{}", id)).await
    }
}
/// Workflow CRUD operations
impl ApiClient {
    /// List all workflows
    pub async fn list_workflows(&self) -> Result<ListWorkflowsResponse, String> {
        self.get("/workflows").await
    }

    /// Get a specific workflow by ID
    pub async fn get_workflow(&self, id: &str) -> Result<GetWorkflowResponse, String> {
        self.get(&format!("/workflows/{}", id)).await
    }

    /// Create a new workflow
    pub async fn create_workflow(
        &self,
        request: &CreateWorkflowRequest,
    ) -> Result<CrudResponse, String> {
        self.post("/workflows", request).await
    }

    /// Update an existing workflow
    pub async fn update_workflow(
        &self,
        id: &str,
        request: &UpdateWorkflowRequest,
    ) -> Result<CrudResponse, String> {
        self.put(&format!("/workflows/{}", id), request).await
    }

    /// Delete a workflow
    pub async fn delete_workflow(&self, id: &str) -> Result<CrudResponse, String> {
        self.delete(&format!("/workflows/{}", id)).await
    }
}

/// Script CRUD operations
impl ApiClient {
    /// List all scripts
    pub async fn list_scripts(&self) -> Result<ListScriptsResponse, String> {
        self.get("/scripts").await
    }

    /// Get a specific script by ID
    pub async fn get_script(&self, id: &str) -> Result<GetScriptResponse, String> {
        self.get(&format!("/scripts/{}", id)).await
    }

    /// Create a new script
    pub async fn create_script(
        &self,
        request: &CreateScriptRequest,
    ) -> Result<CrudResponse, String> {
        self.post("/scripts", request).await
    }

    /// Update an existing script
    pub async fn update_script(
        &self,
        id: &str,
        request: &UpdateScriptRequest,
    ) -> Result<CrudResponse, String> {
        self.put(&format!("/scripts/{}", id), request).await
    }

    /// Delete a script
    pub async fn delete_script(&self, id: &str) -> Result<CrudResponse, String> {
        self.delete(&format!("/scripts/{}", id)).await
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

#[derive(serde::Deserialize)]
pub struct ScreenOfferResponse {
    pub session_id: String,
    pub offer: String,
}

#[derive(Serialize)]
pub struct ScreenAnswerRequest {
    pub session_id: String,
    pub answer: String,
}

// ============= CRUD DTOs =============

#[derive(serde::Deserialize)]
pub struct CrudResponse {
    pub status: String,
    #[serde(default)]
    pub id: Option<String>,
    pub message: String,
}

#[derive(Serialize)]
pub struct CreateCommandRequest {
    pub text: String,
    pub action: serde_json::Value,
    pub category: String,
}

#[derive(Serialize, Default)]
pub struct UpdateCommandRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Serialize)]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: String,
}

#[derive(Serialize, Default)]
pub struct UpdateWorkflowRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

#[derive(Serialize)]
pub struct CreateScriptRequest {
    pub name: String,
    pub language: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Serialize, Default)]
pub struct UpdateScriptRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
}

// ============= CRUD Response DTOs =============

#[derive(serde::Deserialize)]
pub struct ListCommandsResponse {
    pub status: String,
    pub commands: Vec<crate::domain::entities::Command>,
}

#[derive(serde::Deserialize)]
pub struct GetCommandResponse {
    pub status: String,
    pub command: crate::domain::entities::Command,
}

#[derive(serde::Deserialize)]
pub struct ListWorkflowsResponse {
    pub status: String,
    pub workflows: Vec<crate::domain::entities::Workflow>,
}

#[derive(serde::Deserialize)]
pub struct GetWorkflowResponse {
    pub status: String,
    pub workflow: crate::domain::entities::Workflow,
}

#[derive(serde::Deserialize)]
pub struct ListScriptsResponse {
    pub status: String,
    pub scripts: Vec<crate::domain::entities::Script>,
}

#[derive(serde::Deserialize)]
pub struct GetScriptResponse {
    pub status: String,
    pub script: crate::domain::entities::Script,
}
