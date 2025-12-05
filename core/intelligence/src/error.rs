//! Error types for WhytCard Intelligence

use thiserror::Error;

/// Result type alias for Intelligence operations
pub type Result<T> = std::result::Result<T, IntelligenceError>;

/// Errors that can occur in the Intelligence module
#[derive(Error, Debug)]
pub enum IntelligenceError {
    /// Database error (boxed to reduce enum size)
    #[error("Database error: {0}")]
    Database(Box<whytcard_database::DatabaseError>),

    /// RAG error (boxed to reduce enum size)
    #[error("RAG error: {0}")]
    Rag(Box<whytcard_rag::RagError>),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Path error
    #[error("Path error: {0}")]
    Path(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Entity not found
    #[error("Entity not found: {0}")]
    EntityNotFound(String),

    /// Relation not found
    #[error("Relation not found: from={from}, to={to}, type={relation_type}")]
    RelationNotFound {
        from: String,
        to: String,
        relation_type: String,
    },

    /// Memory key not found
    #[error("Memory key not found: {0}")]
    KeyNotFound(String),

    /// Invalid operation
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

// Manual From implementations for boxed error types
impl From<whytcard_database::DatabaseError> for IntelligenceError {
    fn from(err: whytcard_database::DatabaseError) -> Self {
        Self::Database(Box::new(err))
    }
}

impl From<whytcard_rag::RagError> for IntelligenceError {
    fn from(err: whytcard_rag::RagError) -> Self {
        Self::Rag(Box::new(err))
    }
}

impl IntelligenceError {
    /// Create a config error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config(msg.into())
    }

    /// Create a path error
    pub fn path(msg: impl Into<String>) -> Self {
        Self::Path(msg.into())
    }

    /// Create an invalid operation error
    pub fn invalid_operation(msg: impl Into<String>) -> Self {
        Self::InvalidOperation(msg.into())
    }
}

impl From<IntelligenceError> for rmcp::ErrorData {
    fn from(err: IntelligenceError) -> Self {
        use rmcp::model::ErrorCode;

        match &err {
            IntelligenceError::EntityNotFound(_) | IntelligenceError::KeyNotFound(_) => {
                rmcp::ErrorData::new(ErrorCode(-32001), err.to_string(), None)
            }
            IntelligenceError::RelationNotFound { .. } => {
                rmcp::ErrorData::new(ErrorCode(-32001), err.to_string(), None)
            }
            IntelligenceError::InvalidOperation(_) => {
                rmcp::ErrorData::new(ErrorCode(-32002), err.to_string(), None)
            }
            _ => rmcp::ErrorData::new(ErrorCode(-32603), err.to_string(), None),
        }
    }
}
