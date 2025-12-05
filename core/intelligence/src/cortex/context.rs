//! Context Manager - Active Session and State
//!
//! Manages the active context across interactions:
//! - Current workspace
//! - Active files
//! - Session state
//! - Recent history
//! - Aggregated memory context (semantic, episodic, procedural, graph)
//!
//! Note: Some types (AggregatedContext, GatherConfig, ContextItem) are reserved
//! for future multi-agent integration and may not be used currently.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::PathBuf;

/// Maximum history items to keep
const MAX_HISTORY: usize = 50;

/// Default semantic search threshold
#[allow(dead_code)]
const DEFAULT_THRESHOLD: f32 = 0.5;

/// Default top-k results per source
#[allow(dead_code)]
const DEFAULT_TOP_K: usize = 5;

// ============================================================================
// Aggregated Context - Gathered from all memory systems
// ============================================================================

/// Source type for context items
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextSource {
    /// Semantic memory (embeddings/RAG)
    Semantic,
    /// Episodic memory (events/interactions)
    Episodic,
    /// Procedural memory (rules/patterns)
    Procedural,
    /// Knowledge graph (entities/relations)
    Graph,
    /// User-provided context
    User,
    /// System context
    System,
}

/// A single item from any context source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextItem {
    /// Source of this context item
    pub source: ContextSource,
    /// Content text or summary
    pub content: String,
    /// Relevance score (0.0 - 1.0)
    pub relevance: f32,
    /// Additional metadata
    pub metadata: serde_json::Value,
    /// Timestamp when gathered
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[allow(dead_code)]
impl ContextItem {
    /// Create a new context item
    pub fn new(source: ContextSource, content: String, relevance: f32) -> Self {
        Self {
            source,
            content,
            relevance,
            metadata: serde_json::Value::Null,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Add metadata to the context item
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Relevance scores from each memory source
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelevanceScores {
    /// Semantic memory relevance (0.0 - 1.0)
    pub semantic: f32,
    /// Episodic memory relevance (0.0 - 1.0)
    pub episodic: f32,
    /// Procedural memory relevance (0.0 - 1.0)
    pub procedural: f32,
    /// Knowledge graph relevance (0.0 - 1.0)
    pub graph: f32,
}

impl RelevanceScores {
    /// Calculate overall relevance (average of non-zero scores)
    pub fn overall(&self) -> f32 {
        let scores = [self.semantic, self.episodic, self.procedural, self.graph];
        let non_zero: Vec<f32> = scores.into_iter().filter(|&s| s > 0.0).collect();
        if non_zero.is_empty() {
            0.0
        } else {
            non_zero.iter().sum::<f32>() / non_zero.len() as f32
        }
    }
}

/// Aggregated context gathered from all memory systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedContext {
    /// Original query that triggered context gathering
    pub query: String,

    // === Memory items by source ===
    /// Items from semantic memory (documents, knowledge)
    pub semantic_items: Vec<ContextItem>,
    /// Items from episodic memory (past interactions)
    pub episodic_items: Vec<ContextItem>,
    /// Matched rules from procedural memory
    pub procedural_rules: Vec<ContextItem>,
    /// Matched patterns from procedural memory
    pub procedural_patterns: Vec<ContextItem>,
    /// Entities from knowledge graph
    pub graph_entities: Vec<ContextItem>,
    /// Relations from knowledge graph
    pub graph_relations: Vec<serde_json::Value>,

    // === User context ===
    /// User-provided context
    pub user_context: serde_json::Value,

    // === Scores ===
    /// Relevance scores by source
    pub scores: RelevanceScores,

    // === Metadata ===
    /// Session ID if available
    pub session_id: Option<String>,
    /// Timestamp when context was gathered
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for AggregatedContext {
    fn default() -> Self {
        Self {
            query: String::new(),
            semantic_items: Vec::new(),
            episodic_items: Vec::new(),
            procedural_rules: Vec::new(),
            procedural_patterns: Vec::new(),
            graph_entities: Vec::new(),
            graph_relations: Vec::new(),
            user_context: serde_json::Value::Null,
            scores: RelevanceScores::default(),
            session_id: None,
            timestamp: chrono::Utc::now(),
        }
    }
}

#[allow(dead_code)]
impl AggregatedContext {
    /// Create a new aggregated context for a query
    pub fn new(query: impl Into<String>) -> Self {
        Self {
            query: query.into(),
            ..Default::default()
        }
    }

    /// Set the session ID
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Set user context
    pub fn with_user_context(mut self, user_context: serde_json::Value) -> Self {
        self.user_context = user_context;
        self
    }

    /// Total number of context items
    pub fn total_items(&self) -> usize {
        self.semantic_items.len()
            + self.episodic_items.len()
            + self.procedural_rules.len()
            + self.procedural_patterns.len()
            + self.graph_entities.len()
    }

    /// Overall relevance score
    pub fn overall_relevance(&self) -> f32 {
        self.scores.overall()
    }

    /// Check if context is empty
    pub fn is_empty(&self) -> bool {
        self.total_items() == 0
    }

    /// Get a text summary of the context
    pub fn summary(&self) -> String {
        let mut parts = Vec::new();

        if !self.semantic_items.is_empty() {
            parts.push(format!("Semantic: {} items", self.semantic_items.len()));
        }
        if !self.episodic_items.is_empty() {
            parts.push(format!("Episodic: {} items", self.episodic_items.len()));
        }
        if !self.procedural_rules.is_empty() {
            parts.push(format!("Rules: {}", self.procedural_rules.len()));
        }
        if !self.procedural_patterns.is_empty() {
            parts.push(format!("Patterns: {}", self.procedural_patterns.len()));
        }
        if !self.graph_entities.is_empty() {
            parts.push(format!("Entities: {}", self.graph_entities.len()));
        }

        if parts.is_empty() {
            "Empty context".to_string()
        } else {
            parts.join(" | ")
        }
    }

    /// Generate a detailed text summary for LLM consumption
    pub fn to_prompt_text(&self) -> String {
        let mut parts = Vec::new();

        // Semantic knowledge
        if !self.semantic_items.is_empty() {
            parts.push("**Relevant Knowledge:**".to_string());
            for item in self.semantic_items.iter().take(3) {
                let content = if item.content.len() > 200 {
                    format!("{}...", &item.content[..200])
                } else {
                    item.content.clone()
                };
                parts.push(format!("- {}", content));
            }
        }

        // Episodic history
        if !self.episodic_items.is_empty() {
            parts.push("\n**Recent History:**".to_string());
            for item in self.episodic_items.iter().take(3) {
                let content = if item.content.len() > 100 {
                    format!("{}...", &item.content[..100])
                } else {
                    item.content.clone()
                };
                parts.push(format!("- {}", content));
            }
        }

        // Applicable rules
        if !self.procedural_rules.is_empty() {
            parts.push("\n**Applicable Rules:**".to_string());
            for item in self.procedural_rules.iter().take(2) {
                let content = if item.content.len() > 100 {
                    format!("{}...", &item.content[..100])
                } else {
                    item.content.clone()
                };
                parts.push(format!("- {}", content));
            }
        }

        // Graph relations
        if !self.graph_relations.is_empty() {
            parts.push("\n**Related Concepts:**".to_string());
            for rel in self.graph_relations.iter().take(3) {
                if let (Some(source), Some(relation), Some(target)) = (
                    rel.get("source").and_then(|v| v.as_str()),
                    rel.get("relation").and_then(|v| v.as_str()),
                    rel.get("target").and_then(|v| v.as_str()),
                ) {
                    parts.push(format!("- {} {} {}", source, relation, target));
                }
            }
        }

        if parts.is_empty() {
            "No relevant context found.".to_string()
        } else {
            parts.join("\n")
        }
    }

    /// Add semantic items and update relevance score
    pub fn add_semantic_items(&mut self, items: Vec<ContextItem>) {
        if !items.is_empty() {
            self.scores.semantic =
                items.iter().map(|i| i.relevance).sum::<f32>() / items.len() as f32;
            self.semantic_items.extend(items);
        }
    }

    /// Add episodic items and update relevance score
    pub fn add_episodic_items(&mut self, items: Vec<ContextItem>) {
        if !items.is_empty() {
            self.scores.episodic =
                items.iter().map(|i| i.relevance).sum::<f32>() / items.len() as f32;
            self.episodic_items.extend(items);
        }
    }

    /// Add procedural rules and update relevance score
    pub fn add_procedural_rules(&mut self, items: Vec<ContextItem>) {
        if !items.is_empty() {
            self.scores.procedural =
                items.iter().map(|i| i.relevance).sum::<f32>() / items.len() as f32;
            self.procedural_rules.extend(items);
        }
    }

    /// Add graph entities and relations
    pub fn add_graph_data(
        &mut self,
        entities: Vec<ContextItem>,
        relations: Vec<serde_json::Value>,
    ) {
        if !entities.is_empty() {
            self.scores.graph = 0.7; // Base relevance for graph matches
            self.graph_entities.extend(entities);
        }
        self.graph_relations.extend(relations);
    }
}

// ============================================================================
// Context Gathering Configuration
// Reserved for multi-agent system integration
// ============================================================================

/// Configuration for context gathering
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatherConfig {
    /// Maximum items per source
    pub top_k: usize,
    /// Minimum relevance threshold
    pub threshold: f32,
    /// Whether to include semantic memory
    pub include_semantic: bool,
    /// Whether to include episodic memory
    pub include_episodic: bool,
    /// Whether to include procedural memory
    pub include_procedural: bool,
    /// Whether to include knowledge graph
    pub include_graph: bool,
}

impl Default for GatherConfig {
    fn default() -> Self {
        Self {
            top_k: DEFAULT_TOP_K,
            threshold: DEFAULT_THRESHOLD,
            include_semantic: true,
            include_episodic: true,
            include_procedural: true,
            include_graph: true,
        }
    }
}

// ============================================================================
// Original ActiveContext and ContextManager (preserved)
// ============================================================================

/// Active context for CORTEX operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveContext {
    /// Current workspace path
    pub workspace: Option<PathBuf>,

    /// Active files being worked on
    pub active_files: Vec<PathBuf>,

    /// Current session ID
    pub session_id: Option<String>,

    /// Recent query history
    pub history: Vec<HistoryItem>,

    /// Custom metadata
    pub metadata: serde_json::Value,

    /// Context created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Last updated timestamp
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl Default for ActiveContext {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            workspace: None,
            active_files: Vec::new(),
            session_id: None,
            history: Vec::new(),
            metadata: serde_json::Value::Null,
            created_at: now,
            updated_at: now,
        }
    }
}

impl ActiveContext {
    /// Create a new active context
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the workspace
    pub fn with_workspace(mut self, workspace: PathBuf) -> Self {
        self.workspace = Some(workspace);
        self.updated_at = chrono::Utc::now();
        self
    }

    /// Add an active file
    pub fn with_file(mut self, file: PathBuf) -> Self {
        if !self.active_files.contains(&file) {
            self.active_files.push(file);
            self.updated_at = chrono::Utc::now();
        }
        self
    }

    /// Set the session ID
    pub fn with_session(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self.updated_at = chrono::Utc::now();
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self.updated_at = chrono::Utc::now();
        self
    }
}

/// A history item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem {
    /// Query
    pub query: String,

    /// Intent detected
    pub intent: String,

    /// Whether successful
    pub success: bool,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Context Manager for managing active context
pub struct ContextManager {
    /// Current active context
    context: ActiveContext,

    /// History queue
    history: VecDeque<HistoryItem>,

    /// Maximum history size
    max_history: usize,
}

impl Default for ContextManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextManager {
    /// Create a new context manager
    pub fn new() -> Self {
        Self {
            context: ActiveContext::new(),
            history: VecDeque::with_capacity(MAX_HISTORY),
            max_history: MAX_HISTORY,
        }
    }

    /// Get the current context
    pub fn get_context(&self) -> &ActiveContext {
        &self.context
    }

    /// Get mutable context
    #[allow(dead_code)]
    pub fn get_context_mut(&mut self) -> &mut ActiveContext {
        &mut self.context
    }

    /// Set the workspace
    pub fn set_workspace(&mut self, workspace: PathBuf) {
        self.context.workspace = Some(workspace);
        self.context.updated_at = chrono::Utc::now();
    }

    /// Add an active file
    #[allow(dead_code)]
    pub fn add_active_file(&mut self, file: PathBuf) {
        if !self.context.active_files.contains(&file) {
            self.context.active_files.push(file);
            self.context.updated_at = chrono::Utc::now();
        }
    }

    /// Remove an active file
    #[allow(dead_code)]
    pub fn remove_active_file(&mut self, file: &PathBuf) {
        self.context.active_files.retain(|f| f != file);
        self.context.updated_at = chrono::Utc::now();
    }

    /// Clear active files
    #[allow(dead_code)]
    pub fn clear_active_files(&mut self) {
        self.context.active_files.clear();
        self.context.updated_at = chrono::Utc::now();
    }

    /// Start a new session
    pub fn start_session(&mut self) -> String {
        let session_id = uuid::Uuid::new_v4().to_string();
        self.context.session_id = Some(session_id.clone());
        self.context.created_at = chrono::Utc::now();
        self.context.updated_at = chrono::Utc::now();
        self.history.clear();
        session_id
    }

    /// End the current session
    pub fn end_session(&mut self) {
        self.context.session_id = None;
        self.context.updated_at = chrono::Utc::now();
    }

    /// Add a history item
    pub fn add_history(&mut self, item: HistoryItem) {
        if self.history.len() >= self.max_history {
            self.history.pop_front();
        }
        self.history.push_back(item);

        // Update context history
        self.context.history = self.history.iter().cloned().collect();
        self.context.updated_at = chrono::Utc::now();
    }

    /// Record a query
    pub fn record_query(&mut self, query: &str, intent: &str, success: bool) {
        self.add_history(HistoryItem {
            query: query.to_string(),
            intent: intent.to_string(),
            success,
            timestamp: chrono::Utc::now(),
        });
    }

    /// Get recent history
    #[allow(dead_code)]
    pub fn get_recent_history(&self, count: usize) -> Vec<&HistoryItem> {
        self.history.iter().rev().take(count).collect()
    }

    /// Get successful queries from history
    #[allow(dead_code)]
    pub fn get_successful_queries(&self, count: usize) -> Vec<&HistoryItem> {
        self.history
            .iter()
            .rev()
            .filter(|h| h.success)
            .take(count)
            .collect()
    }

    /// Set metadata
    #[allow(dead_code)]
    pub fn set_metadata(&mut self, metadata: serde_json::Value) {
        self.context.metadata = metadata;
        self.context.updated_at = chrono::Utc::now();
    }

    /// Get workspace
    pub fn get_workspace(&self) -> Option<&PathBuf> {
        self.context.workspace.as_ref()
    }

    /// Export context as JSON
    #[allow(dead_code)]
    pub fn export(&self) -> serde_json::Value {
        serde_json::json!({
            "workspace": self.context.workspace,
            "active_files": self.context.active_files,
            "session_id": self.context.session_id,
            "history_count": self.history.len(),
            "created_at": self.context.created_at,
            "updated_at": self.context.updated_at,
        })
    }

    /// Import context from JSON
    #[allow(dead_code)]
    pub fn import(&mut self, data: serde_json::Value) -> Result<(), serde_json::Error> {
        if let Some(workspace) = data.get("workspace").and_then(|v| v.as_str()) {
            self.context.workspace = Some(PathBuf::from(workspace));
        }

        if let Some(files) = data.get("active_files").and_then(|v| v.as_array()) {
            self.context.active_files = files
                .iter()
                .filter_map(|v| v.as_str())
                .map(PathBuf::from)
                .collect();
        }

        self.context.updated_at = chrono::Utc::now();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_context() {
        let ctx = ActiveContext::new()
            .with_workspace(PathBuf::from("/test"))
            .with_file(PathBuf::from("/test/file.rs"));

        assert_eq!(ctx.workspace, Some(PathBuf::from("/test")));
        assert_eq!(ctx.active_files.len(), 1);
    }

    #[test]
    fn test_context_manager() {
        let mut mgr = ContextManager::new();

        // Start session
        let session = mgr.start_session();
        assert!(mgr.get_context().session_id.is_some());
        assert_eq!(mgr.get_context().session_id.as_ref(), Some(&session));

        // Add history
        mgr.record_query("test query", "create", true);
        assert_eq!(mgr.history.len(), 1);

        // Check recent
        let recent = mgr.get_recent_history(10);
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].query, "test query");
    }

    #[test]
    fn test_history_limit() {
        let mut mgr = ContextManager::new();
        mgr.max_history = 5;

        for i in 0..10 {
            mgr.record_query(&format!("query {}", i), "test", true);
        }

        assert_eq!(mgr.history.len(), 5);
        assert_eq!(mgr.history.front().unwrap().query, "query 5");
    }
}
