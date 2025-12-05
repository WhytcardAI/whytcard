//! Main RAG engine.
//!
//! Combines chunker, embedder, and vector store for complete RAG pipeline.
//! Uses spawn_blocking for CPU-intensive embedding operations to avoid
//! blocking the async runtime.

use crate::chunker::{Chunker, ChunkingStrategy};
use crate::config::RagConfig;
use crate::embedder::Embedder;
use crate::error::{RagError, Result};
use crate::store::VectorStore;
use crate::types::{Document, SearchResult};
use std::sync::{Arc, Mutex};

/// Main RAG engine combining all components.
pub struct RagEngine {
    chunker: Chunker,
    embedder: Arc<Mutex<Embedder>>,
    store: VectorStore,
    config: RagConfig,
}

impl RagEngine {
    /// Create a new RAG engine with the given config.
    pub async fn new(config: RagConfig) -> Result<Self> {
        let chunker = Chunker::with_config(config.chunking.clone());
        let embedder = Embedder::with_model(config.embedding_model.clone())?;
        let store = VectorStore::open(config.clone()).await?;

        Ok(Self {
            chunker,
            embedder: Arc::new(Mutex::new(embedder)),
            store,
            config,
        })
    }

    /// Create engine with custom chunking strategy.
    pub async fn with_strategy(config: RagConfig, strategy: ChunkingStrategy) -> Result<Self> {
        let chunker = Chunker::with_config(config.chunking.clone()).with_strategy(strategy);
        let embedder = Embedder::with_model(config.embedding_model.clone())?;
        let store = VectorStore::open(config.clone()).await?;

        Ok(Self {
            chunker,
            embedder: Arc::new(Mutex::new(embedder)),
            store,
            config,
        })
    }

    /// Get the configuration.
    pub fn config(&self) -> &RagConfig {
        &self.config
    }

    /// Index a document.
    ///
    /// Chunks the document, generates embeddings, and stores in vector DB.
    /// Uses spawn_blocking for CPU-intensive embedding to avoid blocking async runtime.
    pub async fn index(&mut self, document: &Document) -> Result<usize> {
        // Chunk the document (fast, doesn't need spawn_blocking)
        let chunks = self.chunker.chunk(document)?;

        if chunks.is_empty() {
            return Ok(0);
        }

        // Generate embeddings in blocking task
        let embedder = Arc::clone(&self.embedder);
        let chunks_clone = chunks.clone();
        
        let chunks_with_embeddings = tokio::task::spawn_blocking(move || {
            let mut embedder = embedder.lock()
                .map_err(|_| RagError::Embedding("Failed to lock embedder".to_string()))?;
            embedder.embed_chunks(&chunks_clone)
        })
        .await
        .map_err(|e| RagError::Embedding(format!("Embedding task failed: {e}")))??;

        let count = chunks_with_embeddings.len();

        // Store in vector DB
        self.store.insert(chunks_with_embeddings).await?;

        Ok(count)
    }

    /// Index multiple documents.
    pub async fn index_many(&mut self, documents: &[Document]) -> Result<usize> {
        let mut total = 0;

        for doc in documents {
            total += self.index(doc).await?;
        }

        Ok(total)
    }

    /// Search for relevant chunks.
    /// Uses spawn_blocking for CPU-intensive embedding to avoid blocking async runtime.
    pub async fn search(&mut self, query: &str, limit: Option<usize>) -> Result<Vec<SearchResult>> {
        let embedder = Arc::clone(&self.embedder);
        let query_owned = query.to_string();
        
        let query_embedding = tokio::task::spawn_blocking(move || {
            let mut embedder = embedder.lock()
                .map_err(|_| RagError::Embedding("Failed to lock embedder".to_string()))?;
            embedder.embed_query(&query_owned)
        })
        .await
        .map_err(|e| RagError::Embedding(format!("Embedding task failed: {e}")))??;
        
        self.store.search(query_embedding, limit).await
    }

    /// Search and return only the text content.
    pub async fn search_text(&mut self, query: &str, limit: Option<usize>) -> Result<Vec<String>> {
        let results = self.search(query, limit).await?;
        Ok(results.into_iter().map(|r| r.chunk.text).collect())
    }

    /// Search and format as context for LLM.
    pub async fn search_context(
        &mut self,
        query: &str,
        limit: Option<usize>,
    ) -> Result<String> {
        let results = self.search(query, limit).await?;

        let context = results
            .iter()
            .enumerate()
            .map(|(i, r)| {
                format!(
                    "[{i}] (score: {:.3})\n{}\n",
                    r.score,
                    r.chunk.text.trim()
                )
            })
            .collect::<Vec<_>>()
            .join("\n---\n");

        Ok(context)
    }

    /// Delete a document and its chunks.
    pub async fn delete_document(&mut self, document_id: &str) -> Result<()> {
        self.store.delete_by_document(document_id).await
    }

    /// Get number of indexed chunks.
    pub async fn count(&self) -> Result<usize> {
        self.store.count().await
    }

    /// Reindex a document (delete old chunks, index new).
    pub async fn reindex(&mut self, document: &Document) -> Result<usize> {
        self.store.delete_by_document(&document.id).await?;
        self.index(document).await
    }
}

/// Builder for RagEngine with fluent API.
pub struct RagEngineBuilder {
    config: RagConfig,
    strategy: ChunkingStrategy,
}

impl RagEngineBuilder {
    /// Create builder with default config.
    pub fn new() -> Self {
        Self {
            config: RagConfig::default(),
            strategy: ChunkingStrategy::default(),
        }
    }

    /// Set the database path.
    pub fn db_path(mut self, path: impl Into<String>) -> Self {
        self.config.db_path = path.into();
        self
    }

    /// Set the table name.
    pub fn table_name(mut self, name: impl Into<String>) -> Self {
        self.config.table_name = name.into();
        self
    }

    /// Set the embedding model.
    pub fn embedding_model(mut self, model: crate::config::EmbeddingModel) -> Self {
        self.config.embedding_model = model;
        self
    }

    /// Alias for `embedding_model` for convenience.
    pub fn model(self, model: crate::config::EmbeddingModel) -> Self {
        self.embedding_model(model)
    }

    /// Set the chunking configuration.
    pub fn chunking_config(mut self, config: crate::config::ChunkingConfig) -> Self {
        self.config.chunking = config;
        self
    }

    /// Set the search configuration.
    pub fn search_config(mut self, config: crate::config::SearchConfig) -> Self {
        self.config.search = config;
        self
    }

    /// Set chunk size.
    pub fn chunk_size(mut self, size: usize) -> Self {
        self.config.chunking.chunk_size = size;
        self
    }

    /// Set chunk overlap.
    pub fn chunk_overlap(mut self, overlap: usize) -> Self {
        self.config.chunking.chunk_overlap = overlap;
        self
    }

    /// Set minimum chunk size.
    pub fn min_chunk_size(mut self, size: usize) -> Self {
        self.config.chunking.min_chunk_size = size;
        self
    }

    /// Set chunking strategy.
    pub fn chunking_strategy(mut self, strategy: ChunkingStrategy) -> Self {
        self.strategy = strategy;
        self
    }

    /// Set search limit.
    pub fn default_search_limit(mut self, limit: usize) -> Self {
        self.config.search.default_limit = limit;
        self
    }

    /// Set minimum score threshold.
    pub fn min_score(mut self, score: f32) -> Self {
        self.config.search.min_score = score;
        self
    }

    /// Build the engine.
    pub async fn build(self) -> Result<RagEngine> {
        RagEngine::with_strategy(self.config, self.strategy).await
    }
}

impl Default for RagEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_config() {
        let builder = RagEngineBuilder::new()
            .db_path("/tmp/test.lance")
            .table_name("test")
            .chunk_size(256)
            .chunk_overlap(32);

        assert_eq!(builder.config.db_path, "/tmp/test.lance");
        assert_eq!(builder.config.table_name, "test");
        assert_eq!(builder.config.chunking.chunk_size, 256);
        assert_eq!(builder.config.chunking.chunk_overlap, 32);
    }

    #[tokio::test]
    async fn test_engine_creation() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance").to_string_lossy().to_string();

        let engine = RagEngineBuilder::new()
            .db_path(db_path)
            .build()
            .await;

        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_index_and_search() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance").to_string_lossy().to_string();

        let mut engine = RagEngineBuilder::new()
            .db_path(db_path)
            .chunk_size(500)
            .min_chunk_size(10)
            .build()
            .await
            .unwrap();

        // Index a document with enough content
        let doc = Document::new("Rust is a systems programming language focused on safety, speed, and concurrency. It provides memory safety without garbage collection.");
        let count = engine.index(&doc).await.unwrap();
        assert!(count > 0);

        // Search
        let results = engine.search("programming language", Some(5)).await.unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_delete_document() {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance").to_string_lossy().to_string();

        let mut engine = RagEngineBuilder::new()
            .db_path(db_path)
            .min_chunk_size(10)
            .build()
            .await
            .unwrap();

        // Index with enough content
        let doc = Document::new("Test content for deletion that is long enough to be indexed properly.");
        engine.index(&doc).await.unwrap();

        let count_before = engine.count().await.unwrap();
        assert!(count_before > 0);

        // Delete
        engine.delete_document(&doc.id).await.unwrap();

        let count_after = engine.count().await.unwrap();
        assert_eq!(count_after, 0);
    }
}
