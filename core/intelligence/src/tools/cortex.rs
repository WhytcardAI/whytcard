//! CORTEX MCP Tools
//!
//! Tools for cognitive processing via CORTEX engine:
//! - cortex_process: Main cognitive processing entry point
//! - cortex_feedback: Provide feedback for learning
//! - cortex_stats: Get cognitive engine statistics
//! - cortex_cleanup: Cleanup old data

use crate::cortex::{CortexEngine, CortexResult};
use crate::cortex::CortexConfig;
use crate::error::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::OnceCell;

// ============================================================================
// Task Type Classification (for prompt loading)
// ============================================================================

/// Type of task for automatic prompt loading
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TaskType {
    /// Code writing or modification
    Code,
    /// Information research
    Research,
    /// Error or bug fixing
    Fix,
    /// File or project creation
    Create,
    /// Code review
    Review,
    /// Documentation writing
    Document,
}

impl TaskType {
    /// Get the prompt key suffix for this task type
    pub fn prompt_key(&self) -> &'static str {
        match self {
            TaskType::Code => "code",
            TaskType::Research => "research:general",
            TaskType::Fix => "fix:error",
            TaskType::Create => "create:file",
            TaskType::Review => "review",
            TaskType::Document => "document",
        }
    }
}

// ============================================================================
// Global CORTEX Engine Instance
// ============================================================================

static CORTEX_ENGINE: OnceCell<Arc<CortexEngine>> = OnceCell::const_new();

/// Initialize the global CORTEX engine
pub async fn init_cortex(data_path: &Path, config: CortexConfig) -> Result<()> {
    let engine = CortexEngine::new(data_path, config).await?;
    CORTEX_ENGINE
        .set(Arc::new(engine))
        .map_err(|_| crate::error::IntelligenceError::config("CORTEX already initialized"))?;
    Ok(())
}

/// Get the global CORTEX engine
pub fn get_cortex() -> Result<Arc<CortexEngine>> {
    CORTEX_ENGINE
        .get()
        .cloned()
        .ok_or_else(|| crate::error::IntelligenceError::config("CORTEX not initialized"))
}

// ============================================================================
// cortex_process - Main cognitive processing
// ============================================================================

/// Input parameters for cortex_process tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CortexProcessParams {
    /// The query or task to process
    pub query: String,

    /// Optional session ID for context continuity
    #[serde(default)]
    pub session_id: Option<String>,

    /// Optional additional context
    #[serde(default)]
    pub context: Option<String>,

    /// Whether to enable auto-learning from this interaction
    #[serde(default = "default_true")]
    pub auto_learn: bool,

    /// Task type for automatic prompt loading (Code, Research, Fix, Create, Review, Document)
    #[serde(default)]
    pub task_type: Option<TaskType>,

    /// Programming language for code-related prompts (rust, typescript, python)
    #[serde(default)]
    pub language: Option<String>,

    /// Whether to inject doubt rules (default: true)
    #[serde(default = "default_true")]
    pub inject_doubt: bool,

    /// Optional file path to filter instructions by applyTo pattern
    #[serde(default)]
    pub file_path: Option<String>,

    /// Whether to inject instructions from .instructions.md files (default: true)
    #[serde(default = "default_true")]
    pub inject_instructions: bool,
}

fn default_true() -> bool {
    true
}

/// Output from cortex_process tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CortexProcessResult {
    /// Whether processing was successful
    pub success: bool,

    /// The main output/response
    pub output: String,

    /// Detected intent
    pub intent: String,

    /// Detected task labels
    pub labels: Vec<String>,

    /// Confidence score (0.0-1.0)
    pub confidence: f32,

    /// Whether external research was needed
    pub research_needed: bool,

    /// Number of steps executed
    pub steps_executed: usize,

    /// Execution duration in milliseconds
    pub duration_ms: u128,

    /// Recommendations for follow-up
    pub recommendations: Vec<String>,

    /// Session ID for continuity
    pub session_id: Option<String>,

    /// Prompts that were loaded and injected
    pub loaded_prompts: Vec<String>,

    /// Number of instructions injected
    pub instructions_count: usize,
}

impl From<CortexResult> for CortexProcessResult {
    fn from(result: CortexResult) -> Self {
        Self {
            success: result.success,
            output: result.result.to_string(),
            intent: format!("{:?}", result.perception.intent),
            labels: result.perception.labels.iter().map(|l| l.as_str().to_string()).collect(),
            confidence: result.confidence,
            research_needed: result.execution.research_performed,
            steps_executed: result.execution.steps_executed,
            duration_ms: result.execution.duration_ms as u128,
            recommendations: result.next_actions,
            session_id: None,
            loaded_prompts: Vec::new(),
            instructions_count: 0,
        }
    }
}

/// Process a query through CORTEX cognitive engine
pub async fn cortex_process(params: CortexProcessParams) -> Result<CortexProcessResult> {
    let engine = get_cortex()?;

    // Start session if provided
    let session_id = if params.session_id.is_some() {
        let sid = engine.start_session(None).await?;
        Some(sid)
    } else {
        None
    };

    // Build context JSON
    let mut context_obj = serde_json::Map::new();

    // Add user context if provided
    if let Some(user_ctx) = &params.context {
        context_obj.insert("user_context".to_string(), serde_json::json!(user_ctx));
    }

    // Inject instructions if enabled
    let mut instructions_count = 0;
    if params.inject_instructions {
        // Get instructions prompt (filtered by file if provided)
        let instructions_prompt = engine.get_instructions_prompt(params.file_path.as_deref()).await;

        if !instructions_prompt.is_empty() {
            // Count instructions
            if let Some(file_path) = &params.file_path {
                // If filtering by file, count matching instructions
                instructions_count = engine.get_instructions_for_file(file_path).await.len();
            } else {
                // Count all loaded instructions from stats
                let stats = engine.get_instructions_stats().await;
                instructions_count = stats.get("count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
            }

            context_obj.insert("system_instructions".to_string(), serde_json::json!(instructions_prompt));
        }
    }

    // Build final context
    let context = if context_obj.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(context_obj))
    };

    // Process through CORTEX
    let result = engine.process(&params.query, context).await?;

    // End session if we started one
    if session_id.is_some() {
        engine.end_session().await?;
    }

    let mut output: CortexProcessResult = result.into();
    output.session_id = session_id;
    output.instructions_count = instructions_count;

    Ok(output)
}

// ============================================================================
// cortex_feedback - Provide feedback for learning
// ============================================================================

/// Input parameters for cortex_feedback tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CortexFeedbackParams {
    /// The rule ID to provide feedback for
    pub rule_id: String,

    /// Whether the outcome was successful
    pub success: bool,

    /// Optional feedback message
    #[serde(default)]
    pub message: Option<String>,
}

/// Output from cortex_feedback tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CortexFeedbackResult {
    /// Whether feedback was recorded
    pub recorded: bool,

    /// New confidence score after feedback
    pub new_confidence: f32,

    /// Message
    pub message: String,
}

/// Provide feedback to update learning
pub async fn cortex_feedback(params: CortexFeedbackParams) -> Result<CortexFeedbackResult> {
    let engine = get_cortex()?;

    let new_confidence = engine.provide_feedback(&params.rule_id, params.success).await?;

    Ok(CortexFeedbackResult {
        recorded: true,
        new_confidence,
        message: format!(
            "Feedback recorded for rule {}. New confidence: {:.2}%",
            params.rule_id,
            new_confidence * 100.0
        ),
    })
}

// ============================================================================
// cortex_stats - Get cognitive engine statistics
// ============================================================================

/// Input parameters for cortex_stats (empty - no params needed)
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct CortexStatsParams {}

/// Output from cortex_stats tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CortexStatsResult {
    /// Memory statistics
    pub memory: MemoryStatsDetail,

    /// Engine status
    pub status: String,

    /// Uptime in seconds
    pub uptime_secs: u64,
}

/// Detailed memory statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryStatsDetail {
    /// Semantic memory facts count
    pub semantic_facts: usize,

    /// Episodic memory events count
    pub episodic_events: usize,

    /// Procedural memory rules count
    pub procedural_rules: usize,
}

/// Get CORTEX engine statistics
pub async fn cortex_stats(_params: CortexStatsParams) -> Result<CortexStatsResult> {
    let engine = get_cortex()?;

    let stats = engine.get_stats().await;

    // Extract values from JSON
    let semantic_facts = stats.get("semantic")
        .and_then(|s| s.get("total_facts"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let episodic_events = stats.get("episodic")
        .and_then(|s| s.get("total_episodes"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let procedural_rules = stats.get("procedural")
        .and_then(|s| s.get("total_rules"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    Ok(CortexStatsResult {
        memory: MemoryStatsDetail {
            semantic_facts,
            episodic_events,
            procedural_rules,
        },
        status: "running".to_string(),
        uptime_secs: 0, // TODO: Track actual uptime
    })
}

// ============================================================================
// cortex_cleanup - Cleanup old data
// ============================================================================

/// Input parameters for cortex_cleanup tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CortexCleanupParams {
    /// Number of days to retain data (default: 30)
    #[serde(default = "default_retention_days")]
    pub retention_days: i64,
}

fn default_retention_days() -> i64 {
    30
}

/// Output from cortex_cleanup tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CortexCleanupResult {
    /// Number of items cleaned up
    pub cleaned_count: usize,

    /// Message
    pub message: String,
}

/// Cleanup old data from CORTEX memory
pub async fn cortex_cleanup(params: CortexCleanupParams) -> Result<CortexCleanupResult> {
    let engine = get_cortex()?;

    let cleaned = engine.cleanup(params.retention_days).await?;

    Ok(CortexCleanupResult {
        cleaned_count: cleaned,
        message: format!(
            "Cleaned {} old records (retention: {} days)",
            cleaned, params.retention_days
        ),
    })
}

// ============================================================================
// cortex_execute - Execute shell commands
// ============================================================================

/// Input parameters for cortex_execute tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CortexExecuteParams {
    /// The command to execute
    pub command: String,

    /// Working directory for the command (optional)
    #[serde(default)]
    pub cwd: Option<String>,

    /// Environment variables to set (optional)
    #[serde(default)]
    pub env: Option<std::collections::HashMap<String, String>>,

    /// Timeout in seconds (default: 60)
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Whether to capture stderr separately (default: false, merged with stdout)
    #[serde(default)]
    pub separate_stderr: bool,
}

fn default_timeout() -> u64 {
    60
}

/// Output from cortex_execute tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CortexExecuteResult {
    /// Whether the command succeeded (exit code 0)
    pub success: bool,

    /// Exit code of the command
    pub exit_code: i32,

    /// Standard output
    pub stdout: String,

    /// Standard error (if separate_stderr was true, otherwise empty)
    pub stderr: String,

    /// Execution duration in milliseconds
    pub duration_ms: u64,

    /// The command that was executed
    pub command: String,
}

// ============================================================================
// cortex_instructions - Manage workspace instructions
// ============================================================================

/// Action to perform on instructions
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum InstructionsAction {
    /// List all loaded instructions
    List,
    /// Reload instructions from workspace
    Reload,
    /// Get content of a specific instruction file
    Get,
    /// Get instructions filtered by file path
    ForFile,
}

/// Input parameters for cortex_instructions tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CortexInstructionsParams {
    /// Action to perform: list, reload, get, or for_file
    pub action: InstructionsAction,

    /// Instruction name (for action=get)
    #[serde(default)]
    pub name: Option<String>,

    /// File path to filter by (for action=for_file)
    #[serde(default)]
    pub file_path: Option<String>,
}

/// Information about a single instruction file
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InstructionInfo {
    /// Name of the instruction file
    pub name: String,

    /// Description from frontmatter
    pub description: Option<String>,

    /// ApplyTo pattern from frontmatter
    pub apply_to: Option<String>,
}

/// Output from cortex_instructions tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CortexInstructionsResult {
    /// Whether the operation was successful
    pub success: bool,

    /// Action that was performed
    pub action: String,

    /// Total number of instructions loaded
    pub count: usize,

    /// List of instruction info (for list action)
    #[serde(default)]
    pub instructions: Vec<InstructionInfo>,

    /// Content of instruction (for get action)
    #[serde(default)]
    pub content: Option<String>,

    /// Message
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cortex_process_params() {
        let params = CortexProcessParams {
            query: "Test query".to_string(),
            session_id: None,
            context: None,
            auto_learn: true,
            task_type: Some(TaskType::Code),
            language: Some("rust".to_string()),
            inject_doubt: true,
            file_path: Some("src/main.rs".to_string()),
            inject_instructions: true,
        };

        assert_eq!(params.query, "Test query");
        assert!(params.auto_learn);
        assert_eq!(params.task_type, Some(TaskType::Code));
        assert_eq!(params.language, Some("rust".to_string()));
        assert_eq!(params.file_path, Some("src/main.rs".to_string()));
        assert!(params.inject_instructions);
    }

    #[test]
    fn test_task_type_prompt_key() {
        assert_eq!(TaskType::Code.prompt_key(), "code");
        assert_eq!(TaskType::Research.prompt_key(), "research:general");
        assert_eq!(TaskType::Fix.prompt_key(), "fix:error");
        assert_eq!(TaskType::Create.prompt_key(), "create:file");
    }

    #[test]
    fn test_cortex_feedback_params() {
        let params = CortexFeedbackParams {
            rule_id: "rule_123".to_string(),
            success: true,
            message: Some("Great result".to_string()),
        };

        assert_eq!(params.rule_id, "rule_123");
        assert!(params.success);
    }

    #[test]
    fn test_cortex_cleanup_params_default() {
        let json = r#"{}"#;
        let params: CortexCleanupParams = serde_json::from_str(json).unwrap();

        // Default should be 30 days
        assert_eq!(params.retention_days, 30);
    }

    #[test]
    fn test_cortex_process_params_defaults() {
        let json = r#"{"query": "test"}"#;
        let params: CortexProcessParams = serde_json::from_str(json).unwrap();

        assert_eq!(params.query, "test");
        assert!(params.auto_learn);
        assert!(params.inject_doubt);
        assert!(params.inject_instructions);
        assert!(params.task_type.is_none());
        assert!(params.language.is_none());
        assert!(params.file_path.is_none());
    }

    #[test]
    fn test_cortex_process_result_with_instructions() {
        let result = CortexProcessResult {
            success: true,
            output: "Done".to_string(),
            intent: "Code".to_string(),
            labels: vec!["rust".to_string()],
            confidence: 0.95,
            research_needed: false,
            steps_executed: 3,
            duration_ms: 150,
            recommendations: vec![],
            session_id: None,
            loaded_prompts: vec![],
            instructions_count: 5,
        };

        assert_eq!(result.instructions_count, 5);
        assert!(result.success);
    }
}
