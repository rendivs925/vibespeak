use crate::domain::entities::RecognitionResult;
use crate::domain::services::SpeechRecognitionService;
use crate::shared::{AudioSample, Error, Result};
use async_trait::async_trait;
use std::sync::Arc;
use vosk::{Model, Recognizer};

pub struct VoskAdapter {
    model: Arc<Model>,
    sample_rate: f32,
}

impl VoskAdapter {
    pub fn new(model_path: &str, sample_rate: f32) -> Result<Self> {
        let model = Model::new(model_path).ok_or_else(|| {
            Error::Infrastructure(format!(
                "Failed to load Vosk model from path: {}",
                model_path
            ))
        })?;

        Ok(Self {
            model: Arc::new(model),
            sample_rate,
        })
    }
}

#[async_trait]
impl SpeechRecognitionService for VoskAdapter {
    async fn recognize(&self, audio: AudioSample) -> Result<RecognitionResult> {
        // Create recognizer for this audio session
        let mut recognizer = Recognizer::new(&self.model, self.sample_rate)
            .ok_or_else(|| Error::Infrastructure("Failed to create Vosk recognizer".to_string()))?;

        // Process the audio - Vosk expects &[i16]
        let _state = recognizer
            .accept_waveform(&audio.data)
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
