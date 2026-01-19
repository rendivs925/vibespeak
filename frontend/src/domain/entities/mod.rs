//! Domain entities - Core business objects

use serde::{Deserialize, Serialize};

/// Core application configuration entity
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub commands: Vec<Command>,
    pub workflows: Vec<Workflow>,
    pub scripts: Vec<Script>,
    pub settings: SystemSettings,
}

/// Voice command entity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Command {
    pub id: String,
    pub text: String,
    pub action: CommandAction,
    pub category: String,
    pub enabled: bool,
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
    pub language: ScriptLanguage,
    pub content: String,
    pub enabled: bool,
}

/// System settings entity
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct SystemSettings {
    pub vosk_model_path: String,
    pub sample_rate: f32,
    pub audio_device: Option<String>,
    pub web_server_port: u16,
    pub enable_tts: bool,
    pub enable_webrtc: bool,
    pub security_level: SecurityLevel,
    pub tailscale_enabled: bool,
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

/// Command action variants
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum CommandAction {
    ShellCommand { command: String },
    Workflow { workflow_id: String },
    Script { script_id: String },
    RemoteControl { action: RemoteAction },
}

impl std::fmt::Display for CommandAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandAction::ShellCommand { command } => write!(f, "Shell: {}", command),
            CommandAction::Workflow { workflow_id } => write!(f, "Workflow: {}", workflow_id),
            CommandAction::Script { script_id } => write!(f, "Script: {}", script_id),
            CommandAction::RemoteControl { action } => write!(f, "Remote: {:?}", action),
        }
    }
}

/// Remote control actions
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum RemoteAction {
    MouseClick { x: i32, y: i32 },
    MouseMove { x: i32, y: i32 },
    KeyPress { key: String },
    TextType { text: String },
}

/// Workflow step entity
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub action: CommandAction,
    pub delay_ms: Option<u64>,
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
