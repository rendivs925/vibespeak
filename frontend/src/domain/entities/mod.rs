//! Domain entities - Core business objects

use serde::{Deserialize, Serialize};

/// Core application configuration entity
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub commands: Vec<Command>,
    #[serde(default)]
    pub workflows: Vec<Workflow>,
    #[serde(default)]
    pub scripts: Vec<Script>,
    #[serde(default)]
    pub settings: SystemSettings,
}

/// Voice command entity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub text: String,
    pub action: serde_json::Value,  // Backend sends this as JSON value
    pub category: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

fn default_enabled() -> bool {
    true
}

impl Command {
    /// Get a display string for the action
    pub fn action_display(&self) -> String {
        if let Some(shell) = self.action.get("ShellCommand") {
            if let Some(cmd) = shell.as_str() {
                return format!("Shell: {}", cmd);
            }
        }
        if let Some(workflow) = self.action.get("Workflow") {
            if let Some(id) = workflow.as_str() {
                return format!("Workflow: {}", id);
            }
        }
        if let Some(script) = self.action.get("Script") {
            if let Some(id) = script.as_str() {
                return format!("Script: {}", id);
            }
        }
        format!("{}", self.action)
    }
}

/// Workflow entity for multi-step automation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Workflow {
    pub id: String,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub enabled: bool,
}

/// Script entity for custom automation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Script {
    pub id: String,
    pub name: String,
    pub language: String,  // Backend sends as string like "Bash", "Python"
    #[serde(default)]
    pub content: String,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

/// System settings entity
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SystemSettings {
    #[serde(default)]
    pub vosk_model_path: String,
    #[serde(default = "default_sample_rate")]
    pub sample_rate: f32,
    #[serde(default)]
    pub audio_device: Option<String>,
    #[serde(default = "default_port")]
    pub web_server_port: u16,
    #[serde(default)]
    pub enable_tts: bool,
    #[serde(default)]
    pub enable_webrtc: bool,
    #[serde(default)]
    pub tailscale_enabled: bool,
}

fn default_sample_rate() -> f32 {
    16000.0
}

fn default_port() -> u16 {
    8080
}

/// Tailscale status entity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TailscaleStatus {
    pub enabled: bool,
    pub connected: bool,
    pub hostname: Option<String>,
    pub port: u16,
    pub error: Option<String>,
}

/// Recognition session entity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecognitionSession {
    pub id: String,
    pub start_time: String,
    pub status: SessionStatus,
    pub results: Vec<RecognitionResult>,
}

/// Recognition result entity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RecognitionResult {
    pub text: String,
    pub confidence: f32,
    pub timestamp: String,
}

/// Workflow step entity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub action: serde_json::Value,
    #[serde(default)]
    pub delay_ms: Option<u64>,
    #[serde(default)]
    pub condition: Option<Condition>,
}

/// Script language variants
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ScriptLanguage {
    Bash,
    Python,
    JavaScript,
    Lua,
}

/// Security level for script execution
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub enum SecurityLevel {
    #[default]
    Trusted,
    Sandboxed,
    Restricted,
}

/// Session status variants
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    Completed,
    Failed,
}

/// Condition for workflow steps
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Condition {
    pub field: String,
    pub operator: ComparisonOperator,
    pub value: serde_json::Value,
}

/// Comparison operators for conditions
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    Contains,
    StartsWith,
    EndsWith,
}
