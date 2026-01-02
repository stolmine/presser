//! Scheduled tasks for presser

use std::sync::Arc;
use anyhow::Result;
use async_trait::async_trait;
use presser_scheduler::Task;

use crate::Engine;

/// Task that updates a single feed
pub struct FeedUpdateTask {
    engine: Arc<Engine>,
    feed_id: String,
}

impl FeedUpdateTask {
    /// Create a new feed update task
    pub fn new(engine: Arc<Engine>, feed_id: String) -> Self {
        Self { engine, feed_id }
    }
}

#[async_trait]
impl Task for FeedUpdateTask {
    async fn execute(&self) -> Result<()> {
        self.engine.update_feed(&self.feed_id).await
    }

    fn name(&self) -> &str {
        &self.feed_id
    }
}
