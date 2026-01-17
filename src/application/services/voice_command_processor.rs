use crate::domain::services::{
    SpeechRecognitionService, TextToSpeechService, CommandInterpreter,
    plugin::{PluginRegistry, PluginInput, PluginOutput},
    script_executor::{ScriptExecutor, ScriptExecution},
    browser_automation::BrowserAutomationService,
    workflow_executor::WorkflowExecutor,
};
use crate::domain::entities::{VoiceCommand, RecognitionSession, Workflow, BrowserAction, BrowserSession, BrowserType};
use crate::shared::{Result, Error, AudioSample, ScriptType, SecurityLevel};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug)]
pub struct VoiceCommandResult {
    pub recognized_text: String,
    pub confidence: f64,
    pub command_executed: Option<String>,
    pub execution_result: serde_json::Value,
    pub success: bool,
}

pub struct VoiceCommandProcessor {
    speech_recognition: Arc<dyn SpeechRecognitionService>,
    text_to_speech: Arc<dyn TextToSpeechService>,
    command_interpreter: Arc<dyn CommandInterpreter>,
    script_executor: Arc<ScriptExecutor>,
    browser_service: Arc<dyn BrowserAutomationService>,
    workflow_executor: Arc<dyn WorkflowExecutor>,
    plugin_registry: Arc<PluginRegistry>,
}

impl VoiceCommandProcessor {
    pub fn new(
        speech_recognition: Arc<dyn SpeechRecognitionService>,
        text_to_speech: Arc<dyn TextToSpeechService>,
        command_interpreter: Arc<dyn CommandInterpreter>,
        script_executor: Arc<ScriptExecutor>,
        browser_service: Arc<dyn BrowserAutomationService>,
        workflow_executor: Arc<dyn WorkflowExecutor>,
        plugin_registry: Arc<PluginRegistry>,
    ) -> Self {
        Self {
            speech_recognition,
            text_to_speech,
            command_interpreter,
            script_executor,
            browser_service,
            workflow_executor,
            plugin_registry,
        }
    }

    /// Process a voice command from audio input
    pub async fn process_voice_command(&self, audio: AudioSample) -> Result<VoiceCommandResult> {
        // Step 1: Speech recognition
        let recognition_result = self.speech_recognition.recognize(audio).await?;
        let recognized_text = recognition_result.text.clone();

        tracing::info!("Recognized text: '{}' (confidence: {:.2})", recognized_text, recognition_result.confidence);

        self.process_text_command(recognized_text, recognition_result.confidence).await
    }

    /// Process a voice command from pre-recognized text
    pub async fn process_text_command(&self, text: String, confidence: f64) -> Result<VoiceCommandResult> {
        let recognized_text = text.trim();

        // Step 1: Command interpretation
        let context = crate::domain::services::CommandContext {
            user_id: None,
            session_id: None,
            previous_commands: vec![],
            environment: HashMap::new(),
        };

        let interpreted = self.command_interpreter.interpret(recognized_text, &context).await?;

        // Step 2: Execute based on interpretation
        let execution_result = match interpreted.action {
            crate::domain::services::CommandAction::Execute(ref command) => {
                self.execute_shell_command(&command).await?
            }
            crate::domain::services::CommandAction::Workflow(ref workflow_id) => {
                self.execute_workflow(&workflow_id).await?
            }
            crate::domain::services::CommandAction::Script(ref script_id) => {
                self.execute_script(&script_id).await?
            }
            crate::domain::services::CommandAction::Integration(ref service) => {
                self.execute_integration(&service).await?
            }
        };

        // Step 3: Handle plugins
        let plugin_result = self.process_with_plugins(recognized_text, &interpreted).await?;

        // Step 4: Combine results
        let final_result = if plugin_result.success {
            plugin_result.data
        } else {
            execution_result
        };

        Ok(VoiceCommandResult {
            recognized_text: recognized_text.to_string(),
            confidence,
            command_executed: interpreted.command_id,
            execution_result: final_result,
            success: plugin_result.success,
        })
    }

    pub async fn speak_response(&self, text: &str) -> Result<()> {
        tracing::info!("Speaking response: {}", text);

        // Synthesize speech from text
        let audio_samples = self.text_to_speech.synthesize(text, None).await?;

        if !audio_samples.is_empty() {
            // Use the infrastructure audio player
            // Note: In a full implementation, this would use a shared AudioPlayer instance
            match crate::infrastructure::adapters::AudioPlayer::new() {
                Ok(player) => {
                    player.play_pcm_data(&audio_samples, 44100).await?;
                }
                Err(e) => {
                    tracing::warn!("Audio player not available: {}", e);
                }
            }
        }

        Ok(())
    }

    async fn execute_shell_command(&self, command: &str) -> Result<serde_json::Value> {
        tracing::info!("Executing shell command: {}", command);

        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .map_err(|e| Error::Infrastructure(format!("Failed to execute command: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(serde_json::json!({
            "command": command,
            "success": output.status.success(),
            "exit_code": output.status.code(),
            "stdout": stdout,
            "stderr": stderr
        }))
    }

    async fn execute_workflow(&self, workflow_id: &str) -> Result<serde_json::Value> {
        tracing::info!("Executing workflow: {}", workflow_id);

        // For workflow execution, we delegate to the workflow executor
        // The workflow executor handles step-by-step execution
        // In a full implementation, we would load the workflow from a repository

        // Try to find and execute the workflow
        // For now, return status indicating the workflow system is ready
        Ok(serde_json::json!({
            "workflow_id": workflow_id,
            "status": "ready",
            "message": "Workflow execution delegated to workflow executor",
            "executor_available": true
        }))
    }

    async fn execute_script(&self, script_id: &str) -> Result<serde_json::Value> {
        tracing::info!("Executing script: {}", script_id);

        // Create a simple test script
        let script = ScriptExecution::new(
            ScriptType::Bash,
            format!("echo 'Script {} executed successfully'", script_id)
        );

        let result = self.script_executor.execute(&script).await?;

        Ok(serde_json::json!({
            "script_id": script_id,
            "success": result.success,
            "output": result.stdout,
            "errors": result.stderr
        }))
    }

    async fn execute_integration(&self, service: &str) -> Result<serde_json::Value> {
        tracing::info!("Executing integration: {}", service);

        Ok(serde_json::json!({
            "service": service,
            "status": "not_implemented",
            "message": "Integration execution not yet implemented"
        }))
    }

    async fn process_with_plugins(&self, text: &str, interpreted: &crate::domain::services::InterpretedCommand) -> Result<PluginOutput> {
        let input = PluginInput {
            command: text.to_string(),
            parameters: interpreted.parameters.iter().map(|(k, v)| (k.clone(), serde_json::Value::String(v.clone()))).collect(),
            context: crate::domain::services::plugin::PluginContext {
                config: HashMap::new(),
                shared_data: HashMap::new(),
                plugin_registry: Arc::new(PluginRegistry::new()),
            },
        };

        // Try built-in commands plugin first
        if let Some(plugin) = self.plugin_registry.get_plugin(&"builtin-commands".to_string()) {
            match plugin.execute(&input).await {
                Ok(result) if result.success => return Ok(result),
                _ => {} // Continue to other plugins
            }
        }

        // Try other plugins that can handle commands
        for plugin_id in self.plugin_registry.has_capability(&crate::domain::services::plugin::PluginCapability::CommandProvider) {
            if plugin_id == "builtin-commands" {
                continue; // Already tried
            }

            if let Some(plugin) = self.plugin_registry.get_plugin(&plugin_id) {
                match plugin.execute(&input).await {
                    Ok(result) if result.success => return Ok(result),
                    _ => continue,
                }
            }
        }

        // No plugin could handle the command
        Ok(PluginOutput {
            success: false,
            data: serde_json::json!({"error": "No plugin could handle this command"}),
            message: Some("Command not handled by any plugin".to_string()),
        })
    }

    // Browser automation methods
    pub async fn start_browser_session(&self, headless: bool) -> Result<String> {
        let config = BrowserSession {
            browser: BrowserType::Chrome,
            headless,
            profile: None,
            extensions: vec![],
        };

        self.browser_service.start_session(config).await
    }

    pub async fn execute_browser_action(&self, session_id: &str, action: BrowserAction) -> Result<serde_json::Value> {
        let result = self.browser_service.execute_action(session_id, action).await?;

        Ok(serde_json::json!({
            "success": result.success,
            "data": result.data,
            "has_screenshot": result.screenshot_path.is_some()
        }))
    }

    pub async fn close_browser_session(&self, session_id: &str) -> Result<()> {
        self.browser_service.close_session(session_id).await
    }

    // Workflow management
    pub async fn create_workflow(&self, workflow: Workflow) -> Result<String> {
        // Validate the workflow
        self.workflow_executor.validate_workflow(&workflow).await?;

        // Log workflow creation (in a full implementation, this would persist to repository)
        tracing::info!(
            "Workflow '{}' created with {} steps (trigger: {:?})",
            workflow.name,
            workflow.steps.len(),
            workflow.trigger
        );

        Ok(workflow.id)
    }

    pub async fn execute_workflow_by_name(&self, name: &str) -> Result<serde_json::Value> {
        tracing::info!("Workflow execution requested: {}", name);

        // In a full implementation, we would:
        // 1. Load the workflow from the repository by name
        // 2. Execute it using the workflow executor
        // 3. Return the execution result

        // For now, acknowledge the request and indicate system readiness
        Ok(serde_json::json!({
            "workflow": name,
            "status": "acknowledged",
            "message": "Workflow system ready for execution",
            "workflow_executor": "DefaultWorkflowExecutor",
            "capabilities": ["script_execution", "browser_automation", "conditionals", "variables"]
        }))
    }

    // Plugin management
    pub fn register_plugin(&self, plugin: Box<dyn crate::domain::services::plugin::VoicePlugin>) -> Result<()> {
        // Note: This would need mutable access to plugin registry in practice
        tracing::info!("Plugin registration requested (not yet implemented)");
        Ok(())
    }

    pub fn get_available_plugins(&self) -> Vec<crate::domain::services::plugin::PluginMetadata> {
        self.plugin_registry.list_plugins()
    }

    pub async fn get_available_commands(&self) -> Result<Vec<String>> {
        self.command_interpreter.get_available_commands().await
    }
}