//! Error types for AI operations

use thiserror::Error;

/// AI-specific errors
#[derive(Debug, Error)]
pub enum AiError {
    /// API request failed
    #[error("API request failed: {0}")]
    ApiError(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    AuthError(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimitError(String),

    /// Invalid response from API
    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    /// Model not available
    #[error("Model not available: {0}")]
    ModelNotAvailable(String),

    /// Local LLM error
    #[error("Local LLM error: {0}")]
    LocalLlmError(String),

    /// HTTP error
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Generic error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
