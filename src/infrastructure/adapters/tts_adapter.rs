use crate::domain::services::TextToSpeechService;
use crate::shared::{Error, Result};
use async_trait::async_trait;

pub struct TtsAdapter {
    // TTS temporarily disabled due to missing system dependencies
    // TODO: Re-enable when speech-dispatcher is available
}

impl TtsAdapter {
    pub fn new() -> Result<Self> {
        tracing::warn!(
            "TTS adapter initialized but TTS is disabled due to missing speech-dispatcher"
        );
        Ok(Self {})
    }
}

#[async_trait]
impl TextToSpeechService for TtsAdapter {
    async fn synthesize(&self, _text: &str, _voice: Option<&str>) -> Result<Vec<i16>> {
        tracing::info!("TTS synthesis requested but TTS is disabled");
        // Return empty audio data when TTS is not available
        Ok(Vec::new())
    }

    async fn get_available_voices(&self) -> Result<Vec<String>> {
        tracing::info!("TTS voices requested but TTS is disabled");
        // Return empty list when TTS is not available
        Ok(Vec::new())
    }

    async fn initialize(&self) -> Result<()> {
        tracing::info!("TTS initialization requested but TTS is disabled");
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("TTS shutdown requested but TTS is disabled");
        Ok(())
    }
}
