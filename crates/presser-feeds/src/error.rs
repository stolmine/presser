//! Error types for feed operations

use thiserror::Error;

/// Feed-specific errors
#[derive(Debug, Error)]
pub enum FeedError {
    /// HTTP request failed
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    /// Feed parsing failed
    #[error("Feed parsing failed: {0}")]
    ParseError(String),

    /// Content extraction failed
    #[error("Content extraction failed: {0}")]
    ExtractionError(String),

    /// Invalid feed URL
    #[error("Invalid feed URL: {0}")]
    InvalidUrl(String),

    /// Feed not found (404)
    #[error("Feed not found: {0}")]
    NotFound(String),

    /// Timeout
    #[error("Request timeout for: {0}")]
    Timeout(String),

    /// Generic error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
