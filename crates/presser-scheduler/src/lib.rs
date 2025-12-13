//! Scheduling engine for Presser
//!
//! This crate provides task scheduling functionality for periodic feed updates.
//! It uses cron expressions to define update schedules and manages the execution
//! of feed update tasks.
//!
//! # Features
//!
//! - Cron-based scheduling
//! - Per-feed custom schedules
//! - Concurrent task execution with limits
//! - Task cancellation and cleanup
//!
//! # Example
//!
//! ```rust,no_run
//! use presser_scheduler::{Scheduler, Task};
//! use std::sync::Arc;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let scheduler = Scheduler::new(10)?;
//!
//! // Schedule a task to run every 6 hours
//! scheduler.schedule("feed-1", "0 */6 * * *", || async {
//!     println!("Updating feed...");
//!     Ok(())
//! }).await?;
//!
//! scheduler.start().await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

pub mod error;
pub mod task;

pub use error::SchedulerError;
pub use task::Task;

/// Scheduler for managing periodic tasks
pub struct Scheduler {
    /// Maximum concurrent tasks
    max_concurrent: usize,

    /// Scheduled tasks
    tasks: Arc<RwLock<HashMap<String, ScheduledTask>>>,

    /// Running task handles
    handles: Arc<RwLock<Vec<JoinHandle<()>>>>,

    /// Whether the scheduler is running
    running: Arc<RwLock<bool>>,
}

/// A scheduled task with its cron schedule
struct ScheduledTask {
    /// Task ID
    id: String,

    /// Cron schedule
    schedule: cron::Schedule,

    /// Last execution time
    last_run: Option<DateTime<Utc>>,

    /// Next execution time
    next_run: DateTime<Utc>,

    /// Task execution function
    executor: Arc<dyn Task>,
}

impl Scheduler {
    /// Create a new scheduler with the given concurrency limit
    pub fn new(max_concurrent: usize) -> Result<Self> {
        if max_concurrent == 0 {
            anyhow::bail!("max_concurrent must be greater than 0");
        }

        Ok(Self {
            max_concurrent,
            tasks: Arc::new(RwLock::new(HashMap::new())),
            handles: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
        })
    }

    /// Add a task to the scheduler
    ///
    /// # Arguments
    ///
    /// * `id` - Unique identifier for the task
    /// * `schedule` - Cron expression (e.g., "0 */6 * * *" for every 6 hours)
    /// * `executor` - Task implementation
    pub async fn schedule(
        &self,
        id: impl Into<String>,
        schedule: &str,
        executor: Arc<dyn Task>,
    ) -> Result<()> {
        let id = id.into();

        // Parse cron schedule
        let schedule: cron::Schedule = schedule
            .parse()
            .context("Failed to parse cron expression")?;

        let next_run = schedule
            .upcoming(Utc)
            .next()
            .context("Failed to calculate next run time")?;

        let task = ScheduledTask {
            id: id.clone(),
            schedule,
            last_run: None,
            next_run,
            executor,
        };

        tracing::info!("Scheduled task: {}", id);

        let mut tasks = self.tasks.write().await;
        tasks.insert(id, task);

        Ok(())
    }

    /// Remove a task from the scheduler
    pub async fn unschedule(&self, id: &str) -> Result<()> {
        let mut tasks = self.tasks.write().await;
        tasks.remove(id);
        tracing::info!("Unscheduled task: {}", id);
        Ok(())
    }

    /// Start the scheduler
    ///
    /// This will begin executing tasks according to their schedules
    pub async fn start(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if *running {
            anyhow::bail!("Scheduler is already running");
        }
        *running = true;
        drop(running);

        tracing::info!("Starting scheduler");

        // TODO: Implement scheduler main loop
        // 1. Check for tasks that need to run
        // 2. Execute tasks respecting concurrency limits
        // 3. Update next_run times
        // 4. Handle errors and retries

        todo!("Implement scheduler main loop")
    }

    /// Stop the scheduler
    ///
    /// This will gracefully shut down the scheduler and wait for running tasks
    pub async fn stop(&self) -> Result<()> {
        let mut running = self.running.write().await;
        if !*running {
            return Ok(());
        }
        *running = false;
        drop(running);

        tracing::info!("Stopping scheduler");

        // Wait for all running tasks to complete
        let mut handles = self.handles.write().await;
        for handle in handles.drain(..) {
            handle.await?;
        }

        Ok(())
    }

    /// Get the number of scheduled tasks
    pub async fn task_count(&self) -> usize {
        self.tasks.read().await.len()
    }

    /// Check if the scheduler is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = Scheduler::new(10);
        assert!(scheduler.is_ok());
    }

    #[tokio::test]
    async fn test_scheduler_zero_concurrency() {
        let scheduler = Scheduler::new(0);
        assert!(scheduler.is_err());
    }

    // TODO: Add more tests
}
