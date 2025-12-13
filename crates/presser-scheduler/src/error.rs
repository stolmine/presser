//! Error types for scheduling

use thiserror::Error;

/// Scheduler-specific errors
#[derive(Debug, Error)]
pub enum SchedulerError {
    /// Invalid cron expression
    #[error("Invalid cron expression: {0}")]
    InvalidCron(String),

    /// Task not found
    #[error("Task not found: {0}")]
    TaskNotFound(String),

    /// Task execution failed
    #[error("Task execution failed: {0}")]
    ExecutionError(String),

    /// Scheduler not running
    #[error("Scheduler is not running")]
    NotRunning,

    /// Generic error
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
