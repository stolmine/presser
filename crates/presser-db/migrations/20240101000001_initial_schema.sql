-- Initial schema for Presser database

-- Feeds table
CREATE TABLE IF NOT EXISTS feeds (
    id TEXT PRIMARY KEY NOT NULL,
    url TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT,
    site_url TEXT,
    last_fetched DATETIME,
    last_successful_fetch DATETIME,
    last_error TEXT,
    entry_count INTEGER NOT NULL DEFAULT 0,
    enabled BOOLEAN NOT NULL DEFAULT 1,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_feeds_enabled ON feeds(enabled);
CREATE INDEX idx_feeds_last_fetched ON feeds(last_fetched);

-- Entries table
CREATE TABLE IF NOT EXISTS entries (
    id TEXT PRIMARY KEY NOT NULL,
    feed_id TEXT NOT NULL,
    title TEXT NOT NULL,
    url TEXT NOT NULL UNIQUE,
    author TEXT,
    published DATETIME,
    updated DATETIME,
    summary TEXT,
    content_html TEXT,
    content_text TEXT,
    categories TEXT, -- JSON array
    read BOOLEAN NOT NULL DEFAULT 0,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (feed_id) REFERENCES feeds(id) ON DELETE CASCADE
);

CREATE INDEX idx_entries_feed_id ON entries(feed_id);
CREATE INDEX idx_entries_published ON entries(published DESC);
CREATE INDEX idx_entries_read ON entries(read);
CREATE INDEX idx_entries_feed_published ON entries(feed_id, published DESC);

-- Summaries table
CREATE TABLE IF NOT EXISTS summaries (
    entry_id TEXT PRIMARY KEY NOT NULL,
    summary_text TEXT NOT NULL,
    model TEXT NOT NULL,
    tokens INTEGER,
    content_hash TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (entry_id) REFERENCES entries(id) ON DELETE CASCADE
);

CREATE INDEX idx_summaries_content_hash ON summaries(content_hash);
CREATE INDEX idx_summaries_created_at ON summaries(created_at DESC);

-- Full-text search virtual table for entries
CREATE VIRTUAL TABLE IF NOT EXISTS entries_fts USING fts5(
    title,
    content_text,
    summary,
    content=entries,
    content_rowid=rowid
);

-- Triggers to keep FTS table in sync
CREATE TRIGGER IF NOT EXISTS entries_fts_insert AFTER INSERT ON entries BEGIN
    INSERT INTO entries_fts(rowid, title, content_text, summary)
    VALUES (new.rowid, new.title, new.content_text, new.summary);
END;

CREATE TRIGGER IF NOT EXISTS entries_fts_delete AFTER DELETE ON entries BEGIN
    DELETE FROM entries_fts WHERE rowid = old.rowid;
END;

CREATE TRIGGER IF NOT EXISTS entries_fts_update AFTER UPDATE ON entries BEGIN
    DELETE FROM entries_fts WHERE rowid = old.rowid;
    INSERT INTO entries_fts(rowid, title, content_text, summary)
    VALUES (new.rowid, new.title, new.content_text, new.summary);
END;
