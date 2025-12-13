//! Task trait and implementations

use anyhow::Result;
use async_trait::async_trait;

/// Trait for executable tasks
#[async_trait]
pub trait Task: Send + Sync {
    /// Execute the task
    async fn execute(&self) -> Result<()>;

    /// Get the task name/description
    fn name(&self) -> &str;
}

/// Example task implementation for feed updates
pub struct FeedUpdateTask {
    feed_id: String,
}

impl FeedUpdateTask {
    /// Create a new feed update task
    pub fn new(feed_id: String) -> Self {
        Self { feed_id }
    }
}

#[async_trait]
impl Task for FeedUpdateTask {
    async fn execute(&self) -> Result<()> {
        tracing::info!("Executing feed update for: {}", self.feed_id);

        // TODO: Implement actual feed update logic
        // This will call into presser-feeds and presser-db

        Ok(())
    }

    fn name(&self) -> &str {
        &self.feed_id
    }
}
