//! Phase D - Document Pipeline
//!
//! Trace and learn after completing work.
//! Combines: memory_store + cortex_feedback + knowledge_add_observation
//!
//! Workflow from instructions:
//! 1. memory_store: log resultat + decisions
//! 2. cortex_feedback: si regle apprise
//! 3. knowledge_add_entity: si nouveau concept

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A log entry for completed work
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskLogEntry {
    /// Task description
    pub task: String,
    /// Outcome (success/partial/failed)
    pub outcome: String,
    /// What was done
    pub actions: Vec<String>,
    /// Files modified
    #[serde(default)]
    pub files_modified: Vec<String>,
    /// Duration in minutes
    #[serde(default)]
    pub duration_minutes: Option<u32>,
    /// Any notes
    #[serde(default)]
    pub notes: Option<String>,
}

/// A decision entry
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DecisionEntry {
    /// What decision was made
    pub decision: String,
    /// Why this decision was made
    pub rationale: String,
    /// Alternatives considered
    #[serde(default)]
    pub alternatives: Vec<String>,
    /// Impact of this decision
    #[serde(default)]
    pub impact: Option<String>,
    /// Related entities
    #[serde(default)]
    pub related_entities: Vec<String>,
}

/// A learned pattern
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PatternEntry {
    /// Pattern name
    pub name: String,
    /// When to use this pattern
    pub when_to_use: String,
    /// How to implement
    pub implementation: String,
    /// Example code if applicable
    #[serde(default)]
    pub example: Option<String>,
    /// Anti-patterns to avoid
    #[serde(default)]
    pub avoid: Vec<String>,
    /// Source of this pattern
    #[serde(default)]
    pub source: Option<String>,
}

/// Feedback for a learned rule
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RuleFeedback {
    /// Rule ID
    pub rule_id: String,
    /// Whether application was successful
    pub success: bool,
    /// Context of application
    #[serde(default)]
    pub context: Option<String>,
    /// Feedback message
    #[serde(default)]
    pub message: Option<String>,
}

/// Knowledge to add to entity
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeEntry {
    /// Entity name (must exist)
    pub entity_name: String,
    /// Observations to add
    pub observations: Vec<String>,
}

/// Error fix documentation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ErrorFixEntry {
    /// Error message or type
    pub error: String,
    /// Root cause
    pub cause: String,
    /// How it was fixed
    pub fix: String,
    /// How to prevent in future
    #[serde(default)]
    pub prevention: Option<String>,
    /// Related files
    #[serde(default)]
    pub files: Vec<String>,
}

/// Parameters for the document pipeline
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
pub struct DocumentParams {
    /// Task logs to record
    #[serde(default)]
    pub task_logs: Vec<TaskLogEntry>,

    /// Decisions to record
    #[serde(default)]
    pub decisions: Vec<DecisionEntry>,

    /// Patterns learned
    #[serde(default)]
    pub patterns: Vec<PatternEntry>,

    /// Rule feedback to provide
    #[serde(default)]
    pub feedbacks: Vec<RuleFeedback>,

    /// Knowledge to add
    #[serde(default)]
    pub knowledge: Vec<KnowledgeEntry>,

    /// Error fixes to document
    #[serde(default)]
    pub error_fixes: Vec<ErrorFixEntry>,

    /// Tags to apply to all entries
    #[serde(default)]
    pub global_tags: Vec<String>,

    /// Session ID for grouping
    #[serde(default)]
    pub session_id: Option<String>,
}

/// Result of documenting a single item
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DocumentedItem {
    /// Type of item documented
    pub item_type: String,
    /// Identifier (key or ID)
    pub id: String,
    /// Whether documentation succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Feedback result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FeedbackDocResult {
    /// Rule ID
    pub rule_id: String,
    /// New confidence after feedback
    pub new_confidence: f32,
}

/// Result from the document pipeline
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DocumentResult {
    /// Items documented
    pub documented: Vec<DocumentedItem>,

    /// Feedback results
    pub feedbacks: Vec<FeedbackDocResult>,

    /// Total items processed
    pub total_processed: usize,

    /// Items successfully documented
    pub total_documented: usize,

    /// Items failed
    pub total_failed: usize,

    /// Summary message
    pub summary: String,

    /// Session ID
    pub session_id: Option<String>,
}

impl DocumentParams {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_params_log_task() {
        let params =
            DocumentParams::log_task("Implement error handling", "success").with_tags(vec![
                "rust".to_string(),
                "error-handling".to_string(),
            ]);

        assert_eq!(params.task_logs.len(), 1);
        assert_eq!(params.task_logs[0].outcome, "success");
        assert_eq!(params.global_tags.len(), 2);
    }

    #[test]
    fn test_document_params_record_decision() {
        let params = DocumentParams::record_decision(
            "Use thiserror for error handling",
            "Better ergonomics than anyhow for libraries",
        );

        assert_eq!(params.decisions.len(), 1);
        assert!(params.decisions[0].decision.contains("thiserror"));
    }

    #[test]
    fn test_document_params_learn_pattern() {
        let params = DocumentParams::learn_pattern(
            "Boxing large errors",
            "When error enum > 200 bytes",
            "Box<ErrorType> instead of ErrorType",
        );

        assert_eq!(params.patterns.len(), 1);
        assert!(params.patterns[0].when_to_use.contains("200 bytes"));
    }

    #[test]
    fn test_document_params_give_feedback() {
        let params =
            DocumentParams::give_feedback("rule_boxing_errors", true).with_session("session_123");

        assert_eq!(params.feedbacks.len(), 1);
        assert!(params.feedbacks[0].success);
        assert_eq!(params.session_id, Some("session_123".to_string()));
    }

    #[test]
    fn test_document_result() {
        let result = DocumentResult {
            documented: vec![DocumentedItem {
                item_type: "task_log".to_string(),
                id: "log_123".to_string(),
                success: true,
                error: None,
            }],
            feedbacks: vec![FeedbackDocResult {
                rule_id: "rule_1".to_string(),
                new_confidence: 0.95,
            }],
            total_processed: 2,
            total_documented: 2,
            total_failed: 0,
            summary: "All items documented".to_string(),
            session_id: Some("session_1".to_string()),
        };

        assert_eq!(result.total_documented, 2);
        assert_eq!(result.feedbacks[0].new_confidence, 0.95);
    }
}
