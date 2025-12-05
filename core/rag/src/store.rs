//! Vector store using SurrealDB via whytcard-database.
//!
//! Provides vector storage and semantic search using the unified
//! whytcard-database module with SurrealDB's HNSW index.

use crate::config::RagConfig;
use crate::error::{RagError, Result};
use crate::types::{Chunk, SearchResult};
use whytcard_database::{
    Config as DbConfig, CreateChunk as DbCreateChunk, Database, DatabaseError,
    DistanceMetric, StorageMode, VectorConfig,
};

/// Vector store backed by SurrealDB.
pub struct VectorStore {
    db: Database,
    config: RagConfig,
}

impl VectorStore {
    /// Open the vector store with the given configuration.
    pub async fn open(config: RagConfig) -> Result<Self> {
        // Determine storage mode
        let storage = if config.db_path.is_empty() || config.db_path == ":memory:" {
            StorageMode::Memory
        } else {
            StorageMode::Persistent(config.db_path.clone().into())
        };

        // Create database config with matching vector dimension
        let db_config = DbConfig {
            storage,
            namespace: "whytcard".into(),
            database: "rag".into(),
            vector_config: VectorConfig {
                dimension: config.embedding_model.dimensions(),
                distance: DistanceMetric::Cosine,
            },
        };

        let db = Database::new(db_config)
            .await
            .map_err(|e| RagError::VectorStore(format!("Failed to open database: {e}")))?;

        Ok(Self { db, config })
    }

    /// Insert chunks with their embeddings.
    pub async fn insert(&mut self, chunks_with_embeddings: Vec<(Chunk, Vec<f32>)>) -> Result<()> {
        if chunks_with_embeddings.is_empty() {
            return Ok(());
        }

        // First, ensure document exists for each unique document_id
        let mut doc_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
        for (chunk, _) in &chunks_with_embeddings {
            doc_ids.insert(chunk.document_id.clone());
        }

        // Create documents if they don't exist
        for doc_id in doc_ids {
            if self.db.get_document_by_key(&doc_id).await.map_err(db_err)?.is_none() {
                // Create a placeholder document
                let doc_input = whytcard_database::CreateDocument {
                    key: Some(doc_id.clone()),
                    content: String::new(), // Will be filled by the actual document
                    title: None,
                    tags: vec![],
                    metadata: None,
                };
                self.db.create_document(doc_input).await.map_err(db_err)?;
            }
        }

        // Now create chunks
        for (chunk, embedding) in chunks_with_embeddings {
            // Get the document record ID
            let doc = self.db.get_document_by_key(&chunk.document_id).await.map_err(db_err)?
                .ok_or_else(|| RagError::VectorStore(format!("Document not found: {}", chunk.document_id)))?;

            let doc_id = doc.id.ok_or_else(|| RagError::VectorStore("Document has no ID".into()))?;

            let db_chunk = DbCreateChunk::new(
                doc_id,
                chunk.text.clone(),
                embedding,
                chunk.index as i32,
            )
            .with_metadata(serde_json::json!({
                "start_char": chunk.start_char,
                "end_char": chunk.end_char,
                "token_count": chunk.token_count,
                "original_id": chunk.id,
            }));

            self.db.create_chunk(db_chunk).await.map_err(db_err)?;
        }

        Ok(())
    }

    /// Search for similar chunks using vector similarity.
    pub async fn search(
        &self,
        query_embedding: Vec<f32>,
        limit: Option<usize>,
    ) -> Result<Vec<SearchResult>> {
        let limit = limit
            .unwrap_or(self.config.search.default_limit)
            .min(self.config.search.max_limit);

        let db_results = self
            .db
            .search_vectors(&query_embedding, limit, None)
            .await
            .map_err(db_err)?;

        let min_score = self.config.search.min_score;

        let results: Vec<SearchResult> = db_results
            .into_iter()
            .filter_map(|r| {
                // Convert distance to similarity score (cosine distance: 0 = identical)
                let score = 1.0 - r.distance;
                if score < min_score {
                    return None;
                }

                // Extract metadata
                let metadata = r.metadata.clone();
                let start_char = metadata
                    .as_ref()
                    .and_then(|m| m.get("start_char"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;
                let end_char = metadata
                    .as_ref()
                    .and_then(|m| m.get("end_char"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;
                let original_id = metadata
                    .as_ref()
                    .and_then(|m| m.get("original_id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                // Get document_id from RecordId
                let document_id = r.document_id.key().to_string();

                let chunk = Chunk {
                    id: original_id,
                    document_id,
                    index: r.chunk_index as usize,
                    text: r.content,
                    start_char,
                    end_char,
                    token_count: (end_char - start_char) / 4, // Rough estimate
                    metadata,
                };

                Some(SearchResult {
                    chunk,
                    score,
                    distance: r.distance,
                })
            })
            .collect();

        Ok(results)
    }

    /// Delete all chunks for a document.
    pub async fn delete_by_document(&mut self, document_id: &str) -> Result<()> {
        // Find the document by key
        if let Some(doc) = self.db.get_document_by_key(document_id).await.map_err(db_err)? {
            if let Some(doc_record_id) = doc.id {
                self.db.delete_chunks_by_document(&doc_record_id).await.map_err(db_err)?;
                // Also delete the document itself
                let doc_key = doc_record_id.key().to_string();
                self.db.delete_document(&doc_key).await.map_err(db_err)?;
            }
        }
        Ok(())
    }

    /// Count total indexed chunks.
    pub async fn count(&self) -> Result<usize> {
        self.db.count_chunks().await.map_err(db_err)
    }

    /// Get database reference for advanced operations.
    pub fn database(&self) -> &Database {
        &self.db
    }
}

/// Convert DatabaseError to RagError.
fn db_err(e: DatabaseError) -> RagError {
    RagError::VectorStore(e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_store() -> VectorStore {
        let config = RagConfig {
            db_path: ":memory:".to_string(),
            ..Default::default()
        };
        VectorStore::open(config).await.unwrap()
    }

    #[tokio::test]
    async fn test_store_creation() {
        let store = create_test_store().await;
        let count = store.count().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_count_empty() {
        let store = create_test_store().await;
        let count = store.count().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_insert_and_count() {
        let mut store = create_test_store().await;

        let chunk = Chunk::new("doc1", 0, "Test content".to_string(), 0, 12);
        let embedding = vec![0.1_f32; 384]; // Match AllMiniLmL6V2 dimension

        store.insert(vec![(chunk, embedding)]).await.unwrap();

        let count = store.count().await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_insert_and_search() {
        let mut store = create_test_store().await;

        // Insert test chunk
        let chunk = Chunk::new("doc1", 0, "Hello world test".to_string(), 0, 16);
        let embedding = vec![0.5_f32; 384];

        store.insert(vec![(chunk, embedding.clone())]).await.unwrap();

        // Search with same embedding should find it
        let results = store.search(embedding, Some(10)).await.unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].chunk.text, "Hello world test");
    }

    #[tokio::test]
    async fn test_delete_by_document() {
        let mut store = create_test_store().await;

        // Insert chunks from two documents
        let chunk1 = Chunk::new("doc1", 0, "Content 1".to_string(), 0, 9);
        let chunk2 = Chunk::new("doc2", 0, "Content 2".to_string(), 0, 9);
        let embedding = vec![0.5_f32; 384];

        store
            .insert(vec![
                (chunk1, embedding.clone()),
                (chunk2, embedding.clone()),
            ])
            .await
            .unwrap();

        assert_eq!(store.count().await.unwrap(), 2);

        // Delete doc1
        store.delete_by_document("doc1").await.unwrap();

        assert_eq!(store.count().await.unwrap(), 1);
    }
}
