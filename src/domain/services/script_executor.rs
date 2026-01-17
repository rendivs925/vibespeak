use crate::shared::{Result, Error, ScriptType, SecurityLevel};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::Duration;
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
        self.validate_security(&script)?;

        // Execute based on script type
        let result = match script.script_type {
            ScriptType::Bash => self.execute_bash(script).await,
            ScriptType::Python => self.execute_python(script).await,
            ScriptType::JavaScript => self.execute_javascript(script).await,
            ScriptType::Ruby => self.execute_ruby(script).await,
            ScriptType::PowerShell => self.execute_powershell(script).await,
            ScriptType::Custom(ref interpreter) => self.execute_custom(script, interpreter).await,
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
                if script.content.contains("rm -rf") || script.content.contains("sudo") {
                    return Err(Error::Infrastructure("Dangerous operations not allowed in sandboxed mode".to_string()));
                }
            }
            SecurityLevel::Trusted => {
                // Allow most operations but log them
                tracing::warn!("Executing trusted script: {}", script.content.chars().take(50).collect::<String>());
            }
            SecurityLevel::Isolated => {
                // TODO: Implement container isolation
                tracing::info!("Isolated execution not yet implemented, falling back to trusted mode");
            }
        }
        Ok(())
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

        // Set up pipes for stdout/stderr
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        // Execute with timeout
        let spawn_result = cmd.spawn();
        match timeout(script.timeout, async move { spawn_result }).await {
            Ok(Ok(mut child)) => {
                match timeout(script.timeout, child.wait_with_output()).await {
                    Ok(Ok(output)) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        Ok((output.status.code().unwrap_or(-1), stdout, stderr))
                    }
                    Ok(Err(e)) => Err(Error::Infrastructure(format!("Failed to read output: {}", e))),
                    Err(_) => {
                        let _ = child.kill().await;
                        Err(Error::Infrastructure("Script execution timed out".to_string()))
                    }
                }
            }
            Ok(Err(e)) => Err(Error::Infrastructure(format!("Failed to spawn process: {}", e))),
            Err(_) => Err(Error::Infrastructure("Script spawn timed out".to_string())),
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

    async fn execute_custom(&self, script: &ScriptExecution, interpreter: &str) -> Result<(i32, String, String)> {
        let mut cmd = Command::new(interpreter);
        cmd.arg(&script.content);

        self.execute_interpreter(cmd, script).await
    }

    async fn execute_interpreter(&self, mut cmd: Command, script: &ScriptExecution) -> Result<(i32, String, String)> {
        // Set working directory if specified
        if let Some(ref dir) = script.working_directory {
            cmd.current_dir(dir);
        }

        // Set environment variables
        for (key, value) in &script.environment {
            cmd.env(key, value);
        }

        // Set up pipes
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

        // Execute with timeout
        let spawn_result = cmd.spawn();
        match timeout(script.timeout, async move { spawn_result }).await {
            Ok(Ok(mut child)) => {
                match timeout(script.timeout, child.wait_with_output()).await {
                    Ok(Ok(output)) => {
                        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        Ok((output.status.code().unwrap_or(-1), stdout, stderr))
                    }
                    Ok(Err(e)) => Err(Error::Infrastructure(format!("Failed to read output: {}", e))),
                    Err(_) => {
                        let _ = child.kill().await;
                        Err(Error::Infrastructure("Script execution timed out".to_string()))
                    }
                }
            }
            Ok(Err(e)) => Err(Error::Infrastructure(format!("Failed to spawn process: {}", e))),
            Err(_) => Err(Error::Infrastructure("Script spawn timed out".to_string())),
        }
    }
}