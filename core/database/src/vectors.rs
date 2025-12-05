//! Vector operations for semantic search

use crate::{Database, DatabaseError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

/// Chunk record with embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Record ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,

    /// Reference to parent document
    pub document_id: RecordId,

    /// Chunk content
    pub content: String,

    /// Vector embedding
    pub embedding: Vec<f32>,

    /// Index of chunk in document
    pub chunk_index: i32,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,

    /// Creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

/// Input for creating a chunk
#[derive(Debug, Clone, Serialize)]
pub struct CreateChunk {
    /// Reference to parent document
    pub document_id: RecordId,

    /// Chunk content
    pub content: String,

    /// Vector embedding
    pub embedding: Vec<f32>,

    /// Index of chunk in document
    pub chunk_index: i32,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl CreateChunk {
    /// Create a new chunk input
    pub fn new(
        document_id: RecordId,
        content: impl Into<String>,
        embedding: Vec<f32>,
        chunk_index: i32,
    ) -> Self {
        Self {
            document_id,
            content: content.into(),
            embedding,
            chunk_index,
            metadata: None,
        }
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Vector search result
#[derive(Debug, Clone, Deserialize)]
pub struct SearchResult {
    /// Chunk ID
    pub id: RecordId,

    /// Document ID
    pub document_id: RecordId,

    /// Chunk content
    pub content: String,

    /// Chunk index
    pub chunk_index: i32,

    /// Similarity distance (lower is better for distance, higher for similarity)
    pub distance: f32,

    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

/// Vector operations
impl Database {
    /// Create a new chunk with embedding
    pub async fn create_chunk(&self, input: CreateChunk) -> Result<Chunk> {
        // Validate embedding dimension
        let expected_dim = self.config().vector_config.dimension;
        if input.embedding.len() != expected_dim {
            return Err(DatabaseError::DimensionMismatch {
                expected: expected_dim,
                got: input.embedding.len(),
            });
        }

        let chunk: Option<Chunk> = self.inner().create("chunk").content(input).await?;
        chunk.ok_or_else(|| DatabaseError::Schema("Failed to create chunk".into()))
    }

    /// Create multiple chunks for a document
    pub async fn create_chunks(&self, inputs: Vec<CreateChunk>) -> Result<Vec<Chunk>> {
        let mut chunks = Vec::with_capacity(inputs.len());
        for input in inputs {
            chunks.push(self.create_chunk(input).await?);
        }
        Ok(chunks)
    }

    /// Search for similar chunks using vector similarity
    pub async fn search_vectors(
        &self,
        query_embedding: &[f32],
        limit: usize,
        min_score: Option<f32>,
    ) -> Result<Vec<SearchResult>> {
        // Validate embedding dimension
        let expected_dim = self.config().vector_config.dimension;
        if query_embedding.len() != expected_dim {
            return Err(DatabaseError::DimensionMismatch {
                expected: expected_dim,
                got: query_embedding.len(),
            });
        }

        let distance = self.config().vector_config.distance.as_surreal_str();

        // Build query with KNN operator
        let query = format!(
            r#"
            SELECT
                id,
                document_id,
                content,
                chunk_index,
                metadata,
                vector::distance::knn() AS distance
            FROM chunk
            WHERE embedding <|{limit},{distance}|> $embedding
            ORDER BY distance
            "#
        );

        let mut result = self
            .inner()
            .query(&query)
            .bind(("embedding", query_embedding.to_vec()))
            .await?;

        let mut results: Vec<SearchResult> = result.take(0)?;

        // Filter by minimum score if specified
        if let Some(min) = min_score {
            results.retain(|r| {
                // For cosine, distance is 1 - similarity, so lower is better
                // Convert to similarity score
                let similarity = 1.0 - r.distance;
                similarity >= min
            });
        }

        Ok(results)
    }

    /// Get all chunks for a document
    pub async fn get_chunks_by_document(&self, document_id: &RecordId) -> Result<Vec<Chunk>> {
        let doc_key = document_id.key().to_string();
        let mut result = self
            .inner()
            .query("SELECT * FROM chunk WHERE document_id = type::thing('document', $doc_key) ORDER BY chunk_index")
            .bind(("doc_key", doc_key))
            .await?;

        let chunks: Vec<Chunk> = result.take(0)?;
        Ok(chunks)
    }

    /// Delete all chunks for a document
    pub async fn delete_chunks_by_document(&self, document_id: &RecordId) -> Result<usize> {
        let doc_key = document_id.key().to_string();
        let mut result = self
            .inner()
            .query("DELETE chunk WHERE document_id = type::thing('document', $doc_key) RETURN BEFORE")
            .bind(("doc_key", doc_key))
            .await?;

        let deleted: Vec<Chunk> = result.take(0)?;
        Ok(deleted.len())
    }

    /// Count chunks
    pub async fn count_chunks(&self) -> Result<usize> {
        let mut result = self
            .inner()
            .query("SELECT count() FROM chunk GROUP ALL")
            .await?;

        #[derive(Deserialize)]
        struct CountResult {
            count: usize,
        }

        let counts: Vec<CountResult> = result.take(0)?;
        Ok(counts.first().map(|c| c.count).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::documents::CreateDocument;

    fn make_embedding(seed: f32) -> Vec<f32> {
        // Create a 384-dimensional embedding with some variation
        (0..384).map(|i| (seed + i as f32 * 0.001).sin()).collect()
    }

    #[tokio::test]
    async fn test_create_and_search_chunks() {
        let db = Database::new_memory().await.unwrap();

        // Create a document first
        let doc = db
            .create_document(CreateDocument::new("Parent document"))
            .await
            .unwrap();
        let doc_id = doc.id.unwrap();

        // Create chunks with embeddings
        let chunks_data = vec![
            ("Rust programming language", 1.0),
            ("Python for data science", 2.0),
            ("JavaScript web development", 3.0),
        ];

        for (i, (content, seed)) in chunks_data.iter().enumerate() {
            let input = CreateChunk::new(doc_id.clone(), *content, make_embedding(*seed), i as i32);
            db.create_chunk(input).await.unwrap();
        }

        // Search with a query embedding similar to the first chunk
        let query_embedding = make_embedding(1.05); // Similar to "Rust programming"
        let results = db.search_vectors(&query_embedding, 3, None).await.unwrap();

        assert!(!results.is_empty());
        // First result should be closest to our query
        assert!(results[0].content.contains("Rust"));
    }

    #[tokio::test]
    async fn test_dimension_validation() {
        let db = Database::new_memory().await.unwrap();

        let doc = db
            .create_document(CreateDocument::new("Test"))
            .await
            .unwrap();
        let doc_id = doc.id.unwrap();

        // Try to create chunk with wrong dimension
        let wrong_dim_embedding = vec![0.1; 100]; // Should be 384
        let input = CreateChunk::new(doc_id, "Content", wrong_dim_embedding, 0);

        let result = db.create_chunk(input).await;
        assert!(matches!(result, Err(DatabaseError::DimensionMismatch { .. })));
    }

    #[tokio::test]
    async fn test_get_chunks_by_document() {
        let db = Database::new_memory().await.unwrap();

        let doc = db
            .create_document(CreateDocument::new("Test doc"))
            .await
            .unwrap();
        let doc_id = doc.id.unwrap();

        // Create 3 chunks
        for i in 0..3 {
            let input = CreateChunk::new(
                doc_id.clone(),
                format!("Chunk {}", i),
                make_embedding(i as f32),
                i,
            );
            db.create_chunk(input).await.unwrap();
        }

        let chunks = db.get_chunks_by_document(&doc_id).await.unwrap();
        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].chunk_index, 0);
        assert_eq!(chunks[1].chunk_index, 1);
        assert_eq!(chunks[2].chunk_index, 2);
    }

    #[tokio::test]
    async fn test_delete_chunks_by_document() {
        let db = Database::new_memory().await.unwrap();

        let doc = db
            .create_document(CreateDocument::new("Test"))
            .await
            .unwrap();
        let doc_id = doc.id.unwrap();

        for i in 0..3 {
            let input = CreateChunk::new(
                doc_id.clone(),
                format!("Chunk {}", i),
                make_embedding(i as f32),
                i,
            );
            db.create_chunk(input).await.unwrap();
        }

        let deleted = db.delete_chunks_by_document(&doc_id).await.unwrap();
        assert_eq!(deleted, 3);

        let remaining = db.get_chunks_by_document(&doc_id).await.unwrap();
        assert!(remaining.is_empty());
    }
}
