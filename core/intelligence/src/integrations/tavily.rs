//! Tavily Client for AI-powered web search
//!
//! Provides intelligent web search capabilities with:
//! - Real-time search results
//! - Content extraction
//! - Domain filtering
//! - Topic-based search (general, news)

use super::{IntegrationClient, SearchResult};
use crate::error::{IntelligenceError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Tavily API client for web search
pub struct TavilyClient {
    /// HTTP client
    client: Option<Client>,

    /// API key
    api_key: Option<String>,

    /// Base URL
    base_url: String,

    /// Whether initialized
    initialized: bool,
}

/// Search request parameters
#[derive(Debug, Serialize)]
struct TavilySearchRequest {
    api_key: String,
    query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    search_depth: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    topic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_results: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exclude_domains: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_raw_content: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    include_images: Option<bool>,
}

/// Search response from Tavily API
#[derive(Debug, Deserialize)]
struct TavilySearchResponse {
    #[serde(rename = "query")]
    _query: String,
    results: Vec<TavilyResult>,
    #[serde(default, rename = "images")]
    _images: Vec<TavilyImage>,
}

#[derive(Debug, Deserialize)]
struct TavilyResult {
    title: String,
    url: String,
    content: String,
    score: f32,
    #[serde(default)]
    raw_content: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TavilyImage {
    #[serde(rename = "url")]
    _url: String,
    #[serde(default, rename = "description")]
    _description: Option<String>,
}

/// Extract request parameters
#[derive(Debug, Serialize)]
struct TavilyExtractRequest {
    api_key: String,
    urls: Vec<String>,
}

/// Extract response from Tavily API
#[derive(Debug, Deserialize)]
struct TavilyExtractResponse {
    results: Vec<TavilyExtractResult>,
    #[serde(default)]
    failed_results: Vec<TavilyFailedResult>,
}

#[derive(Debug, Deserialize)]
struct TavilyExtractResult {
    url: String,
    raw_content: String,
}

#[derive(Debug, Deserialize)]
struct TavilyFailedResult {
    url: String,
    error: String,
}

/// Search depth options
#[derive(Debug, Clone, Copy)]
pub enum SearchDepth {
    /// Basic search (faster)
    Basic,
    /// Advanced search (more comprehensive)
    Advanced,
}

impl SearchDepth {
    fn as_str(&self) -> &str {
        match self {
            Self::Basic => "basic",
            Self::Advanced => "advanced",
        }
    }
}

/// Topic options for search
#[derive(Debug, Clone, Copy)]
pub enum SearchTopic {
    /// General web search
    General,
    /// News-focused search
    News,
}

impl SearchTopic {
    fn as_str(&self) -> &str {
        match self {
            Self::General => "general",
            Self::News => "news",
        }
    }
}

/// Search options
#[derive(Debug, Default)]
pub struct SearchOptions {
    /// Search depth
    pub depth: Option<SearchDepth>,
    /// Search topic
    pub topic: Option<SearchTopic>,
    /// Maximum results
    pub max_results: Option<usize>,
    /// Include only these domains
    pub include_domains: Option<Vec<String>>,
    /// Exclude these domains
    pub exclude_domains: Option<Vec<String>>,
    /// Include raw content
    pub include_raw_content: bool,
    /// Include images
    pub include_images: bool,
}

impl TavilyClient {
    /// Create a new Tavily client
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: None,
            api_key,
            base_url: "https://api.tavily.com".to_string(),
            initialized: false,
        }
    }

    /// Create client from environment variable
    pub fn from_env() -> Self {
        let api_key = std::env::var("TAVILY_API_KEY").ok();
        Self::new(api_key)
    }

    /// Perform a simple search
    pub async fn search(&self, query: &str, max_results: usize) -> Result<Vec<SearchResult>> {
        self.search_with_options(
            query,
            SearchOptions {
                max_results: Some(max_results),
                ..Default::default()
            },
        )
        .await
    }

    /// Perform a search with advanced options
    pub async fn search_with_options(
        &self,
        query: &str,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        if !self.initialized || self.api_key.is_none() {
            return Ok(vec![]);
        }

        let client = self.client.as_ref().unwrap();
        let api_key = self.api_key.as_ref().unwrap();

        let request = TavilySearchRequest {
            api_key: api_key.clone(),
            query: query.to_string(),
            search_depth: options.depth.map(|d| d.as_str().to_string()),
            topic: options.topic.map(|t| t.as_str().to_string()),
            max_results: options.max_results,
            include_domains: options.include_domains,
            exclude_domains: options.exclude_domains,
            include_raw_content: Some(options.include_raw_content),
            include_images: Some(options.include_images),
        };

        let response = client
            .post(format!("{}/search", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| IntelligenceError::Config(format!("Tavily request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            tracing::warn!("Tavily search failed: {} - {}", status, text);
            return Ok(vec![]);
        }

        let data: TavilySearchResponse = response
            .json()
            .await
            .map_err(|e| IntelligenceError::Config(format!("Tavily parse failed: {}", e)))?;

        Ok(data
            .results
            .into_iter()
            .map(|r| SearchResult {
                title: r.title,
                content: r.content,
                url: Some(r.url),
                source: "tavily".into(),
                score: r.score,
                metadata: serde_json::json!({
                    "has_raw_content": r.raw_content.is_some()
                }),
            })
            .collect())
    }

    /// Extract content from URLs
    pub async fn extract(&self, urls: Vec<String>) -> Result<Vec<ExtractedContent>> {
        if !self.initialized || self.api_key.is_none() {
            return Ok(vec![]);
        }

        let client = self.client.as_ref().unwrap();
        let api_key = self.api_key.as_ref().unwrap();

        let request = TavilyExtractRequest {
            api_key: api_key.clone(),
            urls,
        };

        let response = client
            .post(format!("{}/extract", self.base_url))
            .json(&request)
            .send()
            .await
            .map_err(|e| IntelligenceError::Config(format!("Tavily extract failed: {}", e)))?;

        if !response.status().is_success() {
            tracing::warn!("Tavily extract failed: {}", response.status());
            return Ok(vec![]);
        }

        let data: TavilyExtractResponse = response
            .json()
            .await
            .map_err(|e| IntelligenceError::Config(format!("Tavily parse failed: {}", e)))?;

        Ok(data
            .results
            .into_iter()
            .map(|r| ExtractedContent {
                url: r.url,
                content: r.raw_content,
                success: true,
                error: None,
            })
            .chain(data.failed_results.into_iter().map(|r| ExtractedContent {
                url: r.url,
                content: String::new(),
                success: false,
                error: Some(r.error),
            }))
            .collect())
    }

    /// Search for news
    pub async fn search_news(&self, query: &str, max_results: usize) -> Result<Vec<SearchResult>> {
        self.search_with_options(
            query,
            SearchOptions {
                topic: Some(SearchTopic::News),
                max_results: Some(max_results),
                ..Default::default()
            },
        )
        .await
    }

    /// Search with domain filter
    pub async fn search_domains(
        &self,
        query: &str,
        include_domains: Vec<String>,
        max_results: usize,
    ) -> Result<Vec<SearchResult>> {
        self.search_with_options(
            query,
            SearchOptions {
                include_domains: Some(include_domains),
                max_results: Some(max_results),
                ..Default::default()
            },
        )
        .await
    }
}

/// Extracted content from a URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedContent {
    /// Source URL
    pub url: String,
    /// Extracted content
    pub content: String,
    /// Whether extraction succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

#[async_trait]
impl IntegrationClient for TavilyClient {
    fn provider(&self) -> &str {
        "tavily"
    }

    fn is_ready(&self) -> bool {
        self.initialized
    }

    async fn initialize(&mut self) -> Result<bool> {
        if self.api_key.is_none() {
            tracing::warn!("Tavily API key not configured - client disabled");
            return Ok(false);
        }

        self.client = Some(
            Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| IntelligenceError::Config(format!("HTTP client error: {}", e)))?,
        );

        self.initialized = true;
        tracing::info!("Tavily client initialized");
        Ok(true)
    }

    async fn health_check(&self) -> Result<bool> {
        Ok(self.initialized)
    }

    async fn close(&mut self) -> Result<()> {
        self.client = None;
        self.initialized = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_depth() {
        assert_eq!(SearchDepth::Basic.as_str(), "basic");
        assert_eq!(SearchDepth::Advanced.as_str(), "advanced");
    }

    #[test]
    fn test_search_topic() {
        assert_eq!(SearchTopic::General.as_str(), "general");
        assert_eq!(SearchTopic::News.as_str(), "news");
    }

    #[tokio::test]
    async fn test_client_without_key() {
        let client = TavilyClient::new(None);
        let results = client.search("test", 10).await.unwrap();
        assert!(results.is_empty());
    }
}
