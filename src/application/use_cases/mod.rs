// Application use cases - business operations

use crate::application::services::VoiceCommandProcessor;
use crate::application::dtos::{VoiceCommandRequest, VoiceCommandResponse};
use crate::shared::Result;
use std::sync::Arc;

#[derive(Clone)]
pub struct ProcessVoiceCommandUseCase {
    voice_processor: Arc<VoiceCommandProcessor>,
}

impl ProcessVoiceCommandUseCase {
    pub fn new(voice_processor: Arc<VoiceCommandProcessor>) -> Self {
        Self { voice_processor }
    }

    pub async fn execute(&self, request: VoiceCommandRequest) -> Result<VoiceCommandResponse> {
        let start_time = std::time::Instant::now();

        // For now, simulate voice processing without actual audio
        // TODO: Integrate with real audio processing
        let result = self.voice_processor.process_voice_command(
            crate::shared::AudioSample::new(vec![], 16000, 1)
        ).await?;

        let execution_time = start_time.elapsed();

        Ok(VoiceCommandResponse {
            success: result.success,
            command_id: result.command_executed,
            result: result.execution_result,
            message: Some(format!("Command processed successfully")),
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
        let workflow = crate::domain::entities::Workflow::new(
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

        if let Some(desc) = request.description {
            // Note: Workflow entity doesn't have description field yet
            // TODO: Add description to Workflow entity
        }

        // Add steps (simplified conversion)
        for step_dto in request.steps {
            // TODO: Convert DTO steps to domain steps
            // This is a placeholder for now
        }

        self.voice_processor.create_workflow(workflow).await
    }
}

#[derive(Clone)]
pub struct GetSystemStatusUseCase {
    voice_processor: Arc<VoiceCommandProcessor>,
}

impl GetSystemStatusUseCase {
    pub fn new(voice_processor: Arc<VoiceCommandProcessor>) -> Self {
        Self { voice_processor }
    }

    pub async fn execute(&self) -> Result<crate::application::dtos::SystemStatusResponse> {
        // Get basic system information
        let plugins = self.voice_processor.get_available_plugins();

        Ok(crate::application::dtos::SystemStatusResponse {
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime: 0, // TODO: Track actual uptime
            active_plugins: plugins.len(),
            loaded_commands: 0, // TODO: Get from repository
            active_workflows: 0, // TODO: Get from repository
            memory_usage_mb: None, // TODO: System monitoring
            cpu_usage_percent: None, // TODO: System monitoring
        })
    }
}
