//! WhytCard LLM - Local LLM inference engine
//!
//! This crate provides a high-level interface for running LLMs locally
//! using llama.cpp bindings via llama-cpp-2.
//!
//! # Features
//!
//! - Model management (load/unload GGUF models)
//! - Chat sessions with history
//! - Token streaming
//! - GPU acceleration (CUDA/Metal)
//! - Sampling strategies
//!
//! # Example
//!
//! ```rust,ignore
//! use whytcard_llm::{LlmEngine, ChatMessage, GenerationConfig};
//!
//! let engine = LlmEngine::new()?;
//! engine.load_model("path/to/model.gguf", None)?;
//!
//! let response = engine.generate("Hello!", &GenerationConfig::default())?;
//! println!("{}", response);
//! ```

pub mod config;
pub mod engine;
pub mod error;
pub mod model;
pub mod session;
pub mod sampling;
pub mod streaming;

pub use config::{LlmConfig, ModelConfig, GenerationConfig};
pub use engine::LlmEngine;
pub use error::{LlmError, Result};
pub use model::{LoadedModel, ModelInfo, ModelManager};
pub use session::{ChatSession, ChatMessage, MessageRole};
pub use sampling::SamplingStrategy;
pub use streaming::{TokenStream, StreamEvent};
