// Application use cases - business operations

use crate::application::services::VoiceCommandProcessor;
use crate::application::dtos::{VoiceCommandRequest, VoiceCommandResponse, WorkflowStepDto};
use crate::shared::{Result, ScriptType, SecurityLevel};
use crate::domain::entities::{WorkflowStep, BrowserAction, Condition, ComparisonOperator, VariableValue};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::sync::atomic::{AtomicU64, Ordering};

/// Global start time for uptime tracking
static START_TIME: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();

fn get_start_time() -> Instant {
    *START_TIME.get_or_init(Instant::now)
}

#[derive(Clone)]
pub struct ProcessVoiceCommandUseCase {
    voice_processor: Arc<VoiceCommandProcessor>,
}

impl ProcessVoiceCommandUseCase {
    pub fn new(voice_processor: Arc<VoiceCommandProcessor>) -> Self {
        Self { voice_processor }
    }

    pub async fn execute(&self, request: VoiceCommandRequest) -> Result<VoiceCommandResponse> {
        let start_time = Instant::now();

        // Process the voice command with the provided text
        // In a real implementation, this would handle actual audio data
        let result = self.voice_processor.process_voice_command(
            crate::shared::AudioSample::new(vec![], 16000, 1)
        ).await?;

        let execution_time = start_time.elapsed();

        Ok(VoiceCommandResponse {
            success: result.success,
            command_id: result.command_executed,
            result: result.execution_result,
            message: Some(format!("Command '{}' processed", request.text)),
            execution_time_ms: execution_time.as_millis() as u64,
        })
    }
}

#[derive(Clone)]
pub struct CreateWorkflowUseCase {
    voice_processor: Arc<VoiceCommandProcessor>,
}

impl CreateWorkflowUseCase {
    pub fn new(voice_processor: Arc<VoiceCommandProcessor>) -> Self {
        Self { voice_processor }
    }

    pub async fn execute(&self, request: crate::application::dtos::WorkflowCreationRequest) -> Result<String> {
        // Convert DTO to domain entity
        let mut workflow = crate::domain::entities::Workflow::new(
            request.name,
            match request.trigger {
                crate::application::dtos::WorkflowTriggerDto::VoiceCommand(cmd) =>
                    crate::domain::entities::WorkflowTrigger::VoiceCommand(cmd),
                crate::application::dtos::WorkflowTriggerDto::Scheduled(cron) =>
                    crate::domain::entities::WorkflowTrigger::Scheduled(cron),
                crate::application::dtos::WorkflowTriggerDto::Event(event) =>
                    crate::domain::entities::WorkflowTrigger::Event(event),
                crate::application::dtos::WorkflowTriggerDto::Manual =>
                    crate::domain::entities::WorkflowTrigger::Manual,
            }
        );

        // Set description if provided
        if let Some(desc) = request.description {
            workflow.description = desc;
        }

        // Convert DTO steps to domain steps
        for step_dto in request.steps {
            if let Some(step) = Self::convert_step_dto(step_dto) {
                workflow.add_step(step);
            }
        }

        self.voice_processor.create_workflow(workflow).await
    }

    fn convert_step_dto(dto: WorkflowStepDto) -> Option<WorkflowStep> {
        match dto {
            WorkflowStepDto::ExecuteCommand(cmd) => Some(WorkflowStep::ExecuteCommand(cmd)),
            WorkflowStepDto::RunScript(script) => {
                let script_type = match script.script_type.to_lowercase().as_str() {
                    "bash" | "sh" => ScriptType::Bash,
                    "python" | "py" => ScriptType::Python,
                    "javascript" | "js" | "node" => ScriptType::JavaScript,
                    "ruby" | "rb" => ScriptType::Ruby,
                    "powershell" | "ps1" => ScriptType::PowerShell,
                    other => ScriptType::Custom(other.to_string()),
                };

                let security_level = match script.security_level.to_lowercase().as_str() {
                    "sandboxed" => SecurityLevel::Sandboxed,
                    "isolated" => SecurityLevel::Isolated,
                    _ => SecurityLevel::Trusted,
                };

                Some(WorkflowStep::RunScript(crate::domain::entities::ScriptExecution {
                    script_type,
                    content: script.content,
                    arguments: vec![],
                    timeout: Duration::from_secs(script.timeout_seconds as u64),
                    security_level,
                    working_directory: None,
                    environment: std::collections::HashMap::new(),
                }))
            }
            WorkflowStepDto::BrowserAction(action) => {
                let browser_action = match action.action_type.to_lowercase().as_str() {
                    "navigate" | "goto" => action.value.map(BrowserAction::Navigate),
                    "click" => action.selector.map(BrowserAction::Click),
                    "type" | "input" => {
                        match (action.selector, action.value) {
                            (Some(sel), Some(val)) => Some(BrowserAction::Type(sel, val)),
                            _ => None,
                        }
                    }
                    "wait" => action.selector.map(BrowserAction::WaitForElement),
                    "screenshot" => action.value.map(BrowserAction::Screenshot),
                    "script" | "execute" => action.value.map(BrowserAction::ExecuteScript),
                    "gettext" => action.selector.map(BrowserAction::GetText),
                    _ => None,
                };
                browser_action.map(WorkflowStep::BrowserAction)
            }
            WorkflowStepDto::IntegrationCall(service, params) => {
                Some(WorkflowStep::IntegrationCall(service, params))
            }
            WorkflowStepDto::Wait(ms) => {
                Some(WorkflowStep::Wait(Duration::from_millis(ms)))
            }
            WorkflowStepDto::SetVariable(name, value) => {
                let var_value = match value {
                    crate::application::dtos::VariableValueDto::String(s) => VariableValue::String(s),
                    crate::application::dtos::VariableValueDto::Number(n) => VariableValue::Number(n),
                    crate::application::dtos::VariableValueDto::Boolean(b) => VariableValue::Boolean(b),
                    crate::application::dtos::VariableValueDto::Json(j) => VariableValue::Json(j),
                };
                Some(WorkflowStep::SetVariable(name, var_value))
            }
            WorkflowStepDto::UserPrompt(prompt) => {
                Some(WorkflowStep::UserPrompt(prompt))
            }
            WorkflowStepDto::Conditional(cond, then_step, else_step) => {
                let operator = match cond.operator.to_lowercase().as_str() {
                    "eq" | "equals" | "==" => ComparisonOperator::Equals,
                    "neq" | "not_equals" | "!=" => ComparisonOperator::NotEquals,
                    "gt" | "greater_than" | ">" => ComparisonOperator::GreaterThan,
                    "lt" | "less_than" | "<" => ComparisonOperator::LessThan,
                    "contains" => ComparisonOperator::Contains,
                    "starts_with" => ComparisonOperator::StartsWith,
                    "ends_with" => ComparisonOperator::EndsWith,
                    _ => ComparisonOperator::Equals,
                };

                let value = match cond.value {
                    serde_json::Value::String(s) => VariableValue::String(s),
                    serde_json::Value::Number(n) => VariableValue::Number(n.as_f64().unwrap_or(0.0)),
                    serde_json::Value::Bool(b) => VariableValue::Boolean(b),
                    other => VariableValue::Json(other),
                };

                let condition = Condition {
                    variable: cond.variable,
                    operator,
                    value,
                };

                match (Self::convert_step_dto(*then_step), Self::convert_step_dto(*else_step)) {
                    (Some(then_s), Some(else_s)) => {
                        Some(WorkflowStep::Conditional(condition, Box::new(then_s), Box::new(else_s)))
                    }
                    _ => None,
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct GetSystemStatusUseCase {
    voice_processor: Arc<VoiceCommandProcessor>,
}

impl GetSystemStatusUseCase {
    pub fn new(voice_processor: Arc<VoiceCommandProcessor>) -> Self {
        // Initialize start time on first use
        get_start_time();
        Self { voice_processor }
    }

    pub async fn execute(&self) -> Result<crate::application::dtos::SystemStatusResponse> {
        // Get basic system information
        let plugins = self.voice_processor.get_available_plugins();

        // Calculate uptime
        let uptime_secs = get_start_time().elapsed().as_secs();

        // Get system metrics
        let (memory_mb, cpu_percent) = Self::get_system_metrics();

        Ok(crate::application::dtos::SystemStatusResponse {
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime: uptime_secs,
            active_plugins: plugins.len(),
            loaded_commands: plugins.iter().map(|p| p.capabilities.len()).sum(),
            active_workflows: 0, // Would come from workflow repository
            memory_usage_mb: memory_mb,
            cpu_usage_percent: cpu_percent,
        })
    }

    fn get_system_metrics() -> (Option<f64>, Option<f64>) {
        // Try to read from /proc for Linux systems
        let memory_mb = Self::get_process_memory();
        let cpu_percent = Self::get_process_cpu();
        (memory_mb, cpu_percent)
    }

    fn get_process_memory() -> Option<f64> {
        // Read /proc/self/statm for memory info (Linux)
        if let Ok(content) = std::fs::read_to_string("/proc/self/statm") {
            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.len() >= 2 {
                // Second value is resident set size in pages
                if let Ok(rss_pages) = parts[1].parse::<u64>() {
                    let page_size = 4096u64; // Common page size
                    let rss_bytes = rss_pages * page_size;
                    return Some(rss_bytes as f64 / (1024.0 * 1024.0));
                }
            }
        }
        None
    }

    fn get_process_cpu() -> Option<f64> {
        // Read /proc/self/stat for CPU info (Linux)
        // This is a simplified implementation - real CPU tracking requires
        // measuring over time intervals
        if let Ok(content) = std::fs::read_to_string("/proc/self/stat") {
            let parts: Vec<&str> = content.split_whitespace().collect();
            if parts.len() >= 15 {
                // Fields 14 and 15 are utime and stime in clock ticks
                let utime: u64 = parts[13].parse().unwrap_or(0);
                let stime: u64 = parts[14].parse().unwrap_or(0);
                let total_time = utime + stime;

                // Get system uptime
                if let Ok(uptime_content) = std::fs::read_to_string("/proc/uptime") {
                    let uptime_parts: Vec<&str> = uptime_content.split_whitespace().collect();
                    if let Some(uptime_str) = uptime_parts.first() {
                        if let Ok(uptime_secs) = uptime_str.parse::<f64>() {
                            // Clock ticks per second (usually 100 on Linux)
                            let clk_tck = 100.0;
                            let process_time_secs = total_time as f64 / clk_tck;
                            let cpu_percent = (process_time_secs / uptime_secs) * 100.0;
                            return Some(cpu_percent.min(100.0));
                        }
                    }
                }
            }
        }
        None
    }
}
