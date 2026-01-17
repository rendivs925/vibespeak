mod domain;
mod application;
mod infrastructure;
mod presentation;
mod shared;

use crate::infrastructure::config::SystemConfig;
use crate::infrastructure::adapters::{VoskAdapter, TtsAdapter, FuzzyCommandInterpreter};
use crate::application::services::VoiceProcessingService;
use crate::presentation::web::WebServer;
use crate::shared::{Result, Error};
use std::sync::Arc;

const MODEL_PATH: &str = "model/vosk-model-small-en-us-0.15";
const CONFIG_PATH: &str = "config/system.json";
const WEB_PORT: u16 = 8080;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!("Starting Vibespeak Voice Automation System");

    // Load or create system configuration
    let system_config = if std::path::Path::new(CONFIG_PATH).exists() {
        SystemConfig::load_from_file(CONFIG_PATH)?
    } else {
        tracing::info!("Creating default system configuration");
        let config = SystemConfig::default();
        config.save_to_file(CONFIG_PATH)?;
        config
    };

    // Initialize infrastructure adapters
    let speech_recognition = Arc::new(VoskAdapter::new(
        &system_config.settings.vosk_model_path,
        system_config.settings.sample_rate
    )?);

    let text_to_speech = Arc::new(TtsAdapter::new()?);

    // Create command interpreter with system commands
    let command_interpreter = Arc::new(FuzzyCommandInterpreter::new(
        system_config.commands.iter()
            .map(|cmd| (cmd.text.clone(), format!("{:?}", cmd.action)))
            .collect()
    ));

    // Initialize application services
    let voice_service = VoiceProcessingService::new(
        speech_recognition.clone(),
        text_to_speech.clone(),
        command_interpreter,
    );

    // Initialize services
    voice_service.initialize().await?;
    tracing::info!("Voice services initialized successfully");

    // Start web server for configuration and control
    let voice_service = Arc::new(voice_service);
    let web_server = WebServer::new(voice_service.clone(), system_config);
    let server_handle = tokio::spawn(async move {
        if let Err(e) = web_server.run(WEB_PORT).await {
            tracing::error!("Web server error: {}", e);
        }
    });

    tracing::info!("Vibespeak web interface available at http://localhost:{}", WEB_PORT);
    tracing::info!("System ready. Press Ctrl+C to exit.");

    // Wait for shutdown signal
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            tracing::info!("Received shutdown signal, stopping services...");
            server_handle.abort();
            voice_service.shutdown().await?;
            tracing::info!("Shutdown complete");
        }
        Err(err) => {
            tracing::error!("Failed to listen for shutdown signal: {}", err);
        }
    }

    Ok(())
}

async fn run_legacy_cli(voice_service: &VoiceProcessingService) -> Result<()> {
    // TODO: Implement legacy CLI using new architecture
    // For now, just show that the system is working
    tracing::info!("Legacy CLI mode - voice recognition ready");

    // Get available commands
    let available_commands = voice_service.command_interpreter.get_available_commands().await?;
    tracing::info!("Loaded {} commands", available_commands.len());

    // Get available voices
    let voices = voice_service.text_to_speech.get_available_voices().await?;
    tracing::info!("Available TTS voices: {:?}", voices);

    tracing::info!("System ready. Press Ctrl+C to exit.");

    // Keep running until interrupted
    match tokio::signal::ctrl_c().await {
        Ok(()) => tracing::info!("Received Ctrl+C, shutting down..."),
        Err(err) => tracing::error!("Failed to listen for shutdown signal: {}", err),
    }

    Ok(())
}