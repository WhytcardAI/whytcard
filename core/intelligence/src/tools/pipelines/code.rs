//! Phase C - Code Pipeline
//!
//! Execute and verify code during development.
//! Combines: cortex_execute + cortex_feedback
//!
//! Workflow from instructions:
//! 1. Write TEST first (TDD)
//! 2. Write CODE minimal
//! 3. cortex_execute: compile?
//! 4. runTests: tests pass?
//! 5. get_errors: IDE errors?

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Command to execute
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecuteCommand {
    /// The command to execute
    pub command: String,
    /// Working directory (optional)
    #[serde(default)]
    pub cwd: Option<String>,
    /// Environment variables (optional)
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Timeout in seconds (default: 60)
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
    /// Label for this command (for reporting)
    #[serde(default)]
    pub label: Option<String>,
}

fn default_timeout() -> u64 {
    60
}

/// Feedback for learning
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FeedbackDef {
    /// Rule ID that was applied
    pub rule_id: String,
    /// Whether the outcome was successful
    pub success: bool,
    /// Optional feedback message
    #[serde(default)]
    pub message: Option<String>,
}

/// Parameters for the code pipeline
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CodeParams {
    /// Commands to execute (in order)
    #[serde(default)]
    pub commands: Vec<ExecuteCommand>,

    /// Feedback to provide for learning
    #[serde(default)]
    pub feedback: Vec<FeedbackDef>,

    /// Stop on first failure (default: true)
    #[serde(default = "default_true")]
    pub stop_on_failure: bool,

    /// Capture stderr separately (default: false)
    #[serde(default)]
    pub separate_stderr: bool,

    /// Project root for relative paths
    #[serde(default)]
    pub project_root: Option<String>,

    /// Language context (rust, typescript, python)
    #[serde(default)]
    pub language: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Result of a single command execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecuteResult {
    /// Command that was executed
    pub command: String,
    /// Label if provided
    pub label: Option<String>,
    /// Whether command succeeded (exit code 0)
    pub success: bool,
    /// Exit code
    pub exit_code: i32,
    /// Standard output
    pub stdout: String,
    /// Standard error (if separate_stderr)
    pub stderr: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
}

/// Result of feedback operation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FeedbackResult {
    /// Rule ID
    pub rule_id: String,
    /// Whether feedback was recorded
    pub recorded: bool,
    /// New confidence after feedback
    pub new_confidence: f32,
}

/// Detected error from output
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DetectedError {
    /// Error type (compile, lint, test, runtime)
    pub error_type: String,
    /// File path if detected
    pub file: Option<String>,
    /// Line number if detected
    pub line: Option<u32>,
    /// Error message
    pub message: String,
    /// Severity (error, warning, info)
    pub severity: String,
}

/// Result from the code pipeline
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CodeResult {
    /// Results from command executions
    pub executions: Vec<ExecuteResult>,

    /// Results from feedback operations
    pub feedbacks: Vec<FeedbackResult>,

    /// All commands succeeded
    pub all_success: bool,

    /// Total commands executed
    pub commands_executed: usize,

    /// Commands that failed
    pub commands_failed: usize,

    /// Detected errors from output parsing
    #[serde(default)]
    pub detected_errors: Vec<DetectedError>,

    /// Detected warnings from output parsing
    #[serde(default)]
    pub detected_warnings: Vec<DetectedError>,

    /// Summary message
    pub summary: String,

    /// Recommended next action
    pub recommendation: Option<String>,
}

impl Default for CodeParams {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            feedback: Vec::new(),
            stop_on_failure: true,
            separate_stderr: false,
            project_root: None,
            language: None,
        }
    }
}

impl CodeParams {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_result_serialization() {
        let result = CodeResult {
            executions: vec![ExecuteResult {
                command: "cargo check".to_string(),
                label: Some("check".to_string()),
                success: true,
                exit_code: 0,
                stdout: "Compiling...".to_string(),
                stderr: String::new(),
                duration_ms: 1500,
            }],
            feedbacks: vec![],
            all_success: true,
            commands_executed: 1,
            commands_failed: 0,
            detected_errors: vec![],
            detected_warnings: vec![],
            summary: "All commands passed".to_string(),
            recommendation: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("all_success"));
        assert!(json.contains("cargo check"));
    }

    #[test]
    fn test_detected_error() {
        let error = DetectedError {
            error_type: "compile".to_string(),
            file: Some("src/main.rs".to_string()),
            line: Some(42),
            message: "expected `;`".to_string(),
            severity: "error".to_string(),
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("src/main.rs"));
        assert!(json.contains("42"));
    }
}
