//! Executor Module - Execute with Adaptation
//!
//! The Executor runs the plan with OODA loops:
//! - Observe: Execute step and capture result
//! - Orient: Interpret the result
//! - Decide: Continue, adjust, or stop
//! - Act: Record and proceed

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An execution plan with steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    /// Plan ID
    pub id: String,

    /// Plan name/description
    pub name: String,

    /// Ordered execution steps
    pub steps: Vec<ExecutionStep>,

    /// Plan metadata
    pub metadata: HashMap<String, serde_json::Value>,

    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ExecutionPlan {
    /// Create a new execution plan
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            steps: Vec::new(),
            metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
        }
    }

    /// Add a step to the plan
    pub fn add_step(&mut self, step: ExecutionStep) {
        self.steps.push(step);
    }

    /// Add metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

/// A single execution step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStep {
    /// Step ID
    pub id: String,

    /// Step name
    pub name: String,

    /// Step action type
    pub action: StepAction,

    /// Tool to use (if applicable)
    pub tool: Option<String>,

    /// Parameters for the tool
    pub params: HashMap<String, serde_json::Value>,

    /// Expected outcome description
    pub expected_outcome: Option<String>,

    /// Whether this step is critical (stops on failure)
    pub critical: bool,

    /// Retry count on failure
    pub retry_count: u32,
}

impl ExecutionStep {
    /// Create a new execution step
    pub fn new(name: impl Into<String>, action: StepAction) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            action,
            tool: None,
            params: HashMap::new(),
            expected_outcome: None,
            critical: true,
            retry_count: 0,
        }
    }

    /// Set the tool for this step
    #[allow(dead_code)]
    pub fn with_tool(mut self, tool: impl Into<String>) -> Self {
        self.tool = Some(tool.into());
        self
    }

    /// Add a parameter
    pub fn with_param(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.params.insert(key.into(), value);
        self
    }

    /// Set expected outcome
    #[allow(dead_code)]
    pub fn with_expected(mut self, expected: impl Into<String>) -> Self {
        self.expected_outcome = Some(expected.into());
        self
    }

    /// Set as non-critical
    pub fn non_critical(mut self) -> Self {
        self.critical = false;
        self
    }

    /// Set retry count
    #[allow(dead_code)]
    pub fn with_retries(mut self, count: u32) -> Self {
        self.retry_count = count;
        self
    }
}

/// Types of step actions
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepAction {
    /// Analyze something
    Analyze,
    /// Generate code or content
    Generate,
    /// Execute a tool
    Tool,
    /// Search/query
    Search,
    /// Validate/verify
    Validate,
    /// Transform data
    Transform,
    /// Wait/delay
    Wait,
    /// Checkpoint for progress
    Checkpoint,
    /// Custom action
    Custom(String),
}

/// Result of a single step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    /// Step ID
    pub step_id: String,

    /// Whether the step succeeded
    pub success: bool,

    /// Output from the step
    pub output: Option<serde_json::Value>,

    /// Error message if failed
    pub error: Option<String>,

    /// Duration in milliseconds
    pub duration_ms: u64,

    /// Number of retries used
    pub retries_used: u32,
}

impl StepResult {
    /// Create a successful step result
    pub fn success(step_id: String, output: serde_json::Value, duration_ms: u64) -> Self {
        Self {
            step_id,
            success: true,
            output: Some(output),
            error: None,
            duration_ms,
            retries_used: 0,
        }
    }

    /// Create a failed step result
    pub fn failure(step_id: String, error: String, duration_ms: u64) -> Self {
        Self {
            step_id,
            success: false,
            output: None,
            error: Some(error),
            duration_ms,
            retries_used: 0,
        }
    }
}

/// Result of the full execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// Plan ID
    pub plan_id: String,

    /// Overall success
    pub success: bool,

    /// Step results
    pub step_results: Vec<StepResult>,

    /// Final output
    pub output: Option<serde_json::Value>,

    /// Total duration in milliseconds
    pub total_duration_ms: u64,

    /// Number of successful steps
    pub successful_steps: usize,

    /// Number of failed steps
    pub failed_steps: usize,

    /// Adjustments made during execution
    pub adjustments: Vec<String>,
}

impl ExecutionResult {
    /// Create a new execution result
    pub fn new(plan_id: String) -> Self {
        Self {
            plan_id,
            success: false,
            step_results: Vec::new(),
            output: None,
            total_duration_ms: 0,
            successful_steps: 0,
            failed_steps: 0,
            adjustments: Vec::new(),
        }
    }

    /// Add a step result
    pub fn add_step_result(&mut self, result: StepResult) {
        self.total_duration_ms += result.duration_ms;
        if result.success {
            self.successful_steps += 1;
        } else {
            self.failed_steps += 1;
        }
        self.step_results.push(result);
    }

    /// Record an adjustment
    pub fn add_adjustment(&mut self, adjustment: impl Into<String>) {
        self.adjustments.push(adjustment.into());
    }

    /// Finalize the result
    pub fn finalize(&mut self, output: Option<serde_json::Value>) {
        self.success = self.failed_steps == 0;
        self.output = output;
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f32 {
        let total = self.successful_steps + self.failed_steps;
        if total == 0 {
            return 0.0;
        }
        self.successful_steps as f32 / total as f32
    }
}

/// OODA loop decision
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum OodaDecision {
    /// Continue with next step
    Continue,
    /// Adjust the plan
    Adjust,
    /// Stop execution
    Stop,
    /// Retry current step
    Retry,
}

/// The Executor module
pub struct Executor {
    /// Maximum steps to execute
    max_steps: usize,

    /// Maximum retries per step
    max_retries: u32,
}

impl Executor {
    /// Create a new Executor
    pub fn new(max_steps: usize) -> Self {
        Self {
            max_steps,
            max_retries: 3,
        }
    }

    /// Execute a plan
    pub async fn execute(&self, plan: ExecutionPlan) -> Result<ExecutionResult> {
        let mut result = ExecutionResult::new(plan.id.clone());
        let start_time = std::time::Instant::now();

        for (idx, step) in plan.steps.iter().enumerate() {
            if idx >= self.max_steps {
                result.add_adjustment(format!("Stopped at step {} (max steps reached)", idx));
                break;
            }

            // Execute step with OODA
            let step_result = self.execute_step_ooda(step).await;

            // Orient: Interpret result
            let decision = self.orient(&step_result, step);

            // Record result
            result.add_step_result(step_result.clone());

            // Decide and Act
            match decision {
                OodaDecision::Continue => continue,
                OodaDecision::Adjust => {
                    result.add_adjustment(format!("Adjusted after step: {}", step.name));
                    continue;
                }
                OodaDecision::Stop => {
                    result.add_adjustment(format!("Stopped at step: {}", step.name));
                    break;
                }
                OodaDecision::Retry => {
                    // Already handled in execute_step_ooda
                    continue;
                }
            }
        }

        result.total_duration_ms = start_time.elapsed().as_millis() as u64;
        result.finalize(None);

        Ok(result)
    }

    /// Execute a single step with OODA loop
    async fn execute_step_ooda(&self, step: &ExecutionStep) -> StepResult {
        let mut retries = 0;

        loop {
            let start_time = std::time::Instant::now();

            // Observe: Execute the step
            let outcome = self.observe(step).await;

            let duration_ms = start_time.elapsed().as_millis() as u64;

            match outcome {
                Ok(output) => {
                    let mut result = StepResult::success(step.id.clone(), output, duration_ms);
                    result.retries_used = retries;
                    return result;
                }
                Err(e) => {
                    if retries < step.retry_count && retries < self.max_retries {
                        retries += 1;
                        tracing::warn!("Step {} failed, retry {}/{}", step.name, retries, step.retry_count);
                        continue;
                    }

                    let mut result = StepResult::failure(step.id.clone(), e.to_string(), duration_ms);
                    result.retries_used = retries;
                    return result;
                }
            }
        }
    }

    /// Observe: Execute the step and capture result
    async fn observe(&self, step: &ExecutionStep) -> Result<serde_json::Value> {
        // For now, return a placeholder
        // In full implementation, this would dispatch to actual tool execution
        tracing::debug!("Executing step: {} ({:?})", step.name, step.action);

        Ok(serde_json::json!({
            "step": step.name,
            "action": format!("{:?}", step.action),
            "status": "completed"
        }))
    }

    /// Orient: Interpret the result and decide next action
    fn orient(&self, result: &StepResult, step: &ExecutionStep) -> OodaDecision {
        if result.success {
            OodaDecision::Continue
        } else if step.critical {
            OodaDecision::Stop
        } else {
            OodaDecision::Continue
        }
    }

    /// Create a simple plan from a perception result
    pub fn create_plan_from_perception(
        &self,
        perception: &super::perceiver::PerceptionResult,
    ) -> ExecutionPlan {
        let mut plan = ExecutionPlan::new(format!("{:?} task", perception.intent));

        // Add analysis step
        plan.add_step(
            ExecutionStep::new("Analyze requirements", StepAction::Analyze)
                .with_param("query", serde_json::Value::String(perception.query.clone()))
        );

        // Add research step if needed
        if perception.needs_research {
            plan.add_step(
                ExecutionStep::new("Research documentation", StepAction::Search)
                    .with_param("labels", serde_json::json!(perception.labels.iter().map(|l| l.as_str()).collect::<Vec<_>>()))
                    .non_critical()
            );
        }

        // Add main execution step based on intent
        match perception.intent {
            super::perceiver::Intent::Create => {
                plan.add_step(
                    ExecutionStep::new("Generate content", StepAction::Generate)
                        .with_param("type", serde_json::Value::String("create".to_string()))
                );
            }
            super::perceiver::Intent::Search => {
                plan.add_step(
                    ExecutionStep::new("Search codebase", StepAction::Search)
                        .with_param("type", serde_json::Value::String("search".to_string()))
                );
            }
            super::perceiver::Intent::Debug => {
                plan.add_step(
                    ExecutionStep::new("Analyze issue", StepAction::Analyze)
                        .with_param("type", serde_json::Value::String("debug".to_string()))
                );
                plan.add_step(
                    ExecutionStep::new("Apply fix", StepAction::Generate)
                        .with_param("type", serde_json::Value::String("fix".to_string()))
                );
            }
            _ => {
                plan.add_step(
                    ExecutionStep::new("Execute task", StepAction::Tool)
                        .with_param("intent", serde_json::Value::String(perception.intent.as_str().to_string()))
                );
            }
        }

        // Add validation step
        plan.add_step(
            ExecutionStep::new("Validate result", StepAction::Validate)
                .non_critical()
        );

        plan
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_plan() {
        let mut plan = ExecutionPlan::new("Test plan");
        plan.add_step(ExecutionStep::new("Step 1", StepAction::Analyze));
        plan.add_step(ExecutionStep::new("Step 2", StepAction::Generate));

        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.name, "Test plan");
    }

    #[test]
    fn test_step_result() {
        let success = StepResult::success(
            "step-1".to_string(),
            serde_json::json!({"result": "ok"}),
            100
        );
        assert!(success.success);

        let failure = StepResult::failure(
            "step-2".to_string(),
            "Something went wrong".to_string(),
            50
        );
        assert!(!failure.success);
    }

    #[test]
    fn test_execution_result() {
        let mut result = ExecutionResult::new("plan-1".to_string());
        result.add_step_result(StepResult::success("s1".into(), serde_json::json!({}), 100));
        result.add_step_result(StepResult::success("s2".into(), serde_json::json!({}), 100));
        result.add_step_result(StepResult::failure("s3".into(), "error".into(), 50));

        assert_eq!(result.successful_steps, 2);
        assert_eq!(result.failed_steps, 1);
        assert!((result.success_rate() - 0.666).abs() < 0.01);
    }
}
