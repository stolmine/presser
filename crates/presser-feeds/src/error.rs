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

    /// HTTP status error (4xx, 5xx)
    #[error("HTTP {status} for: {url}")]
    HttpStatus { url: String, status: u16 },

    /// Timeout
    #[error("Request timeout for: {0}")]
    Timeout(String),

    /// Generic error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
