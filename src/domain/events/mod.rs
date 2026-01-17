// Domain events for event sourcing

use crate::shared::CommandId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    VoiceCommandExecuted {
        command_id: CommandId,
        text: String,
        success: bool,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    WorkflowStarted {
        workflow_id: String,
        name: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    WorkflowCompleted {
        workflow_id: String,
        success: bool,
        execution_time_ms: u64,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    ScriptExecuted {
        script_id: String,
        language: String,
        success: bool,
        execution_time_ms: u64,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    PluginLoaded {
        plugin_id: String,
        name: String,
        version: String,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    VoiceRecognized {
        text: String,
        confidence: f64,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
}

pub trait DomainEventPublisher: Send + Sync {
    fn publish(&self, event: DomainEvent);
}

// Simple in-memory event publisher for now
pub struct InMemoryEventPublisher {
    events: std::sync::Mutex<Vec<DomainEvent>>,
}

impl InMemoryEventPublisher {
    pub fn new() -> Self {
        Self {
            events: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn get_events(&self) -> Vec<DomainEvent> {
        self.events.lock().unwrap().clone()
    }

    pub fn clear_events(&self) {
        self.events.lock().unwrap().clear();
    }
}

impl DomainEventPublisher for InMemoryEventPublisher {
    fn publish(&self, event: DomainEvent) {
        tracing::info!("Domain event: {:?}", event);
        self.events.lock().unwrap().push(event);
    }
}
