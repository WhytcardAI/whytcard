//! Episodic Memory - Events and temporal interactions
//!
//! Stores events, interactions, and history in chronological order.
//! Allows finding "what happened" and "in what context".

use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::Path;
use whytcard_database::{Config as DbConfig, Database, StorageMode, VectorConfig};

/// Type of episode event
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeType {
    /// User query
    Query,
    /// System response
    Response,
    /// Error occurred
    Error,
    /// Decision made
    Decision,
    /// Learning event
    Learning,
    /// Tool call
    ToolCall,
    /// Context change
    Context,
    /// User feedback
    Feedback,
}

impl EpisodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Query => "query",
            Self::Response => "response",
            Self::Error => "error",
            Self::Decision => "decision",
            Self::Learning => "learning",
            Self::ToolCall => "tool_call",
            Self::Context => "context",
            Self::Feedback => "feedback",
        }
    }
}

/// Episodic memory for event/interaction storage
pub struct EpisodicMemory {
    /// Database connection
    db: Database,

    /// Current session ID
    current_session: Option<String>,

    /// Whether initialized
    initialized: bool,
}

impl EpisodicMemory {
    /// Create new episodic memory at the given path
    pub async fn new(db_path: &Path) -> Result<Self> {
        let db_config = DbConfig {
            storage: StorageMode::Persistent(db_path.to_path_buf()),
            namespace: "whytcard".into(),
            database: "episodic".into(),
            vector_config: VectorConfig::default(),
        };

        let db = Database::new(db_config).await?;

        Ok(Self {
            db,
            current_session: None,
            initialized: true,
        })
    }

    /// Create in-memory episodic memory for testing
    #[cfg(test)]
    pub async fn in_memory() -> Result<Self> {
        let db_config = DbConfig {
            storage: StorageMode::Memory,
            namespace: "whytcard".into(),
            database: "episodic_test".into(),
            vector_config: VectorConfig::default(),
        };

        let db = Database::new(db_config).await?;

        Ok(Self {
            db,
            current_session: None,
            initialized: true,
        })
    }

    /// Start a new session
    pub async fn start_session(&mut self, workspace: Option<String>) -> Result<String> {
        let session_id = uuid::Uuid::new_v4().to_string();
        let now = Utc::now();

        let session = Session {
            id: session_id.clone(),
            workspace,
            started_at: now,
            ended_at: None,
            metadata: None,
        };

        // Store session in database
        let doc = whytcard_database::CreateDocument::new(&serde_json::to_string(&session)?)
            .with_key(format!("session:{}", session_id))
            .with_tags(vec!["session".to_string()]);

        self.db.create_document(doc).await?;
        self.current_session = Some(session_id.clone());

        tracing::info!("Started session: {}", session_id);
        Ok(session_id)
    }

    /// End the current session
    pub async fn end_session(&mut self) -> Result<bool> {
        if let Some(session_id) = &self.current_session {
            // Update session with end time
            // Note: In a real implementation, we'd update the document
            tracing::info!("Ended session: {}", session_id);
            self.current_session = None;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Record an episode
    pub async fn record(&self, episode: Episode) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();

        let session_id = episode.session_id
            .or_else(|| self.current_session.clone())
            .unwrap_or_else(|| "default".to_string());

        let stored_episode = StoredEpisode {
            id: id.clone(),
            session_id,
            episode_type: episode.episode_type,
            content: episode.content,
            context: episode.context,
            metadata: episode.metadata,
            created_at: Utc::now(),
        };

        let doc = whytcard_database::CreateDocument::new(&serde_json::to_string(&stored_episode)?)
            .with_key(format!("episode:{}", id))
            .with_tags(vec!["episode".to_string(), stored_episode.episode_type.as_str().to_string()]);

        self.db.create_document(doc).await?;

        tracing::debug!("Recorded episode: {} ({})", id, stored_episode.episode_type.as_str());
        Ok(id)
    }

    /// Get recent episodes
    pub async fn get_recent(
        &self,
        limit: usize,
        episode_type: Option<EpisodeType>,
        session_id: Option<String>,
    ) -> Result<Vec<StoredEpisode>> {
        // Query episodes from database
        let tags = if let Some(et) = episode_type {
            vec!["episode".to_string(), et.as_str().to_string()]
        } else {
            vec!["episode".to_string()]
        };

        let docs = self.db.list_documents(Some(&tags), limit, 0).await?;

        let mut episodes: Vec<StoredEpisode> = docs
            .into_iter()
            .filter(|d| d.tags.contains(&"episode".to_string()))
            .filter_map(|d| serde_json::from_str(&d.content).ok())
            .collect();

        // Filter by session if provided
        if let Some(sid) = session_id {
            episodes.retain(|e| e.session_id == sid);
        }

        // Filter by type if provided
        if let Some(et) = episode_type {
            episodes.retain(|e| e.episode_type == et);
        }

        // Sort by created_at descending
        episodes.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        episodes.truncate(limit);

        Ok(episodes)
    }

    /// Search episodes by content
    pub async fn search(
        &self,
        query: &str,
        episode_type: Option<EpisodeType>,
        limit: usize,
    ) -> Result<Vec<StoredEpisode>> {
        let episodes = self.get_recent(limit * 2, episode_type, None).await?;

        let query_lower = query.to_lowercase();
        let filtered: Vec<_> = episodes
            .into_iter()
            .filter(|e| e.content.to_lowercase().contains(&query_lower))
            .take(limit)
            .collect();

        Ok(filtered)
    }

    /// Cleanup old episodes (retention policy)
    pub async fn cleanup_old(&self, retention_days: i64) -> Result<usize> {
        let cutoff = Utc::now() - chrono::Duration::days(retention_days);

        let episodes = self.get_recent(1000, None, None).await?;
        let mut deleted = 0;

        for episode in episodes {
            if episode.created_at < cutoff {
                self.db.delete_document_by_key(&format!("episode:{}", episode.id)).await?;
                deleted += 1;
            }
        }

        tracing::info!("Cleaned up {} old episodes", deleted);
        Ok(deleted)
    }

    /// Get statistics
    pub async fn get_stats(&self) -> EpisodicStats {
        let episodes = self.get_recent(10000, None, None).await.unwrap_or_default();

        let mut by_type = std::collections::HashMap::new();
        for ep in &episodes {
            *by_type.entry(ep.episode_type.as_str().to_string()).or_insert(0) += 1;
        }

        EpisodicStats {
            total_episodes: episodes.len(),
            current_session: self.current_session.clone(),
            episodes_by_type: by_type,
            initialized: self.initialized,
        }
    }
}

/// An episode to record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    /// Type of episode
    pub episode_type: EpisodeType,

    /// Content of the episode
    pub content: String,

    /// Session ID (uses current if not provided)
    pub session_id: Option<String>,

    /// Additional context
    pub context: Option<serde_json::Value>,

    /// Metadata
    pub metadata: Option<serde_json::Value>,
}

impl Episode {
    pub fn new(episode_type: EpisodeType, content: impl Into<String>) -> Self {
        Self {
            episode_type,
            content: content.into(),
            session_id: None,
            context: None,
            metadata: None,
        }
    }

    pub fn query(content: impl Into<String>) -> Self {
        Self::new(EpisodeType::Query, content)
    }

    pub fn response(content: impl Into<String>) -> Self {
        Self::new(EpisodeType::Response, content)
    }

    pub fn error(content: impl Into<String>) -> Self {
        Self::new(EpisodeType::Error, content)
    }

    pub fn tool_call(content: impl Into<String>) -> Self {
        Self::new(EpisodeType::ToolCall, content)
    }

    pub fn with_context(mut self, context: serde_json::Value) -> Self {
        self.context = Some(context);
        self
    }
}

/// A stored episode with ID and timestamps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEpisode {
    pub id: String,
    pub session_id: String,
    pub episode_type: EpisodeType,
    pub content: String,
    pub context: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// A session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub workspace: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
}

/// Statistics for episodic memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpisodicStats {
    pub total_episodes: usize,
    pub current_session: Option<String>,
    pub episodes_by_type: std::collections::HashMap<String, usize>,
    pub initialized: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_episodic_memory() {
        let mem = EpisodicMemory::in_memory().await.unwrap();
        assert!(mem.initialized);
    }

    #[tokio::test]
    async fn test_episode_builder() {
        let ep = Episode::query("What is Rust?")
            .with_context(serde_json::json!({"workspace": "/test"}));

        assert_eq!(ep.episode_type, EpisodeType::Query);
        assert_eq!(ep.content, "What is Rust?");
        assert!(ep.context.is_some());
    }
}
