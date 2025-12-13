//! Database models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

/// Feed model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Feed {
    /// Unique feed identifier
    pub id: String,

    /// Feed URL
    pub url: String,

    /// Feed title
    pub title: String,

    /// Feed description
    pub description: Option<String>,

    /// Website URL
    pub site_url: Option<String>,

    /// Last fetch time
    pub last_fetched: Option<DateTime<Utc>>,

    /// Last successful fetch time
    pub last_successful_fetch: Option<DateTime<Utc>>,

    /// Last error message
    pub last_error: Option<String>,

    /// Number of entries
    pub entry_count: i64,

    /// Whether the feed is enabled
    pub enabled: bool,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

impl Default for Feed {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: String::new(),
            url: String::new(),
            title: String::new(),
            description: None,
            site_url: None,
            last_fetched: None,
            last_successful_fetch: None,
            last_error: None,
            entry_count: 0,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Entry/Article model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Entry {
    /// Unique entry identifier (usually URL or GUID)
    pub id: String,

    /// Feed ID this entry belongs to
    pub feed_id: String,

    /// Entry title
    pub title: String,

    /// Entry URL
    pub url: String,

    /// Author name
    pub author: Option<String>,

    /// Publication date
    pub published: Option<DateTime<Utc>>,

    /// Last updated date
    pub updated: Option<DateTime<Utc>>,

    /// Entry summary/description
    pub summary: Option<String>,

    /// Full content (HTML)
    pub content_html: Option<String>,

    /// Extracted clean text content
    pub content_text: Option<String>,

    /// Categories/tags (JSON array)
    pub categories: Option<String>,

    /// Whether this entry has been read
    pub read: bool,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Updated timestamp
    pub updated_at: DateTime<Utc>,
}

impl Default for Entry {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: String::new(),
            feed_id: String::new(),
            title: String::new(),
            url: String::new(),
            author: None,
            published: None,
            updated: None,
            summary: None,
            content_html: None,
            content_text: None,
            categories: None,
            read: false,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Summary model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Summary {
    /// Entry ID this summary belongs to
    pub entry_id: String,

    /// The summary text
    pub summary_text: String,

    /// AI model used
    pub model: String,

    /// Token count
    pub tokens: Option<i64>,

    /// Content hash (for caching)
    pub content_hash: String,

    /// Created timestamp
    pub created_at: DateTime<Utc>,
}

impl Default for Summary {
    fn default() -> Self {
        Self {
            entry_id: String::new(),
            summary_text: String::new(),
            model: String::new(),
            tokens: None,
            content_hash: String::new(),
            created_at: Utc::now(),
        }
    }
}
