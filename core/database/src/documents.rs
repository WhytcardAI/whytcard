//! Document operations

use crate::{Database, DatabaseError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

/// Document record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Record ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,

    /// Optional unique key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// Document content
    pub content: String,

    /// Optional title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Tags for filtering
    #[serde(default)]
    pub tags: Vec<String>,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,

    /// Creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,

    /// Last update timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Input for creating a document
#[derive(Debug, Clone, Serialize)]
pub struct CreateDocument {
    /// Optional unique key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,

    /// Document content
    pub content: String,

    /// Optional title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Tags for filtering
    #[serde(default)]
    pub tags: Vec<String>,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl CreateDocument {
    /// Create a new document input
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            key: None,
            content: content.into(),
            title: None,
            tags: Vec::new(),
            metadata: None,
        }
    }

    /// Set the key
    pub fn with_key(mut self, key: impl Into<String>) -> Self {
        self.key = Some(key.into());
        self
    }

    /// Set the title
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Add a tag
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Set tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Document operations
impl Database {
    /// Create a new document
    pub async fn create_document(&self, input: CreateDocument) -> Result<Document> {
        let doc: Option<Document> = self.inner().create("document").content(input).await?;
        doc.ok_or_else(|| DatabaseError::Schema("Failed to create document".into()))
    }

    /// Get a document by ID
    pub async fn get_document(&self, id: &str) -> Result<Option<Document>> {
        let doc: Option<Document> = self.inner().select(("document", id)).await?;
        Ok(doc)
    }

    /// Get a document by key
    pub async fn get_document_by_key(&self, key: &str) -> Result<Option<Document>> {
        let key_owned = key.to_string();
        let mut result = self
            .inner()
            .query("SELECT * FROM document WHERE key = $key LIMIT 1")
            .bind(("key", key_owned))
            .await?;

        let docs: Vec<Document> = result.take(0)?;
        Ok(docs.into_iter().next())
    }

    /// Update a document
    pub async fn update_document(&self, id: &str, input: CreateDocument) -> Result<Document> {
        let doc: Option<Document> = self
            .inner()
            .update(("document", id))
            .merge(serde_json::json!({
                "content": input.content,
                "title": input.title,
                "tags": input.tags,
                "metadata": input.metadata,
                "updated_at": Utc::now(),
            }))
            .await?;

        doc.ok_or_else(|| DatabaseError::NotFound {
            table: "document".into(),
            id: id.into(),
        })
    }

    /// Delete a document
    pub async fn delete_document(&self, id: &str) -> Result<bool> {
        let doc: Option<Document> = self.inner().delete(("document", id)).await?;
        Ok(doc.is_some())
    }

    /// Delete a document by key
    pub async fn delete_document_by_key(&self, key: &str) -> Result<bool> {
        let key_owned = key.to_string();
        let mut result = self
            .inner()
            .query("DELETE FROM document WHERE key = $key RETURN BEFORE")
            .bind(("key", key_owned))
            .await?;

        let docs: Vec<Document> = result.take(0)?;
        Ok(!docs.is_empty())
    }

    /// List documents with optional tag filter
    pub async fn list_documents(
        &self,
        tags: Option<&[String]>,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Document>> {
        let query = match tags {
            Some(tags) if !tags.is_empty() => {
                let tags_json = serde_json::to_string(tags)?;
                format!(
                    "SELECT * FROM document WHERE tags CONTAINSANY {} ORDER BY created_at DESC LIMIT {} START {}",
                    tags_json, limit, offset
                )
            }
            _ => format!(
                "SELECT * FROM document ORDER BY created_at DESC LIMIT {} START {}",
                limit, offset
            ),
        };

        let mut result = self.inner().query(&query).await?;
        let docs: Vec<Document> = result.take(0)?;
        Ok(docs)
    }

    /// Count documents
    pub async fn count_documents(&self) -> Result<usize> {
        let mut result = self
            .inner()
            .query("SELECT count() FROM document GROUP ALL")
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

    #[tokio::test]
    async fn test_create_and_get_document() {
        let db = Database::new_memory().await.unwrap();

        let input = CreateDocument::new("Test content")
            .with_title("Test Title")
            .with_tag("test");

        let doc = db.create_document(input).await.unwrap();
        assert!(doc.id.is_some());
        assert_eq!(doc.content, "Test content");
        assert_eq!(doc.title, Some("Test Title".to_string()));
        assert_eq!(doc.tags, vec!["test"]);
    }

    #[tokio::test]
    async fn test_get_document_by_key() {
        let db = Database::new_memory().await.unwrap();

        let input = CreateDocument::new("Content with key").with_key("my-unique-key");

        db.create_document(input).await.unwrap();

        let doc = db.get_document_by_key("my-unique-key").await.unwrap();
        assert!(doc.is_some());
        assert_eq!(doc.unwrap().content, "Content with key");
    }

    #[tokio::test]
    async fn test_list_documents() {
        let db = Database::new_memory().await.unwrap();

        for i in 0..5 {
            let input = CreateDocument::new(format!("Content {}", i))
                .with_tag(if i % 2 == 0 { "even" } else { "odd" });
            db.create_document(input).await.unwrap();
        }

        let all = db.list_documents(None, 10, 0).await.unwrap();
        assert_eq!(all.len(), 5);

        let even = db
            .list_documents(Some(&["even".to_string()]), 10, 0)
            .await
            .unwrap();
        assert_eq!(even.len(), 3);
    }

    #[tokio::test]
    async fn test_delete_document() {
        let db = Database::new_memory().await.unwrap();

        let input = CreateDocument::new("To delete");
        let doc = db.create_document(input).await.unwrap();

        let id = doc.id.unwrap().key().to_string();
        let deleted = db.delete_document(&id).await.unwrap();
        assert!(deleted);

        let gone = db.get_document(&id).await.unwrap();
        assert!(gone.is_none());
    }
}
