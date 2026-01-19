//! Axum-based web server for the Vibespeak voice automation system
//!
//! This module provides a modular, clean architecture for the web API:
//! - `state` - Application state management
//! - `routes` - Route definitions
//! - `handlers` - Request handlers organized by feature
//! - `extractors` - Custom extractors for request parsing

pub mod handlers;
pub mod routes;
pub mod state;

use crate::application::services::VoiceProcessingService;
use crate::infrastructure::config::SystemConfig;
use crate::shared::Result;
use state::AppState;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

pub struct AxumServer {
    state: AppState,
}

impl AxumServer {
    pub fn new(voice_service: Arc<VoiceProcessingService>, config: SystemConfig) -> Self {
        Self {
            state: AppState::new(voice_service, config),
        }
    }

    pub async fn run(self, port: u16) -> Result<()> {
        let config = self.state.config.read().await;

        let addr = if config.settings.tailscale_enabled {
            if let Some(ref bind_addr) = config.settings.web_server_bind {
                parse_bind_address(bind_addr)?
            } else {
                SocketAddr::from(([127, 0, 0, 1], port))
            }
        } else {
            SocketAddr::from(([127, 0, 0, 1], port))
        };

        drop(config);

        let app = routes::create_router(self.state);

        tracing::info!("Starting Axum server on {}", addr);

        let listener = TcpListener::bind(addr)
            .await
            .map_err(|e| crate::shared::Error::Configuration(format!("Failed to bind: {}", e)))?;

        axum::serve(listener, app)
            .await
            .map_err(|e| crate::shared::Error::Configuration(format!("Server error: {}", e)))?;

        Ok(())
    }
}

fn parse_bind_address(bind_addr: &str) -> Result<SocketAddr> {
    bind_addr
        .parse()
        .map_err(|_| crate::shared::Error::Configuration(format!("Invalid bind address: {}", bind_addr)))
}
