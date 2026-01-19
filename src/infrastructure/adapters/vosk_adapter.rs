use crate::domain::entities::RecognitionResult;
use crate::domain::services::SpeechRecognitionService;
use crate::shared::{AudioSample, Error, Result};
use async_trait::async_trait;
use std::sync::Arc;
use vosk::{Model, Recognizer};

pub struct VoskAdapter {
    model: Arc<Model>,
    default_sample_rate: f32,
    grammar: Vec<String>,
}

impl VoskAdapter {
    pub fn new(model_path: &str, sample_rate: f32, grammar: Vec<String>) -> Result<Self> {
        let model = Model::new(model_path).ok_or_else(|| {
            Error::Infrastructure(format!(
                "Failed to load Vosk model from path: {}",
                model_path
            ))
        })?;

        Ok(Self {
            model: Arc::new(model),
            default_sample_rate: sample_rate,
            grammar,
        })
    }
}

#[async_trait]
impl SpeechRecognitionService for VoskAdapter {
    async fn recognize(&self, audio: AudioSample) -> Result<RecognitionResult> {
        // Preprocess audio to 16kHz mono for optimal Vosk performance
        let processed_audio = audio
            .to_16khz_mono()
            .map_err(|e| Error::Infrastructure(format!("Failed to preprocess audio: {}", e)))?;

        tracing::debug!(
            "Audio preprocessed: {}Hz {}ch -> {}Hz {}ch, {} samples",
            audio.sample_rate,
            audio.channels,
            processed_audio.sample_rate,
            processed_audio.channels,
            processed_audio.data.len()
        );

        // Create recognizer with grammar for command recognition
        let grammar_refs: Vec<&str> = self.grammar.iter().map(|s| s.as_str()).collect();
        let mut recognizer = Recognizer::new_with_grammar(
            &self.model,
            processed_audio.sample_rate as f32,
            &grammar_refs,
        )
        .ok_or_else(|| {
            Error::Infrastructure("Failed to create Vosk recognizer with grammar".to_string())
        })?;

        // Process the audio - Vosk expects &[i16]
        let _state = recognizer
            .accept_waveform(&processed_audio.data)
            .map_err(|e| Error::Audio(format!("Failed to process audio waveform: {:?}", e)))?;

        // Get the final result
        match recognizer.result() {
            vosk::CompleteResult::Single(result) => {
                Ok(RecognitionResult::new(result.text.to_string(), 0.8)) // Default confidence for now
            }
            _ => Ok(RecognitionResult::new("".to_string(), 0.0)),
        }
    }

    async fn initialize(&self) -> Result<()> {
        // Vosk model is already loaded in constructor
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        // Vosk cleanup happens automatically
        Ok(())
    }
}
