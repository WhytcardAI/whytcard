-- Library files table
-- Stores file metadata for RAG system
-- Embeddings are stored separately in LanceDB (core/rag)

CREATE TABLE IF NOT EXISTS library_files (
    id TEXT PRIMARY KEY NOT NULL,
    -- File identification
    file_path TEXT NOT NULL UNIQUE,
    file_name TEXT NOT NULL,
    file_hash TEXT NOT NULL,
    -- File metadata
    file_size INTEGER NOT NULL,
    mime_type TEXT,
    -- RAG processing status
    rag_status TEXT NOT NULL DEFAULT 'pending' CHECK (rag_status IN ('pending', 'processing', 'indexed', 'failed', 'skipped')),
    rag_error TEXT,
    chunk_count INTEGER DEFAULT 0,
    -- Timestamps
    file_modified_at INTEGER NOT NULL,
    indexed_at INTEGER,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    updated_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now'))
);

-- Index for finding files by hash (deduplication)
CREATE INDEX IF NOT EXISTS idx_library_files_hash ON library_files(file_hash);

-- Index for filtering by RAG status
CREATE INDEX IF NOT EXISTS idx_library_files_rag_status ON library_files(rag_status);

-- Index for path lookups
CREATE INDEX IF NOT EXISTS idx_library_files_path ON library_files(file_path);

-- Session-File link table (N:N relationship)
-- Allows attaching files to specific sessions for context
CREATE TABLE IF NOT EXISTS session_files (
    session_id TEXT NOT NULL,
    file_id TEXT NOT NULL,
    attached_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    PRIMARY KEY (session_id, file_id),
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (file_id) REFERENCES library_files(id) ON DELETE CASCADE
);

-- Index for listing files in a session
CREATE INDEX IF NOT EXISTS idx_session_files_session ON session_files(session_id);

-- Index for finding sessions using a file
CREATE INDEX IF NOT EXISTS idx_session_files_file ON session_files(file_id);
