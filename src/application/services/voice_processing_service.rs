use crate::domain::services::{
    CommandContext, CommandInterpreter, InterpretedCommand, SpeechRecognitionService,
    TextToSpeechService,
};
use crate::shared::{AudioSample, Result};
use std::sync::Arc;

pub struct VoiceProcessingService {
    pub speech_recognition: Arc<dyn SpeechRecognitionService>,
    pub text_to_speech: Arc<dyn TextToSpeechService>,
    pub command_interpreter: Arc<dyn CommandInterpreter>,
}

impl VoiceProcessingService {
    pub fn new(
        speech_recognition: Arc<dyn SpeechRecognitionService>,
        text_to_speech: Arc<dyn TextToSpeechService>,
        command_interpreter: Arc<dyn CommandInterpreter>,
    ) -> Self {
        Self {
            speech_recognition,
            text_to_speech,
            command_interpreter,
        }
    }

    pub async fn process_audio(&self, audio: AudioSample) -> Result<RecognitionResult> {
        // Recognize speech from audio
        let recognition_result = self.speech_recognition.recognize(audio).await?;

        // Create command context
        let context = CommandContext {
            user_id: None,
            session_id: None,
            previous_commands: Vec::new(),
            environment: std::collections::HashMap::new(),
        };

        // Interpret the recognized text as a command
        let interpreted = self
            .command_interpreter
            .interpret(&recognition_result.text, &context)
            .await?;

        Ok(RecognitionResult {
            recognition: recognition_result,
            interpreted_command: interpreted,
        })
    }

    pub async fn speak_text(&self, text: &str, voice: Option<&str>) -> Result<()> {
        let _audio_samples = self.text_to_speech.synthesize(text, voice).await?;
        // TODO: Play audio samples using rodio
        Ok(())
    }

    pub async fn initialize(&self) -> Result<()> {
        self.speech_recognition.initialize().await?;
        self.text_to_speech.initialize().await?;
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<()> {
        self.speech_recognition.shutdown().await?;
        self.text_to_speech.shutdown().await?;
        Ok(())
    }
}

pub struct RecognitionResult {
    pub recognition: crate::domain::entities::RecognitionResult,
    pub interpreted_command: InterpretedCommand,
}
