//! Semantic Memory - Facts, knowledge, and patterns
//!
//! Vector-based memory for semantic search and retrieval.
//! Stores facts, knowledge, API patterns, and definitions.

use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use whytcard_database::{Config as DbConfig, Database, StorageMode, VectorConfig};
use whytcard_rag::{Document, RagEngine, RagEngineBuilder};

/// Semantic memory for vector-based knowledge storage
pub struct SemanticMemory {
    /// Database connection
    db: Database,

    /// RAG engine for embeddings and search
    rag: RagEngine,

    /// Whether initialized
    initialized: bool,
}

impl SemanticMemory {
    /// Create new semantic memory at the given path
    pub async fn new(base_path: &Path) -> Result<Self> {
        // Use separate subdirectories for db and rag to avoid lock conflicts
        let db_path = base_path.join("db");
        let rag_path = base_path.join("rag");

        std::fs::create_dir_all(&db_path)?;
        std::fs::create_dir_all(&rag_path)?;

        let db_config = DbConfig {
            storage: StorageMode::Persistent(db_path),
            namespace: "whytcard".into(),
            database: "semantic".into(),
            vector_config: VectorConfig {
                dimension: 384,
                distance: whytcard_database::DistanceMetric::Cosine,
            },
        };

        let db = Database::new(db_config).await?;

        let rag = RagEngineBuilder::new()
            .db_path(rag_path.to_str().unwrap_or("semantic_rag"))
            .build()
            .await?;

        Ok(Self {
            db,
            rag,
            initialized: true,
        })
    }

    /// Create in-memory semantic memory for testing
    #[cfg(test)]
    pub async fn in_memory() -> Result<Self> {
        let db_config = DbConfig {
            storage: StorageMode::Memory,
            namespace: "whytcard".into(),
            database: "semantic_test".into(),
            vector_config: VectorConfig::default(),
        };

        let db = Database::new(db_config).await?;

        let rag = RagEngineBuilder::new()
            .db_path(":memory:")
            .min_chunk_size(10)
            .build()
            .await?;

        Ok(Self {
            db,
            rag,
            initialized: true,
        })
    }

    /// Store a fact/knowledge in semantic memory
    pub async fn store(&mut self, fact: SemanticFact) -> Result<String> {
        let id = fact.id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        // Index in RAG for vector search
        let doc = Document::new(fact.content.clone())
            .with_id(id.clone())
            .with_metadata(serde_json::json!({
                "source": fact.source,
                "category": fact.category,
            }));
        self.rag.index(&doc).await?;

        // Store metadata in database
        let doc_input = whytcard_database::CreateDocument::new(&fact.content)
            .with_key(&id)
            .with_metadata(serde_json::json!({
                "source": fact.source,
                "category": fact.category,
                "relevance_score": fact.relevance_score,
            }))
            .with_tags(fact.tags);

        self.db.create_document(doc_input).await?;

        tracing::debug!("Stored semantic fact: {}", id);
        Ok(id)
    }

    /// Search semantic memory by query
    pub async fn search(&mut self, query: &str, top_k: usize, min_score: Option<f32>) -> Result<Vec<SemanticSearchResult>> {
        let results = self.rag.search(query, Some(top_k)).await?;

        let min_score = min_score.unwrap_or(0.0);

        Ok(results
            .into_iter()
            .filter(|r| r.score >= min_score)
            .map(|r| SemanticSearchResult {
                id: r.chunk.id.clone(),
                content: r.chunk.text.clone(),
                score: r.score,
                source: None,
                category: None,
            })
            .collect())
    }

    /// Get a specific fact by ID
    pub async fn get(&self, id: &str) -> Result<Option<SemanticFact>> {
        match self.db.get_document_by_key(id).await? {
            Some(doc) => {
                let metadata = doc.metadata.unwrap_or_default();
                Ok(Some(SemanticFact {
                    id: Some(doc.key.unwrap_or_default()),
                    content: doc.content,
                    source: metadata.get("source").and_then(|v| v.as_str()).map(String::from),
                    category: metadata.get("category").and_then(|v| v.as_str()).map(String::from).unwrap_or_default(),
                    tags: doc.tags,
                    relevance_score: metadata.get("relevance_score").and_then(|v| v.as_f64()).unwrap_or(1.0) as f32,
                }))
            }
            None => Ok(None),
        }
    }

    /// Delete a fact by ID
    pub async fn delete(&mut self, id: &str) -> Result<bool> {
        self.db.delete_document_by_key(id).await?;
        self.rag.delete_document(id).await?;
        Ok(true)
    }

    /// Get statistics
    pub async fn get_stats(&self) -> SemanticStats {
        let count = self.db.count_documents().await.unwrap_or(0);

        SemanticStats {
            total_facts: count,
            initialized: self.initialized,
        }
    }
}

/// A semantic fact/knowledge entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticFact {
    /// Unique ID (auto-generated if not provided)
    pub id: Option<String>,

    /// Content of the fact
    pub content: String,

    /// Source of the knowledge
    pub source: Option<String>,

    /// Category for filtering
    pub category: String,

    /// Tags for organization
    pub tags: Vec<String>,

    /// Relevance score (decays if not used)
    pub relevance_score: f32,
}

impl SemanticFact {
    pub fn new(content: impl Into<String>, category: impl Into<String>) -> Self {
        Self {
            id: None,
            content: content.into(),
            source: None,
            category: category.into(),
            tags: Vec::new(),
            relevance_score: 1.0,
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// Search result from semantic memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticSearchResult {
    pub id: String,
    pub content: String,
    pub score: f32,
    pub source: Option<String>,
    pub category: Option<String>,
}

/// Statistics for semantic memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticStats {
    pub total_facts: usize,
    pub initialized: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_semantic_memory() {
        let mem = SemanticMemory::in_memory().await.unwrap();
        assert!(mem.initialized);
    }
}
