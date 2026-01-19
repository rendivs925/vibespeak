use crate::shared::{Error, Result, ScriptType, SecurityLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptExecution {
    pub script_type: ScriptType,
    pub content: String,
    pub arguments: Vec<String>,
    pub timeout: Duration,
    pub security_level: SecurityLevel,
    pub working_directory: Option<String>,
    pub environment: HashMap<String, String>,
}

impl ScriptExecution {
    pub fn new(script_type: ScriptType, content: String) -> Self {
        Self {
            script_type,
            content,
            arguments: Vec::new(),
            timeout: Duration::from_secs(30),
            security_level: SecurityLevel::Trusted,
            working_directory: None,
            environment: HashMap::new(),
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn with_security_level(mut self, level: SecurityLevel) -> Self {
        self.security_level = level;
        self
    }

    pub fn with_argument(mut self, arg: String) -> Self {
        self.arguments.push(arg);
        self
    }

    pub fn with_environment(mut self, key: String, value: String) -> Self {
        self.environment.insert(key, value);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptResult {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub execution_time: Duration,
}

pub struct ScriptExecutor;

impl ScriptExecutor {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, script: &ScriptExecution) -> Result<ScriptResult> {
        let start_time = std::time::Instant::now();

        // Validate security level
        self.validate_security(script)?;

        // Execute based on security level and script type
        let result = if script.security_level == SecurityLevel::Isolated {
            // Use containerized execution for isolated scripts
            self.execute_in_container(script).await
        } else {
            // Execute directly based on script type
            match &script.script_type {
                ScriptType::Bash => self.execute_bash(script).await,
                ScriptType::Python => self.execute_python(script).await,
                ScriptType::JavaScript => self.execute_javascript(script).await,
                ScriptType::Ruby => self.execute_ruby(script).await,
                ScriptType::PowerShell => self.execute_powershell(script).await,
                ScriptType::Custom(interpreter) => self.execute_custom(script, interpreter).await,
            }
        };

        let execution_time = start_time.elapsed();

        match result {
            Ok((exit_code, stdout, stderr)) => Ok(ScriptResult {
                success: exit_code == 0,
                exit_code: Some(exit_code),
                stdout,
                stderr,
                execution_time,
            }),
            Err(e) => Ok(ScriptResult {
                success: false,
                exit_code: None,
                stdout: String::new(),
                stderr: format!("Execution error: {}", e),
                execution_time,
            }),
        }
    }

    fn validate_security(&self, script: &ScriptExecution) -> Result<()> {
        match script.security_level {
            SecurityLevel::Sandboxed => {
                // Restrict dangerous operations
                let dangerous_patterns = [
                    "rm -rf",
                    "sudo",
                    "chmod 777",
                    "mkfs",
                    "dd if=",
                    "> /dev/",
                    "curl | sh",
                    "wget | sh",
                    "eval",
                    ":(){ :|:& };:", // Fork bomb
                ];

                for pattern in dangerous_patterns {
                    if script.content.contains(pattern) {
                        return Err(Error::Infrastructure(format!(
                            "Dangerous operation '{}' not allowed in sandboxed mode",
                            pattern
                        )));
                    }
                }
                tracing::info!("Sandboxed execution validated");
            }
            SecurityLevel::Trusted => {
                // Allow most operations but log them
                tracing::warn!(
                    "Executing trusted script: {}...",
                    script.content.chars().take(50).collect::<String>()
                );
            }
            SecurityLevel::Isolated => {
                // Container isolation - requires Docker to be available
                tracing::info!("Isolated execution mode - using container sandboxing");
            }
        }
        Ok(())
    }

    /// Check if Docker is available for containerized execution
    fn is_docker_available() -> bool {
        std::process::Command::new("docker")
            .arg("--version")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }

    /// Execute script in a Docker container for isolation
    async fn execute_in_container(
        &self,
        script: &ScriptExecution,
    ) -> Result<(i32, String, String)> {
        if !Self::is_docker_available() {
            return Err(Error::Infrastructure(
                "Docker not available for isolated execution. Install Docker or use a different security level.".to_string()
            ));
        }

        // Determine the Docker image based on script type
        let image = match &script.script_type {
            ScriptType::Bash => "alpine:latest",
            ScriptType::Python => "python:3-slim",
            ScriptType::JavaScript => "node:slim",
            ScriptType::Ruby => "ruby:slim",
            ScriptType::PowerShell => "mcr.microsoft.com/powershell:latest",
            ScriptType::Custom(_) => "alpine:latest",
        };

        // Build the Docker command
        let mut docker_cmd = Command::new("docker");
        docker_cmd
            .arg("run")
            .arg("--rm") // Remove container after execution
            .arg("--network=none") // No network access
            .arg("--memory=256m") // Memory limit
            .arg("--cpus=0.5") // CPU limit
            .arg("--pids-limit=100") // Process limit
            .arg("--read-only") // Read-only filesystem
            .arg("--tmpfs=/tmp:size=64m,mode=1777") // Writable /tmp
            .arg("--security-opt=no-new-privileges") // No privilege escalation
            .arg(image);

        // Add the script command
        match &script.script_type {
            ScriptType::Bash => {
                docker_cmd.args(["sh", "-c", &script.content]);
            }
            ScriptType::Python => {
                docker_cmd.args(["python", "-c", &script.content]);
            }
            ScriptType::JavaScript => {
                docker_cmd.args(["node", "-e", &script.content]);
            }
            ScriptType::Ruby => {
                docker_cmd.args(["ruby", "-e", &script.content]);
            }
            ScriptType::PowerShell => {
                docker_cmd.args(["pwsh", "-Command", &script.content]);
            }
            ScriptType::Custom(interpreter) => {
                docker_cmd.args([interpreter, "-c", &script.content]);
            }
        }

        tracing::info!("Executing in container with image: {}", image);

        // Execute with timeout
        match timeout(script.timeout, docker_cmd.output()).await {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                Ok((output.status.code().unwrap_or(-1), stdout, stderr))
            }
            Ok(Err(e)) => Err(Error::Infrastructure(format!(
                "Docker execution failed: {}",
                e
            ))),
            Err(_) => {
                // Kill any running containers on timeout (best effort)
                let _ = std::process::Command::new("docker")
                    .args(["container", "prune", "-f"])
                    .output();
                Err(Error::Infrastructure(
                    "Containerized script execution timed out".to_string(),
                ))
            }
        }
    }

    async fn execute_bash(&self, script: &ScriptExecution) -> Result<(i32, String, String)> {
        let mut cmd = Command::new("bash");
        cmd.arg("-c").arg(&script.content);

        // Set working directory if specified
        if let Some(ref dir) = script.working_directory {
            cmd.current_dir(dir);
        }

        // Set environment variables
        for (key, value) in &script.environment {
            cmd.env(key, value);
        }

        // Execute with timeout using tokio::process::Command
        match tokio::time::timeout(script.timeout, cmd.output()).await {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                Ok((output.status.code().unwrap_or(-1), stdout, stderr))
            }
            Ok(Err(e)) => Err(Error::Infrastructure(format!(
                "Failed to execute command: {}",
                e
            ))),
            Err(_) => Err(Error::Infrastructure(
                "Script execution timed out".to_string(),
            )),
        }
    }

    async fn execute_python(&self, script: &ScriptExecution) -> Result<(i32, String, String)> {
        let mut cmd = Command::new("python3");
        cmd.arg("-c").arg(&script.content);

        self.execute_interpreter(cmd, script).await
    }

    async fn execute_javascript(&self, script: &ScriptExecution) -> Result<(i32, String, String)> {
        let mut cmd = Command::new("node");
        cmd.arg("-e").arg(&script.content);

        self.execute_interpreter(cmd, script).await
    }

    async fn execute_ruby(&self, script: &ScriptExecution) -> Result<(i32, String, String)> {
        let mut cmd = Command::new("ruby");
        cmd.arg("-e").arg(&script.content);

        self.execute_interpreter(cmd, script).await
    }

    async fn execute_powershell(&self, script: &ScriptExecution) -> Result<(i32, String, String)> {
        let mut cmd = Command::new("powershell");
        cmd.arg("-Command").arg(&script.content);

        self.execute_interpreter(cmd, script).await
    }

    async fn execute_custom(
        &self,
        script: &ScriptExecution,
        interpreter: &str,
    ) -> Result<(i32, String, String)> {
        let mut cmd = Command::new(interpreter);
        cmd.arg(&script.content);

        self.execute_interpreter(cmd, script).await
    }

    async fn execute_interpreter(
        &self,
        mut cmd: Command,
        script: &ScriptExecution,
    ) -> Result<(i32, String, String)> {
        // Set working directory if specified
        if let Some(ref dir) = script.working_directory {
            cmd.current_dir(dir);
        }

        // Set environment variables
        for (key, value) in &script.environment {
            cmd.env(key, value);
        }

        // Execute with timeout using tokio::process::Command
        match tokio::time::timeout(script.timeout, cmd.output()).await {
            Ok(Ok(output)) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                Ok((output.status.code().unwrap_or(-1), stdout, stderr))
            }
            Ok(Err(e)) => Err(Error::Infrastructure(format!(
                "Failed to execute command: {}",
                e
            ))),
            Err(_) => Err(Error::Infrastructure(
                "Script execution timed out".to_string(),
            )),
        }
    }
}
