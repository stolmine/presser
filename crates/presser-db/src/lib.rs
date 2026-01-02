//! Database layer for Presser using SQLite
//!
//! This crate manages all database operations for Presser, including:
//! - Feed metadata storage
//! - Article/entry storage
//! - Summary caching
//! - Read/unread status tracking
//! - Full-text search
//!
//! # Schema
//!
//! The database consists of several tables:
//! - `feeds`: Feed metadata and configuration
//! - `entries`: Individual feed entries/articles
//! - `summaries`: AI-generated summaries
//! - `read_status`: User read/unread tracking
//!
//! # Example
//!
//! ```rust,no_run
//! use presser_db::{Database, Feed};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let db = Database::open("presser.db").await?;
//! db.migrate().await?;
//!
//! // Insert a feed
//! let feed = Feed {
//!     id: "tech-news".to_string(),
//!     url: "https://example.com/feed.xml".to_string(),
//!     title: "Tech News".to_string(),
//!     ..Default::default()
//! };
//! db.upsert_feed(&feed).await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;

pub mod error;
pub mod models;
pub mod queries;

pub use error::DatabaseError;
pub use models::*;

/// Database connection pool and operations
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Open a database connection
    ///
    /// Creates the database file if it doesn't exist
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", path.display()))?
            .create_if_missing(true)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await
            .context("Failed to connect to database")?;

        Ok(Self { pool })
    }

    /// Run database migrations
    ///
    /// This creates all necessary tables and indices
    pub async fn migrate(&self) -> Result<()> {
        tracing::info!("Running database migrations");

        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .context("Failed to run migrations")?;

        Ok(())
    }

    /// Insert or update a feed
    pub async fn upsert_feed(&self, feed: &Feed) -> Result<()> {
        queries::upsert_feed(&self.pool, feed).await
    }

    /// Get a feed by ID
    pub async fn get_feed(&self, id: &str) -> Result<Option<Feed>> {
        queries::get_feed(&self.pool, id).await
    }

    /// Get all feeds
    pub async fn get_all_feeds(&self) -> Result<Vec<Feed>> {
        queries::get_all_feeds(&self.pool).await
    }

    /// Delete a feed and all its entries
    pub async fn delete_feed(&self, id: &str) -> Result<()> {
        queries::delete_feed(&self.pool, id).await
    }

    /// Insert or update an entry
    pub async fn upsert_entry(&self, entry: &Entry) -> Result<()> {
        queries::upsert_entry(&self.pool, entry).await
    }

    /// Get an entry by ID
    pub async fn get_entry(&self, id: &str) -> Result<Option<Entry>> {
        queries::get_entry(&self.pool, id).await
    }

    /// Get entries for a feed
    pub async fn get_entries_for_feed(&self, feed_id: &str, limit: i64) -> Result<Vec<Entry>> {
        queries::get_entries_for_feed(&self.pool, feed_id, limit).await
    }

    /// Get unread entries
    pub async fn get_unread_entries(&self, limit: i64) -> Result<Vec<Entry>> {
        queries::get_unread_entries(&self.pool, limit).await
    }

    /// Mark an entry as read
    pub async fn mark_read(&self, entry_id: &str) -> Result<()> {
        queries::mark_read(&self.pool, entry_id).await
    }

    /// Mark an entry as unread
    pub async fn mark_unread(&self, entry_id: &str) -> Result<()> {
        queries::mark_unread(&self.pool, entry_id).await
    }

    /// Insert or update a summary
    pub async fn upsert_summary(&self, summary: &Summary) -> Result<()> {
        queries::upsert_summary(&self.pool, summary).await
    }

    /// Get summary for an entry
    pub async fn get_summary(&self, entry_id: &str) -> Result<Option<Summary>> {
        queries::get_summary(&self.pool, entry_id).await
    }

    /// Search entries by text
    pub async fn search_entries(&self, query: &str, limit: i64) -> Result<Vec<Entry>> {
        queries::search_entries(&self.pool, query, limit).await
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<DatabaseStats> {
        queries::get_stats(&self.pool).await
    }

    /// Get a reference to the connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Close the database connection
    pub async fn close(self) {
        self.pool.close().await;
    }
}

/// Database statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub total_feeds: i64,
    pub total_entries: i64,
    pub unread_entries: i64,
    pub total_summaries: i64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    async fn setup_db() -> (Database, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::open(&db_path).await.unwrap();
        db.migrate().await.unwrap();
        (db, temp_dir) // Return temp_dir to keep it alive
    }

    #[tokio::test]
    async fn test_database_open() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = Database::open(&db_path).await;
        assert!(db.is_ok());
    }

    #[tokio::test]
    async fn test_feed_crud() {
        let (db, _dir) = setup_db().await;

        // Create
        let feed = Feed {
            id: "test-feed".into(),
            url: "https://example.com/feed.xml".into(),
            title: "Test Feed".into(),
            ..Default::default()
        };
        db.upsert_feed(&feed).await.unwrap();

        // Read
        let fetched = db.get_feed("test-feed").await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().title, "Test Feed");

        // Update
        let updated_feed = Feed {
            title: "Updated Title".into(),
            ..feed.clone()
        };
        db.upsert_feed(&updated_feed).await.unwrap();
        let fetched = db.get_feed("test-feed").await.unwrap().unwrap();
        assert_eq!(fetched.title, "Updated Title");

        // List all
        let all = db.get_all_feeds().await.unwrap();
        assert_eq!(all.len(), 1);

        // Delete
        db.delete_feed("test-feed").await.unwrap();
        assert!(db.get_feed("test-feed").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_entry_operations() {
        let (db, _dir) = setup_db().await;

        // Setup feed first
        let feed = Feed {
            id: "feed1".into(),
            url: "https://ex.com/f".into(),
            title: "F".into(),
            ..Default::default()
        };
        db.upsert_feed(&feed).await.unwrap();

        // Create entry
        let entry = Entry {
            id: "entry1".into(),
            feed_id: "feed1".into(),
            title: "Article Title".into(),
            url: "https://ex.com/article".into(),
            content_text: Some("searchable content here".into()),
            ..Default::default()
        };
        db.upsert_entry(&entry).await.unwrap();

        // Read
        let fetched = db.get_entry("entry1").await.unwrap().unwrap();
        assert_eq!(fetched.title, "Article Title");
        assert!(!fetched.read);

        // Mark read
        db.mark_read("entry1").await.unwrap();
        let fetched = db.get_entry("entry1").await.unwrap().unwrap();
        assert!(fetched.read);

        // Unread entries (should be empty now)
        let unread = db.get_unread_entries(100).await.unwrap();
        assert!(unread.is_empty());

        // Mark unread
        db.mark_unread("entry1").await.unwrap();
        let unread = db.get_unread_entries(100).await.unwrap();
        assert_eq!(unread.len(), 1);

        // Get entries for feed
        let entries = db.get_entries_for_feed("feed1", 100).await.unwrap();
        assert_eq!(entries.len(), 1);
    }

    #[tokio::test]
    async fn test_summary_operations() {
        let (db, _dir) = setup_db().await;

        // Setup feed and entry
        let feed = Feed {
            id: "feed1".into(),
            url: "https://ex.com/f".into(),
            title: "F".into(),
            ..Default::default()
        };
        db.upsert_feed(&feed).await.unwrap();

        let entry = Entry {
            id: "entry1".into(),
            feed_id: "feed1".into(),
            title: "Article".into(),
            url: "https://ex.com/a".into(),
            ..Default::default()
        };
        db.upsert_entry(&entry).await.unwrap();

        // Create summary
        let summary = Summary {
            entry_id: "entry1".into(),
            summary_text: "This is a summary".into(),
            model: "gpt-4".into(),
            tokens: Some(50),
            content_hash: "abc123".into(),
            ..Default::default()
        };
        db.upsert_summary(&summary).await.unwrap();

        // Read summary
        let fetched = db.get_summary("entry1").await.unwrap().unwrap();
        assert_eq!(fetched.summary_text, "This is a summary");
        assert_eq!(fetched.model, "gpt-4");
    }

    #[tokio::test]
    async fn test_stats() {
        let (db, _dir) = setup_db().await;

        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.total_feeds, 0);
        assert_eq!(stats.total_entries, 0);

        // Add a feed
        let feed = Feed {
            id: "f1".into(),
            url: "https://ex.com/f".into(),
            title: "F".into(),
            ..Default::default()
        };
        db.upsert_feed(&feed).await.unwrap();

        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.total_feeds, 1);

        // Add an entry
        let entry = Entry {
            id: "e1".into(),
            feed_id: "f1".into(),
            title: "E".into(),
            url: "https://ex.com/e".into(),
            ..Default::default()
        };
        db.upsert_entry(&entry).await.unwrap();

        let stats = db.get_stats().await.unwrap();
        assert_eq!(stats.total_entries, 1);
        assert_eq!(stats.unread_entries, 1);
    }

    #[tokio::test]
    async fn test_fts_search() {
        let (db, _dir) = setup_db().await;

        let feed = Feed {
            id: "feed1".into(),
            url: "https://ex.com/f".into(),
            title: "F".into(),
            ..Default::default()
        };
        db.upsert_feed(&feed).await.unwrap();

        let entry = Entry {
            id: "entry1".into(),
            feed_id: "feed1".into(),
            title: "Rust Programming".into(),
            url: "https://ex.com/rust".into(),
            content_text: Some("Learn async await in Rust language".into()),
            ..Default::default()
        };
        db.upsert_entry(&entry).await.unwrap();

        // Search by title
        let results = db.search_entries("Rust", 10).await.unwrap();
        assert_eq!(results.len(), 1);

        // Search by content
        let results = db.search_entries("async", 10).await.unwrap();
        assert_eq!(results.len(), 1);

        // No results for unrelated query
        let results = db.search_entries("Python", 10).await.unwrap();
        assert!(results.is_empty());
    }
}
