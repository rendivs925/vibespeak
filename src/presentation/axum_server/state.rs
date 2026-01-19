//! Application state for the Axum server

use crate::application::services::VoiceProcessingService;
use crate::infrastructure::config::SystemConfig;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub voice_service: Arc<VoiceProcessingService>,
    pub config: Arc<RwLock<SystemConfig>>,
}

impl AppState {
    pub fn new(voice_service: Arc<VoiceProcessingService>, config: SystemConfig) -> Self {
        Self {
            voice_service,
            config: Arc::new(RwLock::new(config)),
        }
    }
}
