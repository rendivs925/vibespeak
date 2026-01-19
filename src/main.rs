use clap::{Arg, Command};
use std::collections::HashMap;
use std::sync::Arc;
use vibespeak::application::services::{VoiceCommandProcessor, VoiceProcessingService};
use vibespeak::domain::entities::CommandAction;
use vibespeak::domain::services::{
    browser_automation::ChromiumBrowserService,
    plugin::{BuiltinCommandsPlugin, PluginRegistry},
    script_executor::ScriptExecutor,
    workflow_executor::DefaultWorkflowExecutor,
};
use vibespeak::infrastructure::adapters::{
    FuzzyCommandInterpreter, MicrophoneCapture, MicrophoneConfig, TtsAdapter, VoskAdapter,
};
use vibespeak::infrastructure::config::{CommandConfig, SystemConfig};
use vibespeak::presentation::axum_server::AxumServer;
use vibespeak::shared::Result;

const MODEL_PATH: &str = "model/vosk-model-en-us-0.22-lgraph";
const CONFIG_PATH: &str = "config/system.json";
// Web port is now read from configuration

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments
    let matches = Command::new("vibespeak")
        .version("1.0")
        .author("Vibespeak Team")
        .about("Voice-controlled automation system")
        .arg(
            Arg::new("mode")
                .short('m')
                .long("mode")
                .value_parser(["web", "listen"])
                .default_value("web")
                .help("Run mode: 'web' for web interface (default), 'listen' for continuous voice listening")
        )
        .get_matches();

    let mode = matches.get_one::<String>("mode").unwrap();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    tracing::info!(
        "Starting Vibespeak Voice Automation System (mode: {})",
        mode
    );

    // Load or create system configuration
    let mut system_config = if std::path::Path::new(CONFIG_PATH).exists() {
        SystemConfig::load_from_file(CONFIG_PATH)?
    } else {
        tracing::info!("Creating default system configuration");
        let config = SystemConfig::default();
        config.save_to_file(CONFIG_PATH)?;
        config
    };

    // Load commands from TOML if available
    let commands_toml_path = "config/commands.toml";
    if std::path::Path::new(commands_toml_path).exists() {
        tracing::info!("Loading commands from {}", commands_toml_path);
        let toml_config = CommandConfig::load_from_file(commands_toml_path)?;
        system_config = SystemConfig::migrate_from_toml(&toml_config);
        // Save the migrated config
        system_config.save_to_file(CONFIG_PATH)?;
    }

    // Extract command texts for grammar-based recognition
    let command_grammar: Vec<String> = system_config
        .commands
        .iter()
        .map(|cmd| cmd.text.clone())
        .collect();
    tracing::info!(
        "Loaded {} commands for grammar-based recognition",
        command_grammar.len()
    );

    // Initialize infrastructure adapters
    let speech_recognition = Arc::new(VoskAdapter::new(
        &system_config.settings.vosk_model_path,
        system_config.settings.sample_rate,
        command_grammar,
    )?);

    let text_to_speech = Arc::new(TtsAdapter::new()?);

    // Create command interpreter with system commands
    let command_interpreter = Arc::new(FuzzyCommandInterpreter::new(
        system_config
            .commands
            .iter()
            .filter_map(|cmd| {
                match &cmd.action {
                    CommandAction::ShellCommand(command) => {
                        Some((cmd.text.clone(), command.clone()))
                    }
                    _ => None, // Skip other command types for now
                }
            })
            .collect(),
    ));

    // Initialize plugin system
    let mut plugin_registry = PluginRegistry::new();
    plugin_registry
        .register(Box::new(BuiltinCommandsPlugin))
        .unwrap();
    let plugin_registry = Arc::new(plugin_registry);

    // Initialize script executor
    let script_executor = Arc::new(ScriptExecutor::new());

    // Initialize browser automation
    let browser_service = Arc::new(ChromiumBrowserService::new());

    // Initialize workflow executor
    let workflow_executor = Arc::new(DefaultWorkflowExecutor::new(
        script_executor.clone(),
        browser_service.clone(),
    ));

    // Initialize voice command processor
    let voice_processor = VoiceCommandProcessor::new(
        speech_recognition.clone(),
        text_to_speech.clone(),
        command_interpreter,
        script_executor,
        browser_service,
        workflow_executor,
        plugin_registry,
    );

    // Initialize services
    let voice_service = VoiceProcessingService::new(
        speech_recognition,
        text_to_speech,
        Arc::new(FuzzyCommandInterpreter::new(HashMap::new())), // Simplified for legacy compatibility
    );

    voice_service.initialize().await?;
    tracing::info!("Voice services initialized successfully");

    let voice_processor = Arc::new(voice_processor);
    let voice_service = Arc::new(voice_service);

    match mode.as_str() {
        "web" => run_web_mode(voice_service, voice_processor, system_config).await,
        "listen" => run_listen_mode(voice_processor, voice_service, system_config).await,
        _ => unreachable!(),
    }
}

async fn run_web_mode(
    voice_service: Arc<VoiceProcessingService>,
    voice_processor: Arc<VoiceCommandProcessor>,
    system_config: SystemConfig,
) -> Result<()> {
    let web_port = system_config.settings.web_server_port;
    let axum_server = AxumServer::new(
        voice_service.clone(),
        voice_processor.clone(),
        system_config,
    );
    let server_handle = tokio::spawn(async move {
        if let Err(e) = axum_server.run(web_port).await {
            tracing::error!("Axum server error: {}", e);
        }
    });

    tracing::info!(
        "Vibespeak web interface available at http://localhost:{}",
        web_port
    );
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

async fn run_listen_mode(
    voice_processor: Arc<VoiceCommandProcessor>,
    voice_service: Arc<VoiceProcessingService>,
    system_config: SystemConfig,
) -> Result<()> {
    tracing::info!("Initializing microphone for continuous voice listening...");

    // Initialize microphone with speech recognition optimized settings
    tracing::info!("Available microphone devices:");
    for device in MicrophoneCapture::list_devices()? {
        tracing::info!("  - {}", device);
    }

    tracing::info!("Initializing microphone...");
    let mut mic_config = MicrophoneConfig::for_speech_recognition();
    let configured_rate = system_config.settings.sample_rate.round() as u32;
    if configured_rate > 0 {
        mic_config.sample_rate = configured_rate;
    }

    let microphone = match system_config.settings.audio_device.as_deref() {
        Some(device_name) => {
            MicrophoneCapture::with_config_and_device(mic_config, Some(device_name))?
        }
        None => MicrophoneCapture::with_config(mic_config)?,
    };
    tracing::info!("Microphone initialized successfully");

    tracing::info!("Microphone ready. Starting continuous voice listening...");
    tracing::info!("Available commands:");
    // Use the voice processor's command interpreter which has the loaded commands
    if let Ok(commands) = voice_processor.get_available_commands().await {
        for (i, cmd) in commands.iter().enumerate() {
            tracing::info!("  {}. {}", i + 1, cmd);
        }
    } else {
        tracing::warn!("Could not retrieve available commands");
    }
    tracing::info!("Say 'stop listening' or press Ctrl+C to exit.");

    loop {
        tokio::select! {
            // Listen for voice input
            result = microphone.record_until_silence() => {
                match result {
                    Ok(audio_sample) => {
                        tracing::info!("Audio captured: {:.2}s, processing...", audio_sample.duration_seconds());

                        // Process the audio through speech recognition
                        match voice_service.speech_recognition.recognize(audio_sample).await {
                            Ok(recognition_result) => {
                                let recognized_text = recognition_result.text.trim();
                                tracing::info!("Recognized: \"{}\"", recognized_text);

                                if recognized_text.is_empty() {
                                    tracing::info!("No speech detected, continuing to listen...");
                                    continue;
                                }

                                // Check for exit commands
                                if recognized_text.to_lowercase().contains("stop listening") ||
                                   recognized_text.to_lowercase().contains("exit") ||
                                   recognized_text.to_lowercase().contains("quit") {
                                    tracing::info!("Exit command detected, stopping...");
                                    break;
                                }

                                // Process the voice command using the recognized text
                                match voice_processor.process_text_command(recognized_text.to_string(), recognition_result.confidence).await {
                                    Ok(result) => {
                                        tracing::info!("Command processed successfully: {}", result.success);

                                        // Provide audio feedback
                                        let feedback_message = if result.success {
                                            "Command executed successfully"
                                        } else {
                                            "Command failed"
                                        };

                                        if let Err(e) = voice_service.text_to_speech.synthesize(feedback_message, Some("male")).await {
                                            tracing::error!("TTS feedback failed: {}", e);
                                        }
                                    }
                                    Err(e) => {
                                        tracing::error!("Voice command processing failed: {}", e);
                                        if let Err(e) = voice_service.text_to_speech.synthesize("Sorry, I didn't understand that command", Some("male")).await {
                                            tracing::error!("TTS error feedback failed: {}", e);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("Speech recognition failed: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Audio recording failed: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }

            // Listen for Ctrl+C
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Received shutdown signal, stopping voice listening...");
                break;
            }
        }
    }

    tracing::info!("Voice listening mode ended");
    voice_service.shutdown().await?;
    Ok(())
}

async fn run_legacy_cli(voice_service: &VoiceProcessingService) -> Result<()> {
    // Legacy CLI mode using the new architecture
    tracing::info!("Legacy CLI mode - voice recognition ready");

    // Get available commands
    let available_commands = voice_service
        .command_interpreter
        .get_available_commands()
        .await?;
    tracing::info!("Loaded {} commands", available_commands.len());

    // Get available voices
    let voices = voice_service.text_to_speech.get_available_voices().await?;
    tracing::info!("Available TTS voices: {:?}", voices);

    // Initialize TTS
    voice_service.text_to_speech.initialize().await?;

    tracing::info!("System ready. Press Ctrl+C to exit.");

    // Keep running until interrupted
    match tokio::signal::ctrl_c().await {
        Ok(()) => tracing::info!("Received Ctrl+C, shutting down..."),
        Err(err) => tracing::error!("Failed to listen for shutdown signal: {}", err),
    }

    Ok(())
}
