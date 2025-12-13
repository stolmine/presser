//! Error types for configuration management

use thiserror::Error;

/// Configuration-specific errors
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Invalid configuration value
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Missing required field
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Invalid URL format
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    /// Invalid cron expression
    #[error("Invalid cron expression: {0}")]
    InvalidCron(String),

    /// Invalid AI provider
    #[error("Invalid AI provider: {0}")]
    InvalidProvider(String),

    /// File I/O error
    #[error("File error: {0}")]
    FileError(#[from] std::io::Error),

    /// TOML parsing error
    #[error("TOML parse error: {0}")]
    TomlError(#[from] toml::de::Error),

    /// Generic error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
