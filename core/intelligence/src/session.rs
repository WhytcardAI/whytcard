//! Session Management for Multi-Client Support
//!
//! This module provides session isolation for multiple concurrent MCP clients.
//! Each client gets its own session context while sharing the underlying
//! resources (database, RAG, CORTEX) safely via Arc/RwLock.
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    MultiSessionManager                       │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐                  │
//! │  │ Session1 │  │ Session2 │  │ Session3 │  ...             │
//! │  │ (VS Code)│  │ (Cursor) │  │ (Claude) │                  │
//! │  └────┬─────┘  └────┬─────┘  └────┬─────┘                  │
//! │       │             │             │                         │
//! │       └─────────────┴─────────────┘                         │
//! │                     │                                       │
//! │              Shared Resources (Arc)                         │
//! │       ┌─────────────┴─────────────┐                         │
//! │       │  DB  │  RAG  │  CORTEX   │                         │
//! │       └─────────────────────────────┘                       │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Session Isolation
//!
//! Each session maintains:
//! - Its own CORTEX session ID for episodic memory tracking
//! - Client identification (name, version)
//! - Activity timestamps for cleanup
//! - Optional namespace override
//!
//! Shared across sessions (thread-safe):
//! - Database (SurrealDB - inherently concurrent)
//! - RAG engine (via RwLock)
//! - CORTEX engine (via internal RwLocks)
//! - MCP client connections

use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Session identifier type
pub type SessionId = String;

/// Information about a connected client
#[derive(Debug, Clone)]
pub struct ClientInfo {
    /// Client name (e.g., "vscode", "cursor", "claude-desktop")
    pub name: String,
    /// Client version
    pub version: Option<String>,
    /// Connection timestamp
    pub connected_at: Instant,
    /// Last activity timestamp
    pub last_activity: Instant,
    /// Optional namespace override for this session
    pub namespace: Option<String>,
}

impl ClientInfo {
    /// Create new client info
    pub fn new(name: impl Into<String>) -> Self {
        let now = Instant::now();
        Self {
            name: name.into(),
            version: None,
            connected_at: now,
            last_activity: now,
            namespace: None,
        }
    }

    /// Set client version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }

    /// Set namespace override
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    /// Update last activity timestamp
    pub fn touch(&mut self) {
        self.last_activity = Instant::now();
    }

    /// Check if session is stale (no activity for given duration)
    pub fn is_stale(&self, timeout: Duration) -> bool {
        self.last_activity.elapsed() > timeout
    }
}

/// A single client session
#[derive(Debug)]
pub struct ClientSession {
    /// Unique session identifier
    pub id: SessionId,
    /// Client information
    pub client: ClientInfo,
    /// CORTEX session ID (for episodic memory tracking)
    pub cortex_session_id: Option<String>,
    /// Whether session is active
    pub active: bool,
}

impl ClientSession {
    /// Create a new session
    pub fn new(client: ClientInfo) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            client,
            cortex_session_id: None,
            active: true,
        }
    }

    /// Set CORTEX session ID
    pub fn with_cortex_session(mut self, session_id: String) -> Self {
        self.cortex_session_id = Some(session_id);
        self
    }

    /// Mark session as inactive
    pub fn deactivate(&mut self) {
        self.active = false;
    }
}

/// Manager for multiple concurrent client sessions
///
/// Thread-safe session management that tracks connected clients
/// and provides session isolation while sharing resources.
pub struct MultiSessionManager {
    /// Active sessions by ID
    sessions: RwLock<HashMap<SessionId, ClientSession>>,
    /// Maximum allowed sessions (0 = unlimited)
    max_sessions: usize,
    /// Session timeout for cleanup
    session_timeout: Duration,
}

impl MultiSessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: RwLock::new(HashMap::new()),
            max_sessions: 0, // unlimited by default
            session_timeout: Duration::from_secs(3600), // 1 hour default
        }
    }

    /// Set maximum allowed sessions
    pub fn with_max_sessions(mut self, max: usize) -> Self {
        self.max_sessions = max;
        self
    }

    /// Set session timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.session_timeout = timeout;
        self
    }

    /// Create a new session for a client
    pub async fn create_session(&self, client: ClientInfo) -> Result<SessionId, SessionError> {
        let mut sessions = self.sessions.write().await;

        // Check max sessions limit
        if self.max_sessions > 0 && sessions.len() >= self.max_sessions {
            return Err(SessionError::MaxSessionsReached(self.max_sessions));
        }

        let session = ClientSession::new(client);
        let session_id = session.id.clone();

        tracing::info!(
            session_id = %session_id,
            client = %session.client.name,
            "New session created"
        );

        sessions.insert(session_id.clone(), session);
        Ok(session_id)
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Option<ClientSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).map(|s| ClientSession {
            id: s.id.clone(),
            client: s.client.clone(),
            cortex_session_id: s.cortex_session_id.clone(),
            active: s.active,
        })
    }

    /// Update session activity timestamp
    pub async fn touch_session(&self, session_id: &str) -> bool {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.client.touch();
            true
        } else {
            false
        }
    }

    /// Set CORTEX session ID for a session
    pub async fn set_cortex_session(&self, session_id: &str, cortex_session_id: String) -> bool {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.cortex_session_id = Some(cortex_session_id);
            true
        } else {
            false
        }
    }

    /// End a session
    pub async fn end_session(&self, session_id: &str) -> bool {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.remove(session_id) {
            tracing::info!(
                session_id = %session_id,
                client = %session.client.name,
                duration_secs = %session.client.connected_at.elapsed().as_secs(),
                "Session ended"
            );
            true
        } else {
            false
        }
    }

    /// Get all active session IDs
    pub async fn list_sessions(&self) -> Vec<SessionId> {
        let sessions = self.sessions.read().await;
        sessions.keys().cloned().collect()
    }

    /// Get session count
    pub async fn session_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.len()
    }

    /// Cleanup stale sessions
    pub async fn cleanup_stale_sessions(&self) -> usize {
        let mut sessions = self.sessions.write().await;
        let before = sessions.len();

        sessions.retain(|id, session| {
            let stale = session.client.is_stale(self.session_timeout);
            if stale {
                tracing::warn!(
                    session_id = %id,
                    client = %session.client.name,
                    "Removing stale session"
                );
            }
            !stale
        });

        before - sessions.len()
    }

    /// Get session statistics
    pub async fn stats(&self) -> SessionStats {
        let sessions = self.sessions.read().await;

        let active_count = sessions.values().filter(|s| s.active).count();
        let clients: Vec<_> = sessions
            .values()
            .map(|s| s.client.name.clone())
            .collect();

        SessionStats {
            total_sessions: sessions.len(),
            active_sessions: active_count,
            max_sessions: self.max_sessions,
            connected_clients: clients,
        }
    }
}

impl Default for MultiSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Session statistics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SessionStats {
    /// Total number of sessions
    pub total_sessions: usize,
    /// Number of active sessions
    pub active_sessions: usize,
    /// Maximum allowed sessions (0 = unlimited)
    pub max_sessions: usize,
    /// List of connected client names
    pub connected_clients: Vec<String>,
}

/// Session errors
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Maximum sessions reached: {0}")]
    MaxSessionsReached(usize),

    #[error("Session not found: {0}")]
    NotFound(String),

    #[error("Session expired: {0}")]
    Expired(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_session() {
        let manager = MultiSessionManager::new();
        let client = ClientInfo::new("test-client").with_version("1.0.0");

        let session_id = manager.create_session(client).await.unwrap();
        assert!(!session_id.is_empty());

        let session = manager.get_session(&session_id).await.unwrap();
        assert_eq!(session.client.name, "test-client");
        assert!(session.active);
    }

    #[tokio::test]
    async fn test_max_sessions() {
        let manager = MultiSessionManager::new().with_max_sessions(2);

        // Create 2 sessions
        let client1 = ClientInfo::new("client1");
        let client2 = ClientInfo::new("client2");
        manager.create_session(client1).await.unwrap();
        manager.create_session(client2).await.unwrap();

        // Third should fail
        let client3 = ClientInfo::new("client3");
        let result = manager.create_session(client3).await;
        assert!(matches!(result, Err(SessionError::MaxSessionsReached(2))));
    }

    #[tokio::test]
    async fn test_end_session() {
        let manager = MultiSessionManager::new();
        let client = ClientInfo::new("test");

        let session_id = manager.create_session(client).await.unwrap();
        assert_eq!(manager.session_count().await, 1);

        manager.end_session(&session_id).await;
        assert_eq!(manager.session_count().await, 0);
    }

    #[tokio::test]
    async fn test_touch_session() {
        let manager = MultiSessionManager::new();
        let client = ClientInfo::new("test");

        let session_id = manager.create_session(client).await.unwrap();
        let result = manager.touch_session(&session_id).await;
        assert!(result);

        let result = manager.touch_session("nonexistent").await;
        assert!(!result);
    }
}
