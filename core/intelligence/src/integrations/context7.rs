//! Context7 Client for library documentation
//!
//! Provides access to official library documentation via Context7 API.
//! Used for retrieving up-to-date documentation and code examples.

use super::{DocResult, IntegrationClient, SearchResult};
use crate::error::{IntelligenceError, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Context7 API client for library documentation
pub struct Context7Client {
    /// HTTP client
    client: Option<Client>,

    /// API key
    api_key: Option<String>,

    /// Base URL
    base_url: String,

    /// Library ID cache (name -> id)
    library_cache: Arc<RwLock<HashMap<String, String>>>,

    /// Whether initialized
    initialized: bool,
}

/// Response from library resolution
#[derive(Debug, Deserialize)]
struct ResolveResponse {
    library_id: Option<String>,
    #[serde(rename = "name")]
    _name: Option<String>,
    #[serde(rename = "description")]
    _description: Option<String>,
}

/// Response from docs retrieval
#[derive(Debug, Deserialize)]
struct DocsResponse {
    content: String,
    url: Option<String>,
    #[serde(rename = "library_id")]
    _library_id: Option<String>,
}

/// Response from search
#[derive(Debug, Deserialize)]
struct SearchResponse {
    results: Vec<SearchItem>,
}

#[derive(Debug, Deserialize)]
struct SearchItem {
    title: String,
    snippet: String,
    url: Option<String>,
    library_id: Option<String>,
    score: Option<f32>,
}

impl Context7Client {
    /// Create a new Context7 client
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: None,
            api_key,
            base_url: "https://context7.com/api".to_string(),
            library_cache: Arc::new(RwLock::new(Self::common_library_mappings())),
            initialized: false,
        }
    }

    /// Create client from environment variable
    pub fn from_env() -> Self {
        let api_key = std::env::var("CONTEXT7_API_KEY").ok();
        Self::new(api_key)
    }

    /// Common library ID mappings for fallback
    fn common_library_mappings() -> HashMap<String, String> {
        let mut map = HashMap::new();

        // Frontend
        map.insert("react".into(), "/facebook/react".into());
        map.insert("vue".into(), "/vuejs/vue".into());
        map.insert("angular".into(), "/angular/angular".into());
        map.insert("svelte".into(), "/sveltejs/svelte".into());
        map.insert("next".into(), "/vercel/next.js".into());
        map.insert("nextjs".into(), "/vercel/next.js".into());
        map.insert("vite".into(), "/vitejs/vite".into());
        map.insert("tailwind".into(), "/tailwindlabs/tailwindcss".into());
        map.insert("tailwindcss".into(), "/tailwindlabs/tailwindcss".into());
        map.insert("framer-motion".into(), "/framer/motion".into());

        // Backend Rust
        map.insert("axum".into(), "/tokio-rs/axum".into());
        map.insert("tokio".into(), "/tokio-rs/tokio".into());
        map.insert("tauri".into(), "/tauri-apps/tauri".into());
        map.insert("serde".into(), "/serde-rs/serde".into());
        map.insert("sqlx".into(), "/launchbadge/sqlx".into());
        map.insert("reqwest".into(), "/seanmonstar/reqwest".into());
        map.insert("tracing".into(), "/tokio-rs/tracing".into());

        // Python
        map.insert("fastapi".into(), "/tiangolo/fastapi".into());
        map.insert("django".into(), "/django/django".into());
        map.insert("flask".into(), "/pallets/flask".into());
        map.insert("pydantic".into(), "/pydantic/pydantic".into());

        // Database
        map.insert("lancedb".into(), "/lancedb/lancedb".into());
        map.insert("surrealdb".into(), "/surrealdb/surrealdb".into());
        map.insert("postgresql".into(), "/postgres/postgres".into());
        map.insert("redis".into(), "/redis/redis".into());

        // AI/ML
        map.insert("langchain".into(), "/langchain-ai/langchain".into());
        map.insert("openai".into(), "/openai/openai-python".into());

        // Other
        map.insert("typescript".into(), "/microsoft/TypeScript".into());
        map.insert("nodejs".into(), "/nodejs/node".into());

        map
    }

    /// Resolve a library name to a Context7-compatible ID
    pub async fn resolve_library_id(&self, library_name: &str) -> Result<Option<String>> {
        let name_lower = library_name.to_lowercase();

        // Check cache first
        {
            let cache = self.library_cache.read().await;
            if let Some(id) = cache.get(&name_lower) {
                return Ok(Some(id.clone()));
            }
        }

        // If API key is available, try to resolve via API
        if self.initialized {
            if let Some(ref client) = self.client {
                let response = client
                    .get(format!("{}/v1/resolve", self.base_url))
                    .query(&[("name", library_name)])
                    .send()
                    .await;

                if let Ok(resp) = response {
                    if resp.status().is_success() {
                        if let Ok(data) = resp.json::<ResolveResponse>().await {
                            if let Some(library_id) = data.library_id {
                                // Cache the result
                                let mut cache = self.library_cache.write().await;
                                cache.insert(name_lower.clone(), library_id.clone());
                                return Ok(Some(library_id));
                            }
                        }
                    }
                }
            }
        }

        // Return None if not found
        Ok(None)
    }

    /// Get documentation for a library
    pub async fn get_library_docs(
        &self,
        library: &str,
        topic: Option<&str>,
        tokens: u32,
    ) -> Result<Option<DocResult>> {
        // First resolve the library ID
        let library_id = if library.starts_with('/') {
            library.to_string()
        } else {
            match self.resolve_library_id(library).await? {
                Some(id) => id,
                None => return Ok(None),
            }
        };

        if !self.initialized {
            // Return cached/fallback info if not initialized
            return Ok(Some(DocResult {
                source: library_id.clone(),
                topic: topic.map(String::from),
                content: format!(
                    "Library: {}\nTopic: {}\n\nNote: Context7 API not initialized. Provide CONTEXT7_API_KEY to get real documentation.",
                    library_id,
                    topic.unwrap_or("general")
                ),
                code_snippets: vec![],
                url: None,
                provider: "context7".into(),
            }));
        }

        if let Some(ref client) = self.client {
            let tokens_str = tokens.to_string();
            let mut query_params = vec![
                ("library_id", library_id.as_str()),
                ("tokens", tokens_str.as_str()),
            ];

            let topic_str;
            if let Some(t) = topic {
                topic_str = t.to_string();
                query_params.push(("topic", topic_str.as_str()));
            }

            let response = client
                .get(format!("{}/v1/docs", self.base_url))
                .query(&query_params)
                .send()
                .await
                .map_err(|e| IntelligenceError::Config(format!("Context7 request failed: {}", e)))?;

            if !response.status().is_success() {
                tracing::warn!("Context7 docs request failed: {}", response.status());
                return Ok(None);
            }

            let data: DocsResponse = response
                .json()
                .await
                .map_err(|e| IntelligenceError::Config(format!("Context7 parse failed: {}", e)))?;

            // Extract code snippets from content
            let code_snippets = Self::extract_code_snippets(&data.content);

            return Ok(Some(DocResult {
                source: library_id,
                topic: topic.map(String::from),
                content: data.content,
                code_snippets,
                url: data.url,
                provider: "context7".into(),
            }));
        }

        Ok(None)
    }

    /// Search documentation
    pub async fn search_docs(
        &self,
        query: &str,
        library_id: Option<&str>,
    ) -> Result<Vec<SearchResult>> {
        if !self.initialized || self.client.is_none() {
            return Ok(vec![]);
        }

        let client = self.client.as_ref().unwrap();

        let mut query_params: Vec<(&str, &str)> = vec![("query", query)];
        if let Some(lib_id) = library_id {
            query_params.push(("library_id", lib_id));
        }

        let response = client
            .get(format!("{}/v1/search", self.base_url))
            .query(&query_params)
            .send()
            .await;

        match response {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(data) = resp.json::<SearchResponse>().await {
                    return Ok(data
                        .results
                        .into_iter()
                        .map(|item| SearchResult {
                            title: item.title,
                            content: item.snippet,
                            url: item.url,
                            source: "context7".into(),
                            score: item.score.unwrap_or(0.5),
                            metadata: serde_json::json!({
                                "library": item.library_id
                            }),
                        })
                        .collect());
                }
            }
            _ => {}
        }

        Ok(vec![])
    }

    /// Extract code snippets from markdown content
    fn extract_code_snippets(content: &str) -> Vec<String> {
        let mut snippets = Vec::new();
        let mut in_code_block = false;
        let mut current_snippet = Vec::new();

        for line in content.lines() {
            if line.starts_with("```") {
                if in_code_block {
                    if !current_snippet.is_empty() {
                        snippets.push(current_snippet.join("\n"));
                    }
                    current_snippet.clear();
                }
                in_code_block = !in_code_block;
            } else if in_code_block {
                current_snippet.push(line.to_string());
            }
        }

        snippets
    }
}

#[async_trait]
impl IntegrationClient for Context7Client {
    fn provider(&self) -> &str {
        "context7"
    }

    fn is_ready(&self) -> bool {
        self.initialized
    }

    async fn initialize(&mut self) -> Result<bool> {
        if self.api_key.is_none() {
            tracing::warn!("Context7 API key not configured - client disabled");
            return Ok(false);
        }

        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            "application/json".parse().unwrap(),
        );

        if let Some(ref key) = self.api_key {
            headers.insert(
                reqwest::header::AUTHORIZATION,
                format!("Bearer {}", key).parse().unwrap(),
            );
        }

        self.client = Some(
            Client::builder()
                .default_headers(headers)
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .map_err(|e| IntelligenceError::Config(format!("HTTP client error: {}", e)))?,
        );

        self.initialized = true;
        tracing::info!("Context7 client initialized");
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
    fn test_common_library_mappings() {
        let mappings = Context7Client::common_library_mappings();
        assert_eq!(mappings.get("react"), Some(&"/facebook/react".to_string()));
        assert_eq!(mappings.get("axum"), Some(&"/tokio-rs/axum".to_string()));
        assert_eq!(mappings.get("tauri"), Some(&"/tauri-apps/tauri".to_string()));
    }

    #[test]
    fn test_extract_code_snippets() {
        let content = r#"
Here is some documentation.

```rust
fn main() {
    println!("Hello");
}
```

More text here.

```python
def hello():
    print("Hello")
```
"#;

        let snippets = Context7Client::extract_code_snippets(content);
        assert_eq!(snippets.len(), 2);
        assert!(snippets[0].contains("fn main"));
        assert!(snippets[1].contains("def hello"));
    }

    #[tokio::test]
    async fn test_resolve_library_from_cache() {
        let client = Context7Client::new(None);
        let result = client.resolve_library_id("react").await.unwrap();
        assert_eq!(result, Some("/facebook/react".to_string()));
    }
}
