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
        let _global_path = dir.join("global.toml");

        // TODO: Implement configuration loading
        // 1. Read and parse global.toml
        // 2. Read and parse all files in feeds/ directory
        // 3. Merge feed configs with global defaults
        // 4. Validate the resulting configuration

        todo!("Implement config loading from {}", dir.display())
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
fn default_update_interval() -> String { "0 */6 * * *".to_string() } // Every 6 hours

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        assert_eq!(default_max_concurrent_fetches(), 10);
        assert_eq!(default_fetch_timeout(), 30);
        assert!(default_true());
    }
}
