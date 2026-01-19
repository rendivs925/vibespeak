// Integration tests for the complete system

mod config_tests {
    #[test]
    fn test_system_config_default() {
        use vibespeak::infrastructure::config::SystemConfig;

        let config = SystemConfig::default();

        // Default config starts with empty collections
        assert!(config.commands.is_empty());
        assert!(config.workflows.is_empty());
        assert!(config.scripts.is_empty());
        // But has valid settings
        assert!(config.settings.sample_rate > 0.0);
        assert!(config.settings.web_server_port > 0);
    }

    #[test]
    fn test_system_config_serialization() {
        use vibespeak::infrastructure::config::SystemConfig;

        let config = SystemConfig::default();

        let json = serde_json::to_string(&config);
        assert!(json.is_ok());

        let json = json.unwrap();
        assert!(json.contains("commands"));
        assert!(json.contains("settings"));
    }

    #[test]
    fn test_system_config_deserialization() {
        use vibespeak::infrastructure::config::SystemConfig;

        let config = SystemConfig::default();
        let json = serde_json::to_string(&config).unwrap();

        let deserialized: Result<SystemConfig, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok());
    }
}

mod plugin_tests {
    #[test]
    fn test_plugin_registry_creation() {
        use vibespeak::domain::services::plugin::PluginRegistry;

        let registry = PluginRegistry::new();
        assert!(registry.list_plugins().is_empty());
    }

    #[test]
    fn test_builtin_commands_plugin() {
        use vibespeak::domain::services::plugin::{BuiltinCommandsPlugin, PluginRegistry};

        let mut registry = PluginRegistry::new();
        let result = registry.register(Box::new(BuiltinCommandsPlugin));

        assert!(result.is_ok());
        assert_eq!(registry.list_plugins().len(), 1);
    }

    #[test]
    fn test_plugin_metadata() {
        use vibespeak::domain::services::plugin::{
            BuiltinCommandsPlugin, PluginRegistry, VoicePlugin,
        };

        let plugin = BuiltinCommandsPlugin;
        let metadata = plugin.metadata();

        assert!(!metadata.name.is_empty());
        assert!(!metadata.version.is_empty());
        assert!(!metadata.capabilities.is_empty());
    }

    #[test]
    fn test_plugin_capability_check() {
        use vibespeak::domain::services::plugin::{
            BuiltinCommandsPlugin, PluginCapability, PluginRegistry,
        };

        let mut registry = PluginRegistry::new();
        registry.register(Box::new(BuiltinCommandsPlugin)).unwrap();

        let providers = registry.has_capability(&PluginCapability::CommandProvider);
        assert!(!providers.is_empty());
    }
}

mod error_tests {
    #[test]
    fn test_error_variants() {
        use vibespeak::shared::Error;

        let domain_err = Error::Domain("domain error".to_string());
        let infra_err = Error::Infrastructure("infra error".to_string());
        let config_err = Error::Configuration("config error".to_string());
        let plugin_err = Error::Plugin("plugin error".to_string());
        let audio_err = Error::Audio("audio error".to_string());
        let network_err = Error::Network("network error".to_string());

        assert!(format!("{}", domain_err).contains("domain"));
        assert!(format!("{}", infra_err).contains("infra"));
        assert!(format!("{}", config_err).contains("config"));
        assert!(format!("{}", plugin_err).contains("plugin"));
        assert!(format!("{}", audio_err).contains("audio"));
        assert!(format!("{}", network_err).contains("network"));
    }

    #[test]
    fn test_error_from_io() {
        use std::io;
        use vibespeak::shared::Error;

        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let error: Error = io_error.into();

        assert!(format!("{}", error).contains("IO error"));
    }

    #[test]
    fn test_error_from_json() {
        use vibespeak::shared::Error;

        let json_result: Result<serde_json::Value, _> = serde_json::from_str("invalid json");
        if let Err(json_err) = json_result {
            let error: Error = json_err.into();
            assert!(format!("{}", error).contains("JSON"));
        }
    }
}

mod fuzzy_matching_tests {
    #[test]
    fn test_strsim_jaro_winkler() {
        // Test the fuzzy matching algorithm we use
        let similarity = strsim::jaro_winkler("hello", "hello");
        assert_eq!(similarity, 1.0);

        let similarity = strsim::jaro_winkler("hello", "helo");
        assert!(similarity > 0.9);

        let similarity = strsim::jaro_winkler("hello", "world");
        assert!(similarity < 0.5);
    }

    #[test]
    fn test_fuzzy_command_matching() {
        let commands = vec!["hello", "help", "status", "stop", "start"];
        let input = "helo"; // typo for "hello"

        let best_match = commands
            .iter()
            .map(|cmd| (cmd, strsim::jaro_winkler(input, cmd)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        assert!(best_match.is_some());
        let (matched, score) = best_match.unwrap();
        assert_eq!(*matched, "hello");
        assert!(score > 0.8);
    }

    #[test]
    fn test_case_insensitive_matching() {
        let input = "HELLO".to_lowercase();
        let command = "hello".to_lowercase();

        let similarity = strsim::jaro_winkler(&input, &command);
        assert_eq!(similarity, 1.0);
    }
}

mod workflow_executor_tests {
    #[tokio::test]
    async fn test_workflow_executor_validate() {
        use std::sync::Arc;
        use vibespeak::domain::entities::{Workflow, WorkflowStep, WorkflowTrigger};
        use vibespeak::domain::services::browser_automation::ChromiumBrowserService;
        use vibespeak::domain::services::script_executor::ScriptExecutor;
        use vibespeak::domain::services::workflow_executor::{
            DefaultWorkflowExecutor, WorkflowExecutor,
        };

        let script_executor = Arc::new(ScriptExecutor::new());
        let browser_service: Arc<
            dyn vibespeak::domain::services::browser_automation::BrowserAutomationService,
        > = Arc::new(ChromiumBrowserService::new());
        let executor = DefaultWorkflowExecutor::new(script_executor, browser_service);

        let mut workflow = Workflow::new("Test".to_string(), WorkflowTrigger::Manual);
        workflow.add_step(WorkflowStep::ExecuteCommand("echo test".to_string()));

        let result = executor.validate_workflow(&workflow).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_workflow_executor_execute_simple() {
        use std::collections::HashMap;
        use std::sync::Arc;
        use vibespeak::domain::entities::{VariableValue, Workflow, WorkflowStep, WorkflowTrigger};
        use vibespeak::domain::services::browser_automation::ChromiumBrowserService;
        use vibespeak::domain::services::script_executor::ScriptExecutor;
        use vibespeak::domain::services::workflow_executor::{
            DefaultWorkflowExecutor, WorkflowExecutor,
        };

        let script_executor = Arc::new(ScriptExecutor::new());
        let browser_service: Arc<
            dyn vibespeak::domain::services::browser_automation::BrowserAutomationService,
        > = Arc::new(ChromiumBrowserService::new());
        let executor = DefaultWorkflowExecutor::new(script_executor, browser_service);

        let mut workflow = Workflow::new("Simple".to_string(), WorkflowTrigger::Manual);
        workflow.add_step(WorkflowStep::ExecuteCommand("echo 'step 1'".to_string()));
        workflow.add_step(WorkflowStep::ExecuteCommand("echo 'step 2'".to_string()));

        let variables: HashMap<String, VariableValue> = HashMap::new();
        let result = executor.execute_workflow(&workflow, variables).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.success);
        assert_eq!(result.steps_executed, 2);
    }
}

mod voice_processing_tests {
    #[tokio::test]
    async fn test_voice_processing_service_creation() {
        use std::collections::HashMap;
        use std::sync::Arc;
        use vibespeak::application::services::VoiceProcessingService;
        use vibespeak::infrastructure::adapters::{
            FuzzyCommandInterpreter, TtsAdapter, VoskAdapter,
        };

        // Try to create the Vosk adapter (may fail if model not present)
        let vosk_result = VoskAdapter::new("model/vosk-model-en-us-0.22-lgraph", 16000.0, vec![]);

        if let Ok(vosk) = vosk_result {
            let tts = TtsAdapter::new().expect("TTS creation failed");
            let interpreter = FuzzyCommandInterpreter::new(HashMap::new());

            let service =
                VoiceProcessingService::new(Arc::new(vosk), Arc::new(tts), Arc::new(interpreter));

            let init_result = service.initialize().await;
            // May fail if hardware not available, but should not panic
            let _ = init_result;
        }
    }
}

mod concurrent_access_tests {
    use std::sync::Arc;

    #[tokio::test]
    async fn test_repository_concurrent_access() {
        use vibespeak::domain::entities::{CommandAction, VoiceCommand};
        use vibespeak::infrastructure::repositories::command_repository::InMemoryCommandRepository;
        use vibespeak::infrastructure::repositories::CommandRepository;

        let repo = Arc::new(InMemoryCommandRepository::new());

        let mut handles = vec![];

        for i in 0..10 {
            let repo_clone = repo.clone();
            let handle = tokio::spawn(async move {
                let cmd = VoiceCommand::new(
                    format!("command {}", i),
                    CommandAction::ShellCommand(format!("echo {}", i)),
                );
                repo_clone.save(&cmd).await
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }

        let all = repo.find_all().await.expect("Find all failed");
        assert_eq!(all.len(), 10);
    }

    #[tokio::test]
    async fn test_workflow_repository_concurrent_access() {
        use vibespeak::domain::entities::{Workflow, WorkflowTrigger};
        use vibespeak::infrastructure::repositories::workflow_repository::{
            InMemoryWorkflowRepository, WorkflowRepository,
        };

        let repo = Arc::new(InMemoryWorkflowRepository::new());

        let mut handles = vec![];

        for i in 0..5 {
            let repo_clone = repo.clone();
            let handle = tokio::spawn(async move {
                let workflow = Workflow::new(format!("workflow {}", i), WorkflowTrigger::Manual);
                repo_clone.save(&workflow).await
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }

        let count = repo.count().await.expect("Count failed");
        assert_eq!(count, 5);
    }
}
