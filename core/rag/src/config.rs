//! Configuration for the RAG module.

use serde::{Deserialize, Serialize};

/// RAG engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagConfig {
    /// Path to vector database
    pub db_path: String,
    /// Table name for chunks
    pub table_name: String,
    /// Embedding model name
    pub embedding_model: EmbeddingModel,
    /// Chunking configuration
    pub chunking: ChunkingConfig,
    /// Search configuration
    pub search: SearchConfig,
}

impl Default for RagConfig {
    fn default() -> Self {
        Self {
            db_path: "data/vectors".to_string(),
            table_name: "chunks".to_string(),
            embedding_model: EmbeddingModel::default(),
            chunking: ChunkingConfig::default(),
            search: SearchConfig::default(),
        }
    }
}

impl RagConfig {
    /// Create config with custom database path.
    pub fn with_db_path(mut self, path: impl Into<String>) -> Self {
        self.db_path = path.into();
        self
    }

    /// Create config with custom table name.
    pub fn with_table_name(mut self, name: impl Into<String>) -> Self {
        self.table_name = name.into();
        self
    }

    /// Create config with custom embedding model.
    pub fn with_embedding_model(mut self, model: EmbeddingModel) -> Self {
        self.embedding_model = model;
        self
    }
}

/// Embedding model selection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmbeddingModel {
    /// all-MiniLM-L6-v2 (384 dimensions, fast)
    AllMiniLmL6V2,
    /// BGE-small-en-v1.5 (384 dimensions, quality)
    BgeSmallEnV15,
    /// BGE-base-en-v1.5 (768 dimensions, high quality)
    BgeBaseEnV15,
}

impl Default for EmbeddingModel {
    fn default() -> Self {
        Self::AllMiniLmL6V2
    }
}

impl EmbeddingModel {
    /// Get embedding dimension for the model.
    pub fn dimensions(&self) -> usize {
        match self {
            Self::AllMiniLmL6V2 => 384,
            Self::BgeSmallEnV15 => 384,
            Self::BgeBaseEnV15 => 768,
        }
    }

    /// Get fastembed model name.
    pub fn fastembed_name(&self) -> &'static str {
        match self {
            Self::AllMiniLmL6V2 => "sentence-transformers/all-MiniLM-L6-v2",
            Self::BgeSmallEnV15 => "BAAI/bge-small-en-v1.5",
            Self::BgeBaseEnV15 => "BAAI/bge-base-en-v1.5",
        }
    }
}

/// Chunking configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfig {
    /// Target chunk size in characters
    pub chunk_size: usize,
    /// Overlap between chunks in characters
    pub chunk_overlap: usize,
    /// Minimum chunk size (smaller chunks are merged)
    pub min_chunk_size: usize,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            chunk_size: 512,
            chunk_overlap: 50,
            min_chunk_size: 100,
        }
    }
}

/// Search configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Default number of results
    pub default_limit: usize,
    /// Maximum number of results
    pub max_limit: usize,
    /// Minimum similarity score (0.0 - 1.0)
    pub min_score: f32,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            default_limit: 5,
            max_limit: 50,
            min_score: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RagConfig::default();
        assert_eq!(config.db_path, "data/vectors");
        assert_eq!(config.table_name, "chunks");
        assert_eq!(config.embedding_model.dimensions(), 384);
    }

    #[test]
    fn test_config_builder() {
        let config = RagConfig::default()
            .with_db_path("/custom/path")
            .with_table_name("my_chunks")
            .with_embedding_model(EmbeddingModel::BgeBaseEnV15);

        assert_eq!(config.db_path, "/custom/path");
        assert_eq!(config.table_name, "my_chunks");
        assert_eq!(config.embedding_model.dimensions(), 768);
    }

    #[test]
    fn test_embedding_model_dimensions() {
        assert_eq!(EmbeddingModel::AllMiniLmL6V2.dimensions(), 384);
        assert_eq!(EmbeddingModel::BgeSmallEnV15.dimensions(), 384);
        assert_eq!(EmbeddingModel::BgeBaseEnV15.dimensions(), 768);
    }
}
