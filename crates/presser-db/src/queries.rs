//! Database query implementations
//!
//! Uses runtime queries to avoid requiring a database during compilation.

use crate::models::{Entry, Feed, Summary};
use crate::DatabaseStats;
use anyhow::{Context, Result};
use sqlx::{Row, SqlitePool};

// =============================================================================
// Feed Operations
// =============================================================================

/// Insert or update a feed
pub async fn upsert_feed(pool: &SqlitePool, feed: &Feed) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO feeds (id, url, title, description, site_url, last_fetched,
                          last_successful_fetch, last_error, entry_count, enabled,
                          created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
        ON CONFLICT(id) DO UPDATE SET
            url = excluded.url,
            title = excluded.title,
            description = excluded.description,
            site_url = excluded.site_url,
            last_fetched = excluded.last_fetched,
            last_successful_fetch = excluded.last_successful_fetch,
            last_error = excluded.last_error,
            entry_count = excluded.entry_count,
            enabled = excluded.enabled,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(&feed.id)
    .bind(&feed.url)
    .bind(&feed.title)
    .bind(&feed.description)
    .bind(&feed.site_url)
    .bind(&feed.last_fetched)
    .bind(&feed.last_successful_fetch)
    .bind(&feed.last_error)
    .bind(feed.entry_count)
    .bind(feed.enabled)
    .bind(&feed.created_at)
    .bind(&feed.updated_at)
    .execute(pool)
    .await
    .context("Failed to upsert feed")?;
    Ok(())
}

/// Get a feed by ID
pub async fn get_feed(pool: &SqlitePool, id: &str) -> Result<Option<Feed>> {
    sqlx::query_as::<_, Feed>("SELECT * FROM feeds WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .context("Failed to get feed")
}

/// Get all feeds ordered by title
pub async fn get_all_feeds(pool: &SqlitePool) -> Result<Vec<Feed>> {
    sqlx::query_as::<_, Feed>("SELECT * FROM feeds ORDER BY title COLLATE NOCASE")
        .fetch_all(pool)
        .await
        .context("Failed to get all feeds")
}

/// Delete a feed (entries cascade via foreign key)
pub async fn delete_feed(pool: &SqlitePool, id: &str) -> Result<()> {
    sqlx::query("DELETE FROM feeds WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .context("Failed to delete feed")?;
    Ok(())
}

// =============================================================================
// Entry Operations
// =============================================================================

/// Insert or update an entry (preserves read status on update)
pub async fn upsert_entry(pool: &SqlitePool, entry: &Entry) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO entries (id, feed_id, title, url, author, published, updated,
                            summary, content_html, content_text, categories, read,
                            created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
        ON CONFLICT(id) DO UPDATE SET
            feed_id = excluded.feed_id,
            title = excluded.title,
            url = excluded.url,
            author = excluded.author,
            published = excluded.published,
            updated = excluded.updated,
            summary = excluded.summary,
            content_html = excluded.content_html,
            content_text = excluded.content_text,
            categories = excluded.categories,
            updated_at = CURRENT_TIMESTAMP
        "#,
    )
    .bind(&entry.id)
    .bind(&entry.feed_id)
    .bind(&entry.title)
    .bind(&entry.url)
    .bind(&entry.author)
    .bind(&entry.published)
    .bind(&entry.updated)
    .bind(&entry.summary)
    .bind(&entry.content_html)
    .bind(&entry.content_text)
    .bind(&entry.categories)
    .bind(entry.read)
    .bind(&entry.created_at)
    .bind(&entry.updated_at)
    .execute(pool)
    .await
    .context("Failed to upsert entry")?;
    Ok(())
}

/// Get an entry by ID
pub async fn get_entry(pool: &SqlitePool, id: &str) -> Result<Option<Entry>> {
    sqlx::query_as::<_, Entry>("SELECT * FROM entries WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await
        .context("Failed to get entry")
}

/// Get entries for a feed, ordered by published date descending
pub async fn get_entries_for_feed(
    pool: &SqlitePool,
    feed_id: &str,
    limit: i64,
) -> Result<Vec<Entry>> {
    sqlx::query_as::<_, Entry>(
        "SELECT * FROM entries WHERE feed_id = ? ORDER BY published DESC LIMIT ?",
    )
    .bind(feed_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .context("Failed to get entries for feed")
}

/// Get unread entries, ordered by published date descending
pub async fn get_unread_entries(pool: &SqlitePool, limit: i64) -> Result<Vec<Entry>> {
    sqlx::query_as::<_, Entry>(
        "SELECT * FROM entries WHERE read = 0 ORDER BY published DESC LIMIT ?",
    )
    .bind(limit)
    .fetch_all(pool)
    .await
    .context("Failed to get unread entries")
}

/// Mark an entry as read
pub async fn mark_read(pool: &SqlitePool, entry_id: &str) -> Result<()> {
    sqlx::query("UPDATE entries SET read = 1, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(entry_id)
        .execute(pool)
        .await
        .context("Failed to mark entry as read")?;
    Ok(())
}

/// Mark an entry as unread
pub async fn mark_unread(pool: &SqlitePool, entry_id: &str) -> Result<()> {
    sqlx::query("UPDATE entries SET read = 0, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(entry_id)
        .execute(pool)
        .await
        .context("Failed to mark entry as unread")?;
    Ok(())
}

// =============================================================================
// Summary Operations
// =============================================================================

/// Insert or update a summary
pub async fn upsert_summary(pool: &SqlitePool, summary: &Summary) -> Result<()> {
    sqlx::query(
        r#"
        INSERT INTO summaries (entry_id, summary_text, model, tokens, content_hash, created_at)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6)
        ON CONFLICT(entry_id) DO UPDATE SET
            summary_text = excluded.summary_text,
            model = excluded.model,
            tokens = excluded.tokens,
            content_hash = excluded.content_hash,
            created_at = excluded.created_at
        "#,
    )
    .bind(&summary.entry_id)
    .bind(&summary.summary_text)
    .bind(&summary.model)
    .bind(summary.tokens)
    .bind(&summary.content_hash)
    .bind(&summary.created_at)
    .execute(pool)
    .await
    .context("Failed to upsert summary")?;
    Ok(())
}

/// Get summary for an entry
pub async fn get_summary(pool: &SqlitePool, entry_id: &str) -> Result<Option<Summary>> {
    sqlx::query_as::<_, Summary>("SELECT * FROM summaries WHERE entry_id = ?")
        .bind(entry_id)
        .fetch_optional(pool)
        .await
        .context("Failed to get summary")
}

// =============================================================================
// Search and Statistics
// =============================================================================

/// Search entries using FTS5 full-text search
pub async fn search_entries(pool: &SqlitePool, query: &str, limit: i64) -> Result<Vec<Entry>> {
    sqlx::query_as::<_, Entry>(
        r#"
        SELECT e.*
        FROM entries e
        JOIN entries_fts fts ON e.rowid = fts.rowid
        WHERE entries_fts MATCH ?1
        ORDER BY bm25(entries_fts)
        LIMIT ?2
        "#,
    )
    .bind(query)
    .bind(limit)
    .fetch_all(pool)
    .await
    .context("Failed to search entries")
}

/// Get database statistics
pub async fn get_stats(pool: &SqlitePool) -> Result<DatabaseStats> {
    let row = sqlx::query(
        r#"
        SELECT
            (SELECT COUNT(*) FROM feeds) as total_feeds,
            (SELECT COUNT(*) FROM entries) as total_entries,
            (SELECT COUNT(*) FROM entries WHERE read = 0) as unread_entries,
            (SELECT COUNT(*) FROM summaries) as total_summaries
        "#,
    )
    .fetch_one(pool)
    .await
    .context("Failed to get database stats")?;

    Ok(DatabaseStats {
        total_feeds: row.get("total_feeds"),
        total_entries: row.get("total_entries"),
        unread_entries: row.get("unread_entries"),
        total_summaries: row.get("total_summaries"),
    })
}
