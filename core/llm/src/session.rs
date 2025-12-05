//! Chat session management with history

use crate::error::{LlmError, Result};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Role of a message in a conversation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System message (instructions)
    System,
    /// User message
    User,
    /// Assistant response
    Assistant,
}

impl MessageRole {
    /// Convert to string for chat template
    pub fn as_str(&self) -> &str {
        match self {
            Self::System => "system",
            Self::User => "user",
            Self::Assistant => "assistant",
        }
    }
}

/// A single message in a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    /// Message role
    pub role: MessageRole,
    
    /// Message content
    pub content: String,
    
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    
    /// Token count (if computed)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_count: Option<usize>,
}

impl ChatMessage {
    /// Create a new message
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            timestamp: Utc::now(),
            token_count: None,
        }
    }
    
    /// Create a system message
    pub fn system(content: impl Into<String>) -> Self {
        Self::new(MessageRole::System, content)
    }
    
    /// Create a user message
    pub fn user(content: impl Into<String>) -> Self {
        Self::new(MessageRole::User, content)
    }
    
    /// Create an assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(MessageRole::Assistant, content)
    }
    
    /// Set token count
    pub fn with_token_count(mut self, count: usize) -> Self {
        self.token_count = Some(count);
        self
    }
}

/// A chat session with history management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSession {
    /// Session ID
    pub id: String,
    
    /// Session name/title
    pub name: Option<String>,
    
    /// Message history
    pub messages: Vec<ChatMessage>,
    
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    
    /// Last activity timestamp
    pub updated_at: DateTime<Utc>,
    
    /// System prompt for this session
    pub system_prompt: Option<String>,
    
    /// Maximum context tokens to keep
    pub max_context_tokens: usize,
    
    /// Model used for this session
    pub model_name: Option<String>,
    
    /// Custom metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for ChatSession {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatSession {
    /// Create a new session
    pub fn new() -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: None,
            messages: Vec::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            system_prompt: None,
            max_context_tokens: 4096,
            model_name: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Create with a specific ID
    pub fn with_id(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ..Self::new()
        }
    }
    
    /// Set session name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    /// Set system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }
    
    /// Set max context tokens
    pub fn with_max_context_tokens(mut self, tokens: usize) -> Self {
        self.max_context_tokens = tokens;
        self
    }
    
    /// Set model name
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model_name = Some(model.into());
        self
    }
    
    /// Add a message to the session
    pub fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        self.updated_at = Utc::now();
    }
    
    /// Add a user message
    pub fn add_user_message(&mut self, content: impl Into<String>) {
        self.add_message(ChatMessage::user(content));
    }
    
    /// Add an assistant message
    pub fn add_assistant_message(&mut self, content: impl Into<String>) {
        self.add_message(ChatMessage::assistant(content));
    }
    
    /// Get all messages for context
    pub fn get_messages(&self) -> &[ChatMessage] {
        &self.messages
    }
    
    /// Get messages with system prompt prepended
    pub fn get_messages_with_system(&self) -> Vec<ChatMessage> {
        let mut messages = Vec::new();
        
        if let Some(system) = &self.system_prompt {
            messages.push(ChatMessage::system(system.clone()));
        }
        
        messages.extend(self.messages.iter().cloned());
        messages
    }
    
    /// Get the last N messages
    pub fn get_recent_messages(&self, n: usize) -> &[ChatMessage] {
        let start = self.messages.len().saturating_sub(n);
        &self.messages[start..]
    }
    
    /// Clear all messages
    pub fn clear(&mut self) {
        self.messages.clear();
        self.updated_at = Utc::now();
    }
    
    /// Get message count
    pub fn message_count(&self) -> usize {
        self.messages.len()
    }
    
    /// Check if session is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
    
    /// Get last message
    pub fn last_message(&self) -> Option<&ChatMessage> {
        self.messages.last()
    }
    
    /// Get last user message
    pub fn last_user_message(&self) -> Option<&ChatMessage> {
        self.messages.iter()
            .rev()
            .find(|m| m.role == MessageRole::User)
    }
    
    /// Get last assistant message
    pub fn last_assistant_message(&self) -> Option<&ChatMessage> {
        self.messages.iter()
            .rev()
            .find(|m| m.role == MessageRole::Assistant)
    }
    
    /// Estimate total token count
    pub fn estimated_tokens(&self) -> usize {
        self.messages.iter()
            .map(|m| m.token_count.unwrap_or(m.content.len() / 4))
            .sum()
    }
    
    /// Truncate to fit within token limit
    pub fn truncate_to_fit(&mut self, max_tokens: usize) {
        // Keep system prompt space if present
        let system_tokens = self.system_prompt.as_ref()
            .map(|s| s.len() / 4)
            .unwrap_or(0);
        
        let available = max_tokens.saturating_sub(system_tokens);
        let mut total = 0;
        let mut keep_from = 0;
        
        // Count from end to find cutoff
        for (i, msg) in self.messages.iter().rev().enumerate() {
            let tokens = msg.token_count.unwrap_or(msg.content.len() / 4);
            total += tokens;
            if total > available {
                keep_from = self.messages.len().saturating_sub(i);
                break;
            }
        }
        
        if keep_from > 0 {
            self.messages = self.messages[keep_from..].to_vec();
            self.updated_at = Utc::now();
        }
    }
}

/// Manager for multiple chat sessions
pub struct SessionManager {
    sessions: HashMap<String, ChatSession>,
    active_session: Option<String>,
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            active_session: None,
        }
    }
    
    /// Create a new session and return its ID
    pub fn create_session(&mut self) -> String {
        let session = ChatSession::new();
        let id = session.id.clone();
        self.sessions.insert(id.clone(), session);
        id
    }
    
    /// Create a session with options
    pub fn create_session_with(&mut self, session: ChatSession) -> String {
        let id = session.id.clone();
        self.sessions.insert(id.clone(), session);
        id
    }
    
    /// Get a session by ID
    pub fn get(&self, id: &str) -> Option<&ChatSession> {
        self.sessions.get(id)
    }
    
    /// Get a mutable session by ID
    pub fn get_mut(&mut self, id: &str) -> Option<&mut ChatSession> {
        self.sessions.get_mut(id)
    }
    
    /// Get or create active session
    pub fn get_or_create_active(&mut self) -> &mut ChatSession {
        let id = self.active_session.clone().unwrap_or_else(|| {
            let new_id = self.create_session();
            self.active_session = Some(new_id.clone());
            new_id
        });
        
        self.sessions.get_mut(&id).expect("Session should exist")
    }
    
    /// Set active session
    pub fn set_active(&mut self, id: &str) -> Result<()> {
        if self.sessions.contains_key(id) {
            self.active_session = Some(id.to_string());
            Ok(())
        } else {
            Err(LlmError::SessionNotFound(id.to_string()))
        }
    }
    
    /// Get active session
    pub fn active(&self) -> Option<&ChatSession> {
        self.active_session.as_ref()
            .and_then(|id| self.sessions.get(id))
    }
    
    /// Get active session mutably
    pub fn active_mut(&mut self) -> Option<&mut ChatSession> {
        let id = self.active_session.clone()?;
        self.sessions.get_mut(&id)
    }
    
    /// Delete a session
    pub fn delete(&mut self, id: &str) -> bool {
        if self.active_session.as_deref() == Some(id) {
            self.active_session = None;
        }
        self.sessions.remove(id).is_some()
    }
    
    /// List all session IDs
    pub fn list(&self) -> Vec<&str> {
        self.sessions.keys().map(|s| s.as_str()).collect()
    }
    
    /// Get session count
    pub fn count(&self) -> usize {
        self.sessions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_chat_message() {
        let user = ChatMessage::user("Hello");
        assert_eq!(user.role, MessageRole::User);
        assert_eq!(user.content, "Hello");
        
        let assistant = ChatMessage::assistant("Hi there!");
        assert_eq!(assistant.role, MessageRole::Assistant);
    }
    
    #[test]
    fn test_chat_session() {
        let mut session = ChatSession::new()
            .with_system_prompt("You are a helpful assistant.");
        
        session.add_user_message("Hello");
        session.add_assistant_message("Hi! How can I help?");
        
        assert_eq!(session.message_count(), 2);
        assert!(session.system_prompt.is_some());
        
        let messages = session.get_messages_with_system();
        assert_eq!(messages.len(), 3);
        assert_eq!(messages[0].role, MessageRole::System);
    }
    
    #[test]
    fn test_session_manager() {
        let mut manager = SessionManager::new();
        
        let id = manager.create_session();
        assert_eq!(manager.count(), 1);
        
        manager.set_active(&id).unwrap();
        let session = manager.active_mut().unwrap();
        session.add_user_message("Test");
        
        assert_eq!(manager.active().unwrap().message_count(), 1);
    }
    
    #[test]
    fn test_message_serialization() {
        let msg = ChatMessage::user("Test message");
        let json = serde_json::to_string(&msg).unwrap();
        let parsed: ChatMessage = serde_json::from_str(&json).unwrap();
        
        assert_eq!(parsed.role, MessageRole::User);
        assert_eq!(parsed.content, "Test message");
    }
}
