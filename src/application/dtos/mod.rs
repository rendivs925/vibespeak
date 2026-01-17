// Data transfer objects for application layer communication

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCommandRequest {
    pub text: String,
    pub voice: Option<String>,
    pub context: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceCommandResponse {
    pub success: bool,
    pub command_id: Option<String>,
    pub result: serde_json::Value,
    pub message: Option<String>,
    pub execution_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCreationRequest {
    pub name: String,
    pub description: Option<String>,
    pub trigger: WorkflowTriggerDto,
    pub steps: Vec<WorkflowStepDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowTriggerDto {
    VoiceCommand(String),
    Scheduled(String),
    Event(String),
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStepDto {
    ExecuteCommand(String),
    RunScript(ScriptExecutionDto),
    BrowserAction(BrowserActionDto),
    IntegrationCall(String, serde_json::Value),
    Conditional(ConditionDto, Box<WorkflowStepDto>, Box<WorkflowStepDto>),
    Wait(u64), // milliseconds
    SetVariable(String, VariableValueDto),
    UserPrompt(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecutionDto {
    pub script_type: String,
    pub content: String,
    pub timeout_seconds: u32,
    pub security_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserActionDto {
    pub action_type: String,
    pub selector: Option<String>,
    pub value: Option<String>,
    pub wait_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionDto {
    pub variable: String,
    pub operator: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableValueDto {
    String(String),
    Number(f64),
    Boolean(bool),
    Json(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionResponse {
    pub workflow_id: String,
    pub success: bool,
    pub steps_executed: usize,
    pub total_steps: usize,
    pub execution_time_ms: u64,
    pub results: Vec<serde_json::Value>,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfoResponse {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub loaded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatusResponse {
    pub version: String,
    pub uptime: u64,
    pub active_plugins: usize,
    pub loaded_commands: usize,
    pub active_workflows: usize,
    pub memory_usage_mb: Option<f64>,
    pub cpu_usage_percent: Option<f64>,
}
