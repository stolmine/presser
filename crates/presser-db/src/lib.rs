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
//! db.insert_feed(&feed).await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
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
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_database_open() {
        let temp_file = NamedTempFile::new().unwrap();
        let db = Database::open(temp_file.path()).await;
        assert!(db.is_ok());
    }

    // TODO: Add more tests
}
