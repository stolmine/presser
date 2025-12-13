//! Feed fetching and parsing for Presser
//!
//! This crate handles fetching RSS/Atom feeds, parsing them, and extracting
//! full article content using readability algorithms.
//!
//! # Features
//!
//! - Fetch RSS and Atom feeds
//! - Parse feed entries
//! - Extract full article content from HTML
//! - Convert HTML to clean text
//! - Handle various feed formats and edge cases
//!
//! # Example
//!
//! ```rust,no_run
//! use presser_feeds::{FeedFetcher, FeedEntry};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let fetcher = FeedFetcher::new()?;
//! let entries = fetcher.fetch("https://example.com/feed.xml").await?;
//! for entry in entries {
//!     println!("{}: {}", entry.title, entry.url);
//! }
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub mod error;
pub mod extractor;
pub mod parser;

pub use error::FeedError;
pub use extractor::ContentExtractor;
pub use parser::FeedParser;

/// Feed fetcher that handles HTTP requests and parsing
pub struct FeedFetcher {
    client: reqwest::Client,
    parser: FeedParser,
    extractor: ContentExtractor,
}

/// Represents a single feed entry/article
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedEntry {
    /// Unique identifier for the entry
    pub id: String,

    /// Entry title
    pub title: String,

    /// Entry URL
    pub url: String,

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

    /// Author name
    pub author: Option<String>,

    /// Categories/tags
    pub categories: Vec<String>,
}

/// Feed metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedMetadata {
    /// Feed title
    pub title: String,

    /// Feed description
    pub description: Option<String>,

    /// Feed URL
    pub url: String,

    /// Feed website URL
    pub site_url: Option<String>,

    /// Last updated time
    pub last_updated: Option<DateTime<Utc>>,
}

impl FeedFetcher {
    /// Create a new feed fetcher with default settings
    pub fn new() -> Result<Self> {
        Self::with_timeout(Duration::from_secs(30))
    }

    /// Create a new feed fetcher with custom timeout
    pub fn with_timeout(timeout: Duration) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(timeout)
            .user_agent(format!("Presser/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            parser: FeedParser::new(),
            extractor: ContentExtractor::new(),
        })
    }

    /// Fetch and parse a feed from the given URL
    ///
    /// Returns the feed metadata and list of entries
    pub async fn fetch(&self, url: &str) -> Result<(FeedMetadata, Vec<FeedEntry>)> {
        tracing::info!("Fetching feed: {}", url);

        // TODO: Implement feed fetching
        // 1. Make HTTP GET request
        // 2. Parse response body as RSS/Atom
        // 3. Convert to FeedMetadata and Vec<FeedEntry>
        // 4. Optionally extract full content for each entry

        todo!("Implement feed fetching for {}", url)
    }

    /// Fetch and parse a feed, extracting full content for each entry
    pub async fn fetch_with_content(&self, url: &str) -> Result<(FeedMetadata, Vec<FeedEntry>)> {
        let (metadata, mut entries) = self.fetch(url).await?;

        // Extract full content for each entry
        for entry in &mut entries {
            if let Ok(content) = self.extract_content(&entry.url).await {
                entry.content_text = Some(content);
            }
        }

        Ok((metadata, entries))
    }

    /// Extract full article content from a URL
    pub async fn extract_content(&self, url: &str) -> Result<String> {
        tracing::debug!("Extracting content from: {}", url);

        // TODO: Implement content extraction
        // 1. Fetch HTML from URL
        // 2. Apply readability algorithm
        // 3. Convert to clean text
        // 4. Return extracted content

        todo!("Implement content extraction for {}", url)
    }

    /// Get a reference to the HTTP client
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
}

impl Default for FeedFetcher {
    fn default() -> Self {
        Self::new().expect("Failed to create default FeedFetcher")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fetcher_creation() {
        let fetcher = FeedFetcher::new();
        assert!(fetcher.is_ok());
    }

    // TODO: Add more tests with mock HTTP responses
}
