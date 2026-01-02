//! AI integration for Presser
//!
//! This crate provides AI-powered summarization using various providers:
//! - OpenAI (GPT-4, GPT-3.5, etc.)
//! - Anthropic (Claude)
//! - Local LLMs (via llama.cpp)
//!
//! # Features
//!
//! - Multiple AI provider support
//! - Content-based caching to avoid redundant API calls
//! - Streaming responses (for supported providers)
//! - Customizable prompts and parameters
//!
//! # Example
//!
//! ```rust,no_run
//! use presser_ai::{AiClient, AiProvider, AiConfig};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = AiConfig {
//!     provider: AiProvider::OpenAI,
//!     api_key: Some("sk-...".to_string()),
//!     model: "gpt-4".to_string(),
//!     ..Default::default()
//! };
//!
//! let client = AiClient::new(config)?;
//! let summary = client.summarize("Long article content...").await?;
//! println!("Summary: {}", summary.text);
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod error;
pub mod providers;

pub use error::AiError;

/// AI provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiProvider {
    OpenAI,
    Anthropic,
    Local,
}

/// AI client configuration
#[derive(Debug, Clone)]
pub struct AiConfig {
    /// AI provider
    pub provider: AiProvider,

    /// API key (for cloud providers)
    pub api_key: Option<String>,

    /// Model name
    pub model: String,

    /// API endpoint (for custom endpoints)
    pub endpoint: Option<String>,

    /// System prompt
    pub system_prompt: String,

    /// Maximum tokens for response
    pub max_tokens: u32,

    /// Temperature for generation
    pub temperature: f32,

    /// Enable caching
    pub enable_cache: bool,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            provider: AiProvider::OpenAI,
            api_key: None,
            model: "gpt-4".to_string(),
            endpoint: None,
            system_prompt: "You are a helpful assistant that creates concise summaries.".to_string(),
            max_tokens: 500,
            temperature: 0.7,
            enable_cache: true,
        }
    }
}

/// AI client for summarization
pub struct AiClient {
    config: AiConfig,
    client: reqwest::Client,
    cache: Arc<RwLock<HashMap<String, String>>>,
}

/// Summary response from AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Summary {
    /// The generated summary
    pub text: String,

    /// Whether this was served from cache
    pub cached: bool,

    /// Model used for generation
    pub model: String,

    /// Token count (if available)
    pub tokens: Option<u32>,
}

impl AiClient {
    /// Create a new AI client with the given configuration
    pub fn new(config: AiConfig) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            config,
            client,
            cache: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Summarize the given content
    ///
    /// # Arguments
    ///
    /// * `content` - The content to summarize
    ///
    /// # Returns
    ///
    /// A `Summary` containing the generated summary and metadata
    pub async fn summarize(&self, content: &str) -> Result<Summary> {
        // Check cache first if enabled
        if self.config.enable_cache {
            let cache_key = self.cache_key(content);
            let cache = self.cache.read().await;

            if let Some(cached_summary) = cache.get(&cache_key) {
                tracing::debug!("Cache hit for content");
                return Ok(Summary {
                    text: cached_summary.clone(),
                    cached: true,
                    model: self.config.model.clone(),
                    tokens: None,
                });
            }
        }

        // Generate summary using the configured provider
        let summary = match self.config.provider {
            AiProvider::OpenAI => self.summarize_openai(content).await?,
            AiProvider::Anthropic => self.summarize_anthropic(content).await?,
            AiProvider::Local => self.summarize_local(content).await?,
        };

        // Cache the result if enabled
        if self.config.enable_cache {
            let cache_key = self.cache_key(content);
            let mut cache = self.cache.write().await;
            cache.insert(cache_key, summary.text.clone());
        }

        Ok(Summary {
            text: summary.text,
            cached: false,
            model: self.config.model.clone(),
            tokens: summary.tokens,
        })
    }

    /// Summarize using OpenAI API
    async fn summarize_openai(&self, content: &str) -> Result<Summary> {
        tracing::debug!("Generating summary using OpenAI");

        // TODO: Implement OpenAI API call
        // 1. Prepare request with system prompt and content
        // 2. Make API call to OpenAI
        // 3. Parse response and extract summary
        // 4. Return Summary with token count

        todo!("Implement OpenAI summarization")
    }

    /// Summarize using Anthropic API
    async fn summarize_anthropic(&self, content: &str) -> Result<Summary> {
        tracing::debug!("Generating summary using Anthropic");

        // TODO: Implement Anthropic API call
        // Similar to OpenAI but using Anthropic's API format

        todo!("Implement Anthropic summarization")
    }

    /// Summarize using local LLM
    async fn summarize_local(&self, content: &str) -> Result<Summary> {
        tracing::debug!("Generating summary using local LLM");

        // TODO: Implement local LLM inference
        // This will use llama-cpp-rs when the feature is enabled

        #[cfg(feature = "local-llm")]
        {
            todo!("Implement local LLM summarization")
        }

        #[cfg(not(feature = "local-llm"))]
        {
            anyhow::bail!("Local LLM support not enabled. Compile with --features local-llm")
        }
    }

    /// Generate a cache key for content
    fn cache_key(&self, content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        hasher.update(self.config.system_prompt.as_bytes());
        hasher.update(self.config.model.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Clear the cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        tracing::info!("Cleared AI cache");
    }

    /// Get cache size
    pub async fn cache_size(&self) -> usize {
        self.cache.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = AiConfig::default();
        assert_eq!(config.provider, AiProvider::OpenAI);
        assert_eq!(config.model, "gpt-4");
        assert!(config.enable_cache);
    }

    #[tokio::test]
    async fn test_client_creation() {
        let config = AiConfig::default();
        let client = AiClient::new(config);
        assert!(client.is_ok());
    }

    // TODO: Add more tests with mock API responses
}
