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
//! let (_metadata, entries) = fetcher.fetch("https://example.com/feed.xml").await?;
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

        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    FeedError::Timeout(url.to_string())
                } else {
                    FeedError::HttpError(e)
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            return Err(FeedError::HttpStatus {
                url: url.to_string(),
                status: status.as_u16(),
            }.into());
        }

        let bytes = response.bytes().await
            .map_err(FeedError::HttpError)?;

        let (mut metadata, entries) = self.parser.parse(&bytes)?;

        if metadata.url.is_empty() {
            metadata.url = url.to_string();
        }

        Ok((metadata, entries))
    }

    /// Fetch and parse a feed, extracting full content for each entry
    pub async fn fetch_with_content(&self, url: &str) -> Result<(FeedMetadata, Vec<FeedEntry>)> {
        let (metadata, mut entries) = self.fetch(url).await?;

        // Extract full content for each entry
        for entry in &mut entries {
            match self.extract_content(&entry.url).await {
                Ok(content) => entry.content_text = Some(content),
                Err(e) => tracing::warn!("Failed to extract content for {}: {}", entry.url, e),
            }
        }

        Ok((metadata, entries))
    }

    /// Extract full article content from a URL
    pub async fn extract_content(&self, url: &str) -> Result<String> {
        tracing::debug!("Extracting content from: {}", url);

        let response = self.client
            .get(url)
            .send()
            .await
            .map_err(FeedError::HttpError)?;

        let status = response.status();
        if !status.is_success() {
            return Err(FeedError::HttpStatus {
                url: url.to_string(),
                status: status.as_u16(),
            }.into());
        }

        let html = response.text().await
            .map_err(FeedError::HttpError)?;

        Ok(self.extractor.extract(&html, url)?)
    }

    /// Get a reference to the HTTP client
    pub fn client(&self) -> &reqwest::Client {
        &self.client
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
