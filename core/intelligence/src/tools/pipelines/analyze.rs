//! Phase A - Analyze Pipeline
//!
//! Research and understand before coding.
//! Combines: sequential_thinking + memory_search + knowledge_search + external_docs + external_search
//!
//! Workflow from instructions:
//! 1. sequential_thinking: "Quelle est la demande exacte ?"
//! 2. memory_search: contexte projet, decisions passees
//! 3. grep_search: existe deja dans le code ?
//! 4. context7: doc officielle lib/framework
//! 5. tavily: best practices actuelles

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Sources to search during analysis
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AnalyzeSource {
    /// Search internal memory (semantic facts)
    Memory,
    /// Search knowledge graph (entities/relations)
    Knowledge,
    /// Get official documentation (Context7)
    Docs,
    /// Search web for best practices (Tavily)
    Web,
    /// Search Microsoft Learn documentation
    Microsoft,
}

/// Parameters for the analyze pipeline
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnalyzeParams {
    /// The query or problem to analyze
    pub query: String,

    /// Sources to search (default: all)
    #[serde(default = "default_sources")]
    pub sources: Vec<AnalyzeSource>,

    /// Whether to run sequential thinking first (default: true)
    #[serde(default = "default_true")]
    pub think: bool,

    /// Estimated thinking steps if think=true (default: 5)
    #[serde(default = "default_steps")]
    pub think_steps: u32,

    /// Library name for docs search (required if sources contains Docs)
    #[serde(default)]
    pub library: Option<String>,

    /// Specific topic to focus on for docs
    #[serde(default)]
    pub topic: Option<String>,

    /// Maximum results per source (default: 5)
    #[serde(default = "default_max_per_source")]
    pub max_per_source: usize,

    /// Minimum relevance score (0.0-1.0, default: 0.3)
    #[serde(default = "default_min_score")]
    pub min_score: f32,

    /// Tags to filter memory search
    #[serde(default)]
    pub tags: Vec<String>,

    /// File path context for filtering instructions
    #[serde(default)]
    pub file_path: Option<String>,
}

fn default_sources() -> Vec<AnalyzeSource> {
    vec![
        AnalyzeSource::Memory,
        AnalyzeSource::Knowledge,
        AnalyzeSource::Docs,
        AnalyzeSource::Web,
    ]
}

fn default_true() -> bool {
    true
}

fn default_steps() -> u32 {
    5
}

fn default_max_per_source() -> usize {
    5
}

fn default_min_score() -> f32 {
    0.3
}

/// A thinking step from sequential thinking
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThinkingStep {
    /// Step number
    pub number: u32,
    /// Step content
    pub content: String,
    /// Whether this was a revision of previous thinking
    #[serde(default)]
    pub is_revision: bool,
}

/// A memory search result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MemoryResult {
    /// Unique key
    pub key: String,
    /// Content snippet
    pub content: String,
    /// Title if available
    pub title: Option<String>,
    /// Relevance score
    pub score: f32,
    /// Tags
    pub tags: Vec<String>,
}

/// A knowledge graph result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeResult {
    /// Entity name
    pub name: String,
    /// Entity type
    pub entity_type: String,
    /// Observations
    pub observations: Vec<String>,
    /// Related entities
    pub related: Vec<String>,
}

/// A documentation result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DocsResult {
    /// Library name
    pub library: String,
    /// Content snippet
    pub content: String,
    /// Code examples extracted
    pub code_snippets: Vec<String>,
    /// Source URL
    pub url: Option<String>,
    /// Provider (context7, mslearn)
    pub provider: String,
}

/// A web search result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WebResult {
    /// Result title
    pub title: String,
    /// Content snippet
    pub content: String,
    /// Source URL
    pub url: Option<String>,
    /// Relevance score
    pub score: f32,
}

/// Result from the analyze pipeline
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AnalyzeResult {
    /// Original query
    pub query: String,

    /// Sequential thinking steps (if think=true)
    #[serde(default)]
    pub thinking: Vec<ThinkingStep>,

    /// Conclusion from thinking (if think=true)
    pub thinking_conclusion: Option<String>,

    /// Memory search results
    #[serde(default)]
    pub memory_results: Vec<MemoryResult>,

    /// Knowledge graph results
    #[serde(default)]
    pub knowledge_results: Vec<KnowledgeResult>,

    /// Documentation results
    #[serde(default)]
    pub docs_results: Vec<DocsResult>,

    /// Web search results
    #[serde(default)]
    pub web_results: Vec<WebResult>,

    /// Aggregated context summary
    pub summary: String,

    /// Sources that were searched
    pub sources_searched: Vec<String>,

    /// Overall confidence in the analysis (0.0-1.0)
    pub confidence: f32,

    /// Recommended action based on analysis
    pub recommendation: String,

    /// Whether more research is needed
    pub needs_more_research: bool,

    /// Suggested next query if needs_more_research is true
    pub suggested_query: Option<String>,
}

impl Default for AnalyzeParams {
    fn default() -> Self {
        Self {
            query: String::new(),
            sources: default_sources(),
            think: true,
            think_steps: 5,
            library: None,
            topic: None,
            max_per_source: 5,
            min_score: 0.3,
            tags: Vec::new(),
            file_path: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_params_defaults() {
        let json = r#"{"query": "test"}"#;
        let params: AnalyzeParams = serde_json::from_str(json).unwrap();

        assert_eq!(params.query, "test");
        assert!(params.think);
        assert_eq!(params.think_steps, 5);
        assert_eq!(params.sources.len(), 4);
        assert_eq!(params.max_per_source, 5);
    }

    #[test]
    fn test_analyze_params_custom_sources() {
        let json = r#"{"query": "test", "sources": ["memory", "docs"]}"#;
        let params: AnalyzeParams = serde_json::from_str(json).unwrap();

        assert_eq!(params.sources.len(), 2);
        assert!(params.sources.contains(&AnalyzeSource::Memory));
        assert!(params.sources.contains(&AnalyzeSource::Docs));
    }

    #[test]
    fn test_analyze_result_serialization() {
        let result = AnalyzeResult {
            query: "test".to_string(),
            thinking: vec![],
            thinking_conclusion: Some("conclusion".to_string()),
            memory_results: vec![],
            knowledge_results: vec![],
            docs_results: vec![],
            web_results: vec![],
            summary: "summary".to_string(),
            sources_searched: vec!["memory".to_string()],
            confidence: 0.8,
            recommendation: "proceed".to_string(),
            needs_more_research: false,
            suggested_query: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("confidence"));
        assert!(json.contains("0.8"));
    }
}
