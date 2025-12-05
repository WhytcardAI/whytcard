//! Microsoft Learn Client for official Microsoft/Azure documentation
//!
//! Provides access to official Microsoft documentation via Learn API.
//! Used for retrieving Azure, .NET, Microsoft 365, and other Microsoft technologies docs.

use super::{DocResult, IntegrationClient, SearchResult};
use crate::error::{IntelligenceError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Microsoft Learn API client
pub struct MSLearnClient {
    /// HTTP client
    client: Option<Client>,

    /// Base search URL
    search_url: String,

    /// Whether initialized
    initialized: bool,
}

/// Search response from Microsoft Learn API
#[derive(Debug, Deserialize)]
struct MSLearnSearchResponse {
    results: Vec<MSLearnResult>,
    #[serde(default, rename = "count")]
    _count: usize,
}

#[derive(Debug, Deserialize)]
struct MSLearnResult {
    title: String,
    url: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    last_updated: Option<String>,
    #[serde(default)]
    product: Option<String>,
    #[serde(default)]
    score: Option<f32>,
}

/// Code sample from Microsoft Learn
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSample {
    /// Sample title
    pub title: String,
    /// Programming language
    pub language: String,
    /// Code content
    pub code: String,
    /// Source URL
    pub url: Option<String>,
}

impl MSLearnClient {
    /// Create a new Microsoft Learn client
    pub fn new() -> Self {
        Self {
            client: None,
            search_url: "https://learn.microsoft.com/api/search".to_string(),
            initialized: false,
        }
    }

    /// Search Microsoft Learn documentation
    pub async fn search(&self, query: &str, max_results: usize) -> Result<Vec<SearchResult>> {
        if !self.initialized || self.client.is_none() {
            return Ok(vec![]);
        }

        let client = self.client.as_ref().unwrap();

        let response = client
            .get(&self.search_url)
            .query(&[
                ("search", query),
                ("locale", "en-us"),
                ("$top", &max_results.to_string()),
            ])
            .send()
            .await
            .map_err(|e| IntelligenceError::Config(format!("MS Learn request failed: {}", e)))?;

        if !response.status().is_success() {
            tracing::warn!("MS Learn search failed: {}", response.status());
            return Ok(vec![]);
        }

        let data: MSLearnSearchResponse = response
            .json()
            .await
            .map_err(|e| IntelligenceError::Config(format!("MS Learn parse failed: {}", e)))?;

        Ok(data
            .results
            .into_iter()
            .map(|r| SearchResult {
                title: r.title,
                content: r.description,
                url: Some(r.url),
                source: "microsoft_learn".into(),
                score: r.score.unwrap_or(0.5),
                metadata: serde_json::json!({
                    "product": r.product,
                    "last_updated": r.last_updated
                }),
            })
            .collect())
    }

    /// Fetch a specific documentation page
    pub async fn fetch_docs(&self, query: &str) -> Result<Option<DocResult>> {
        // First search for the most relevant page
        let results = self.search(query, 1).await?;

        if results.is_empty() {
            return Ok(None);
        }

        let first = &results[0];

        // In a real implementation, we would fetch and parse the actual page
        // For now, we return the search result as a DocResult
        Ok(Some(DocResult {
            source: "microsoft_learn".into(),
            topic: Some(query.to_string()),
            content: first.content.clone(),
            code_snippets: vec![],
            url: first.url.clone(),
            provider: "microsoft_learn".into(),
        }))
    }

    /// Search for code samples
    pub async fn search_code_samples(
        &self,
        query: &str,
        language: Option<&str>,
        max_results: usize,
    ) -> Result<Vec<CodeSample>> {
        if !self.initialized || self.client.is_none() {
            return Ok(vec![]);
        }

        let client = self.client.as_ref().unwrap();

        let mut query_params = vec![
            ("search", query.to_string()),
            ("locale", "en-us".to_string()),
            ("$top", max_results.to_string()),
            ("category", "code".to_string()),
        ];

        if let Some(lang) = language {
            query_params.push(("language", lang.to_string()));
        }

        let response = client
            .get(&self.search_url)
            .query(&query_params)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(data) = resp.json::<MSLearnSearchResponse>().await {
                    return Ok(data
                        .results
                        .into_iter()
                        .map(|r| CodeSample {
                            title: r.title,
                            language: language.unwrap_or("unknown").to_string(),
                            code: r.description, // Would be actual code in real impl
                            url: Some(r.url),
                        })
                        .collect());
                }
            }
            _ => {}
        }

        Ok(vec![])
    }

    /// Get documentation for Azure services
    pub async fn get_azure_docs(&self, service: &str, topic: Option<&str>) -> Result<Vec<SearchResult>> {
        let query = if let Some(t) = topic {
            format!("azure {} {}", service, t)
        } else {
            format!("azure {}", service)
        };

        self.search(&query, 10).await
    }

    /// Get documentation for .NET
    pub async fn get_dotnet_docs(&self, topic: &str) -> Result<Vec<SearchResult>> {
        let query = format!(".NET {}", topic);
        self.search(&query, 10).await
    }
}

impl Default for MSLearnClient {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl IntegrationClient for MSLearnClient {
    fn provider(&self) -> &str {
        "microsoft_learn"
    }

    fn is_ready(&self) -> bool {
        self.initialized
    }

    async fn initialize(&mut self) -> Result<bool> {
        self.client = Some(
            Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| IntelligenceError::Config(format!("HTTP client error: {}", e)))?,
        );

        self.initialized = true;
        tracing::info!("Microsoft Learn client initialized");
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
    fn test_client_creation() {
        let client = MSLearnClient::new();
        assert!(!client.initialized);
    }

    #[tokio::test]
    async fn test_search_without_init() {
        let client = MSLearnClient::new();
        let results = client.search("test", 10).await.unwrap();
        assert!(results.is_empty());
    }
}
