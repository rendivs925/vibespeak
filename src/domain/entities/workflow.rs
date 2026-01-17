use crate::shared::{Error, Result, WorkflowId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub id: WorkflowId,
    pub name: String,
    pub description: String,
    pub trigger: WorkflowTrigger,
    pub steps: Vec<WorkflowStep>,
    pub variables: HashMap<String, Variable>,
    pub error_handling: ErrorStrategy,
    pub enabled: bool,
}

impl Workflow {
    pub fn new(name: String, trigger: WorkflowTrigger) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: String::new(),
            trigger,
            steps: Vec::new(),
            variables: HashMap::new(),
            error_handling: ErrorStrategy::Stop,
            enabled: true,
        }
    }

    pub fn add_step(&mut self, step: WorkflowStep) {
        self.steps.push(step);
    }

    pub fn validate(&self) -> Result<()> {
        if self.name.trim().is_empty() {
            return Err(Error::Domain("Workflow name cannot be empty".to_string()));
        }
        if self.steps.is_empty() {
            return Err(Error::Domain(
                "Workflow must have at least one step".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowTrigger {
    VoiceCommand(String),
    Scheduled(String), // cron expression
    Event(String),     // from integrations
    Manual,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStep {
    ExecuteCommand(String),
    RunScript(ScriptExecution),
    BrowserAction(BrowserAction),
    IntegrationCall(String, serde_json::Value),
    Conditional(Condition, Box<WorkflowStep>, Box<WorkflowStep>),
    Wait(std::time::Duration),
    SetVariable(String, VariableValue),
    UserPrompt(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BrowserAction {
    Navigate(String),
    Click(String),        // CSS selector
    Type(String, String), // selector, text
    WaitForElement(String),
    Screenshot(String),    // filename
    ExecuteScript(String), // JavaScript code
    GetText(String),       // selector -> text
    Scroll(i32, i32),      // x, y offset
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecution {
    pub script_type: crate::shared::ScriptType,
    pub content: String,
    pub arguments: Vec<String>,
    pub timeout: std::time::Duration,
    pub security_level: crate::shared::SecurityLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Condition {
    pub variable: String,
    pub operator: ComparisonOperator,
    pub value: VariableValue,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
    StartsWith,
    EndsWith,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Variable {
    Static(String),
    Dynamic(String),                 // expression to evaluate
    FromContext(String),             // extract from voice context
    FromIntegration(String, String), // from external service
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Json(serde_json::Value),
}

impl std::fmt::Display for VariableValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableValue::String(s) => write!(f, "{}", s),
            VariableValue::Number(n) => write!(f, "{}", n),
            VariableValue::Boolean(b) => write!(f, "{}", b),
            VariableValue::Json(j) => write!(f, "{}", j),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorStrategy {
    Stop,                        // Stop workflow on error
    Continue,                    // Continue to next step
    Retry(u32),                  // Retry N times
    Fallback(Box<WorkflowStep>), // Execute fallback step
}
