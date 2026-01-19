use crate::domain::entities::{
    ComparisonOperator, Condition, ErrorStrategy, Variable, VariableValue, Workflow, WorkflowStep,
};
use crate::domain::services::{
    browser_automation::BrowserAutomationService,
    script_executor::{ScriptExecution, ScriptExecutor},
};
use crate::shared::{Error, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionResult {
    pub workflow_id: String,
    pub success: bool,
    pub steps_executed: usize,
    pub total_steps: usize,
    pub execution_time: std::time::Duration,
    pub errors: Vec<String>,
    pub outputs: HashMap<String, serde_json::Value>,
}

#[async_trait]
pub trait WorkflowExecutor: Send + Sync {
    async fn execute_workflow(
        &self,
        workflow: &Workflow,
        variables: HashMap<String, VariableValue>,
    ) -> Result<WorkflowExecutionResult>;
    async fn validate_workflow(&self, workflow: &Workflow) -> Result<()>;
}

pub struct DefaultWorkflowExecutor {
    script_executor: Arc<ScriptExecutor>,
    browser_service: Arc<dyn BrowserAutomationService>,
}

impl DefaultWorkflowExecutor {
    pub fn new(
        script_executor: Arc<ScriptExecutor>,
        browser_service: Arc<dyn BrowserAutomationService>,
    ) -> Self {
        Self {
            script_executor,
            browser_service,
        }
    }
}

#[async_trait]
impl WorkflowExecutor for DefaultWorkflowExecutor {
    async fn execute_workflow(
        &self,
        workflow: &Workflow,
        initial_variables: HashMap<String, VariableValue>,
    ) -> Result<WorkflowExecutionResult> {
        let start_time = std::time::Instant::now();
        let mut variables = initial_variables.clone();
        let mut errors = Vec::new();
        let mut outputs = HashMap::new();
        let mut steps_executed = 0;

        for (index, step) in workflow.steps.iter().enumerate() {
            steps_executed = index + 1;

            match self.execute_step(step, &variables).await {
                Ok(step_output) => {
                    // Merge step outputs into variables
                    if let Some(output_vars) = step_output.get("variables") {
                        if let Some(output_map) = output_vars.as_object() {
                            for (key, value) in output_map {
                                variables.insert(key.clone(), VariableValue::Json(value.clone()));
                            }
                        }
                    }

                    // Store step output
                    outputs.insert(format!("step_{}", index), step_output);
                }
                Err(e) => {
                    errors.push(format!("Step {} failed: {}", index, e));

                    // Handle error based on strategy
                    match workflow.error_handling {
                        ErrorStrategy::Stop => {
                            return Ok(WorkflowExecutionResult {
                                workflow_id: workflow.id.clone(),
                                success: false,
                                steps_executed,
                                total_steps: workflow.steps.len(),
                                execution_time: start_time.elapsed(),
                                errors,
                                outputs,
                            });
                        }
                        ErrorStrategy::Continue => {
                            continue;
                        }
                        ErrorStrategy::Retry(retries) => {
                            // Simplified retry logic
                            if retries > 0 {
                                continue; // Would implement retry loop in practice
                            }
                        }
                        ErrorStrategy::Fallback(ref fallback_step) => {
                            // Execute fallback step
                            let _ = self.execute_step(fallback_step, &variables).await;
                            break;
                        }
                    }
                }
            }
        }

        Ok(WorkflowExecutionResult {
            workflow_id: workflow.id.clone(),
            success: errors.is_empty(),
            steps_executed,
            total_steps: workflow.steps.len(),
            execution_time: start_time.elapsed(),
            errors,
            outputs,
        })
    }

    async fn validate_workflow(&self, workflow: &Workflow) -> Result<()> {
        if workflow.name.trim().is_empty() {
            return Err(Error::Domain("Workflow name cannot be empty".to_string()));
        }

        if workflow.steps.is_empty() {
            return Err(Error::Domain(
                "Workflow must have at least one step".to_string(),
            ));
        }

        // Validate each step
        for (index, step) in workflow.steps.iter().enumerate() {
            self.validate_step(step)
                .await
                .map_err(|e| Error::Domain(format!("Step {}: {}", index, e)))?;
        }

        Ok(())
    }
}

impl DefaultWorkflowExecutor {
    async fn execute_step(
        &self,
        step: &WorkflowStep,
        variables: &HashMap<String, VariableValue>,
    ) -> Result<serde_json::Value> {
        match step {
            WorkflowStep::ExecuteCommand(command) => self.execute_command(command, variables).await,
            WorkflowStep::RunScript(script_exec) => {
                self.execute_script(script_exec, variables).await
            }
            WorkflowStep::BrowserAction(action) => {
                self.execute_browser_action(action, variables).await
            }
            WorkflowStep::IntegrationCall(service, params) => {
                self.execute_integration_call(service, params, variables)
                    .await
            }
            WorkflowStep::Conditional(condition, then_step, else_step) => {
                if self.evaluate_condition(condition, variables).await? {
                    Box::pin(self.execute_step(then_step, variables)).await
                } else {
                    Box::pin(self.execute_step(else_step, variables)).await
                }
            }
            WorkflowStep::Wait(duration) => {
                tokio::time::sleep(*duration).await;
                Ok(serde_json::json!({"waited": duration.as_millis()}))
            }
            WorkflowStep::SetVariable(name, value) => {
                Ok(serde_json::json!({"variable_set": name, "value": value}))
            }
            WorkflowStep::UserPrompt(message) => {
                // In practice, this would show a UI prompt
                tracing::info!("User prompt: {}", message);
                Ok(serde_json::json!({"prompt": message}))
            }
        }
    }

    async fn validate_step(&self, step: &WorkflowStep) -> Result<()> {
        match step {
            WorkflowStep::ExecuteCommand(cmd) => {
                if cmd.trim().is_empty() {
                    return Err(Error::Domain("Command cannot be empty".to_string()));
                }
            }
            WorkflowStep::RunScript(script) => {
                if script.content.trim().is_empty() {
                    return Err(Error::Domain("Script content cannot be empty".to_string()));
                }
            }
            WorkflowStep::BrowserAction(_) => {
                // Browser actions are generally valid
            }
            WorkflowStep::IntegrationCall(service, _) => {
                if service.trim().is_empty() {
                    return Err(Error::Domain(
                        "Integration service cannot be empty".to_string(),
                    ));
                }
            }
            WorkflowStep::Conditional(condition, _, _) => {
                self.validate_condition(condition)?;
            }
            WorkflowStep::Wait(_) => {
                // Duration is always valid
            }
            WorkflowStep::SetVariable(name, _) => {
                if name.trim().is_empty() {
                    return Err(Error::Domain("Variable name cannot be empty".to_string()));
                }
            }
            WorkflowStep::UserPrompt(message) => {
                if message.trim().is_empty() {
                    return Err(Error::Domain("Prompt message cannot be empty".to_string()));
                }
            }
        }
        Ok(())
    }

    async fn execute_command(
        &self,
        command: &str,
        variables: &HashMap<String, VariableValue>,
    ) -> Result<serde_json::Value> {
        let resolved_command = self.resolve_variables(command, variables);

        // Execute command using system shell
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&resolved_command)
            .output()
            .map_err(|e| Error::Infrastructure(format!("Failed to execute command: {}", e)))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Ok(serde_json::json!({
            "command": resolved_command,
            "exit_code": output.status.code(),
            "stdout": stdout,
            "stderr": stderr,
            "success": output.status.success()
        }))
    }

    async fn execute_script(
        &self,
        script: &crate::domain::entities::workflow::ScriptExecution,
        variables: &HashMap<String, VariableValue>,
    ) -> Result<serde_json::Value> {
        let resolved_script = crate::domain::services::script_executor::ScriptExecution {
            script_type: script.script_type.clone(),
            content: self.resolve_variables(&script.content, variables),
            arguments: script.arguments.clone(),
            timeout: script.timeout,
            security_level: script.security_level.clone(),
            working_directory: script.working_directory.clone(),
            environment: script.environment.clone(),
        };

        let result = self.script_executor.execute(&resolved_script).await?;

        Ok(serde_json::json!({
            "script_type": format!("{:?}", script.script_type),
            "success": result.success,
            "exit_code": result.exit_code,
            "stdout": result.stdout,
            "stderr": result.stderr,
            "execution_time_ms": result.execution_time.as_millis()
        }))
    }

    async fn execute_browser_action(
        &self,
        action: &crate::domain::entities::BrowserAction,
        variables: &HashMap<String, VariableValue>,
    ) -> Result<serde_json::Value> {
        // Create a temporary browser session for the action
        let session_config = crate::domain::entities::BrowserSession {
            browser: crate::domain::entities::BrowserType::Chrome,
            headless: true,
            profile: None,
            extensions: vec![],
        };

        let session_id = self.browser_service.start_session(session_config).await?;
        let result = self
            .browser_service
            .execute_action(&session_id, action.clone())
            .await?;
        self.browser_service.close_session(&session_id).await?;

        Ok(serde_json::json!({
            "browser_action": format!("{:?}", action),
            "success": result.success,
            "data": result.data,
            "has_screenshot": result.screenshot_path.is_some()
        }))
    }

    async fn execute_integration_call(
        &self,
        service: &str,
        params: &serde_json::Value,
        variables: &HashMap<String, VariableValue>,
    ) -> Result<serde_json::Value> {
        // Placeholder for integration calls
        tracing::info!("Integration call to {} with params: {}", service, params);
        Ok(serde_json::json!({
            "integration": service,
            "params": params,
            "status": "not_implemented"
        }))
    }

    async fn evaluate_condition(
        &self,
        condition: &Condition,
        variables: &HashMap<String, VariableValue>,
    ) -> Result<bool> {
        let left_value = self.resolve_variable_value(&condition.variable, variables);
        let right_value = &condition.value;

        match condition.operator {
            ComparisonOperator::Equals => Ok(left_value == right_value.to_string()),
            ComparisonOperator::NotEquals => Ok(left_value != right_value.to_string()),
            ComparisonOperator::GreaterThan => {
                let left_num = left_value.parse::<f64>().unwrap_or(0.0);
                let right_num = right_value.to_string().parse::<f64>().unwrap_or(0.0);
                Ok(left_num > right_num)
            }
            ComparisonOperator::LessThan => {
                let left_num = left_value.parse::<f64>().unwrap_or(0.0);
                let right_num = right_value.to_string().parse::<f64>().unwrap_or(0.0);
                Ok(left_num < right_num)
            }
            ComparisonOperator::Contains => Ok(left_value.contains(&right_value.to_string())),
            ComparisonOperator::StartsWith => Ok(left_value.starts_with(&right_value.to_string())),
            ComparisonOperator::EndsWith => Ok(left_value.ends_with(&right_value.to_string())),
        }
    }

    fn validate_condition(&self, condition: &Condition) -> Result<()> {
        if condition.variable.trim().is_empty() {
            return Err(Error::Domain(
                "Condition variable cannot be empty".to_string(),
            ));
        }
        Ok(())
    }

    fn resolve_variables(
        &self,
        template: &str,
        variables: &HashMap<String, VariableValue>,
    ) -> String {
        let mut result = template.to_string();

        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            let value_str = match value {
                VariableValue::String(s) => s.clone(),
                VariableValue::Number(n) => n.to_string(),
                VariableValue::Boolean(b) => b.to_string(),
                VariableValue::Json(j) => j.to_string(),
            };
            result = result.replace(&placeholder, &value_str);
        }

        result
    }

    fn resolve_variable_value(
        &self,
        variable: &str,
        variables: &HashMap<String, VariableValue>,
    ) -> String {
        variables
            .get(variable)
            .map(|v| match v {
                VariableValue::String(s) => s.clone(),
                VariableValue::Number(n) => n.to_string(),
                VariableValue::Boolean(b) => b.to_string(),
                VariableValue::Json(j) => j.to_string(),
            })
            .unwrap_or_else(|| "".to_string())
    }
}
