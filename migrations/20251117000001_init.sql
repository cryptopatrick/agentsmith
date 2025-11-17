-- Create sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id TEXT PRIMARY KEY NOT NULL,
    summary TEXT,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Create traces table
CREATE TABLE IF NOT EXISTS traces (
    id TEXT PRIMARY KEY NOT NULL,
    session_id TEXT NOT NULL,
    role TEXT NOT NULL,
    content TEXT NOT NULL,
    metadata TEXT NOT NULL DEFAULT '{}',
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    embedding TEXT,
    FOREIGN KEY (session_id) REFERENCES sessions(id) ON DELETE CASCADE
);

-- Create FTS5 virtual table for full-text search
CREATE VIRTUAL TABLE IF NOT EXISTS traces_fts USING fts5(
    id UNINDEXED,
    session_id UNINDEXED,
    role,
    content,
    metadata,
    content='traces',
    content_rowid='rowid'
);

-- Trigger to keep FTS index in sync on INSERT
CREATE TRIGGER IF NOT EXISTS traces_fts_insert AFTER INSERT ON traces BEGIN
    INSERT INTO traces_fts(rowid, id, session_id, role, content, metadata)
    VALUES (new.rowid, new.id, new.session_id, new.role, new.content, new.metadata);
END;

-- Trigger to keep FTS index in sync on UPDATE
CREATE TRIGGER IF NOT EXISTS traces_fts_update AFTER UPDATE ON traces BEGIN
    UPDATE traces_fts SET
        id = new.id,
        session_id = new.session_id,
        role = new.role,
        content = new.content,
        metadata = new.metadata
    WHERE rowid = old.rowid;
END;

-- Trigger to keep FTS index in sync on DELETE
CREATE TRIGGER IF NOT EXISTS traces_fts_delete AFTER DELETE ON traces BEGIN
    DELETE FROM traces_fts WHERE rowid = old.rowid;
END;

-- Index for faster session lookups
CREATE INDEX IF NOT EXISTS idx_traces_session_id ON traces(session_id);
CREATE INDEX IF NOT EXISTS idx_traces_created_at ON traces(created_at);
