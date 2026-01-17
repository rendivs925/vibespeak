// Repository interfaces for data persistence

use crate::domain::entities::{RecognitionSession, VoiceCommand};
use crate::shared::{CommandId, Result, SessionId};
use async_trait::async_trait;

#[async_trait]
pub trait CommandRepository: Send + Sync {
    async fn save(&self, command: &VoiceCommand) -> Result<()>;
    async fn find_by_id(&self, id: &CommandId) -> Result<Option<VoiceCommand>>;
    async fn find_all(&self) -> Result<Vec<VoiceCommand>>;
    async fn delete(&self, id: &CommandId) -> Result<()>;
}

#[async_trait]
pub trait SessionRepository: Send + Sync {
    async fn save(&self, session: &RecognitionSession) -> Result<()>;
    async fn find_by_id(&self, id: &SessionId) -> Result<Option<RecognitionSession>>;
    async fn find_active_sessions(&self) -> Result<Vec<RecognitionSession>>;
}

// Implementations
pub mod command_repository;
pub mod workflow_repository;
pub mod session_repository;

pub use command_repository::{InMemoryCommandRepository, JsonFileCommandRepository};
pub use workflow_repository::{WorkflowRepository, InMemoryWorkflowRepository, JsonFileWorkflowRepository};
pub use session_repository::{InMemorySessionRepository};