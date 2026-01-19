//! Application state for the Axum server

use crate::application::services::{VoiceCommandProcessor, VoiceProcessingService};
use crate::infrastructure::config::SystemConfig;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub voice_service: Arc<VoiceProcessingService>,
    pub voice_processor: Arc<VoiceCommandProcessor>,
    pub config: Arc<RwLock<SystemConfig>>,
}

impl AppState {
    pub fn new(
        voice_service: Arc<VoiceProcessingService>,
        voice_processor: Arc<VoiceCommandProcessor>,
        config: SystemConfig,
    ) -> Self {
        Self {
            voice_service,
            voice_processor,
            config: Arc::new(RwLock::new(config)),
        }
    }
}
