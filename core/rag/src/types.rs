//! Core types for the RAG module.

use serde::{Deserialize, Serialize};

/// A document to be indexed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    /// Unique identifier
    pub id: String,
    /// File path (if from file)
    pub path: Option<String>,
    /// Document title
    pub title: Option<String>,
    /// Full document content
    pub content: String,
    /// MIME type
    pub mime_type: Option<String>,
    /// Custom metadata
    pub metadata: Option<serde_json::Value>,
    /// Creation timestamp
    pub created_at: i64,
}

impl Document {
    /// Create a new document from content.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            path: None,
            title: None,
            content: content.into(),
            mime_type: None,
            metadata: None,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Create a document from a file path.
    pub fn from_path(path: impl Into<String>, content: impl Into<String>) -> Self {
        let path_str = path.into();
        let title = std::path::Path::new(&path_str)
            .file_name()
            .and_then(|n| n.to_str())
            .map(String::from);

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            path: Some(path_str),
            title,
            content: content.into(),
            mime_type: None,
            metadata: None,
            created_at: chrono::Utc::now().timestamp(),
        }
    }

    /// Set a custom document ID.
    ///
    /// By default, documents are assigned a UUID. Use this method when you need
    /// to associate a specific identifier with the document (e.g., for memory keys).
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Set document title.
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set MIME type.
    pub fn with_mime_type(mut self, mime_type: impl Into<String>) -> Self {
        self.mime_type = Some(mime_type.into());
        self
    }

    /// Set custom metadata as a JSON value.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Set a single metadata field.
    ///
    /// If metadata is None, creates a new object. If metadata exists as an object,
    /// adds or updates the field. If metadata is not an object, replaces it.
    pub fn with_metadata_field(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        let key = key.into();
        let value = value.into();

        self.metadata = Some(match self.metadata.take() {
            Some(serde_json::Value::Object(mut map)) => {
                map.insert(key, value);
                serde_json::Value::Object(map)
            }
            _ => serde_json::json!({ key: value }),
        });
        self
    }
}

/// A chunk of text extracted from a document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Unique chunk identifier
    pub id: String,
    /// Parent document ID
    pub document_id: String,
    /// Chunk index within document
    pub index: usize,
    /// Chunk text content
    pub text: String,
    /// Character start position in document
    pub start_char: usize,
    /// Character end position in document
    pub end_char: usize,
    /// Token count estimate
    pub token_count: usize,
    /// Optional metadata inherited from document
    pub metadata: Option<serde_json::Value>,
}

impl Chunk {
    /// Create a new chunk.
    pub fn new(
        document_id: impl Into<String>,
        index: usize,
        text: impl Into<String>,
        start_char: usize,
        end_char: usize,
    ) -> Self {
        let text_str = text.into();
        let token_count = estimate_tokens(&text_str);

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            document_id: document_id.into(),
            index,
            text: text_str,
            start_char,
            end_char,
            token_count,
            metadata: None,
        }
    }
}

/// Search result from vector store.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// The matching chunk
    pub chunk: Chunk,
    /// Similarity score (0.0 - 1.0, higher is better)
    pub score: f32,
    /// Distance from query vector
    pub distance: f32,
}

impl SearchResult {
    /// Create a new search result.
    pub fn new(chunk: Chunk, score: f32, distance: f32) -> Self {
        Self {
            chunk,
            score,
            distance,
        }
    }
}

/// Estimate token count for text (rough approximation).
fn estimate_tokens(text: &str) -> usize {
    // Rough estimate: ~4 characters per token for English text
    text.len() / 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_new() {
        let doc = Document::new("Hello world");
        assert!(!doc.id.is_empty());
        assert_eq!(doc.content, "Hello world");
        assert!(doc.path.is_none());
    }

    #[test]
    fn test_document_from_path() {
        let doc = Document::from_path("/path/to/file.md", "Content here");
        assert_eq!(doc.path, Some("/path/to/file.md".to_string()));
        assert_eq!(doc.title, Some("file.md".to_string()));
    }

    #[test]
    fn test_document_builder() {
        let doc = Document::new("Content")
            .with_title("My Doc")
            .with_mime_type("text/plain");

        assert_eq!(doc.title, Some("My Doc".to_string()));
        assert_eq!(doc.mime_type, Some("text/plain".to_string()));
    }

    #[test]
    fn test_chunk_new() {
        let chunk = Chunk::new("doc-123", 0, "Hello world", 0, 11);
        assert!(!chunk.id.is_empty());
        assert_eq!(chunk.document_id, "doc-123");
        assert_eq!(chunk.index, 0);
        assert_eq!(chunk.text, "Hello world");
        assert!(chunk.token_count > 0);
    }

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(estimate_tokens(""), 0);
        assert_eq!(estimate_tokens("test"), 1);
        assert_eq!(estimate_tokens("hello world"), 2); // 11 chars / 4
    }
}
