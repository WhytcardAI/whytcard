//! WhytCard Database Layer
//!
//! Provides SurrealDB database initialization and operations for WhytCard.
//! Supports documents, vectors (HNSW), and knowledge graph (relations).
//!
//! # Example
//!
//! ```rust,no_run
//! use whytcard_database::{Database, Config};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // In-memory database
//!     let db = Database::new_memory().await?;
//!
//!     // Or persistent database
//!     let db = Database::new_persistent("./data").await?;
//!
//!     Ok(())
//! }
//! ```

mod config;
mod database;
mod error;
mod schema;

pub mod documents;
pub mod graph;
pub mod vectors;

pub use config::{Config, DistanceMetric, StorageMode, VectorConfig};
pub use database::Database;
pub use error::{DatabaseError, Result};
pub use schema::Schema;

// Re-export document types
pub use documents::{CreateDocument, Document};

// Re-export vector types
pub use vectors::{Chunk, CreateChunk, SearchResult as VectorSearchResult};

// Re-export graph types
pub use graph::{
    CreateEntity, CreateRelation, Entity, EntityWithRelations, RelatedEntity, Relation,
    RelationDirection,
};

/// Re-export SurrealDB types for convenience
pub use surrealdb::RecordId;
