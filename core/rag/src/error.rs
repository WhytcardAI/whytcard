//! Error types for the RAG module.

use thiserror::Error;

/// RAG module error types.
#[derive(Debug, Error)]
pub enum RagError {
    /// Embedding generation failed
    #[error("Embedding error: {0}")]
    Embedding(String),

    /// Vector store operation failed
    #[error("Vector store error: {0}")]
    VectorStore(String),

    /// File I/O error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Chunking error
    #[error("Chunking error: {0}")]
    Chunking(String),

    /// Document not found
    #[error("Document not found: {0}")]
    NotFound(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    Config(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

/// Result type alias for RAG operations.
pub type Result<T> = std::result::Result<T, RagError>;
