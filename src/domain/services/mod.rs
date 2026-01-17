use crate::shared::{AudioSample, Result};
use async_trait::async_trait;

use super::entities::RecognitionResult;

#[async_trait]
pub trait SpeechRecognitionService: Send + Sync {
    async fn recognize(&self, audio: AudioSample) -> Result<RecognitionResult>;
    async fn initialize(&self) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;
}

#[async_trait]
pub trait TextToSpeechService: Send + Sync {
    async fn synthesize(&self, text: &str, voice: Option<&str>) -> Result<Vec<i16>>;
    async fn get_available_voices(&self) -> Result<Vec<String>>;
    async fn initialize(&self) -> Result<()>;
    async fn shutdown(&self) -> Result<()>;
}

#[async_trait]
pub trait CommandInterpreter: Send + Sync {
    async fn interpret(&self, text: &str, context: &CommandContext) -> Result<InterpretedCommand>;
    async fn get_available_commands(&self) -> Result<Vec<String>>;
}

#[derive(Debug, Clone)]
pub struct CommandContext {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub previous_commands: Vec<String>,
    pub environment: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct InterpretedCommand {
    pub command_id: Option<String>,
    pub action: CommandAction,
    pub confidence: f64,
    pub parameters: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub enum CommandAction {
    Execute(String),
    Workflow(String),
    Script(String),
    Integration(String),
}

pub mod plugin;
pub mod script_executor;
pub mod browser_automation;
pub mod workflow_executor;
