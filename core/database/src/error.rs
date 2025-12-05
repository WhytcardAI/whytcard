//! Database error types

use thiserror::Error;

/// Database error type
#[derive(Error, Debug)]
pub enum DatabaseError {
    /// SurrealDB error
    #[error("SurrealDB error: {0}")]
    Surreal(#[from] surrealdb::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Record not found
    #[error("Record not found: {table}:{id}")]
    NotFound { table: String, id: String },

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Schema error
    #[error("Schema error: {0}")]
    Schema(String),

    /// Vector dimension mismatch
    #[error("Vector dimension mismatch: expected {expected}, got {got}")]
    DimensionMismatch { expected: usize, got: usize },

    /// Relation error
    #[error("Relation error: {0}")]
    Relation(String),
}

/// Result type alias
pub type Result<T> = std::result::Result<T, DatabaseError>;
