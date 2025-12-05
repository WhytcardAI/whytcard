//! Perceiver Module - Analyze and Understand Input
//!
//! The Perceiver analyzes user queries to extract:
//! - Intent (what the user wants to do)
//! - Secondary intents (additional detected intents)
//! - Context (workspace, files, history)
//! - Task labels (multi-label classification)
//! - Complexity assessment (simple to very_complex)
//! - Topics and keywords
//! - Recommended actions
//! - Confidence score (triggers research if low)

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

use super::context::AggregatedContext;

// ============================================================================
// Complexity Assessment
// ============================================================================

/// Query complexity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Complexity {
    /// Simple, single-step query
    #[default]
    Simple,
    /// Moderate complexity, may need some context
    Moderate,
    /// Complex, multi-step or conditional
    Complex,
    /// Very complex, requires decomposition
    VeryComplex,
}

impl Complexity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Simple => "simple",
            Self::Moderate => "moderate",
            Self::Complex => "complex",
            Self::VeryComplex => "very_complex",
        }
    }
}

// ============================================================================
// Recommended Actions
// ============================================================================

/// Actions that can be recommended based on perception
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendedAction {
    /// Search semantic memory
    SearchSemantic,
    /// Search all memories
    SearchAllMemories,
    /// Gather context from all sources
    GatherContext,
    /// Decompose query into sub-tasks
    DecomposeQuery,
    /// Provide detailed explanation
    ProvideExplanation,
    /// Generate content
    GenerateContent,
    /// Check existing resources first
    CheckExisting,
    /// Apply changes to target
    ApplyChanges,
    /// Execute deletion with confirmation
    ExecuteDeletion,
    /// Perform analysis
    PerformAnalysis,
    /// Store to memory
    StoreToMemory,
    /// Extract and learn from input
    ExtractKnowledge,
    /// Update procedural memory
    UpdateProcedural,
    /// Search episodic memory
    SearchEpisodic,
    /// Process feedback for learning
    ProcessFeedback,
    /// Gather statistics
    GatherStats,
    /// Provide user guidance
    ProvideGuidance,
    /// Structure comparison response
    StructureComparison,
    /// Rank and filter results
    RankResults,
    /// Validate content before storage
    ValidateContent,
    /// Use external documentation source
    UseExternalDocs,
    /// Use web search
    UseWebSearch,
    /// Custom action
    Custom(String),
}

impl RecommendedAction {
    pub fn as_str(&self) -> &str {
        match self {
            Self::SearchSemantic => "search_semantic",
            Self::SearchAllMemories => "search_all_memories",
            Self::GatherContext => "gather_context",
            Self::DecomposeQuery => "decompose_query",
            Self::ProvideExplanation => "provide_explanation",
            Self::GenerateContent => "generate_content",
            Self::CheckExisting => "check_existing",
            Self::ApplyChanges => "apply_changes",
            Self::ExecuteDeletion => "execute_deletion",
            Self::PerformAnalysis => "perform_analysis",
            Self::StoreToMemory => "store_to_memory",
            Self::ExtractKnowledge => "extract_knowledge",
            Self::UpdateProcedural => "update_procedural",
            Self::SearchEpisodic => "search_episodic",
            Self::ProcessFeedback => "process_feedback",
            Self::GatherStats => "gather_stats",
            Self::ProvideGuidance => "provide_guidance",
            Self::StructureComparison => "structure_comparison",
            Self::RankResults => "rank_results",
            Self::ValidateContent => "validate_content",
            Self::UseExternalDocs => "use_external_docs",
            Self::UseWebSearch => "use_web_search",
            Self::Custom(s) => s,
        }
    }
}

// ============================================================================
// External Source Recommendations
// ============================================================================

/// External sources that may be useful for the query
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalSourceType {
    /// Context7 library documentation
    Context7,
    /// Tavily web search
    Tavily,
    /// Microsoft Learn documentation
    MicrosoftLearn,
    /// Sequential thinking for complex problems
    SequentialThinking,
    /// Custom external source
    Custom(String),
}

/// Recommendation to use an external source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalSourceRecommendation {
    /// Type of external source
    pub source_type: ExternalSourceType,
    /// Reason for recommendation
    pub reason: String,
    /// Suggested query or topic
    pub suggested_query: Option<String>,
    /// Priority (higher = more important)
    pub priority: u8,
}

// ============================================================================
// Enhanced Perception Result
// ============================================================================

/// The result of perception analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerceptionResult {
    /// The detected primary intent
    pub intent: Intent,

    /// Secondary detected intents (if multiple apply)
    pub secondary_intents: Vec<Intent>,

    /// Multi-label task classification
    pub labels: Vec<TaskLabel>,

    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,

    /// Extracted entities from the query
    pub entities: HashMap<String, String>,

    /// Extracted topics from the query
    pub topics: Vec<String>,

    /// Extracted keywords (significant terms)
    pub keywords: Vec<String>,

    /// Query complexity assessment
    pub complexity: Complexity,

    /// Factors that contributed to complexity
    pub complexity_factors: Vec<String>,

    /// Recommended actions based on perception
    pub recommended_actions: Vec<RecommendedAction>,

    /// External sources recommended for this query
    pub external_sources: Vec<ExternalSourceRecommendation>,

    /// Whether research/search is recommended
    pub needs_research: bool,

    /// Whether this should trigger learning
    pub needs_learning: bool,

    /// Context summary if context was provided
    pub context_summary: String,

    /// Rule names from context that are relevant
    pub relevant_rules: Vec<String>,

    /// Original query
    pub query: String,

    /// Processing time in milliseconds
    pub processing_time_ms: f64,

    /// Timestamp of perception
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl Default for PerceptionResult {
    fn default() -> Self {
        Self {
            intent: Intent::Unknown,
            secondary_intents: Vec::new(),
            labels: Vec::new(),
            confidence: 0.0,
            entities: HashMap::new(),
            topics: Vec::new(),
            keywords: Vec::new(),
            complexity: Complexity::Simple,
            complexity_factors: Vec::new(),
            recommended_actions: Vec::new(),
            external_sources: Vec::new(),
            needs_research: false,
            needs_learning: false,
            context_summary: String::new(),
            relevant_rules: Vec::new(),
            query: String::new(),
            processing_time_ms: 0.0,
            timestamp: chrono::Utc::now(),
        }
    }
}

/// User intent classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Intent {
    // === Questions ===
    /// General question
    Question,
    /// Create something new
    Create,
    /// Modify existing
    Modify,
    /// Delete/Remove
    Delete,
    /// Search/Find
    Search,
    /// Explain/Describe
    Explain,
    /// Compare items
    Compare,
    /// Debug/Fix
    Debug,
    /// Analyze/Review
    Analyze,
    /// Execute/Run
    Execute,
    /// Configure/Setup
    Configure,

    // === Learning ===
    /// Store information
    Store,
    /// Learn/Memorize
    Learn,
    /// Recall/Remember
    Remember,

    // === Meta ===
    /// Provide feedback
    Feedback,
    /// Check status
    Status,
    /// Ask for help
    Help,

    /// Unknown intent
    Unknown,
}

impl Intent {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Question => "question",
            Self::Create => "create",
            Self::Modify => "modify",
            Self::Delete => "delete",
            Self::Search => "search",
            Self::Explain => "explain",
            Self::Compare => "compare",
            Self::Debug => "debug",
            Self::Analyze => "analyze",
            Self::Execute => "execute",
            Self::Configure => "configure",
            Self::Store => "store",
            Self::Learn => "learn",
            Self::Remember => "remember",
            Self::Feedback => "feedback",
            Self::Status => "status",
            Self::Help => "help",
            Self::Unknown => "unknown",
        }
    }

    /// Detect intent from query keywords with scoring
    fn detect_all(query: &str) -> Vec<(Intent, f32)> {
        let query_lower = query.to_lowercase();
        let mut scores: Vec<(Intent, f32)> = Vec::new();

        // Intent patterns with weights
        let patterns: &[(Intent, &[&str])] = &[
            // Questions
            (
                Intent::Question,
                &["what", "who", "where", "when", "why", "how", "which", "?"],
            ),
            (
                Intent::Explain,
                &[
                    "explain", "what is", "what are", "describe", "how does", "how do", "define",
                ],
            ),
            (
                Intent::Compare,
                &[
                    "compare",
                    "versus",
                    "vs",
                    "difference between",
                    "better",
                    "worse",
                    "which is",
                ],
            ),
            // Actions
            (
                Intent::Create,
                &[
                    "create",
                    "generate",
                    "make",
                    "build",
                    "implement",
                    "add",
                    "write",
                    "new",
                ],
            ),
            (
                Intent::Modify,
                &[
                    "modify", "change", "update", "edit", "refactor", "revise", "adjust", "fix",
                ],
            ),
            (
                Intent::Delete,
                &["delete", "remove", "drop", "clear", "erase", "get rid of"],
            ),
            (
                Intent::Search,
                &["find", "search", "locate", "where is", "look for", "get"],
            ),
            (
                Intent::Debug,
                &["debug", "fix", "error", "bug", "issue", "problem", "broken"],
            ),
            (
                Intent::Analyze,
                &[
                    "analyze", "review", "examine", "evaluate", "assess", "check", "inspect",
                ],
            ),
            (
                Intent::Execute,
                &["run", "execute", "start", "launch", "trigger"],
            ),
            (
                Intent::Configure,
                &["configure", "setup", "config", "install", "settings"],
            ),
            // Learning
            (
                Intent::Store,
                &["store", "save", "keep", "persist", "record", "note this"],
            ),
            (
                Intent::Learn,
                &[
                    "learn",
                    "train",
                    "memorize",
                    "from now on",
                    "in the future",
                ],
            ),
            (
                Intent::Remember,
                &[
                    "remember",
                    "recall",
                    "what did",
                    "previously",
                    "last time",
                    "earlier",
                ],
            ),
            // Meta
            (
                Intent::Feedback,
                &[
                    "good",
                    "bad",
                    "correct",
                    "wrong",
                    "yes",
                    "no",
                    "exactly",
                    "that's right",
                ],
            ),
            (
                Intent::Status,
                &[
                    "status",
                    "stats",
                    "statistics",
                    "metrics",
                    "how much",
                    "how many",
                ],
            ),
            (
                Intent::Help,
                &[
                    "help",
                    "assist",
                    "guide",
                    "what can you",
                    "how do i use",
                ],
            ),
        ];

        for (intent, keywords) in patterns {
            let mut score = 0.0f32;
            for keyword in *keywords {
                if query_lower.contains(keyword) {
                    score += 1.0 / keywords.len() as f32;
                }
            }
            if score > 0.0 {
                scores.push((*intent, score));
            }
        }

        // Sort by score descending
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scores
    }

    /// Detect primary intent from query keywords
    #[allow(dead_code)]
    fn from_query(query: &str) -> Self {
        let scores = Self::detect_all(query);
        scores.first().map(|(i, _)| *i).unwrap_or(Self::Unknown)
    }
}

/// Task label for multi-label classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskLabel {
    CodeGeneration,
    WebDevelopment,
    Backend,
    Frontend,
    Database,
    Api,
    Testing,
    Documentation,
    DevOps,
    Security,
    Performance,
    Refactoring,
    FileSystem,
    Configuration,
    Research,
    Learning,
    Custom(String),
}

impl TaskLabel {
    pub fn as_str(&self) -> &str {
        match self {
            Self::CodeGeneration => "code_generation",
            Self::WebDevelopment => "web_development",
            Self::Backend => "backend",
            Self::Frontend => "frontend",
            Self::Database => "database",
            Self::Api => "api",
            Self::Testing => "testing",
            Self::Documentation => "documentation",
            Self::DevOps => "devops",
            Self::Security => "security",
            Self::Performance => "performance",
            Self::Refactoring => "refactoring",
            Self::FileSystem => "file_system",
            Self::Configuration => "configuration",
            Self::Research => "research",
            Self::Learning => "learning",
            Self::Custom(s) => s,
        }
    }

    /// Detect labels from query keywords
    fn detect_labels(query: &str) -> Vec<Self> {
        let query_lower = query.to_lowercase();
        let mut labels = Vec::new();

        // Web development
        if query_lower.contains("web") || query_lower.contains("html")
            || query_lower.contains("css") || query_lower.contains("website")
            || query_lower.contains("react") || query_lower.contains("vue")
            || query_lower.contains("next") || query_lower.contains("tailwind") {
            labels.push(Self::WebDevelopment);
        }

        // Frontend
        if query_lower.contains("frontend") || query_lower.contains("ui")
            || query_lower.contains("component") || query_lower.contains("button")
            || query_lower.contains("form") || query_lower.contains("page") {
            labels.push(Self::Frontend);
        }

        // Backend
        if query_lower.contains("backend") || query_lower.contains("server")
            || query_lower.contains("api") || query_lower.contains("endpoint")
            || query_lower.contains("route") {
            labels.push(Self::Backend);
        }

        // Database
        if query_lower.contains("database") || query_lower.contains("db")
            || query_lower.contains("sql") || query_lower.contains("query")
            || query_lower.contains("table") || query_lower.contains("schema") {
            labels.push(Self::Database);
        }

        // API
        if query_lower.contains("api") || query_lower.contains("rest")
            || query_lower.contains("graphql") || query_lower.contains("endpoint") {
            labels.push(Self::Api);
        }

        // Testing
        if query_lower.contains("test") || query_lower.contains("spec")
            || query_lower.contains("coverage") || query_lower.contains("assert") {
            labels.push(Self::Testing);
        }

        // Documentation
        if query_lower.contains("doc") || query_lower.contains("readme")
            || query_lower.contains("comment") || query_lower.contains("explain") {
            labels.push(Self::Documentation);
        }

        // Code generation (default for create intents)
        if query_lower.contains("code") || query_lower.contains("function")
            || query_lower.contains("class") || query_lower.contains("implement") {
            labels.push(Self::CodeGeneration);
        }

        // Security
        if query_lower.contains("security") || query_lower.contains("auth")
            || query_lower.contains("password") || query_lower.contains("token")
            || query_lower.contains("encrypt") {
            labels.push(Self::Security);
        }

        // Performance
        if query_lower.contains("performance") || query_lower.contains("optimize")
            || query_lower.contains("fast") || query_lower.contains("slow")
            || query_lower.contains("cache") {
            labels.push(Self::Performance);
        }

        // File system
        if query_lower.contains("file") || query_lower.contains("folder")
            || query_lower.contains("directory") || query_lower.contains("path") {
            labels.push(Self::FileSystem);
        }

        // Configuration
        if query_lower.contains("config") || query_lower.contains("setup")
            || query_lower.contains("env") || query_lower.contains("settings") {
            labels.push(Self::Configuration);
        }

        // Refactoring
        if query_lower.contains("refactor") || query_lower.contains("clean")
            || query_lower.contains("restructure") {
            labels.push(Self::Refactoring);
        }

        labels
    }
}

/// The Perceiver module
pub struct Perceiver {
    /// Confidence threshold for research
    research_threshold: f32,

    /// Stopwords for keyword extraction
    stopwords: HashSet<&'static str>,
}

impl Default for Perceiver {
    fn default() -> Self {
        Self::new(0.7)
    }
}

impl Perceiver {
    /// Create a new Perceiver
    pub fn new(research_threshold: f32) -> Self {
        let stopwords: HashSet<&'static str> = [
            "the", "a", "an", "is", "are", "was", "were", "be", "been", "being", "have", "has",
            "had", "do", "does", "did", "will", "would", "could", "should", "may", "might",
            "must", "can", "this", "that", "these", "those", "i", "you", "he", "she", "it", "we",
            "they", "what", "which", "who", "whom", "how", "when", "where", "why", "and", "or",
            "but", "if", "then", "so", "as", "of", "in", "on", "at", "to", "for", "with", "by",
            "from", "about", "into", "through", "during", "before", "after", "above", "below",
            "up", "down", "out", "off", "over", "under", "again", "further", "once", "here",
            "there", "all", "each", "few", "more", "most", "other", "some", "such", "no", "not",
            "only", "own", "same", "than", "too", "very", "just", "also", "now", "please", "me",
            "my", "your", "their", "its",
        ]
        .into_iter()
        .collect();

        Self {
            research_threshold,
            stopwords,
        }
    }

    /// Analyze a query and produce a comprehensive PerceptionResult
    pub fn analyze(&self, query: &str, context: Option<&AggregatedContext>) -> PerceptionResult {
        let start = std::time::Instant::now();

        let mut result = PerceptionResult {
            query: query.to_string(),
            timestamp: chrono::Utc::now(),
            ..Default::default()
        };

        // 1. Detect intents (primary + secondary)
        self.detect_intents(query, &mut result);

        // 2. Detect task labels
        result.labels = TaskLabel::detect_labels(query);

        // Add CodeGeneration if Create intent but no labels
        if result.intent == Intent::Create && result.labels.is_empty() {
            result.labels.push(TaskLabel::CodeGeneration);
        }

        // 3. Extract entities
        result.entities = self.extract_entities(query);

        // 4. Extract topics and keywords
        self.extract_topics(query, &mut result);
        self.extract_keywords(query, &mut result);

        // 5. Assess complexity
        self.assess_complexity(query, &mut result);

        // 6. Integrate context if available
        if let Some(ctx) = context {
            self.integrate_context(ctx, &mut result);
        }

        // 7. Calculate confidence
        result.confidence = self.calculate_confidence(&result, context);

        // 8. Determine research/learning needs
        result.needs_research = result.confidence < self.research_threshold
            || result.scores_overall_relevance(context) < 0.5;
        result.needs_learning = result.intent == Intent::Feedback || result.intent == Intent::Learn;

        // 9. Generate recommended actions
        self.generate_recommendations(&mut result);

        // 10. Recommend external sources
        self.recommend_external_sources(&mut result);

        result.processing_time_ms = start.elapsed().as_secs_f64() * 1000.0;

        result
    }

    /// Analyze a query without aggregated context (for backward compatibility)
    pub fn analyze_simple(&self, query: &str) -> PerceptionResult {
        self.analyze(query, None)
    }

    /// Detect primary and secondary intents
    fn detect_intents(&self, query: &str, result: &mut PerceptionResult) {
        let scores = Intent::detect_all(query);

        if let Some((primary, score)) = scores.first() {
            result.intent = *primary;
            result.confidence = *score;
        }

        // Secondary intents (score > 0.3, max 3)
        result.secondary_intents = scores
            .iter()
            .skip(1)
            .filter(|(_, s)| *s > 0.3)
            .take(3)
            .map(|(i, _)| *i)
            .collect();
    }

    /// Extract named entities from query
    fn extract_entities(&self, query: &str) -> HashMap<String, String> {
        let mut entities = HashMap::new();
        let query_lower = query.to_lowercase();

        // Detect programming languages
        let languages = [
            "rust",
            "python",
            "typescript",
            "javascript",
            "go",
            "java",
            "c++",
            "c#",
        ];
        for lang in languages {
            if query_lower.contains(lang) {
                entities.insert("language".to_string(), lang.to_string());
                break;
            }
        }

        // Detect frameworks
        let frameworks = [
            ("react", "React"),
            ("next", "Next.js"),
            ("vue", "Vue"),
            ("angular", "Angular"),
            ("svelte", "Svelte"),
            ("express", "Express"),
            ("fastapi", "FastAPI"),
            ("django", "Django"),
            ("flask", "Flask"),
            ("actix", "Actix"),
            ("axum", "Axum"),
            ("tauri", "Tauri"),
        ];
        for (key, name) in frameworks {
            if query_lower.contains(key) {
                entities.insert("framework".to_string(), name.to_string());
                break;
            }
        }

        // Detect file types
        let file_extensions = [
            ".rs", ".py", ".ts", ".tsx", ".js", ".jsx", ".md", ".json", ".yaml", ".toml",
        ];
        for ext in file_extensions {
            if query.contains(ext) {
                entities.insert("file_type".to_string(), ext.to_string());
                break;
            }
        }

        // Extract code snippets (backticks)
        let code_regex = regex::Regex::new(r"`([^`]+)`").ok();
        if let Some(re) = code_regex {
            for cap in re.captures_iter(query) {
                if let Some(code) = cap.get(1) {
                    entities.insert("code_snippet".to_string(), code.as_str().to_string());
                    break;
                }
            }
        }

        // Extract URLs
        let url_regex = regex::Regex::new(r"https?://[^\s]+").ok();
        if let Some(re) = url_regex {
            if let Some(mat) = re.find(query) {
                entities.insert("url".to_string(), mat.as_str().to_string());
            }
        }

        entities
    }

    /// Extract topics from query
    fn extract_topics(&self, query: &str, result: &mut PerceptionResult) {
        let query_lower = query.to_lowercase();
        let mut topics = Vec::new();

        // Topic patterns
        let topic_patterns = [
            r"(?:about|regarding|concerning|on)\s+(\w+(?:\s+\w+)?)",
            r"the\s+(\w+(?:\s+\w+)?)\b",
        ];

        for pattern in topic_patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                for cap in re.captures_iter(&query_lower) {
                    if let Some(topic) = cap.get(1) {
                        let t = topic.as_str().trim().to_string();
                        if t.len() > 2 && !topics.contains(&t) {
                            topics.push(t);
                        }
                    }
                }
            }
        }

        result.topics = topics.into_iter().take(5).collect();
    }

    /// Extract keywords from query
    fn extract_keywords(&self, query: &str, result: &mut PerceptionResult) {
        let query_lower = query.to_lowercase();

        // Tokenize and filter
        let words: Vec<&str> = query_lower
            .split(|c: char| !c.is_alphanumeric())
            .filter(|w| w.len() > 2 && !self.stopwords.contains(w))
            .collect();

        // Deduplicate while preserving order
        let mut seen = HashSet::new();
        let keywords: Vec<String> = words
            .into_iter()
            .filter(|w| seen.insert(*w))
            .map(|w| w.to_string())
            .take(10)
            .collect();

        result.keywords = keywords;
    }

    /// Assess query complexity
    fn assess_complexity(&self, query: &str, result: &mut PerceptionResult) {
        let mut factors = Vec::new();

        // Word count
        let word_count = query.split_whitespace().count();
        if word_count > 50 {
            factors.push("very_long_query".to_string());
        } else if word_count > 20 {
            factors.push("long_query".to_string());
        }

        // Sentence count
        let sentence_count = query.matches(['.', '?', '!']).count() + 1;
        if sentence_count > 3 {
            factors.push("multiple_sentences".to_string());
        }

        // Conditionals
        let conditionals = ["if", "when", "unless", "assuming", "given that"];
        for cond in conditionals {
            if query.to_lowercase().contains(cond) {
                factors.push(format!("conditional:{}", cond));
                break;
            }
        }

        // Multiple items
        let multi_indicators = ["multiple", "several", "various", "different"];
        for ind in multi_indicators {
            if query.to_lowercase().contains(ind) {
                factors.push("multiple_items".to_string());
                break;
            }
        }

        // Many entities
        if result.entities.len() > 3 {
            factors.push("many_entities".to_string());
        }

        // Multiple intents
        if result.secondary_intents.len() > 1 {
            factors.push("multiple_intents".to_string());
        }

        // Architecture/system keywords
        let complex_keywords = ["architecture", "design", "system", "framework", "integration"];
        for kw in complex_keywords {
            if query.to_lowercase().contains(kw) {
                factors.push(format!("complex_topic:{}", kw));
            }
        }

        // Determine complexity level
        let very_complex_count = factors.iter().filter(|f| f.contains("very")).count();
        let complex_indicators = factors.len();

        result.complexity = if very_complex_count >= 2 || complex_indicators >= 5 {
            Complexity::VeryComplex
        } else if complex_indicators >= 3 {
            Complexity::Complex
        } else if complex_indicators >= 1 {
            Complexity::Moderate
        } else {
            Complexity::Simple
        };

        result.complexity_factors = factors;
    }

    /// Integrate context into perception
    fn integrate_context(&self, context: &AggregatedContext, result: &mut PerceptionResult) {
        // Build context summary
        let mut summary_parts = Vec::new();

        if !context.semantic_items.is_empty() {
            summary_parts.push(format!(
                "{} relevant knowledge items",
                context.semantic_items.len()
            ));
        }

        if !context.episodic_items.is_empty() {
            summary_parts.push(format!("{} history items", context.episodic_items.len()));
        }

        if !context.procedural_rules.is_empty() {
            summary_parts.push(format!("{} applicable rules", context.procedural_rules.len()));
            // Extract rule names
            result.relevant_rules = context
                .procedural_rules
                .iter()
                .filter_map(|r| {
                    r.metadata
                        .get("name")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .collect();
        }

        result.context_summary = if summary_parts.is_empty() {
            "No context".to_string()
        } else {
            summary_parts.join(", ")
        };
    }

    /// Calculate confidence score
    fn calculate_confidence(
        &self,
        result: &PerceptionResult,
        context: Option<&AggregatedContext>,
    ) -> f32 {
        let mut confidence = 0.5; // Base confidence

        // Known intent increases confidence
        if result.intent != Intent::Unknown {
            confidence += 0.2;
        }

        // Labels increase confidence
        confidence += (result.labels.len() as f32 * 0.05).min(0.15);

        // Entities increase confidence
        confidence += (result.entities.len() as f32 * 0.05).min(0.1);

        // Context increases confidence
        if let Some(ctx) = context {
            confidence += ctx.overall_relevance() * 0.1;
        }

        // Keywords increase confidence
        confidence += (result.keywords.len() as f32 * 0.01).min(0.05);

        confidence.min(1.0)
    }

    /// Generate recommended actions based on perception
    fn generate_recommendations(&self, result: &mut PerceptionResult) {
        let mut actions = Vec::new();

        // Intent-based recommendations
        match result.intent {
            Intent::Question | Intent::Explain => {
                actions.push(RecommendedAction::SearchSemantic);
                actions.push(RecommendedAction::ProvideExplanation);
            }
            Intent::Compare => {
                actions.push(RecommendedAction::SearchSemantic);
                actions.push(RecommendedAction::StructureComparison);
            }
            Intent::Create => {
                actions.push(RecommendedAction::CheckExisting);
                actions.push(RecommendedAction::GenerateContent);
            }
            Intent::Modify => {
                actions.push(RecommendedAction::GatherContext);
                actions.push(RecommendedAction::ApplyChanges);
            }
            Intent::Delete => {
                actions.push(RecommendedAction::CheckExisting);
                actions.push(RecommendedAction::ExecuteDeletion);
            }
            Intent::Search => {
                actions.push(RecommendedAction::SearchAllMemories);
                actions.push(RecommendedAction::RankResults);
            }
            Intent::Debug | Intent::Analyze => {
                actions.push(RecommendedAction::GatherContext);
                actions.push(RecommendedAction::PerformAnalysis);
            }
            Intent::Store => {
                actions.push(RecommendedAction::ValidateContent);
                actions.push(RecommendedAction::StoreToMemory);
            }
            Intent::Learn => {
                actions.push(RecommendedAction::ExtractKnowledge);
                actions.push(RecommendedAction::UpdateProcedural);
            }
            Intent::Remember => {
                actions.push(RecommendedAction::SearchEpisodic);
            }
            Intent::Feedback => {
                actions.push(RecommendedAction::ProcessFeedback);
            }
            Intent::Status => {
                actions.push(RecommendedAction::GatherStats);
            }
            Intent::Help => {
                actions.push(RecommendedAction::ProvideGuidance);
            }
            _ => {}
        }

        // Complexity-based additions
        if matches!(result.complexity, Complexity::Complex | Complexity::VeryComplex) {
            actions.insert(0, RecommendedAction::DecomposeQuery);
        }

        // Research recommendation
        if result.needs_research && !actions.contains(&RecommendedAction::SearchSemantic) {
            actions.insert(0, RecommendedAction::SearchSemantic);
        }

        result.recommended_actions = actions;
    }

    /// Recommend external sources based on query analysis
    fn recommend_external_sources(&self, result: &mut PerceptionResult) {
        let mut sources = Vec::new();
        let query_lower = result.query.to_lowercase();

        // Check for library/framework mentions -> Context7
        let libraries = [
            "react", "next", "vue", "angular", "tailwind", "express", "django", "fastapi",
            "rust", "cargo", "tokio", "axum", "framer", "motion",
        ];
        for lib in libraries {
            if query_lower.contains(lib) {
                sources.push(ExternalSourceRecommendation {
                    source_type: ExternalSourceType::Context7,
                    reason: format!("Query mentions {} - official docs may help", lib),
                    suggested_query: Some(lib.to_string()),
                    priority: 8,
                });
                break;
            }
        }

        // Check for Microsoft/Azure mentions -> MSLearn
        let ms_keywords = [
            "azure", "microsoft", ".net", "dotnet", "c#", "visual studio", "typescript",
        ];
        for kw in ms_keywords {
            if query_lower.contains(kw) {
                sources.push(ExternalSourceRecommendation {
                    source_type: ExternalSourceType::MicrosoftLearn,
                    reason: format!("Query mentions {} - MS Learn has official docs", kw),
                    suggested_query: Some(kw.to_string()),
                    priority: 7,
                });
                break;
            }
        }

        // Complex queries may benefit from sequential thinking
        if matches!(result.complexity, Complexity::VeryComplex) {
            sources.push(ExternalSourceRecommendation {
                source_type: ExternalSourceType::SequentialThinking,
                reason: "Very complex query - sequential thinking may help".to_string(),
                suggested_query: None,
                priority: 9,
            });
        }

        // Web search for current info, news, or unknown topics
        let web_indicators = [
            "latest", "new", "recent", "2024", "2025", "best practice", "current",
        ];
        for ind in web_indicators {
            if query_lower.contains(ind) {
                sources.push(ExternalSourceRecommendation {
                    source_type: ExternalSourceType::Tavily,
                    reason: format!("Query mentions '{}' - web search may have current info", ind),
                    suggested_query: Some(result.query.clone()),
                    priority: 6,
                });
                break;
            }
        }

        // If low confidence and few internal matches, suggest web search
        if result.needs_research && sources.is_empty() {
            sources.push(ExternalSourceRecommendation {
                source_type: ExternalSourceType::Tavily,
                reason: "Low confidence - web search may provide additional context".to_string(),
                suggested_query: Some(result.query.clone()),
                priority: 5,
            });
        }

        // Sort by priority descending
        sources.sort_by(|a, b| b.priority.cmp(&a.priority));
        result.external_sources = sources;
    }
}

impl PerceptionResult {
    /// Helper to check overall relevance from context
    fn scores_overall_relevance(&self, context: Option<&AggregatedContext>) -> f32 {
        context.map(|c| c.overall_relevance()).unwrap_or(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_detection() {
        assert_eq!(Intent::from_query("create a new function"), Intent::Create);
        assert_eq!(Intent::from_query("generate a website"), Intent::Create);
        assert_eq!(Intent::from_query("find the bug"), Intent::Search);
        assert_eq!(Intent::from_query("fix the error"), Intent::Debug);
        assert_eq!(Intent::from_query("explain how this works"), Intent::Explain);
        assert_eq!(
            Intent::from_query("compare React vs Vue"),
            Intent::Compare
        );
        assert_eq!(Intent::from_query("remember what we did"), Intent::Remember);
        assert_eq!(Intent::from_query("store this information"), Intent::Store);
    }

    #[test]
    fn test_label_detection() {
        let labels = TaskLabel::detect_labels("create a React component");
        assert!(labels.contains(&TaskLabel::WebDevelopment));

        let labels = TaskLabel::detect_labels("write a test for the API");
        assert!(labels.contains(&TaskLabel::Testing));
        assert!(labels.contains(&TaskLabel::Api));
    }

    #[test]
    fn test_perceiver_basic() {
        let perceiver = Perceiver::new(0.7);
        let result = perceiver.analyze("create a React website", None);

        assert_eq!(result.intent, Intent::Create);
        assert!(result.labels.contains(&TaskLabel::WebDevelopment));
        assert!(!result.query.is_empty());
        assert!(result.processing_time_ms > 0.0);
    }

    #[test]
    fn test_perceiver_complexity() {
        let perceiver = Perceiver::new(0.7);

        // Simple query
        let simple = perceiver.analyze("create a button", None);
        assert_eq!(simple.complexity, Complexity::Simple);

        // Complex query
        let complex = perceiver.analyze(
            "design a microservices architecture with multiple databases and API gateways for a large-scale system",
            None,
        );
        assert!(matches!(
            complex.complexity,
            Complexity::Complex | Complexity::VeryComplex
        ));
    }

    #[test]
    fn test_perceiver_external_sources() {
        let perceiver = Perceiver::new(0.7);

        // Query mentioning React should recommend Context7
        let result = perceiver.analyze("how to use React hooks", None);
        assert!(result
            .external_sources
            .iter()
            .any(|s| s.source_type == ExternalSourceType::Context7));

        // Query mentioning Azure should recommend MSLearn
        let result = perceiver.analyze("deploy to Azure functions", None);
        assert!(result
            .external_sources
            .iter()
            .any(|s| s.source_type == ExternalSourceType::MicrosoftLearn));
    }

    #[test]
    fn test_perceiver_keywords() {
        let perceiver = Perceiver::new(0.7);
        let result = perceiver.analyze("create a React component with TypeScript", None);

        // Should extract meaningful keywords
        assert!(result.keywords.contains(&"react".to_string()));
        assert!(result.keywords.contains(&"component".to_string()));
        assert!(result.keywords.contains(&"typescript".to_string()));
    }

    #[test]
    fn test_perceiver_recommended_actions() {
        let perceiver = Perceiver::new(0.7);

        // Create intent should recommend generation
        let result = perceiver.analyze("create a new API endpoint", None);
        assert!(result
            .recommended_actions
            .contains(&RecommendedAction::GenerateContent));

        // Search intent should recommend search
        let result = perceiver.analyze("find all user handlers", None);
        assert!(result
            .recommended_actions
            .contains(&RecommendedAction::SearchAllMemories));
    }
}
