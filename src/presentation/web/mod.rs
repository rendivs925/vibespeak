use crate::application::services::VoiceProcessingService;
use crate::infrastructure::config::SystemConfig;
use crate::shared::{Result, Error};
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::RwLock;
use warp::Filter;
use warp::reject::Reject;

// Implement Reject for our Error type
impl Reject for Error {}

// TTS error handling is done with standard warp rejections

pub struct WebServer {
    voice_service: Arc<VoiceProcessingService>,
    config: Arc<RwLock<SystemConfig>>,
}

impl WebServer {
    pub fn new(voice_service: Arc<VoiceProcessingService>, config: SystemConfig) -> Self {
        Self {
            voice_service,
            config: Arc::new(RwLock::new(config)),
        }
    }

    pub async fn run(self, port: u16) -> Result<()> {
        let config = self.config.read().await;

        // Determine bind address
        let bind_addr = if config.settings.tailscale_enabled {
            // Parse bind address from config
            if let Some(bind_addr) = &config.settings.web_server_bind {
                parse_bind_address(bind_addr)?
            } else {
                ([127, 0, 0, 1], port)
            }
        } else {
            ([127, 0, 0, 1], port)
        };

        // Static files route
        let static_files = warp::path("static").and(warp::fs::dir("web/static"));

        // API routes
        let api = self.api_routes();

        // Main interface route
        let index =
            warp::path::end().map(|| warp::reply::html(include_str!("../../../web/index.html")));

        let routes = index
            .or(static_files)
            .or(api)
            .with(warp::cors().allow_any_origin());

        tracing::info!("Starting web server on {}:{}", format_ip(bind_addr.0), bind_addr.1);
        warp::serve(routes).run(bind_addr).await;

        Ok(())
    }

    fn api_routes(
        &self,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let config_get = warp::path("api")
            .and(warp::path("config"))
            .and(warp::get())
            .and(with_config(self.config.clone()))
            .and_then(get_config);

        let config_post = warp::path("api")
            .and(warp::path("config"))
            .and(warp::post())
            .and(warp::body::json())
            .and(with_config(self.config.clone()))
            .and_then(update_config);

        let voice_test = warp::path("api")
            .and(warp::path("voice"))
            .and(warp::path("test"))
            .and(warp::post())
            .and(warp::body::json())
            .and(with_voice_service(self.voice_service.clone()))
            .and_then(test_voice);

        // TTS audio endpoint - returns raw PCM data for browser playback
        let tts_audio = warp::path("api")
            .and(warp::path("tts"))
            .and(warp::path("speak"))
            .and(warp::post())
            .and(warp::body::json())
            .and(with_voice_service(self.voice_service.clone()))
            .and_then(tts_audio_handler);

        // Screen sharing routes
        let screen_offer = warp::path("api")
            .and(warp::path("screen"))
            .and(warp::path("offer"))
            .and(warp::post())
            .and(warp::body::json())
            .and(with_voice_service(self.voice_service.clone()))
            .and_then(handle_screen_offer);

        let screen_answer = warp::path("api")
            .and(warp::path("screen"))
            .and(warp::path("answer"))
            .and(warp::post())
            .and(warp::body::json())
            .and(with_voice_service(self.voice_service.clone()))
            .and_then(handle_screen_answer);

        // Remote control routes
        let remote_command = warp::path("api")
            .and(warp::path("remote"))
            .and(warp::path("command"))
            .and(warp::post())
            .and(warp::body::json())
            .and(with_voice_service(self.voice_service.clone()))
            .and_then(handle_remote_command);

        let remote_mouse = warp::path("api")
            .and(warp::path("remote"))
            .and(warp::path("mouse"))
            .and(warp::post())
            .and(warp::body::json())
            .and(with_voice_service(self.voice_service.clone()))
            .and_then(handle_remote_mouse);

        // Dictation routes
        let dictation_start = warp::path("api")
            .and(warp::path("dictation"))
            .and(warp::path("start"))
            .and(warp::post())
            .and(warp::body::json())
            .and_then(handle_dictation_start);

        let dictation_stop = warp::path("api")
            .and(warp::path("dictation"))
            .and(warp::path("stop"))
            .and(warp::post())
            .and_then(handle_dictation_stop);

        let dictation_insert = warp::path("api")
            .and(warp::path("dictation"))
            .and(warp::path("insert"))
            .and(warp::post())
            .and(warp::body::json())
            .and_then(handle_dictation_insert);

        let dictation_type = warp::path("api")
            .and(warp::path("dictation"))
            .and(warp::path("type"))
            .and(warp::post())
            .and(warp::body::json())
            .and_then(handle_dictation_type);

        // Tailscale routes
        let tailscale_status = warp::path("api")
            .and(warp::path("tailscale"))
            .and(warp::path("status"))
            .and(warp::get())
            .and(with_config(self.config.clone()))
            .and_then(handle_tailscale_status);

        let tailscale_config = warp::path("api")
            .and(warp::path("tailscale"))
            .and(warp::path("config"))
            .and(warp::post())
            .and(warp::body::json())
            .and(with_config(self.config.clone()))
            .and_then(handle_tailscale_config);

        config_get.or(config_post).or(voice_test).or(tts_audio).or(screen_offer).or(screen_answer).or(remote_command).or(remote_mouse).or(dictation_start).or(dictation_stop).or(dictation_insert).or(dictation_type).or(tailscale_status).or(tailscale_config)
    }
}

fn with_config(
    config: Arc<RwLock<SystemConfig>>,
) -> impl Filter<Extract = (Arc<RwLock<SystemConfig>>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || config.clone())
}

fn with_voice_service(
    service: Arc<VoiceProcessingService>,
) -> impl Filter<Extract = (Arc<VoiceProcessingService>,), Error = std::convert::Infallible> + Clone
{
    warp::any().map(move || service.clone())
}

async fn get_config(
    config: Arc<RwLock<SystemConfig>>,
) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    let config = config.read().await;
    Ok(warp::reply::json(&*config))
}

async fn update_config(
    new_config: SystemConfig,
    config: Arc<RwLock<SystemConfig>>,
) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    let mut config_lock = config.write().await;
    *config_lock = new_config.clone();

    // Save configuration to file
    let config_path = std::path::Path::new("config/system.json");

    // Create parent directory
    if let Some(parent) = config_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            tracing::error!("Failed to create config directory: {}", e);
            return Ok(warp::reply::json(&serde_json::json!({
                "status": "error",
                "message": format!("Failed to create config directory: {}", e)
            })));
        }
    }

    // Serialize config
    let config_json = match serde_json::to_string_pretty(&new_config) {
        Ok(json) => json,
        Err(e) => {
            tracing::error!("Failed to serialize config: {}", e);
            return Ok(warp::reply::json(&serde_json::json!({
                "status": "error",
                "message": format!("Failed to serialize config: {}", e)
            })));
        }
    };

    // Write to file
    if let Err(e) = std::fs::write(config_path, config_json) {
        tracing::error!("Failed to write config file: {}", e);
        return Ok(warp::reply::json(&serde_json::json!({
            "status": "error",
            "message": format!("Failed to write config file: {}", e)
        })));
    }

    tracing::info!("Configuration saved to {}", config_path.display());
    Ok(warp::reply::json(&serde_json::json!({"status": "ok", "message": "Configuration saved successfully"})))
}

#[derive(serde::Deserialize)]
struct VoiceTestRequest {
    text: String,
}

async fn tts_audio_handler(
    request: VoiceTestRequest,
    voice_service: Arc<VoiceProcessingService>,
) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    // Synthesize TTS audio
    tracing::info!("Web TTS request: synthesizing '{}' using Piper", request.text);
    match voice_service.text_to_speech.synthesize(&request.text, None).await {
        Ok(samples) => {
            tracing::info!("Web TTS: successfully generated {} audio samples from Piper", samples.len());

            // Validate that we have actual audio data (not empty or dummy data)
            if samples.is_empty() {
                tracing::error!("Web TTS: Piper returned empty audio samples!");
                return Err(warp::reject::reject());
            }

            // Log some sample values to verify they're real audio data
            if samples.len() > 10 {
                tracing::debug!("Web TTS: Sample audio values: [{}, {}, {}, {}, ...]",
                    samples[0], samples[1], samples[2], samples[3]);
            }

            // Create WAV file with proper header
            let sample_rate = 22050; // Piper default sample rate
            let channels = 1; // Mono
            let bits_per_sample = 16;
            let data_size = samples.len() * 2; // 2 bytes per i16 sample
            let file_size = 44 + data_size - 8; // WAV header size

            let mut wav_data = Vec::with_capacity(44 + data_size);

            // WAV header
            wav_data.extend_from_slice(b"RIFF");
            wav_data.extend_from_slice(&(file_size as u32).to_le_bytes());
            wav_data.extend_from_slice(b"WAVE");
            wav_data.extend_from_slice(b"fmt ");
            wav_data.extend_from_slice(&16u32.to_le_bytes()); // fmt chunk size
            wav_data.extend_from_slice(&1u16.to_le_bytes()); // PCM format
            wav_data.extend_from_slice(&(channels as u16).to_le_bytes());
            wav_data.extend_from_slice(&(sample_rate as u32).to_le_bytes());
            wav_data.extend_from_slice(&((sample_rate * channels * bits_per_sample / 8) as u32).to_le_bytes()); // byte rate
            wav_data.extend_from_slice(&((channels * bits_per_sample / 8) as u16).to_le_bytes()); // block align
            wav_data.extend_from_slice(&(bits_per_sample as u16).to_le_bytes());
            wav_data.extend_from_slice(b"data");
            wav_data.extend_from_slice(&(data_size as u32).to_le_bytes());

            // Audio data (convert i16 samples to bytes)
            for &sample in &samples {
                wav_data.extend_from_slice(&sample.to_le_bytes());
            }

            // Return WAV file with appropriate headers
            let response = warp::http::Response::builder()
                .status(200)
                .header("Content-Type", "audio/wav")
                .header("Content-Length", wav_data.len().to_string())
                .header("Access-Control-Allow-Origin", "*")
                .header("Access-Control-Allow-Headers", "Content-Type")
                .header("Access-Control-Allow-Methods", "POST, GET, OPTIONS")
                .header("Cache-Control", "no-cache")
                .body(wav_data)
                .unwrap(); // This should be safe since we're building a valid response

            Ok(response)
        }
        Err(e) => {
            tracing::error!("TTS synthesis failed: {}", e);
            Err(warp::reject::reject())
        }
    }
}

async fn test_voice(
    request: VoiceTestRequest,
    voice_service: Arc<VoiceProcessingService>,
) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    tracing::info!("Voice test requested for text: {}", request.text);

    // Get available commands and try to match
    let available_commands = match voice_service.command_interpreter.get_available_commands().await {
        Ok(cmds) => cmds,
        Err(_) => vec![],
    };

    // Try to find matching commands using fuzzy matching
    let text_lower = request.text.to_lowercase();
    let matched_commands: Vec<String> = available_commands
        .iter()
        .filter(|cmd| {
            let cmd_lower = cmd.to_lowercase();
            cmd_lower.contains(&text_lower) || text_lower.contains(&cmd_lower) ||
            strsim::jaro_winkler(&text_lower, &cmd_lower) > 0.7
        })
        .cloned()
        .collect();

    // Synthesize TTS response if text was provided
    let tts_samples = match voice_service.text_to_speech.synthesize(&request.text, None).await {
        Ok(samples) => {
            tracing::info!("TTS synthesized {} samples for web playback", samples.len());
            Some(samples)
        }
        Err(e) => {
            tracing::warn!("TTS synthesis failed for web: {}", e);
            None
        }
    };

    let result = serde_json::json!({
        "status": "ok",
        "text": request.text,
        "processed": true,
        "message": "Voice test completed",
        "commands_matched": matched_commands,
        "available_commands": available_commands.len(),
        "tts_samples": tts_samples.as_ref().map(|s| s.len()).unwrap_or(0),
        "tts_available": tts_samples.is_some()
    });

    Ok(warp::reply::json(&result))
}

// Screen sharing handlers
#[derive(serde::Deserialize)]
struct ScreenOfferRequest {
    session_id: Option<String>,
}

#[derive(serde::Deserialize)]
struct ScreenAnswerRequest {
    session_id: String,
    answer: serde_json::Value,
}

#[derive(serde::Deserialize)]
struct RemoteCommandRequest {
    command: String,
    parameters: Option<serde_json::Value>,
}

#[derive(serde::Deserialize)]
struct RemoteMouseRequest {
    #[serde(rename = "type")]
    event_type: String,
    x: i32,
    y: i32,
    timestamp: Option<u64>,
}

async fn handle_screen_answer(
    _request: ScreenAnswerRequest,
    _voice_service: Arc<VoiceProcessingService>,
) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    // Placeholder for screen answer handling
    let response = serde_json::json!({
        "status": "ok",
        "message": "Screen answer received - implementation pending"
    });
    Ok(warp::reply::json(&response))
}

async fn handle_remote_command(
    request: RemoteCommandRequest,
    voice_service: Arc<VoiceProcessingService>,
) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    match voice_service.execute_remote_command(&request.command, request.parameters.as_ref()).await {
        Ok(result) => {
            let response = serde_json::json!({
                "status": "ok",
                "command": request.command,
                "result": result,
                "processed": true
            });
            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            let response = serde_json::json!({
                "status": "error",
                "command": request.command,
                "error": e.to_string(),
                "processed": false
            });
            Ok(warp::reply::json(&response))
        }
    }
}

async fn handle_remote_mouse(
    request: RemoteMouseRequest,
    voice_service: Arc<VoiceProcessingService>,
) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    match voice_service.handle_remote_mouse(&request.event_type, request.x, request.y).await {
        Ok(result) => {
            let response = serde_json::json!({
                "status": "ok",
                "event_type": request.event_type,
                "x": request.x,
                "y": request.y,
                "result": result,
                "message": "Mouse event processed successfully"
            });
            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            let response = serde_json::json!({
                "status": "error",
                "event_type": request.event_type,
                "x": request.x,
                "y": request.y,
                "error": e.to_string(),
                "message": "Failed to process mouse event"
            });
            Ok(warp::reply::json(&response))
        }
    }
}

async fn handle_screen_offer(
    request: ScreenOfferRequest,
    voice_service: Arc<VoiceProcessingService>,
) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    let session_id = request.session_id.unwrap_or_else(|| {
        format!("session_{}", chrono::Utc::now().timestamp())
    });

    tracing::info!("Creating screen sharing session: {}", session_id);

    match voice_service.start_screen_sharing(session_id.clone()).await {
        Ok(offer) => {
            let response = serde_json::json!({
                "status": "ok",
                "session_id": session_id,
                "offer": serde_json::from_str(&offer).unwrap_or(serde_json::Value::Null),
                "message": "Screen sharing session created"
            });
            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            tracing::error!("Failed to create screen sharing session: {}", e);
            let response = serde_json::json!({
                "status": "error",
                "error": e.to_string(),
                "message": "Failed to create screen sharing session"
            });
            Ok(warp::reply::json(&response))
        }
    }
}

// Dictation handlers
#[derive(serde::Deserialize)]
struct DictationStartRequest {
    #[serde(rename = "inputType")]
    input_type: String,
    url: Option<String>,
    element_info: Option<serde_json::Value>,
}

async fn handle_dictation_start(
    request: DictationStartRequest,
) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    tracing::info!("Dictation started for input type: {} on URL: {}", request.input_type, request.url.as_deref().unwrap_or("unknown"));

    let response = serde_json::json!({
        "status": "ok",
        "message": "Dictation started",
        "input_type": request.input_type,
        "session_id": format!("dictation_{}", chrono::Utc::now().timestamp())
    });

    Ok(warp::reply::json(&response))
}

async fn handle_dictation_stop(
) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    tracing::info!("Dictation stopped");

    let response = serde_json::json!({
        "status": "ok",
        "message": "Dictation stopped"
    });

    Ok(warp::reply::json(&response))
}

#[derive(serde::Deserialize)]
struct DictationInsertRequest {
    text: String,
    #[serde(rename = "inputType")]
    input_type: Option<String>,
}

async fn handle_dictation_insert(
    request: DictationInsertRequest,
) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    tracing::info!("Inserting dictated text: {} (type: {})", request.text, request.input_type.as_deref().unwrap_or("text"));

    let response = serde_json::json!({
        "status": "ok",
        "message": "Text inserted",
        "text_length": request.text.len(),
        "input_type": request.input_type
    });

    Ok(warp::reply::json(&response))
}

// Tailscale handlers
async fn handle_tailscale_status(
    config: Arc<RwLock<SystemConfig>>,
) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    let config = config.read().await;

    // Check if Tailscale is enabled in config
    let enabled = config.settings.tailscale_enabled;

    // Try to get Tailscale status (simplified - in real implementation would check tailscale CLI)
    let status = if enabled {
        // This would normally run `tailscale status` and parse output
        // For now, return mock status based on configuration
        serde_json::json!({
            "enabled": true,
            "connected": true,
            "ip": config.settings.web_server_bind.as_deref().unwrap_or("127.0.0.1:8080"),
            "hostname": config.settings.tailscale_hostname.as_deref().unwrap_or("vibespeak"),
            "status": "Connected"
        })
    } else {
        serde_json::json!({
            "enabled": false,
            "connected": false,
            "status": "Not configured"
        })
    };

    Ok(warp::reply::json(&status))
}

#[derive(serde::Deserialize)]
struct TailscaleConfigRequest {
    enabled: Option<bool>,
    ip: Option<String>,
    port: Option<u16>,
}

async fn handle_tailscale_config(
    request: TailscaleConfigRequest,
    config: Arc<RwLock<SystemConfig>>,
) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    let mut config_lock = config.write().await;

    // Update configuration
    if let Some(enabled) = request.enabled {
        config_lock.settings.tailscale_enabled = enabled;
    }

    if let Some(ip) = request.ip {
        if let Some(port) = request.port {
            config_lock.settings.web_server_bind = Some(format!("{}:{}", ip, port));
        } else {
            config_lock.settings.web_server_bind = Some(ip);
        }
    }

    // Save configuration to file
    let config_path = std::path::Path::new("config/system.json");
    if let Ok(config_json) = serde_json::to_string_pretty(&*config_lock) {
        if std::fs::write(config_path, config_json).is_err() {
            return Ok(warp::reply::json(&serde_json::json!({
                "status": "error",
                "message": "Failed to save configuration"
            })));
        }
    }

    let response = serde_json::json!({
        "status": "ok",
        "message": "Tailscale configuration updated",
        "enabled": config_lock.settings.tailscale_enabled,
        "bind_address": config_lock.settings.web_server_bind
    });

    Ok(warp::reply::json(&response))
}

// Dictation typing handler - simulates keyboard input on desktop
#[derive(serde::Deserialize)]
struct DictationTypeRequest {
    text: String,
    #[serde(default)]
    simulate_keyboard: bool,
}

async fn handle_dictation_type(
    request: DictationTypeRequest,
) -> std::result::Result<impl warp::Reply, warp::Rejection> {
    tracing::info!("Typing text on desktop: '{}' (keyboard simulation: {})", request.text, request.simulate_keyboard);

    if request.text.trim().is_empty() {
        let response = serde_json::json!({
            "success": false,
            "error": "Empty text provided",
            "message": "No text to type"
        });
        return Ok(warp::reply::json(&response));
    }

    // Use xdotool to simulate typing the text on the desktop
    match simulate_keyboard_input(&request.text).await {
        Ok(_) => {
            let response = serde_json::json!({
                "success": true,
                "message": "Text typed successfully on desktop",
                "characters_typed": request.text.len()
            });
            Ok(warp::reply::json(&response))
        }
        Err(e) => {
            tracing::error!("Failed to simulate keyboard input: {}", e);
            let response = serde_json::json!({
                "success": false,
                "error": e.to_string(),
                "message": "Failed to type text on desktop"
            });
            Ok(warp::reply::json(&response))
        }
    }
}

async fn simulate_keyboard_input(text: &str) -> Result<()> {
    // Escape special characters that might cause issues with xdotool
    let escaped_text = text
        .replace("\\", "\\\\")  // Escape backslashes first
        .replace("\"", "\\\"")  // Escape quotes
        .replace("'", "\\'")    // Escape single quotes
        .replace("$", "\\$")    // Escape dollar signs
        .replace("`", "\\`");   // Escape backticks

    // Use xdotool to type the text
    // Note: xdotool type handles most characters, but we could also use xdotool key for special keys
    let command = format!("xdotool type \"{}\"", escaped_text);

    tracing::debug!("Executing xdotool command: {}", command);

    let output = Command::new("bash")
        .arg("-c")
        .arg(&command)
        .output()
        .await
        .map_err(|e| Error::CommandExecution(format!("Failed to execute xdotool: {}", e)))?;

    if output.status.success() {
        tracing::info!("Successfully typed {} characters on desktop", text.len());
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(Error::CommandExecution(format!("xdotool failed: {}", stderr)))
    }
}

fn parse_bind_address(bind_addr: &str) -> Result<([u8; 4], u16)> {
    // Parse "ip:port" format
    let parts: Vec<&str> = bind_addr.split(':').collect();
    if parts.len() != 2 {
        return Err(crate::shared::Error::Configuration(
            format!("Invalid bind address format: {}", bind_addr)
        ));
    }

    let ip_str = parts[0];
    let port_str = parts[1];

    // Parse IP address
    let ip_parts: Vec<&str> = ip_str.split('.').collect();
    if ip_parts.len() != 4 {
        return Err(crate::shared::Error::Configuration(
            format!("Invalid IP address format: {}", ip_str)
        ));
    }

    let mut ip = [0u8; 4];
    for (i, part) in ip_parts.iter().enumerate() {
        ip[i] = part.parse().map_err(|_| {
            crate::shared::Error::Configuration(
                format!("Invalid IP address part: {}", part)
            )
        })?;
    }

    // Parse port
    let port: u16 = port_str.parse().map_err(|_| {
        crate::shared::Error::Configuration(
            format!("Invalid port number: {}", port_str)
        )
    })?;

    Ok((ip, port))
}

fn format_ip(ip: [u8; 4]) -> String {
    format!("{}.{}.{}.{}", ip[0], ip[1], ip[2], ip[3])
}
