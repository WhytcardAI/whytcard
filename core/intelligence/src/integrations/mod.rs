//! External Integrations for WhytCard Intelligence
//!
//! Provides clients for external documentation and search services:
//! - Context7: Library documentation retrieval
//! - Tavily: Web search with AI-powered results
//! - Microsoft Learn: Official Microsoft/Azure documentation

pub mod context7;
pub mod mslearn;
pub mod tavily;

pub use context7::Context7Client;
pub use mslearn::MSLearnClient;
pub use tavily::TavilyClient;

use crate::error::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Common result type for documentation retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocResult {
    /// Library or source name
    pub source: String,

    /// Topic or focus area
    pub topic: Option<String>,

    /// Main content
    pub content: String,

    /// Code snippets extracted
    pub code_snippets: Vec<String>,

    /// Source URL
    pub url: Option<String>,

    /// Provider name
    pub provider: String,
}

/// Common result type for search operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Result title
    pub title: String,

    /// Content snippet
    pub content: String,

    /// Source URL
    pub url: Option<String>,

    /// Provider name
    pub source: String,

    /// Relevance score (0.0 - 1.0)
    pub score: f32,

    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Trait for integration clients
#[async_trait]
pub trait IntegrationClient: Send + Sync {
    /// Get the provider name
    fn provider(&self) -> &str;

    /// Check if the client is initialized and ready
    fn is_ready(&self) -> bool;

    /// Initialize the client
    async fn initialize(&mut self) -> Result<bool>;

    /// Perform a health check
    async fn health_check(&self) -> Result<bool>;

    /// Close the client and release resources
    async fn close(&mut self) -> Result<()>;
}

/// Hub for managing all integrations
pub struct IntegrationHub {
    /// Context7 client for library documentation
    pub context7: Option<Context7Client>,

    /// Tavily client for web search
    pub tavily: Option<TavilyClient>,

    /// Microsoft Learn client for Azure/Microsoft docs
    pub mslearn: Option<MSLearnClient>,
}

impl IntegrationHub {
    /// Create a new integration hub with default configuration
    pub fn new() -> Self {
        Self {
            context7: None,
            tavily: None,
            mslearn: None,
        }
    }

    /// Create hub with Context7 client
    pub fn with_context7(mut self, client: Context7Client) -> Self {
        self.context7 = Some(client);
        self
    }

    /// Create hub with Tavily client
    pub fn with_tavily(mut self, client: TavilyClient) -> Self {
        self.tavily = Some(client);
        self
    }

    /// Create hub with MS Learn client
    pub fn with_mslearn(mut self, client: MSLearnClient) -> Self {
        self.mslearn = Some(client);
        self
    }

    /// Initialize all configured clients
    pub async fn initialize_all(&mut self) -> Result<()> {
        if let Some(ref mut c7) = self.context7 {
            c7.initialize().await?;
        }
        if let Some(ref mut tavily) = self.tavily {
            tavily.initialize().await?;
        }
        if let Some(ref mut mslearn) = self.mslearn {
            mslearn.initialize().await?;
        }
        Ok(())
    }

    /// Close all clients
    pub async fn close_all(&mut self) -> Result<()> {
        if let Some(ref mut c7) = self.context7 {
            c7.close().await?;
        }
        if let Some(ref mut tavily) = self.tavily {
            tavily.close().await?;
        }
        if let Some(ref mut mslearn) = self.mslearn {
            mslearn.close().await?;
        }
        Ok(())
    }

    /// Get documentation from the best available source
    pub async fn get_docs(
        &self,
        library: &str,
        topic: Option<&str>,
    ) -> Result<Option<DocResult>> {
        // Try Context7 first for library documentation
        if let Some(ref c7) = self.context7 {
            if c7.is_ready() {
                if let Some(result) = c7.get_library_docs(library, topic, 5000).await? {
                    return Ok(Some(result));
                }
            }
        }

        // Try MS Learn for Microsoft/Azure libraries
        if let Some(ref mslearn) = self.mslearn {
            if mslearn.is_ready() {
                let query = if let Some(t) = topic {
                    format!("{} {}", library, t)
                } else {
                    library.to_string()
                };
                if let Some(result) = mslearn.fetch_docs(&query).await? {
                    return Ok(Some(result));
                }
            }
        }

        Ok(None)
    }

    /// Search across all available sources
    pub async fn search(&self, query: &str, max_results: usize) -> Result<Vec<SearchResult>> {
        let mut all_results = Vec::new();

        // Search Context7
        if let Some(ref c7) = self.context7 {
            if c7.is_ready() {
                let results = c7.search_docs(query, None).await?;
                all_results.extend(results.into_iter().take(max_results / 3));
            }
        }

        // Search Tavily for web results
        if let Some(ref tavily) = self.tavily {
            if tavily.is_ready() {
                let results = tavily.search(query, max_results / 2).await?;
                all_results.extend(results);
            }
        }

        // Search MS Learn
        if let Some(ref mslearn) = self.mslearn {
            if mslearn.is_ready() {
                let results = mslearn.search(query, max_results / 3).await?;
                all_results.extend(results);
            }
        }

        // Sort by score descending
        all_results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        Ok(all_results.into_iter().take(max_results).collect())
    }
}

impl Default for IntegrationHub {
    fn default() -> Self {
        Self::new()
    }
}
