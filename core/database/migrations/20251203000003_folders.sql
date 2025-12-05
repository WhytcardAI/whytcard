-- Folders table
-- Organization structure for sessions
-- Supports nested folders, colors, and ordering

CREATE TABLE IF NOT EXISTS folders (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    parent_id TEXT,
    color TEXT DEFAULT '#6B7280',
    icon TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    folder_type TEXT NOT NULL DEFAULT 'folder' CHECK (folder_type IN ('folder', 'archive', 'favorites', 'trash')),
    created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
    FOREIGN KEY (parent_id) REFERENCES folders(id) ON DELETE SET NULL
);

-- Index for listing folders by parent (tree structure)
CREATE INDEX IF NOT EXISTS idx_folders_parent ON folders(parent_id);

-- Index for ordering
CREATE INDEX IF NOT EXISTS idx_folders_sort ON folders(sort_order);

-- Add folder_id to sessions for organization
ALTER TABLE sessions ADD COLUMN folder_id TEXT REFERENCES folders(id) ON DELETE SET NULL;

-- Index for filtering sessions by folder
CREATE INDEX IF NOT EXISTS idx_sessions_folder ON sessions(folder_id);
