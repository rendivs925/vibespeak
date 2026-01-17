// Integration tests for repository implementations

use std::path::PathBuf;
use tempfile::TempDir;

// We need to import from the main crate
// These tests verify the repository implementations work correctly

mod command_repository_tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_command_repository_save_and_find() {
        use vibespeak::infrastructure::repositories::command_repository::InMemoryCommandRepository;
        use vibespeak::infrastructure::repositories::CommandRepository;
        use vibespeak::domain::entities::{VoiceCommand, CommandAction};

        let repo = InMemoryCommandRepository::new();

        let cmd = VoiceCommand::new(
            "test command".to_string(),
            CommandAction::ShellCommand("echo test".to_string())
        );
        let cmd_id = cmd.id.clone();

        // Save command
        repo.save(&cmd).await.expect("Failed to save command");

        // Find by ID
        let found = repo.find_by_id(&cmd_id).await.expect("Failed to find command");
        assert!(found.is_some());
        assert_eq!(found.unwrap().text, "test command");
    }

    #[tokio::test]
    async fn test_in_memory_command_repository_find_all() {
        use vibespeak::infrastructure::repositories::command_repository::InMemoryCommandRepository;
        use vibespeak::infrastructure::repositories::CommandRepository;
        use vibespeak::domain::entities::{VoiceCommand, CommandAction};

        let repo = InMemoryCommandRepository::new();

        // Save multiple commands
        for i in 0..5 {
            let cmd = VoiceCommand::new(
                format!("command {}", i),
                CommandAction::ShellCommand(format!("echo {}", i))
            );
            repo.save(&cmd).await.expect("Failed to save command");
        }

        let all = repo.find_all().await.expect("Failed to find all commands");
        assert_eq!(all.len(), 5);
    }

    #[tokio::test]
    async fn test_in_memory_command_repository_delete() {
        use vibespeak::infrastructure::repositories::command_repository::InMemoryCommandRepository;
        use vibespeak::infrastructure::repositories::CommandRepository;
        use vibespeak::domain::entities::{VoiceCommand, CommandAction};

        let repo = InMemoryCommandRepository::new();

        let cmd = VoiceCommand::new(
            "to delete".to_string(),
            CommandAction::ShellCommand("echo delete".to_string())
        );
        let cmd_id = cmd.id.clone();

        repo.save(&cmd).await.expect("Failed to save");
        repo.delete(&cmd_id).await.expect("Failed to delete");

        let found = repo.find_by_id(&cmd_id).await.expect("Find failed");
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_command_repository_with_defaults() {
        use vibespeak::infrastructure::repositories::command_repository::InMemoryCommandRepository;
        use vibespeak::infrastructure::repositories::CommandRepository;

        let repo = InMemoryCommandRepository::new().with_defaults();

        let all = repo.find_all().await.expect("Failed to find all");
        assert!(all.len() >= 2); // At least hello and status
    }

    #[tokio::test]
    async fn test_json_file_command_repository_persistence() {
        use vibespeak::infrastructure::repositories::command_repository::JsonFileCommandRepository;
        use vibespeak::infrastructure::repositories::CommandRepository;
        use vibespeak::domain::entities::{VoiceCommand, CommandAction};

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test_commands.json");

        // Create repository and save a command
        {
            let repo = JsonFileCommandRepository::new(file_path.clone())
                .expect("Failed to create repo");

            let cmd = VoiceCommand::new(
                "persistent test".to_string(),
                CommandAction::ShellCommand("echo persistent".to_string())
            );
            repo.save(&cmd).await.expect("Failed to save");
        }

        // Create new repository instance and verify data persisted
        {
            let repo = JsonFileCommandRepository::new(file_path)
                .expect("Failed to create repo");

            let all = repo.find_all().await.expect("Failed to find all");
            assert_eq!(all.len(), 1);
            assert_eq!(all[0].text, "persistent test");
        }
    }
}

mod workflow_repository_tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_workflow_repository_save_and_find() {
        use vibespeak::infrastructure::repositories::workflow_repository::{
            InMemoryWorkflowRepository, WorkflowRepository
        };
        use vibespeak::domain::entities::{Workflow, WorkflowTrigger};

        let repo = InMemoryWorkflowRepository::new();

        let workflow = Workflow::new(
            "test workflow".to_string(),
            WorkflowTrigger::Manual
        );
        let workflow_id = workflow.id.clone();

        repo.save(&workflow).await.expect("Failed to save");

        let found = repo.find_by_id(&workflow_id).await.expect("Failed to find");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "test workflow");
    }

    #[tokio::test]
    async fn test_in_memory_workflow_repository_find_by_name() {
        use vibespeak::infrastructure::repositories::workflow_repository::{
            InMemoryWorkflowRepository, WorkflowRepository
        };
        use vibespeak::domain::entities::{Workflow, WorkflowTrigger};

        let repo = InMemoryWorkflowRepository::new();

        let workflow = Workflow::new(
            "unique_name".to_string(),
            WorkflowTrigger::VoiceCommand("start workflow".to_string())
        );
        repo.save(&workflow).await.expect("Failed to save");

        let found = repo.find_by_name("unique_name").await.expect("Failed to find");
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "unique_name");

        let not_found = repo.find_by_name("nonexistent").await.expect("Find failed");
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_in_memory_workflow_repository_find_enabled() {
        use vibespeak::infrastructure::repositories::workflow_repository::{
            InMemoryWorkflowRepository, WorkflowRepository
        };
        use vibespeak::domain::entities::{Workflow, WorkflowTrigger};

        let repo = InMemoryWorkflowRepository::new();

        let mut enabled_workflow = Workflow::new(
            "enabled".to_string(),
            WorkflowTrigger::Manual
        );
        enabled_workflow.enabled = true;

        let mut disabled_workflow = Workflow::new(
            "disabled".to_string(),
            WorkflowTrigger::Manual
        );
        disabled_workflow.enabled = false;

        repo.save(&enabled_workflow).await.expect("Failed to save");
        repo.save(&disabled_workflow).await.expect("Failed to save");

        let enabled = repo.find_enabled().await.expect("Failed to find enabled");
        assert_eq!(enabled.len(), 1);
        assert_eq!(enabled[0].name, "enabled");
    }

    #[tokio::test]
    async fn test_in_memory_workflow_repository_count() {
        use vibespeak::infrastructure::repositories::workflow_repository::{
            InMemoryWorkflowRepository, WorkflowRepository
        };
        use vibespeak::domain::entities::{Workflow, WorkflowTrigger};

        let repo = InMemoryWorkflowRepository::new();

        assert_eq!(repo.count().await.expect("Count failed"), 0);

        for i in 0..3 {
            let workflow = Workflow::new(
                format!("workflow {}", i),
                WorkflowTrigger::Manual
            );
            repo.save(&workflow).await.expect("Failed to save");
        }

        assert_eq!(repo.count().await.expect("Count failed"), 3);
    }

    #[tokio::test]
    async fn test_json_file_workflow_repository_persistence() {
        use vibespeak::infrastructure::repositories::workflow_repository::{
            JsonFileWorkflowRepository, WorkflowRepository
        };
        use vibespeak::domain::entities::{Workflow, WorkflowTrigger, WorkflowStep};

        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test_workflows.json");

        // Create and save workflow
        {
            let repo = JsonFileWorkflowRepository::new(file_path.clone())
                .expect("Failed to create repo");

            let mut workflow = Workflow::new(
                "persistent workflow".to_string(),
                WorkflowTrigger::VoiceCommand("run task".to_string())
            );
            workflow.add_step(WorkflowStep::ExecuteCommand("echo hello".to_string()));

            repo.save(&workflow).await.expect("Failed to save");
        }

        // Verify persistence
        {
            let repo = JsonFileWorkflowRepository::new(file_path)
                .expect("Failed to create repo");

            let all = repo.find_all().await.expect("Failed to find all");
            assert_eq!(all.len(), 1);
            assert_eq!(all[0].name, "persistent workflow");
            assert_eq!(all[0].steps.len(), 1);
        }
    }
}

mod session_repository_tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_session_repository_save_and_find() {
        use vibespeak::infrastructure::repositories::session_repository::InMemorySessionRepository;
        use vibespeak::infrastructure::repositories::SessionRepository;
        use vibespeak::domain::entities::RecognitionSession;

        let repo = InMemorySessionRepository::new();

        let session = RecognitionSession::new();
        let session_id = session.id.clone();

        repo.save(&session).await.expect("Failed to save");

        let found = repo.find_by_id(&session_id).await.expect("Failed to find");
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn test_in_memory_session_repository_find_active() {
        use vibespeak::infrastructure::repositories::session_repository::InMemorySessionRepository;
        use vibespeak::infrastructure::repositories::SessionRepository;
        use vibespeak::domain::entities::RecognitionSession;

        let repo = InMemorySessionRepository::new();

        // Create active session
        let active_session = RecognitionSession::new();
        repo.save(&active_session).await.expect("Failed to save");

        // Create completed session
        let mut completed_session = RecognitionSession::new();
        completed_session.end_session();
        repo.save(&completed_session).await.expect("Failed to save");

        let active = repo.find_active_sessions().await.expect("Failed to find active");
        assert_eq!(active.len(), 1);
    }

    #[test]
    fn test_session_repository_count() {
        use vibespeak::infrastructure::repositories::session_repository::InMemorySessionRepository;

        let repo = InMemorySessionRepository::new();
        assert_eq!(repo.count(), 0);
    }
}
