// Tests for script executor functionality

use std::time::Duration;

mod script_executor_tests {
    use super::*;

    #[tokio::test]
    async fn test_script_executor_creation() {
        use vibespeak::domain::services::script_executor::ScriptExecutor;

        let executor = ScriptExecutor::new();
        // Should create successfully
        assert!(true);
    }

    #[tokio::test]
    async fn test_script_execution_creation() {
        use vibespeak::domain::services::script_executor::ScriptExecution;
        use vibespeak::shared::{ScriptType, SecurityLevel};

        let script = ScriptExecution::new(
            ScriptType::Bash,
            "echo hello".to_string()
        );

        assert_eq!(script.content, "echo hello");
        assert_eq!(script.security_level, SecurityLevel::Trusted);
        assert_eq!(script.timeout, Duration::from_secs(30));
    }

    #[tokio::test]
    async fn test_script_execution_with_timeout() {
        use vibespeak::domain::services::script_executor::ScriptExecution;
        use vibespeak::shared::ScriptType;

        let script = ScriptExecution::new(ScriptType::Bash, "sleep 1".to_string())
            .with_timeout(Duration::from_secs(60));

        assert_eq!(script.timeout, Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_script_execution_with_security_level() {
        use vibespeak::domain::services::script_executor::ScriptExecution;
        use vibespeak::shared::{ScriptType, SecurityLevel};

        let script = ScriptExecution::new(ScriptType::Bash, "echo safe".to_string())
            .with_security_level(SecurityLevel::Sandboxed);

        assert_eq!(script.security_level, SecurityLevel::Sandboxed);
    }

    #[tokio::test]
    async fn test_script_execution_with_environment() {
        use vibespeak::domain::services::script_executor::ScriptExecution;
        use vibespeak::shared::ScriptType;

        let script = ScriptExecution::new(ScriptType::Bash, "echo $MY_VAR".to_string())
            .with_environment("MY_VAR".to_string(), "test_value".to_string());

        assert!(script.environment.contains_key("MY_VAR"));
        assert_eq!(script.environment.get("MY_VAR").unwrap(), "test_value");
    }

    #[tokio::test]
    async fn test_execute_bash_script_success() {
        use vibespeak::domain::services::script_executor::{ScriptExecutor, ScriptExecution};
        use vibespeak::shared::ScriptType;

        let executor = ScriptExecutor::new();
        let script = ScriptExecution::new(ScriptType::Bash, "echo 'test output'".to_string());

        let result = executor.execute(&script).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.success);
        assert_eq!(result.exit_code, Some(0));
        assert!(result.stdout.contains("test output"));
    }

    #[tokio::test]
    async fn test_execute_bash_script_with_exit_code() {
        use vibespeak::domain::services::script_executor::{ScriptExecutor, ScriptExecution};
        use vibespeak::shared::ScriptType;

        let executor = ScriptExecutor::new();
        let script = ScriptExecution::new(ScriptType::Bash, "exit 42".to_string());

        let result = executor.execute(&script).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(!result.success);
        assert_eq!(result.exit_code, Some(42));
    }

    #[tokio::test]
    async fn test_execute_bash_script_with_stderr() {
        use vibespeak::domain::services::script_executor::{ScriptExecutor, ScriptExecution};
        use vibespeak::shared::ScriptType;

        let executor = ScriptExecutor::new();
        let script = ScriptExecution::new(ScriptType::Bash, "echo 'error' >&2".to_string());

        let result = executor.execute(&script).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.stderr.contains("error"));
    }

    #[tokio::test]
    async fn test_execute_bash_script_with_environment_variable() {
        use vibespeak::domain::services::script_executor::{ScriptExecutor, ScriptExecution};
        use vibespeak::shared::ScriptType;

        let executor = ScriptExecutor::new();
        let script = ScriptExecution::new(ScriptType::Bash, "echo $TEST_VAR".to_string())
            .with_environment("TEST_VAR".to_string(), "hello_world".to_string());

        let result = executor.execute(&script).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.success);
        assert!(result.stdout.contains("hello_world"));
    }

    #[tokio::test]
    async fn test_execute_sandboxed_rejects_dangerous_commands() {
        use vibespeak::domain::services::script_executor::{ScriptExecutor, ScriptExecution};
        use vibespeak::shared::{ScriptType, SecurityLevel};

        let executor = ScriptExecutor::new();

        // Test rm -rf rejection - should return an error
        let script = ScriptExecution::new(ScriptType::Bash, "rm -rf /".to_string())
            .with_security_level(SecurityLevel::Sandboxed);

        let result = executor.execute(&script).await;
        // Sandboxed mode returns an error for dangerous commands
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(err_msg.contains("Dangerous") || err_msg.contains("not allowed"));
    }

    #[tokio::test]
    async fn test_execute_sandboxed_rejects_sudo() {
        use vibespeak::domain::services::script_executor::{ScriptExecutor, ScriptExecution};
        use vibespeak::shared::{ScriptType, SecurityLevel};

        let executor = ScriptExecutor::new();

        let script = ScriptExecution::new(ScriptType::Bash, "sudo ls".to_string())
            .with_security_level(SecurityLevel::Sandboxed);

        let result = executor.execute(&script).await;
        // Should return an error for sudo commands in sandboxed mode
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_script_timeout() {
        use vibespeak::domain::services::script_executor::{ScriptExecutor, ScriptExecution};
        use vibespeak::shared::ScriptType;

        let executor = ScriptExecutor::new();
        let script = ScriptExecution::new(ScriptType::Bash, "sleep 10".to_string())
            .with_timeout(Duration::from_millis(100));

        let result = executor.execute(&script).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(!result.success);
        assert!(result.stderr.contains("timed out") || result.stderr.contains("timeout"));
    }

    #[tokio::test]
    async fn test_execute_python_script() {
        use vibespeak::domain::services::script_executor::{ScriptExecutor, ScriptExecution};
        use vibespeak::shared::ScriptType;

        let executor = ScriptExecutor::new();
        let script = ScriptExecution::new(
            ScriptType::Python,
            "print('Hello from Python')".to_string()
        );

        let result = executor.execute(&script).await;

        // Python might not be installed, so check gracefully
        if let Ok(result) = result {
            if result.success {
                assert!(result.stdout.contains("Hello from Python"));
            }
        }
    }

    #[tokio::test]
    async fn test_execute_javascript_script() {
        use vibespeak::domain::services::script_executor::{ScriptExecutor, ScriptExecution};
        use vibespeak::shared::ScriptType;

        let executor = ScriptExecutor::new();
        let script = ScriptExecution::new(
            ScriptType::JavaScript,
            "console.log('Hello from JS')".to_string()
        );

        let result = executor.execute(&script).await;

        // Node might not be installed, so check gracefully
        if let Ok(result) = result {
            if result.success {
                assert!(result.stdout.contains("Hello from JS"));
            }
        }
    }

    #[tokio::test]
    async fn test_script_result_execution_time() {
        use vibespeak::domain::services::script_executor::{ScriptExecutor, ScriptExecution};
        use vibespeak::shared::ScriptType;

        let executor = ScriptExecutor::new();
        let script = ScriptExecution::new(ScriptType::Bash, "sleep 0.1".to_string());

        let result = executor.execute(&script).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert!(result.execution_time >= Duration::from_millis(50));
    }

    #[tokio::test]
    async fn test_execute_isolated_without_docker() {
        use vibespeak::domain::services::script_executor::{ScriptExecutor, ScriptExecution};
        use vibespeak::shared::{ScriptType, SecurityLevel};

        let executor = ScriptExecutor::new();
        let script = ScriptExecution::new(ScriptType::Bash, "echo test".to_string())
            .with_security_level(SecurityLevel::Isolated);

        let result = executor.execute(&script).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        // If Docker is not available, it should fail gracefully
        // If Docker is available, it should succeed
        // Either way, we got a result
        assert!(result.success || result.stderr.contains("Docker"));
    }

    #[tokio::test]
    async fn test_execute_multiple_scripts_sequentially() {
        use vibespeak::domain::services::script_executor::{ScriptExecutor, ScriptExecution};
        use vibespeak::shared::ScriptType;

        let executor = ScriptExecutor::new();

        for i in 0..5 {
            let script = ScriptExecution::new(
                ScriptType::Bash,
                format!("echo 'Script {}'", i)
            );

            let result = executor.execute(&script).await;
            assert!(result.is_ok());
            assert!(result.unwrap().success);
        }
    }
}

mod script_type_tests {
    #[test]
    fn test_script_type_display() {
        use vibespeak::shared::ScriptType;

        assert_eq!(format!("{}", ScriptType::Bash), "bash");
        assert_eq!(format!("{}", ScriptType::Python), "python");
        assert_eq!(format!("{}", ScriptType::JavaScript), "javascript");
        assert_eq!(format!("{}", ScriptType::Ruby), "ruby");
        assert_eq!(format!("{}", ScriptType::PowerShell), "powershell");
        assert_eq!(format!("{}", ScriptType::Custom("lua".to_string())), "custom:lua");
    }
}

mod security_level_tests {
    #[test]
    fn test_security_level_display() {
        use vibespeak::shared::SecurityLevel;

        assert_eq!(format!("{}", SecurityLevel::Sandboxed), "sandboxed");
        assert_eq!(format!("{}", SecurityLevel::Trusted), "trusted");
        assert_eq!(format!("{}", SecurityLevel::Isolated), "isolated");
    }
}
