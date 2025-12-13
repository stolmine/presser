//! Configuration validation

use crate::{Config, ConfigError};
use url::Url;

/// Validate the entire configuration
pub fn validate_config(config: &Config) -> Result<(), ConfigError> {
    // Validate global settings
    validate_global(&config.global)?;

    // Validate AI settings
    validate_ai(&config.ai)?;

    // Validate scheduler settings
    validate_scheduler(&config.scheduler)?;

    // Validate each feed
    for (feed_id, feed) in &config.feeds {
        validate_feed(feed_id, feed)?;
    }

    Ok(())
}

/// Validate global configuration
fn validate_global(global: &crate::GlobalConfig) -> Result<(), ConfigError> {
    if global.max_concurrent_fetches == 0 {
        return Err(ConfigError::InvalidConfig(
            "max_concurrent_fetches must be greater than 0".to_string(),
        ));
    }

    if global.fetch_timeout_secs == 0 {
        return Err(ConfigError::InvalidConfig(
            "fetch_timeout_secs must be greater than 0".to_string(),
        ));
    }

    Ok(())
}

/// Validate AI configuration
fn validate_ai(ai: &crate::AiConfig) -> Result<(), ConfigError> {
    // Validate API key for cloud providers
    match ai.provider {
        crate::AiProvider::OpenAI | crate::AiProvider::Anthropic => {
            if ai.api_key.is_none() {
                return Err(ConfigError::InvalidConfig(
                    format!("{:?} provider requires an API key", ai.provider),
                ));
            }
        }
        crate::AiProvider::Local => {
            // Local provider may need an endpoint
            if ai.endpoint.is_none() {
                return Err(ConfigError::InvalidConfig(
                    "Local provider requires an endpoint".to_string(),
                ));
            }
        }
    }

    // Validate endpoint URL if provided
    if let Some(endpoint) = &ai.endpoint {
        Url::parse(endpoint)
            .map_err(|_| ConfigError::InvalidUrl(endpoint.clone()))?;
    }

    // Validate temperature range
    if !(0.0..=2.0).contains(&ai.temperature) {
        return Err(ConfigError::InvalidConfig(
            "temperature must be between 0.0 and 2.0".to_string(),
        ));
    }

    Ok(())
}

/// Validate scheduler configuration
fn validate_scheduler(scheduler: &crate::SchedulerConfig) -> Result<(), ConfigError> {
    // TODO: Validate cron expression syntax
    // For now, just check it's not empty
    if scheduler.default_interval.is_empty() {
        return Err(ConfigError::InvalidCron(
            "default_interval cannot be empty".to_string(),
        ));
    }

    Ok(())
}

/// Validate feed configuration
fn validate_feed(feed_id: &str, feed: &crate::FeedConfig) -> Result<(), ConfigError> {
    // Validate URL
    Url::parse(&feed.url)
        .map_err(|_| ConfigError::InvalidUrl(feed.url.clone()))?;

    // Validate name is not empty
    if feed.name.is_empty() {
        return Err(ConfigError::MissingField(
            format!("Feed '{}' must have a name", feed_id),
        ));
    }

    // Validate custom interval if provided
    if let Some(interval) = &feed.update_interval {
        if interval.is_empty() {
            return Err(ConfigError::InvalidCron(
                format!("Feed '{}' has empty update_interval", feed_id),
            ));
        }
        // TODO: Validate cron expression syntax
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    #[test]
    fn test_validate_global_invalid_concurrent_fetches() {
        let global = GlobalConfig {
            max_concurrent_fetches: 0,
            ..Default::default()
        };

        assert!(validate_global(&global).is_err());
    }

    // TODO: Add more validation tests
}
