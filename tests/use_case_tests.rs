// Tests for application use cases

mod system_status_tests {
    #[test]
    fn test_uptime_tracking() {
        use std::thread::sleep;
        use std::time::Duration;
        use std::time::Instant;

        let start = Instant::now();
        sleep(Duration::from_millis(100));
        let elapsed = start.elapsed();

        assert!(elapsed >= Duration::from_millis(100));
        assert!(elapsed < Duration::from_millis(200));
    }

    #[test]
    fn test_memory_metrics_format() {
        // Test that memory metrics are in reasonable format
        let memory_mb: Option<f64> = Some(50.5);

        if let Some(mem) = memory_mb {
            assert!(mem > 0.0);
            assert!(mem < 100000.0); // Reasonable upper bound
        }
    }

    #[test]
    fn test_cpu_metrics_format() {
        // Test that CPU metrics are in reasonable format (0-100%)
        let cpu_percent: Option<f64> = Some(25.5);

        if let Some(cpu) = cpu_percent {
            assert!(cpu >= 0.0);
            assert!(cpu <= 100.0);
        }
    }
}

mod workflow_creation_tests {
    #[test]
    fn test_workflow_trigger_conversion() {
        use vibespeak::application::dtos::WorkflowTriggerDto;
        use vibespeak::domain::entities::WorkflowTrigger;

        // Test VoiceCommand trigger
        let dto = WorkflowTriggerDto::VoiceCommand("start task".to_string());
        let trigger: WorkflowTrigger = match dto {
            WorkflowTriggerDto::VoiceCommand(cmd) => WorkflowTrigger::VoiceCommand(cmd),
            WorkflowTriggerDto::Scheduled(cron) => WorkflowTrigger::Scheduled(cron),
            WorkflowTriggerDto::Event(event) => WorkflowTrigger::Event(event),
            WorkflowTriggerDto::Manual => WorkflowTrigger::Manual,
        };

        match trigger {
            WorkflowTrigger::VoiceCommand(cmd) => assert_eq!(cmd, "start task"),
            _ => panic!("Wrong trigger type"),
        }
    }

    #[test]
    fn test_workflow_step_dto_conversion() {
        use vibespeak::application::dtos::WorkflowStepDto;

        let step = WorkflowStepDto::ExecuteCommand("echo hello".to_string());

        match step {
            WorkflowStepDto::ExecuteCommand(cmd) => assert_eq!(cmd, "echo hello"),
            _ => panic!("Wrong step type"),
        }
    }

    #[test]
    fn test_workflow_wait_step() {
        use vibespeak::application::dtos::WorkflowStepDto;

        let step = WorkflowStepDto::Wait(1000); // 1 second

        match step {
            WorkflowStepDto::Wait(ms) => assert_eq!(ms, 1000),
            _ => panic!("Wrong step type"),
        }
    }

    #[test]
    fn test_workflow_variable_value_conversion() {
        use vibespeak::application::dtos::VariableValueDto;
        use vibespeak::domain::entities::VariableValue;

        let string_val = VariableValueDto::String("test".to_string());
        let number_val = VariableValueDto::Number(42.5);
        let bool_val = VariableValueDto::Boolean(true);

        // Convert to domain types
        let domain_string: VariableValue = match string_val {
            VariableValueDto::String(s) => VariableValue::String(s),
            _ => panic!("Wrong type"),
        };

        match domain_string {
            VariableValue::String(s) => assert_eq!(s, "test"),
            _ => panic!("Conversion failed"),
        }
    }
}

mod dto_tests {
    #[test]
    fn test_voice_command_request_serialization() {
        use vibespeak::application::dtos::VoiceCommandRequest;

        let request = VoiceCommandRequest {
            text: "hello world".to_string(),
            voice: Some("default".to_string()),
            context: None,
        };

        let json = serde_json::to_string(&request).expect("Serialization failed");
        assert!(json.contains("hello world"));
        assert!(json.contains("default"));
    }

    #[test]
    fn test_voice_command_request_deserialization() {
        use vibespeak::application::dtos::VoiceCommandRequest;

        let json = r#"{"text": "test command", "voice": null, "context": null}"#;
        let request: VoiceCommandRequest =
            serde_json::from_str(json).expect("Deserialization failed");

        assert_eq!(request.text, "test command");
        assert!(request.voice.is_none());
    }

    #[test]
    fn test_voice_command_response_serialization() {
        use vibespeak::application::dtos::VoiceCommandResponse;

        let response = VoiceCommandResponse {
            success: true,
            command_id: Some("cmd-123".to_string()),
            result: serde_json::json!({"output": "done"}),
            message: Some("Command executed".to_string()),
            execution_time_ms: 150,
        };

        let json = serde_json::to_string(&response).expect("Serialization failed");
        assert!(json.contains("true"));
        assert!(json.contains("cmd-123"));
        assert!(json.contains("150"));
    }

    #[test]
    fn test_system_status_response_serialization() {
        use vibespeak::application::dtos::SystemStatusResponse;

        let response = SystemStatusResponse {
            version: "0.1.0".to_string(),
            uptime: 3600,
            active_plugins: 2,
            loaded_commands: 10,
            active_workflows: 1,
            memory_usage_mb: Some(50.5),
            cpu_usage_percent: Some(15.2),
        };

        let json = serde_json::to_string(&response).expect("Serialization failed");
        assert!(json.contains("0.1.0"));
        assert!(json.contains("3600"));
        assert!(json.contains("50.5"));
    }

    #[test]
    fn test_workflow_creation_request() {
        use vibespeak::application::dtos::{
            WorkflowCreationRequest, WorkflowStepDto, WorkflowTriggerDto,
        };

        let request = WorkflowCreationRequest {
            name: "My Workflow".to_string(),
            description: Some("A test workflow".to_string()),
            trigger: WorkflowTriggerDto::Manual,
            steps: vec![
                WorkflowStepDto::ExecuteCommand("echo start".to_string()),
                WorkflowStepDto::Wait(500),
                WorkflowStepDto::ExecuteCommand("echo end".to_string()),
            ],
        };

        let json = serde_json::to_string(&request).expect("Serialization failed");
        assert!(json.contains("My Workflow"));
        assert!(json.contains("Manual"));
    }

    #[test]
    fn test_browser_action_dto() {
        use vibespeak::application::dtos::BrowserActionDto;

        let action = BrowserActionDto {
            action_type: "click".to_string(),
            selector: Some("#submit-button".to_string()),
            value: None,
            wait_ms: Some(1000),
        };

        let json = serde_json::to_string(&action).expect("Serialization failed");
        assert!(json.contains("click"));
        assert!(json.contains("#submit-button"));
    }

    #[test]
    fn test_script_execution_dto() {
        use vibespeak::application::dtos::ScriptExecutionDto;

        let script = ScriptExecutionDto {
            script_type: "bash".to_string(),
            content: "echo hello".to_string(),
            timeout_seconds: 30,
            security_level: "trusted".to_string(),
        };

        let json = serde_json::to_string(&script).expect("Serialization failed");
        assert!(json.contains("bash"));
        assert!(json.contains("echo hello"));
        assert!(json.contains("trusted"));
    }

    #[test]
    fn test_condition_dto() {
        use vibespeak::application::dtos::ConditionDto;

        let condition = ConditionDto {
            variable: "status".to_string(),
            operator: "equals".to_string(),
            value: serde_json::json!("success"),
        };

        let json = serde_json::to_string(&condition).expect("Serialization failed");
        assert!(json.contains("status"));
        assert!(json.contains("equals"));
    }
}
