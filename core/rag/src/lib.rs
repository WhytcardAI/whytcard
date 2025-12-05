//! WhytCard RAG Module
//!
//! Provides Retrieval-Augmented Generation capabilities:
//! - Document chunking
//! - Text embeddings via fastembed
//! - Vector storage via SurrealDB (unified whytcard-database)
//! - Semantic search
//!
//! # Architecture
//!
//! ```text
//! Document -> Chunker -> Embedder -> SurrealDB (HNSW)
//!                                       |
//! Query -> Embedder -> Search -----------+-> Results
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use whytcard_rag::{RagEngine, RagEngineBuilder, Document};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut engine = RagEngineBuilder::new()
//!         .db_path(":memory:") // In-memory for testing
//!         .build()
//!         .await?;
//!
//!     // Index a document
//!     let doc = Document::new("Rust is a systems programming language.");
//!     engine.index(&doc).await?;
//!
//!     // Search
//!     let results = engine.search("programming", Some(5)).await?;
//!     for result in results {
//!         println!("{}: {}", result.score, result.chunk.text);
//!     }
//!
//!     Ok(())
//! }
//! ```

mod chunker;
mod config;
mod embedder;
mod engine;
mod error;
mod store;
mod types;

pub use chunker::{Chunker, ChunkingStrategy};
pub use config::{ChunkingConfig, EmbeddingModel, RagConfig, SearchConfig};
pub use embedder::Embedder;
pub use engine::{RagEngine, RagEngineBuilder};
pub use error::{RagError, Result};
pub use store::VectorStore;
pub use types::{Chunk, Document, SearchResult};
