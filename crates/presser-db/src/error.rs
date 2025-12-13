//! Error types for database operations

use thiserror::Error;

/// Database-specific errors
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// Record not found
    #[error("Record not found: {0}")]
    NotFound(String),

    /// Duplicate key/constraint violation
    #[error("Duplicate record: {0}")]
    Duplicate(String),

    /// Migration failed
    #[error("Migration failed: {0}")]
    MigrationError(String),

    /// SQLx error
    #[error("Database error: {0}")]
    SqlxError(#[from] sqlx::Error),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Generic error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
