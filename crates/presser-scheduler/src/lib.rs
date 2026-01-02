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
//! use async_trait::async_trait;
//!
//! struct FeedUpdater;
//!
//! #[async_trait]
//! impl Task for FeedUpdater {
//!     async fn execute(&self) -> anyhow::Result<()> {
//!         println!("Updating feed...");
//!         Ok(())
//!     }
//!     fn name(&self) -> &str { "feed-updater" }
//! }
//!
//! # async fn example() -> anyhow::Result<()> {
//! let scheduler = Scheduler::new(10)?;
//!
//! // Schedule a task to run every 6 hours (6-field cron: sec min hour day month weekday)
//! scheduler.schedule("feed-1", "0 0 */6 * * *", Arc::new(FeedUpdater)).await?;
//!
//! scheduler.start().await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock, Semaphore};
use tokio::task::JoinHandle;

pub mod error;
pub mod task;

pub use error::SchedulerError;
pub use task::Task;

/// Scheduler for managing periodic tasks
pub struct Scheduler {
    /// Scheduled tasks
    tasks: Arc<RwLock<HashMap<String, ScheduledTask>>>,

    /// Running task handles
    handles: Arc<RwLock<Vec<JoinHandle<()>>>>,

    /// Whether the scheduler is running
    running: Arc<RwLock<bool>>,

    /// Shutdown signal
    shutdown_tx: broadcast::Sender<()>,

    /// Concurrency limiter
    semaphore: Arc<Semaphore>,
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

        let (shutdown_tx, _) = broadcast::channel(1);
        let semaphore = Arc::new(Semaphore::new(max_concurrent));

        Ok(Self {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            handles: Arc::new(RwLock::new(Vec::new())),
            running: Arc::new(RwLock::new(false)),
            shutdown_tx,
            semaphore,
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

        let mut shutdown_rx = self.shutdown_tx.subscribe();

        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    tracing::info!("Scheduler received shutdown signal");
                    break;
                }
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    self.tick().await;
                }
            }
        }

        Ok(())
    }

    /// Process one scheduler tick
    async fn tick(&self) {
        let now = Utc::now();

        // Collect tasks to run while holding lock briefly
        let tasks_to_run: Vec<_> = {
            let mut tasks = self.tasks.write().await;
            tasks
                .values_mut()
                .filter_map(|task| {
                    if task.next_run <= now {
                        let executor = task.executor.clone();
                        let id = task.id.clone();

                        task.last_run = Some(now);
                        if let Some(next) = task.schedule.upcoming(Utc).next() {
                            task.next_run = next;
                        }

                        Some((id, executor))
                    } else {
                        None
                    }
                })
                .collect()
        };
        // Lock released here

        // Spawn tasks outside the lock
        let mut new_handles = Vec::new();
        for (id, executor) in tasks_to_run {
            let permit = match self.semaphore.clone().try_acquire_owned() {
                Ok(p) => p,
                Err(_) => {
                    tracing::debug!("Concurrency limit reached, skipping task: {}", id);
                    continue;
                }
            };

            tracing::debug!("Executing task: {}", id);

            let handle = tokio::spawn(async move {
                let _permit = permit;
                if let Err(e) = executor.execute().await {
                    tracing::error!("Task {} failed: {}", id, e);
                }
            });

            new_handles.push(handle);
        }

        // Store handles
        if !new_handles.is_empty() {
            let mut handles = self.handles.write().await;
            handles.extend(new_handles);
        }
    }

    /// Stop the scheduler
    ///
    /// This will gracefully shut down the scheduler and wait for running tasks
    pub async fn stop(&self) -> Result<()> {
        let _ = self.shutdown_tx.send(());

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

    #[tokio::test]
    async fn test_task_scheduling() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct CountingTask {
            count: Arc<AtomicUsize>,
        }

        #[async_trait::async_trait]
        impl Task for CountingTask {
            async fn execute(&self) -> Result<()> {
                self.count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
            fn name(&self) -> &str {
                "counter"
            }
        }

        let scheduler = Scheduler::new(2).unwrap();
        let count = Arc::new(AtomicUsize::new(0));

        scheduler
            .schedule(
                "test-task",
                "* * * * * *",
                Arc::new(CountingTask {
                    count: count.clone(),
                }),
            )
            .await
            .unwrap();

        assert_eq!(scheduler.task_count().await, 1);
    }

    #[tokio::test]
    async fn test_shutdown() {
        let scheduler = Scheduler::new(2).unwrap();

        let result = scheduler.stop().await;
        assert!(result.is_ok());
    }
}
