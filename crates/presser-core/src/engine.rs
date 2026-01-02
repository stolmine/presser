//! Core engine that orchestrates all components

use anyhow::Result;
use presser_ai::AiClient;
use presser_config::Config;
use presser_db::Database;
use presser_feeds::FeedFetcher;
use presser_scheduler::Scheduler;

/// Main application engine
pub struct Engine {
    config: Config,
    db: Database,
    fetcher: FeedFetcher,
    ai: AiClient,
    scheduler: Option<Scheduler>,
}

impl Engine {
    /// Create a new engine instance
    pub async fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .map(|d| d.join("presser"))
            .unwrap_or_else(|| std::path::PathBuf::from("."));

        let config = Config::load_from_dir(&config_dir)?;
        Self::with_config(config).await
    }

    /// Initialize from custom config
    pub async fn with_config(config: Config) -> Result<Self> {
        let db_path = config.database.path.clone();

        let db = Database::open(&db_path).await?;
        db.migrate().await?;

        let fetcher = FeedFetcher::new()?;

        let ai_config = presser_ai::AiConfig {
            provider: match config.ai.provider {
                presser_config::AiProvider::OpenAI => presser_ai::AiProvider::OpenAI,
                presser_config::AiProvider::Anthropic => presser_ai::AiProvider::Anthropic,
                presser_config::AiProvider::Local => presser_ai::AiProvider::Local,
            },
            api_key: config.ai.api_key.clone(),
            model: config.ai.model.clone(),
            endpoint: config.ai.endpoint.clone(),
            system_prompt: config.ai.system_prompt.clone(),
            max_tokens: config.ai.max_tokens,
            temperature: config.ai.temperature,
            enable_cache: config.ai.enable_cache,
        };
        let ai = AiClient::new(ai_config)?;

        Ok(Self {
            config,
            db,
            fetcher,
            ai,
            scheduler: None,
        })
    }

    /// Update a single feed
    pub async fn update_feed(&self, feed_id: &str) -> Result<()> {
        tracing::info!("Updating feed: {}", feed_id);

        let feed = self.db.get_feed(feed_id).await?
            .ok_or_else(|| anyhow::anyhow!("Feed not found: {}", feed_id))?;

        let fetch_result = self.fetcher.fetch(&feed.url).await;

        match fetch_result {
            Ok((metadata, entries)) => {
                let updated_feed = presser_db::Feed {
                    title: metadata.title,
                    description: metadata.description,
                    site_url: metadata.site_url,
                    last_fetched: Some(chrono::Utc::now()),
                    last_successful_fetch: Some(chrono::Utc::now()),
                    last_error: None,
                    entry_count: entries.len() as i64,
                    ..feed
                };
                self.db.upsert_feed(&updated_feed).await?;

                for entry in entries {
                    let db_entry = presser_db::Entry {
                        id: entry.id,
                        feed_id: feed_id.to_string(),
                        title: entry.title,
                        url: entry.url,
                        author: entry.author,
                        published: entry.published,
                        updated: entry.updated,
                        summary: entry.summary,
                        content_html: entry.content_html,
                        content_text: entry.content_text,
                        categories: if entry.categories.is_empty() {
                            None
                        } else {
                            Some(serde_json::to_string(&entry.categories)?)
                        },
                        ..Default::default()
                    };
                    self.db.upsert_entry(&db_entry).await?;
                }

                tracing::info!("Feed {} updated with {} entries", feed_id, updated_feed.entry_count);
            }
            Err(e) => {
                let updated_feed = presser_db::Feed {
                    last_fetched: Some(chrono::Utc::now()),
                    last_error: Some(e.to_string()),
                    ..feed
                };
                self.db.upsert_feed(&updated_feed).await?;
                return Err(e.into());
            }
        }

        Ok(())
    }

    /// Update all feeds
    pub async fn update_all_feeds(&self) -> Result<()> {
        let feeds = self.db.get_all_feeds().await?;
        for feed in feeds {
            if feed.enabled {
                if let Err(e) = self.update_feed(&feed.id).await {
                    tracing::warn!("Failed to update feed {}: {}", feed.id, e);
                }
            }
        }
        Ok(())
    }

    /// Generate a digest
    pub async fn generate_digest(&self, days: u32) -> Result<String> {
        todo!("Implement generate_digest")
    }

    /// Get database reference
    pub fn database(&self) -> &Database {
        &self.db
    }

    /// Get config reference
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Get fetcher reference
    pub fn fetcher(&self) -> &FeedFetcher {
        &self.fetcher
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use presser_config::{AiConfig, AiProvider, DatabaseConfig, GlobalConfig, SchedulerConfig};
    use std::collections::HashMap;
    use tempfile::TempDir;

    async fn create_test_engine() -> (Engine, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = Config {
            global: GlobalConfig::default(),
            ai: AiConfig {
                provider: AiProvider::Local,
                api_key: None,
                model: "test-model".to_string(),
                endpoint: Some("http://localhost:8080".to_string()),
                system_prompt: "test prompt".to_string(),
                max_tokens: 100,
                temperature: 0.7,
                enable_cache: true,
            },
            database: DatabaseConfig {
                path: db_path,
                max_connections: 5,
            },
            scheduler: SchedulerConfig {
                default_interval: "0 0 */6 * * *".to_string(),
                auto_update: true,
            },
            feeds: HashMap::new(),
        };

        let engine = Engine::with_config(config).await.unwrap();
        (engine, temp_dir)
    }

    #[tokio::test]
    async fn test_engine_creation() {
        let (_engine, _temp_dir) = create_test_engine().await;
    }

    #[tokio::test]
    async fn test_update_all_feeds_empty() {
        let (engine, _temp_dir) = create_test_engine().await;
        let result = engine.update_all_feeds().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_feed_not_found() {
        let (engine, _temp_dir) = create_test_engine().await;
        let result = engine.update_feed("nonexistent").await;
        assert!(result.is_err());
    }
}
