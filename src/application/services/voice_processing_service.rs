use crate::domain::services::{
    CommandContext, CommandInterpreter, InterpretedCommand, SpeechRecognitionService,
    TextToSpeechService,
};
use crate::screen::{ScreenSharingManager, RemoteControlManager};
use crate::shared::{AudioSample, Result};
use std::sync::Arc;

pub struct VoiceProcessingService {
    pub speech_recognition: Arc<dyn SpeechRecognitionService>,
    pub text_to_speech: Arc<dyn TextToSpeechService>,
    pub command_interpreter: Arc<dyn CommandInterpreter>,
    pub screen_sharing: Arc<ScreenSharingManager>,
    pub remote_control: Arc<RemoteControlManager>,
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
            screen_sharing: Arc::new(ScreenSharingManager::new()),
            remote_control: Arc::new(RemoteControlManager::new()),
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
        // Try to use TTS adapter's speak method if available
        // For now, just synthesize (audio playback handled by voice command processor)
        let _audio_samples = self.text_to_speech.synthesize(text, voice).await?;
        tracing::info!("Text synthesized for speaking: {}", text);
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

    // Screen sharing methods
    pub async fn start_screen_sharing(&self, session_id: String) -> Result<String> {
        self.screen_sharing.create_session(session_id).await
    }

    pub async fn handle_screen_answer(&self, session_id: &str, answer_sdp: &str) -> Result<()> {
        self.screen_sharing.handle_answer(session_id, answer_sdp).await
    }

    pub async fn stop_screen_sharing(&self, session_id: &str) -> Result<()> {
        self.screen_sharing.end_session(session_id).await
    }

    pub async fn get_active_screen_sessions(&self) -> Vec<String> {
        self.screen_sharing.get_active_sessions().await
    }

    // Remote control methods
    pub async fn process_command(&self, command: &str) -> Result<String> {
        self.remote_control.execute_command(command, None).await
    }

    pub async fn execute_remote_command(&self, command: &str, parameters: Option<&serde_json::Value>) -> Result<String> {
        self.remote_control.execute_command(command, parameters).await
    }

    // Remote mouse control
    pub async fn handle_remote_mouse(&self, event_type: &str, x: i32, y: i32) -> Result<String> {
        self.remote_control.handle_mouse_event(event_type, x, y).await
    }

    // Remote voice processing
    pub async fn process_remote_voice(&self, audio: AudioSample) -> Result<String> {
        // Recognize speech from remote audio
        let recognition_result = self.speech_recognition.recognize(audio.clone()).await?;

        // Create command context for remote session
        let context = CommandContext {
            user_id: Some("remote_user".to_string()),
            session_id: Some("remote_session".to_string()),
            previous_commands: Vec::new(),
            environment: std::collections::HashMap::new(),
        };

        // Interpret the recognized text as a command
        let interpreted = self
            .command_interpreter
            .interpret(&recognition_result.text, &context)
            .await?;

        // Execute the interpreted command
        match interpreted.action {
            crate::domain::services::CommandAction::Execute(cmd) => {
                self.remote_control.execute_command(&cmd, None).await
            }
            crate::domain::services::CommandAction::Workflow(workflow_id) => {
                // For now, just acknowledge workflow execution
                tracing::info!("Remote voice triggered workflow: {}", workflow_id);
                Ok(format!("Workflow {} triggered", workflow_id))
            }
            crate::domain::services::CommandAction::Script(script_id) => {
                // For now, just acknowledge script execution
                tracing::info!("Remote voice triggered script: {}", script_id);
                Ok(format!("Script {} executed", script_id))
            }
            crate::domain::services::CommandAction::Integration(integration) => {
                // For now, just acknowledge integration call
                tracing::info!("Remote voice triggered integration: {}", integration);
                Ok(format!("Integration {} called", integration))
            }
        }
    }
}

pub struct RecognitionResult {
    pub recognition: crate::domain::entities::RecognitionResult,
    pub interpreted_command: InterpretedCommand,
}
