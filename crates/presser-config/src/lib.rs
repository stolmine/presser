//! Configuration management for Presser
//!
//! This crate handles all configuration loading, validation, and merging for Presser.
//! It supports a hierarchical configuration system where feed-specific configs can
//! override global settings.
//!
//! # Configuration Hierarchy
//!
//! 1. Global config: `~/.config/presser/global.toml`
//! 2. Feed-specific configs: `~/.config/presser/feeds/*.toml`
//! 3. Feed configs inherit from global and can override specific settings
//!
//! # Example
//!
//! ```rust,no_run
//! use presser_config::Config;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = Config::load()?;
//! println!("AI Provider: {:?}", config.ai.provider);
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub mod error;
pub mod validation;

pub use error::ConfigError;

/// Main configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Global settings
    pub global: GlobalConfig,

    /// AI configuration
    pub ai: AiConfig,

    /// Database configuration
    pub database: DatabaseConfig,

    /// Scheduler configuration
    pub scheduler: SchedulerConfig,

    /// Feed-specific configurations
    pub feeds: HashMap<String, FeedConfig>,
}

/// Global application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Maximum number of concurrent feed fetches
    #[serde(default = "default_max_concurrent_fetches")]
    pub max_concurrent_fetches: usize,

    /// Default fetch timeout in seconds
    #[serde(default = "default_fetch_timeout")]
    pub fetch_timeout_secs: u64,

    /// User agent string for HTTP requests
    #[serde(default = "default_user_agent")]
    pub user_agent: String,

    /// Enable content extraction (readability)
    #[serde(default = "default_true")]
    pub extract_content: bool,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            max_concurrent_fetches: default_max_concurrent_fetches(),
            fetch_timeout_secs: default_fetch_timeout(),
            user_agent: default_user_agent(),
            extract_content: default_true(),
        }
    }
}

/// AI provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    /// AI provider type
    pub provider: AiProvider,

    /// API key (for cloud providers)
    pub api_key: Option<String>,

    /// Model name
    pub model: String,

    /// API endpoint (for custom endpoints)
    pub endpoint: Option<String>,

    /// System prompt for summarization
    #[serde(default = "default_system_prompt")]
    pub system_prompt: String,

    /// Maximum tokens for response
    #[serde(default = "default_max_tokens")]
    pub max_tokens: u32,

    /// Temperature for generation
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Enable caching based on content hash
    #[serde(default = "default_true")]
    pub enable_cache: bool,
}

/// AI provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    OpenAI,
    Anthropic,
    Local,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Path to SQLite database file
    #[serde(default = "default_db_path")]
    pub path: PathBuf,

    /// Maximum number of database connections
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

/// Scheduler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Default update interval (cron expression)
    #[serde(default = "default_update_interval")]
    pub default_interval: String,

    /// Enable automatic updates
    #[serde(default = "default_true")]
    pub auto_update: bool,
}

/// Feed-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedConfig {
    /// Feed URL
    pub url: String,

    /// Feed name/title
    pub name: String,

    /// Custom update interval (overrides global)
    pub update_interval: Option<String>,

    /// Custom AI prompt for this feed
    pub custom_prompt: Option<String>,

    /// Whether to enable AI summarization for this feed
    #[serde(default = "default_true")]
    pub enable_ai: bool,

    /// Whether to extract full content
    pub extract_content: Option<bool>,

    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,

    /// Whether this feed is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// Intermediate struct for parsing global.toml
#[derive(Debug, Clone, Deserialize, Default)]
struct GlobalToml {
    #[serde(default)]
    global: GlobalConfig,
    #[serde(default)]
    ai: Option<AiConfig>,
    #[serde(default)]
    database: Option<DatabaseConfig>,
    #[serde(default)]
    scheduler: Option<SchedulerConfig>,
}

/// Intermediate struct for parsing feed TOML files
#[derive(Debug, Clone, Deserialize)]
struct FeedToml {
    #[serde(default)]
    feed: Vec<FeedConfig>,
}

impl Config {
    /// Load configuration from the default config directory
    ///
    /// This loads the global config and all feed configs from:
    /// - `~/.config/presser/global.toml`
    /// - `~/.config/presser/feeds/*.toml`
    pub fn load() -> Result<Self> {
        let config_dir = Self::config_dir()?;
        Self::load_from_dir(&config_dir)
    }

    /// Load configuration from a specific directory
    pub fn load_from_dir(dir: &Path) -> Result<Self> {
        let global_path = dir.join("global.toml");
        let feeds_dir = dir.join("feeds");

        // 1. Load global.toml (or use defaults if missing)
        let global_toml = if global_path.exists() {
            let content = std::fs::read_to_string(&global_path)
                .with_context(|| format!("Failed to read {}", global_path.display()))?;
            toml::from_str::<GlobalToml>(&content)
                .with_context(|| format!("Failed to parse {}", global_path.display()))?
        } else {
            GlobalToml::default()
        };

        // 2. Load feeds from feeds/ directory
        let mut feeds = HashMap::new();
        if feeds_dir.exists() && feeds_dir.is_dir() {
            for entry in std::fs::read_dir(&feeds_dir)
                .with_context(|| format!("Failed to read {}", feeds_dir.display()))?
            {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "toml") {
                    let content = std::fs::read_to_string(&path)
                        .with_context(|| format!("Failed to read {}", path.display()))?;
                    let feed_toml: FeedToml = toml::from_str(&content)
                        .with_context(|| format!("Failed to parse {}", path.display()))?;
                    for feed in feed_toml.feed {
                        feeds.insert(feed.url.clone(), feed);
                    }
                }
            }
        }

        // 3. Build final Config (AI defaults to Local if not configured)
        let ai = global_toml.ai.unwrap_or_else(|| AiConfig {
            provider: AiProvider::Local,
            api_key: None,
            model: "local".to_string(),
            endpoint: Some("http://localhost:8080".to_string()),
            system_prompt: default_system_prompt(),
            max_tokens: default_max_tokens(),
            temperature: default_temperature(),
            enable_cache: true,
        });

        let config = Config {
            global: global_toml.global,
            ai,
            database: global_toml.database.unwrap_or_else(|| DatabaseConfig {
                path: default_db_path(),
                max_connections: default_max_connections(),
            }),
            scheduler: global_toml.scheduler.unwrap_or_else(|| SchedulerConfig {
                default_interval: default_update_interval(),
                auto_update: default_true(),
            }),
            feeds,
        };

        // 4. Validate
        config.validate().map_err(|e| anyhow::anyhow!(e))?;
        Ok(config)
    }

    /// Get the default configuration directory
    pub fn config_dir() -> Result<PathBuf> {
        dirs::config_dir()
            .map(|p| p.join("presser"))
            .context("Could not determine config directory")
    }

    /// Get the feeds configuration directory
    pub fn feeds_dir() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("feeds"))
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), ConfigError> {
        validation::validate_config(self)
    }
}

// Default value functions
fn default_max_concurrent_fetches() -> usize { 10 }
fn default_fetch_timeout() -> u64 { 30 }
fn default_user_agent() -> String {
    format!("Presser/{}", env!("CARGO_PKG_VERSION"))
}
fn default_true() -> bool { true }
fn default_system_prompt() -> String {
    "You are a helpful assistant that creates concise summaries of articles. \
     Focus on key points and insights.".to_string()
}
fn default_max_tokens() -> u32 { 500 }
fn default_temperature() -> f32 { 0.7 }
fn default_db_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("presser")
        .join("presser.db")
}
fn default_max_connections() -> u32 { 5 }
fn default_update_interval() -> String { "0 0 */6 * * *".to_string() } // Every 6 hours (sec min hour day month weekday)

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_values() {
        assert_eq!(default_max_concurrent_fetches(), 10);
        assert_eq!(default_fetch_timeout(), 30);
        assert!(default_true());
    }

    #[test]
    fn test_load_from_dir_empty() {
        let temp_dir = TempDir::new().unwrap();
        let result = Config::load_from_dir(temp_dir.path());
        // Should succeed with default AI config
        assert!(result.is_ok());
        let config = result.unwrap();
        assert_eq!(config.ai.provider, AiProvider::Local);
    }

    #[test]
    fn test_load_from_dir_global_only() {
        let temp_dir = TempDir::new().unwrap();
        std::fs::write(
            temp_dir.path().join("global.toml"),
            r#"
[ai]
provider = "local"
model = "test-model"
endpoint = "http://localhost:8080"
"#,
        )
        .unwrap();

        let config = Config::load_from_dir(temp_dir.path()).unwrap();
        assert_eq!(config.ai.model, "test-model");
        assert!(config.feeds.is_empty());
    }

    #[test]
    fn test_load_from_dir_with_feeds() {
        let temp_dir = TempDir::new().unwrap();
        let feeds_dir = temp_dir.path().join("feeds");
        std::fs::create_dir(&feeds_dir).unwrap();

        std::fs::write(
            temp_dir.path().join("global.toml"),
            r#"
[ai]
provider = "local"
model = "test-model"
endpoint = "http://localhost:8080"
"#,
        )
        .unwrap();

        std::fs::write(
            feeds_dir.join("test.toml"),
            r#"
[[feed]]
url = "https://example.com/feed"
name = "Test Feed"
"#,
        )
        .unwrap();

        let config = Config::load_from_dir(temp_dir.path()).unwrap();
        assert_eq!(config.feeds.len(), 1);
        assert!(config.feeds.contains_key("https://example.com/feed"));
    }
}
