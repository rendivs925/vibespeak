//! Text-to-speech handlers

use axum::{
    body::Body,
    extract::State,
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

use crate::presentation::axum_server::state::AppState;

#[derive(Debug, Deserialize)]
pub struct SpeakRequest {
    pub text: String,
    pub voice: Option<String>,
}

pub async fn speak(
    State(state): State<AppState>,
    Json(request): Json<SpeakRequest>,
) -> Result<Response, StatusCode> {
    if request.text.trim().is_empty() {
        return Ok((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "Text cannot be empty"
            })),
        )
            .into_response());
    }

    match state
        .voice_service
        .text_to_speech
        .synthesize(&request.text, request.voice.as_deref())
        .await
    {
        Ok(samples) => {
            let wav_data = create_wav_from_samples(&samples);

            Ok(Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "audio/wav")
                .header(header::CONTENT_LENGTH, wav_data.len())
                .body(Body::from(wav_data))
                .unwrap())
        }
        Err(e) => {
            tracing::error!("TTS synthesis failed: {}", e);
            Ok((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": format!("TTS synthesis failed: {}", e)
                })),
            )
                .into_response())
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct TestVoiceRequest {
    pub text: String,
}

pub async fn test_voice(
    State(state): State<AppState>,
    Json(request): Json<TestVoiceRequest>,
) -> Json<Value> {
    let available_commands = state
        .voice_service
        .command_interpreter
        .get_available_commands()
        .await
        .unwrap_or_default();

    let text_lower = request.text.to_lowercase();
    let matched_commands: Vec<String> = available_commands
        .iter()
        .filter(|cmd| {
            let cmd_lower = cmd.to_lowercase();
            cmd_lower.contains(&text_lower)
                || text_lower.contains(&cmd_lower)
                || strsim::jaro_winkler(&text_lower, &cmd_lower) > 0.7
        })
        .cloned()
        .collect();

    let tts_available = state
        .voice_service
        .text_to_speech
        .synthesize(&request.text, None)
        .await
        .is_ok();

    Json(json!({
        "status": "ok",
        "text": request.text,
        "processed": true,
        "message": "Voice test completed",
        "commands_matched": matched_commands,
        "available_commands": available_commands.len(),
        "tts_available": tts_available
    }))
}

fn create_wav_from_samples(samples: &[i16]) -> Vec<u8> {
    let sample_rate: u32 = 22050;
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * (num_channels as u32) * (bits_per_sample as u32) / 8;
    let block_align = num_channels * bits_per_sample / 8;
    let data_size = samples.len() * 2;
    let file_size = 36 + data_size as u32;

    let mut wav = Vec::with_capacity(44 + data_size);

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size
    wav.extend_from_slice(&1u16.to_le_bytes()); // AudioFormat (PCM)
    wav.extend_from_slice(&num_channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data_size as u32).to_le_bytes());

    for sample in samples {
        wav.extend_from_slice(&sample.to_le_bytes());
    }

    wav
}
