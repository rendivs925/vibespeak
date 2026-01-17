// Tests for domain entities and value objects

mod voice_command_tests {
    #[test]
    fn test_voice_command_creation() {
        use vibespeak::domain::entities::{VoiceCommand, CommandAction};

        let cmd = VoiceCommand::new(
            "hello".to_string(),
            CommandAction::ShellCommand("echo hello".to_string())
        );

        assert_eq!(cmd.text, "hello");
        assert!(cmd.enabled);
        assert!(!cmd.id.is_empty());
    }

    #[test]
    fn test_voice_command_with_category() {
        use vibespeak::domain::entities::{VoiceCommand, CommandAction};

        let cmd = VoiceCommand::new(
            "test".to_string(),
            CommandAction::ShellCommand("echo test".to_string())
        ).with_category("custom".to_string());

        assert_eq!(cmd.category, "custom");
    }

    #[test]
    fn test_voice_command_validation_empty_text() {
        use vibespeak::domain::entities::{VoiceCommand, CommandAction};

        let cmd = VoiceCommand::new(
            "   ".to_string(), // whitespace only
            CommandAction::ShellCommand("echo".to_string())
        );

        let result = cmd.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_voice_command_validation_valid() {
        use vibespeak::domain::entities::{VoiceCommand, CommandAction};

        let cmd = VoiceCommand::new(
            "valid command".to_string(),
            CommandAction::ShellCommand("echo valid".to_string())
        );

        let result = cmd.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_command_action_variants() {
        use vibespeak::domain::entities::CommandAction;

        let shell = CommandAction::ShellCommand("ls".to_string());
        let workflow = CommandAction::Workflow("wf-123".to_string());
        let script = CommandAction::Script("script-456".to_string());
        let integration = CommandAction::IntegrationCall("api".to_string());
        let composite = CommandAction::Composite(vec!["cmd1".to_string(), "cmd2".to_string()]);

        // Verify they can be created
        match shell {
            CommandAction::ShellCommand(cmd) => assert_eq!(cmd, "ls"),
            _ => panic!("Wrong variant"),
        }

        match workflow {
            CommandAction::Workflow(id) => assert_eq!(id, "wf-123"),
            _ => panic!("Wrong variant"),
        }

        match composite {
            CommandAction::Composite(cmds) => assert_eq!(cmds.len(), 2),
            _ => panic!("Wrong variant"),
        }
    }
}

mod workflow_tests {
    #[test]
    fn test_workflow_creation() {
        use vibespeak::domain::entities::{Workflow, WorkflowTrigger};

        let workflow = Workflow::new(
            "Test Workflow".to_string(),
            WorkflowTrigger::Manual
        );

        assert_eq!(workflow.name, "Test Workflow");
        assert!(workflow.enabled);
        assert!(workflow.steps.is_empty());
        assert!(!workflow.id.is_empty());
    }

    #[test]
    fn test_workflow_add_step() {
        use vibespeak::domain::entities::{Workflow, WorkflowTrigger, WorkflowStep};

        let mut workflow = Workflow::new(
            "Test".to_string(),
            WorkflowTrigger::Manual
        );

        workflow.add_step(WorkflowStep::ExecuteCommand("echo 1".to_string()));
        workflow.add_step(WorkflowStep::ExecuteCommand("echo 2".to_string()));

        assert_eq!(workflow.steps.len(), 2);
    }

    #[test]
    fn test_workflow_validation_empty_name() {
        use vibespeak::domain::entities::{Workflow, WorkflowTrigger, WorkflowStep};

        let mut workflow = Workflow::new(
            "   ".to_string(), // whitespace only
            WorkflowTrigger::Manual
        );
        workflow.add_step(WorkflowStep::ExecuteCommand("echo".to_string()));

        let result = workflow.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_workflow_validation_no_steps() {
        use vibespeak::domain::entities::{Workflow, WorkflowTrigger};

        let workflow = Workflow::new(
            "Valid Name".to_string(),
            WorkflowTrigger::Manual
        );

        let result = workflow.validate();
        assert!(result.is_err()); // No steps
    }

    #[test]
    fn test_workflow_validation_valid() {
        use vibespeak::domain::entities::{Workflow, WorkflowTrigger, WorkflowStep};

        let mut workflow = Workflow::new(
            "Valid Workflow".to_string(),
            WorkflowTrigger::Manual
        );
        workflow.add_step(WorkflowStep::ExecuteCommand("echo test".to_string()));

        let result = workflow.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_workflow_trigger_variants() {
        use vibespeak::domain::entities::WorkflowTrigger;

        let voice = WorkflowTrigger::VoiceCommand("start".to_string());
        let scheduled = WorkflowTrigger::Scheduled("0 * * * *".to_string());
        let event = WorkflowTrigger::Event("user_login".to_string());
        let manual = WorkflowTrigger::Manual;

        match voice {
            WorkflowTrigger::VoiceCommand(cmd) => assert_eq!(cmd, "start"),
            _ => panic!("Wrong variant"),
        }

        match scheduled {
            WorkflowTrigger::Scheduled(cron) => assert_eq!(cron, "0 * * * *"),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_workflow_step_wait() {
        use vibespeak::domain::entities::WorkflowStep;
        use std::time::Duration;

        let step = WorkflowStep::Wait(Duration::from_secs(5));

        match step {
            WorkflowStep::Wait(duration) => assert_eq!(duration, Duration::from_secs(5)),
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_workflow_step_browser_action() {
        use vibespeak::domain::entities::{WorkflowStep, BrowserAction};

        let step = WorkflowStep::BrowserAction(BrowserAction::Navigate("https://example.com".to_string()));

        match step {
            WorkflowStep::BrowserAction(BrowserAction::Navigate(url)) => {
                assert_eq!(url, "https://example.com");
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_browser_action_variants() {
        use vibespeak::domain::entities::BrowserAction;

        let navigate = BrowserAction::Navigate("https://test.com".to_string());
        let click = BrowserAction::Click("#button".to_string());
        let type_text = BrowserAction::Type("input".to_string(), "hello".to_string());
        let wait = BrowserAction::WaitForElement(".loading".to_string());
        let screenshot = BrowserAction::Screenshot("capture.png".to_string());
        let script = BrowserAction::ExecuteScript("alert('test')".to_string());
        let get_text = BrowserAction::GetText("h1".to_string());
        let scroll = BrowserAction::Scroll(0, 500);

        match navigate {
            BrowserAction::Navigate(url) => assert!(url.contains("test.com")),
            _ => panic!("Wrong variant"),
        }

        match scroll {
            BrowserAction::Scroll(x, y) => {
                assert_eq!(x, 0);
                assert_eq!(y, 500);
            }
            _ => panic!("Wrong variant"),
        }
    }

    #[test]
    fn test_variable_value_display() {
        use vibespeak::domain::entities::VariableValue;

        let string_val = VariableValue::String("hello".to_string());
        let number_val = VariableValue::Number(42.5);
        let bool_val = VariableValue::Boolean(true);

        assert_eq!(format!("{}", string_val), "hello");
        assert_eq!(format!("{}", number_val), "42.5");
        assert_eq!(format!("{}", bool_val), "true");
    }

    #[test]
    fn test_comparison_operators() {
        use vibespeak::domain::entities::ComparisonOperator;

        let operators = vec![
            ComparisonOperator::Equals,
            ComparisonOperator::NotEquals,
            ComparisonOperator::GreaterThan,
            ComparisonOperator::LessThan,
            ComparisonOperator::Contains,
            ComparisonOperator::StartsWith,
            ComparisonOperator::EndsWith,
        ];

        assert_eq!(operators.len(), 7);
    }

    #[test]
    fn test_error_strategy_variants() {
        use vibespeak::domain::entities::{ErrorStrategy, WorkflowStep};

        let stop = ErrorStrategy::Stop;
        let continue_strat = ErrorStrategy::Continue;
        let retry = ErrorStrategy::Retry(3);
        let fallback = ErrorStrategy::Fallback(Box::new(
            WorkflowStep::ExecuteCommand("echo fallback".to_string())
        ));

        match retry {
            ErrorStrategy::Retry(n) => assert_eq!(n, 3),
            _ => panic!("Wrong variant"),
        }
    }
}

mod recognition_session_tests {
    #[test]
    fn test_recognition_session_creation() {
        use vibespeak::domain::entities::RecognitionSession;

        let session = RecognitionSession::new();

        assert!(!session.id.is_empty());
        assert!(session.end_time.is_none());
        assert!(session.audio_samples.is_empty());
        assert!(session.recognition_results.is_empty());
    }

    #[test]
    fn test_recognition_session_end() {
        use vibespeak::domain::entities::{RecognitionSession, SessionStatus};

        let mut session = RecognitionSession::new();
        assert!(session.end_time.is_none());

        session.end_session();

        assert!(session.end_time.is_some());
        assert_eq!(session.status, SessionStatus::Completed);
    }

    #[test]
    fn test_recognition_session_add_result() {
        use vibespeak::domain::entities::{RecognitionSession, RecognitionResult};

        let mut session = RecognitionSession::new();

        let result = RecognitionResult::new("hello world".to_string(), 0.95);
        session.add_recognition_result(result);

        assert_eq!(session.recognition_results.len(), 1);
        assert_eq!(session.recognition_results[0].text, "hello world");
        assert_eq!(session.recognition_results[0].confidence, 0.95);
    }

    #[test]
    fn test_recognition_result_creation() {
        use vibespeak::domain::entities::RecognitionResult;

        let result = RecognitionResult::new("test text".to_string(), 0.85);

        assert_eq!(result.text, "test text");
        assert_eq!(result.confidence, 0.85);
        assert!(result.alternatives.is_empty());
    }

    #[test]
    fn test_session_status_variants() {
        use vibespeak::domain::entities::SessionStatus;

        let statuses = vec![
            SessionStatus::Active,
            SessionStatus::Paused,
            SessionStatus::Completed,
            SessionStatus::Failed,
        ];

        assert_eq!(statuses.len(), 4);
        assert_eq!(statuses[0], SessionStatus::Active);
    }

    #[test]
    fn test_session_metadata_default() {
        use vibespeak::domain::entities::SessionMetadata;

        let metadata = SessionMetadata::default();

        assert!(metadata.user_id.is_none());
        assert!(metadata.device_info.is_empty());
        assert!(metadata.environment.is_empty());
    }
}

mod audio_sample_tests {
    #[test]
    fn test_audio_sample_creation() {
        use vibespeak::shared::AudioSample;

        let samples = vec![0i16, 100, -100, 500, -500];
        let audio = AudioSample::new(samples.clone(), 44100, 1);

        assert_eq!(audio.data, samples);
        assert_eq!(audio.sample_rate, 44100);
        assert_eq!(audio.channels, 1);
    }

    #[test]
    fn test_audio_sample_stereo() {
        use vibespeak::shared::AudioSample;

        let audio = AudioSample::new(vec![0; 1000], 48000, 2);

        assert_eq!(audio.sample_rate, 48000);
        assert_eq!(audio.channels, 2);
    }
}
