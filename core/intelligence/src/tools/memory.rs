//! Memory tools for MCP
//!
//! Tools for storing, searching, and managing memories.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for memory_store tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryStoreParams {
    /// Unique key for this memory (auto-generated if not provided)
    #[serde(default)]
    pub key: Option<String>,

    /// The content to store
    pub content: String,

    /// Optional title/summary
    #[serde(default)]
    pub title: Option<String>,

    /// Optional tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Optional metadata as JSON
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,

    /// Whether to index for semantic search (default: true)
    #[serde(default = "default_true")]
    pub index: bool,
}

/// Result from memory_store
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryStoreResult {
    /// The key assigned to this memory
    pub key: String,

    /// Whether the memory was indexed for semantic search
    pub indexed: bool,

    /// Timestamp when stored
    pub stored_at: i64,
}

/// Parameters for memory_search tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemorySearchParams {
    /// Search query (semantic search)
    pub query: String,

    /// Maximum number of results (default: 10)
    #[serde(default = "default_limit")]
    pub limit: usize,

    /// Minimum similarity score (0.0 - 1.0)
    #[serde(default)]
    pub min_score: Option<f32>,

    /// Filter by tags (AND logic)
    #[serde(default)]
    pub tags: Vec<String>,
}

/// A single search result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemorySearchResultItem {
    /// Memory key
    pub key: String,

    /// Memory content
    pub content: String,

    /// Optional title
    pub title: Option<String>,

    /// Similarity score (0.0 - 1.0)
    pub score: f32,

    /// Tags
    pub tags: Vec<String>,

    /// When stored
    pub stored_at: i64,
}

/// Result from memory_search
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemorySearchResult {
    /// Search results
    pub results: Vec<MemorySearchResultItem>,

    /// Total results found
    pub total: usize,

    /// Query used
    pub query: String,
}

/// Parameters for memory_get tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryGetParams {
    /// Key of the memory to retrieve
    pub key: String,
}

/// Result from memory_get
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryGetResult {
    /// Memory key
    pub key: String,

    /// Memory content
    pub content: String,

    /// Optional title
    pub title: Option<String>,

    /// Tags
    pub tags: Vec<String>,

    /// Metadata
    pub metadata: Option<serde_json::Value>,

    /// When stored
    pub stored_at: i64,

    /// When last updated
    pub updated_at: i64,
}

/// Parameters for memory_delete tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryDeleteParams {
    /// Key of the memory to delete
    pub key: String,
}

/// Result from memory_delete
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryDeleteResult {
    /// Key that was deleted
    pub key: String,

    /// Whether deletion was successful
    pub deleted: bool,
}

/// Parameters for memory_list tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryListParams {
    /// Maximum number of results
    #[serde(default = "default_limit")]
    pub limit: usize,

    /// Offset for pagination
    #[serde(default)]
    pub offset: usize,

    /// Filter by tags
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Result from memory_list
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryListResult {
    /// List of memory summaries
    pub memories: Vec<MemorySummary>,

    /// Total count
    pub total: usize,

    /// Whether there are more results
    pub has_more: bool,
}

/// Summary of a memory (for listing)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemorySummary {
    /// Memory key
    pub key: String,

    /// Title or content preview
    pub title: String,

    /// Tags
    pub tags: Vec<String>,

    /// When stored
    pub stored_at: i64,
}

// ============================================================================
// BATCH STORE (from Python v2.0)
// ============================================================================

/// A single item for batch storage
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BatchStoreItem {
    /// The content to store
    pub content: String,

    /// Source/origin of content
    pub source: String,

    /// Optional category
    #[serde(default = "default_category")]
    pub category: String,

    /// Optional tags
    #[serde(default)]
    pub tags: Vec<String>,

    /// Optional metadata
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

/// Parameters for batch_store tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BatchStoreParams {
    /// Items to store in batch
    pub items: Vec<BatchStoreItem>,
}

/// Result from batch_store
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BatchStoreResult {
    /// Number of items successfully stored
    pub stored: usize,

    /// Keys assigned to stored items
    pub keys: Vec<String>,

    /// Any errors that occurred
    #[serde(default)]
    pub errors: Vec<String>,
}

// ============================================================================
// HYBRID SEARCH (from Python v2.0)
// ============================================================================

/// Parameters for hybrid_search tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HybridSearchParams {
    /// Search query
    pub query: String,

    /// Maximum results per source (default: 10)
    #[serde(default = "default_limit")]
    pub top_k: usize,

    /// Minimum relevance threshold (default: 0.3)
    #[serde(default = "default_min_relevance")]
    pub min_relevance: f32,
}

/// A semantic search result item
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SemanticItem {
    /// Document ID
    pub id: String,

    /// Content snippet
    pub content: String,

    /// Source of the content
    pub source: String,

    /// Relevance score (0.0 - 1.0)
    pub score: f32,

    /// Category
    pub category: String,

    /// Tags
    pub tags: Vec<String>,
}

/// An episodic memory item
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EpisodicItem {
    /// Episode ID
    pub id: String,

    /// Content
    pub content: String,

    /// Episode type
    pub episode_type: String,

    /// Session ID if available
    pub session_id: Option<String>,

    /// Timestamp
    pub timestamp: i64,
}

/// A procedural rule item
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProceduralItem {
    /// Rule ID
    pub id: String,

    /// Rule name
    pub name: String,

    /// Rule description
    pub description: String,

    /// Confidence level
    pub confidence: f32,
}

/// Result from hybrid_search
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct HybridSearchResult {
    /// Semantic memory results
    pub semantic: Vec<SemanticItem>,

    /// Episodic memory results
    pub episodic: Vec<EpisodicItem>,

    /// Procedural memory results
    pub procedural: Vec<ProceduralItem>,

    /// Knowledge graph results
    pub graph: Vec<serde_json::Value>,

    /// Summary of results
    pub summary: String,
}

// ============================================================================
// TAG MANAGEMENT (from Python v2.0)
// ============================================================================

/// Parameters for manage_tags tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ManageTagsParams {
    /// Action to perform: "add", "remove", "get", or "search"
    pub action: String,

    /// Document ID (for add/remove/get)
    #[serde(default)]
    pub doc_id: Option<String>,

    /// Tags to add/remove or search for
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Result from manage_tags
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ManageTagsResult {
    /// Whether the operation succeeded
    pub success: bool,

    /// Document ID if applicable
    #[serde(default)]
    pub doc_id: Option<String>,

    /// Tags involved in the operation
    pub tags: Vec<String>,

    /// Search results if action was "search"
    #[serde(default)]
    pub results: Vec<MemorySummary>,

    /// Message describing the result
    pub message: String,
}

// ============================================================================
// GET CONTEXT (from Python v2.0)
// ============================================================================

/// Parameters for get_context tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetContextParams {
    /// Query to get context for
    pub query: String,

    /// Type of context gathering: "query", "search", or "session"
    #[serde(default = "default_context_type")]
    pub context_type: String,
}

/// Aggregated context result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct GetContextResult {
    /// Original query
    pub query: String,

    /// Semantic items found
    pub semantic_items: Vec<SemanticItem>,

    /// Episodic items found
    pub episodic_items: Vec<EpisodicItem>,

    /// Applicable rules
    pub procedural_rules: Vec<ProceduralItem>,

    /// Graph entities found
    pub graph_entities: Vec<serde_json::Value>,

    /// Overall relevance scores
    pub scores: ContextScores,

    /// Summary text
    pub summary: String,
}

/// Context relevance scores
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContextScores {
    /// Semantic relevance (0.0 - 1.0)
    pub semantic: f32,

    /// Episodic relevance (0.0 - 1.0)
    pub episodic: f32,

    /// Procedural relevance (0.0 - 1.0)
    pub procedural: f32,

    /// Graph relevance (0.0 - 1.0)
    pub graph: f32,

    /// Overall relevance (0.0 - 1.0)
    pub overall: f32,
}

// Default helpers
fn default_true() -> bool {
    true
}

fn default_category() -> String {
    "general".to_string()
}

fn default_min_relevance() -> f32 {
    0.3
}

fn default_context_type() -> String {
    "query".to_string()
}

fn default_limit() -> usize {
    10
}

#[cfg(test)]
mod tests {
    use super::*;

    impl MemoryStoreParams {
        fn new(content: impl Into<String>) -> Self {
            Self {
                key: None,
                content: content.into(),
                title: None,
                tags: Vec::new(),
                metadata: None,
                index: true,
            }
        }

        fn with_key(mut self, key: impl Into<String>) -> Self {
            self.key = Some(key.into());
            self
        }

        fn with_title(mut self, title: impl Into<String>) -> Self {
            self.title = Some(title.into());
            self
        }

        fn with_tag(mut self, tag: impl Into<String>) -> Self {
            self.tags.push(tag.into());
            self
        }
    }

    impl MemorySearchParams {
        fn new(query: impl Into<String>) -> Self {
            Self {
                query: query.into(),
                limit: super::default_limit(),
                min_score: None,
                tags: Vec::new(),
            }
        }

        fn with_limit(mut self, limit: usize) -> Self {
            self.limit = limit;
            self
        }

        fn with_min_score(mut self, score: f32) -> Self {
            self.min_score = Some(score);
            self
        }
    }

    #[test]
    fn test_memory_store_params_builder() {
        let params = MemoryStoreParams::new("Test content")
            .with_key("test-key")
            .with_title("Test Title")
            .with_tag("important")
            .with_tag("work");

        assert_eq!(params.key, Some("test-key".to_string()));
        assert_eq!(params.content, "Test content");
        assert_eq!(params.title, Some("Test Title".to_string()));
        assert_eq!(params.tags, vec!["important", "work"]);
        assert!(params.index);
    }

    #[test]
    fn test_memory_search_params_builder() {
        let params = MemorySearchParams::new("find something")
            .with_limit(5)
            .with_min_score(0.7);

        assert_eq!(params.query, "find something");
        assert_eq!(params.limit, 5);
        assert_eq!(params.min_score, Some(0.7));
    }

    #[test]
    fn test_serialization() {
        let params = MemoryStoreParams::new("content");
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("content"));

        let parsed: MemoryStoreParams = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.content, "content");
    }
}
