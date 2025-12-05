//! Error types for the LLM module

use thiserror::Error;

/// Result type for LLM operations
pub type Result<T> = std::result::Result<T, LlmError>;

/// LLM-specific errors
#[derive(Error, Debug)]
pub enum LlmError {
    /// Model file not found
    #[error("Model not found: {0}")]
    ModelNotFound(String),

    /// Model loading failed
    #[error("Failed to load model: {0}")]
    ModelLoadError(String),

    /// No model currently loaded
    #[error("No model loaded")]
    NoModelLoaded,

    /// Context creation failed
    #[error("Failed to create context: {0}")]
    ContextError(String),

    /// Tokenization error
    #[error("Tokenization failed: {0}")]
    TokenizationError(String),

    /// Generation error
    #[error("Generation failed: {0}")]
    GenerationError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    ConfigError(String),

    /// Backend initialization error
    #[error("Backend initialization failed: {0}")]
    BackendError(String),

    /// Chat template error
    #[error("Chat template error: {0}")]
    ChatTemplateError(String),

    /// Sampling error
    #[error("Sampling error: {0}")]
    SamplingError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Channel error (for streaming)
    #[error("Channel error: {0}")]
    ChannelError(String),

    /// Session not found
    #[error("Session not found: {0}")]
    SessionNotFound(String),

    /// Model already loaded
    #[error("Model already loaded: {0}")]
    ModelAlreadyLoaded(String),
}
