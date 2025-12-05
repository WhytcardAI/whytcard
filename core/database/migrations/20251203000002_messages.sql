-- Messages table
-- Stores conversation messages linked to sessions
-- NO model_config here - messages are pure user data

CREATE TABLE IF NOT EXISTS messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

-- Index for fetching messages by session (most common query)
CREATE INDEX IF NOT EXISTS idx_messages_session_id ON messages(session_id);

-- Index for chronological ordering within a session
CREATE INDEX IF NOT EXISTS idx_messages_session_created ON messages(session_id, created_at ASC);
