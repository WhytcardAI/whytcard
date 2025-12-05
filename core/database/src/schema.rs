//! Database schema definitions

use crate::{Config, Result};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;

/// Database schema manager
pub struct Schema;

impl Schema {
    /// Initialize all schemas
    pub async fn init(db: &Surreal<Db>, config: &Config) -> Result<()> {
        Self::init_documents(db).await?;
        Self::init_vectors(db, config).await?;
        Self::init_graph(db).await?;
        Ok(())
    }

    /// Initialize document tables
    async fn init_documents(db: &Surreal<Db>) -> Result<()> {
        db.query(
            r#"
            -- Document table for storing any content
            DEFINE TABLE document SCHEMAFULL;
            DEFINE FIELD key ON document TYPE option<string>;
            DEFINE FIELD content ON document TYPE string;
            DEFINE FIELD title ON document TYPE option<string>;
            DEFINE FIELD tags ON document TYPE array<string> DEFAULT [];
            DEFINE FIELD metadata ON document TYPE option<object>;
            DEFINE FIELD created_at ON document TYPE datetime DEFAULT time::now();
            DEFINE FIELD updated_at ON document TYPE datetime DEFAULT time::now();

            -- Unique index on key if provided
            DEFINE INDEX idx_document_key ON document FIELDS key UNIQUE;

            -- Index for tag filtering
            DEFINE INDEX idx_document_tags ON document FIELDS tags;
            "#,
        )
        .await?;

        tracing::info!("Document schema initialized");
        Ok(())
    }

    /// Initialize vector tables with HNSW index
    async fn init_vectors(db: &Surreal<Db>, config: &Config) -> Result<()> {
        let dimension = config.vector_config.dimension;
        let distance = config.vector_config.distance.as_surreal_str();

        db.query(format!(
            r#"
            -- Chunk table for storing text chunks with embeddings
            DEFINE TABLE chunk SCHEMAFULL;
            DEFINE FIELD document_id ON chunk TYPE record<document>;
            DEFINE FIELD content ON chunk TYPE string;
            DEFINE FIELD embedding ON chunk TYPE array<float>;
            DEFINE FIELD chunk_index ON chunk TYPE int;
            DEFINE FIELD metadata ON chunk TYPE option<object>;
            DEFINE FIELD created_at ON chunk TYPE datetime DEFAULT time::now();

            -- HNSW vector index for semantic search
            DEFINE INDEX idx_chunk_embedding ON chunk FIELDS embedding
                HNSW DIMENSION {dimension} DIST {distance};

            -- Index for document lookup
            DEFINE INDEX idx_chunk_document ON chunk FIELDS document_id;
            "#
        ))
        .await?;

        tracing::info!(
            "Vector schema initialized (dimension={}, distance={})",
            dimension,
            distance
        );
        Ok(())
    }

    /// Initialize knowledge graph tables
    async fn init_graph(db: &Surreal<Db>) -> Result<()> {
        db.query(
            r#"
            -- Entity table for knowledge graph nodes
            DEFINE TABLE entity SCHEMAFULL;
            DEFINE FIELD name ON entity TYPE string;
            DEFINE FIELD entity_type ON entity TYPE string;
            DEFINE FIELD observations ON entity TYPE array<string> DEFAULT [];
            DEFINE FIELD metadata ON entity TYPE option<object>;
            DEFINE FIELD created_at ON entity TYPE datetime DEFAULT time::now();
            DEFINE FIELD updated_at ON entity TYPE datetime DEFAULT time::now();

            -- Unique index on name + type combination
            DEFINE INDEX idx_entity_name_type ON entity FIELDS name, entity_type UNIQUE;

            -- Index for type filtering
            DEFINE INDEX idx_entity_type ON entity FIELDS entity_type;

            -- Relation table for graph edges (using SurrealDB graph feature)
            DEFINE TABLE relates_to SCHEMAFULL TYPE RELATION IN entity OUT entity;
            DEFINE FIELD relation_type ON relates_to TYPE string;
            DEFINE FIELD weight ON relates_to TYPE option<float>;
            DEFINE FIELD metadata ON relates_to TYPE option<object>;
            DEFINE FIELD created_at ON relates_to TYPE datetime DEFAULT time::now();

            -- Index for relation type
            DEFINE INDEX idx_relation_type ON relates_to FIELDS relation_type;
            "#,
        )
        .await?;

        tracing::info!("Knowledge graph schema initialized");
        Ok(())
    }
}
