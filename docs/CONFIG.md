# Presser Configuration Guide

This guide covers all configuration options for Presser.

## Configuration File Locations

Presser uses a hierarchical configuration system:

**Linux/macOS**:
- Global config: `~/.config/presser/global.toml`
- Feed configs: `~/.config/presser/feeds/*.toml`
- Database: `~/.local/share/presser/presser.db`

**Windows**:
- Global config: `%APPDATA%\presser\global.toml`
- Feed configs: `%APPDATA%\presser\feeds\*.toml`
- Database: `%APPDATA%\presser\presser.db`

## Global Configuration

The global configuration file (`global.toml`) contains default settings that apply to all feeds unless overridden.

### Complete Example

```toml
[global]
max_concurrent_fetches = 10
fetch_timeout_secs = 30
user_agent = "Presser/0.1.0"
extract_content = true

[ai]
provider = "openai"
model = "gpt-4"
system_prompt = "Create concise summaries..."
max_tokens = 500
temperature = 0.7
enable_cache = true

[database]
max_connections = 5

[scheduler]
default_interval = "0 0 */6 * * *"
auto_update = true
```

### Global Section

#### `max_concurrent_fetches`

- **Type**: Integer
- **Default**: `10`
- **Description**: Maximum number of feeds to fetch simultaneously
- **Example**: `max_concurrent_fetches = 5`

#### `fetch_timeout_secs`

- **Type**: Integer
- **Default**: `30`
- **Description**: Timeout for HTTP requests in seconds
- **Example**: `fetch_timeout_secs = 60`

#### `user_agent`

- **Type**: String
- **Default**: `"Presser/0.1.0"`
- **Description**: User agent string for HTTP requests
- **Example**: `user_agent = "MyPresser/1.0"`

#### `extract_content`

- **Type**: Boolean
- **Default**: `true`
- **Description**: Enable content extraction using readability by default
- **Example**: `extract_content = false`

### AI Section

#### `provider`

- **Type**: String (enum)
- **Required**: Yes
- **Options**: `"openai"`, `"anthropic"`, `"local"`
- **Description**: AI provider to use for summarization
- **Example**: `provider = "anthropic"`

#### `api_key`

- **Type**: String
- **Required**: Yes (for cloud providers)
- **Description**: API key for the chosen provider
- **Environment Variables**:
  - OpenAI: `OPENAI_API_KEY`
  - Anthropic: `ANTHROPIC_API_KEY`
- **Example**: `api_key = "sk-..."`
- **Note**: It's recommended to use environment variables instead of hardcoding keys

#### `model`

- **Type**: String
- **Required**: Yes
- **Description**: Model to use for generation
- **Examples**:
  - OpenAI: `"gpt-4"`, `"gpt-4-turbo-preview"`, `"gpt-3.5-turbo"`
  - Anthropic: `"claude-3-opus-20240229"`, `"claude-3-sonnet-20240229"`, `"claude-3-haiku-20240307"`
  - Local: Model name or path

#### `endpoint`

- **Type**: String (URL)
- **Required**: No (Yes for local provider)
- **Description**: Custom API endpoint
- **Use Cases**:
  - Local LLM servers
  - OpenAI-compatible APIs
  - Custom proxies
- **Example**: `endpoint = "http://localhost:8080/v1"`

#### `system_prompt`

- **Type**: String
- **Default**: `"You are a helpful assistant that creates concise summaries..."`
- **Description**: System prompt for AI summarization
- **Example**:

```toml
system_prompt = """You are an expert at summarizing technical articles.
Focus on key insights, methodologies, and practical takeaways.
Keep summaries concise but informative."""
```

#### `max_tokens`

- **Type**: Integer
- **Default**: `500`
- **Description**: Maximum tokens in AI response
- **Range**: `1` to `4096` (varies by model)
- **Example**: `max_tokens = 300`

#### `temperature`

- **Type**: Float
- **Default**: `0.7`
- **Range**: `0.0` to `2.0`
- **Description**: Creativity/randomness of generation
  - `0.0`: Deterministic, consistent
  - `1.0`: Balanced
  - `2.0`: Very creative, varied
- **Example**: `temperature = 0.5`

#### `enable_cache`

- **Type**: Boolean
- **Default**: `true`
- **Description**: Cache summaries by content hash to avoid redundant API calls
- **Example**: `enable_cache = false`

### Database Section

#### `path`

- **Type**: String (path)
- **Default**: Platform-specific (see above)
- **Description**: Path to SQLite database file
- **Example**: `path = "/custom/path/presser.db"`

#### `max_connections`

- **Type**: Integer
- **Default**: `5`
- **Description**: Maximum database connections in pool
- **Example**: `max_connections = 10`

### Scheduler Section

#### `default_interval`

- **Type**: String (cron expression)
- **Default**: `"0 0 */6 * * *"` (every 6 hours)
- **Description**: Default update interval for feeds
- **Examples**:
  - `"0 0 */3 * * *"` - Every 3 hours
  - `"0 0 9,17 * * *"` - 9 AM and 5 PM daily
  - `"0 0 0 * * *"` - Daily at midnight
  - `"0 0 9 * * 1-5"` - 9 AM on weekdays

#### `auto_update`

- **Type**: Boolean
- **Default**: `true`
- **Description**: Enable automatic updates via scheduler
- **Example**: `auto_update = false`

## Feed Configuration

Feed-specific configuration files override global settings. You can have multiple feed configs, typically organized by topic or source.

### Feed Config Structure

```toml
[[feed]]
url = "https://example.com/feed.xml"
name = "Example Feed"
tags = ["tech", "news"]
enabled = true
update_interval = "0 0 */2 * * *"
custom_prompt = "Custom summarization instructions..."
extract_content = true
enable_ai = true
```

### Feed Fields

#### `url`

- **Type**: String (URL)
- **Required**: Yes
- **Description**: RSS or Atom feed URL
- **Example**: `url = "https://hnrss.org/frontpage"`

#### `name`

- **Type**: String
- **Required**: Yes
- **Description**: Human-readable feed name
- **Example**: `name = "Hacker News"`

#### `tags`

- **Type**: Array of strings
- **Default**: `[]`
- **Description**: Tags for categorization and filtering
- **Example**: `tags = ["tech", "programming", "startup"]`

#### `enabled`

- **Type**: Boolean
- **Default**: `true`
- **Description**: Whether this feed is active
- **Example**: `enabled = false`

#### `update_interval`

- **Type**: String (cron expression)
- **Default**: From global config
- **Description**: Custom update interval for this feed
- **Example**: `update_interval = "0 0 */2 * * *"`

#### `custom_prompt`

- **Type**: String
- **Default**: From global config
- **Description**: Custom AI prompt for this feed
- **Example**:

```toml
custom_prompt = """Summarize this Hacker News article.
Focus on the technical innovation and why it matters to developers."""
```

#### `extract_content`

- **Type**: Boolean
- **Default**: From global config
- **Description**: Whether to extract full article content
- **Example**: `extract_content = false`

#### `enable_ai`

- **Type**: Boolean
- **Default**: `true`
- **Description**: Whether to generate AI summaries for this feed
- **Example**: `enable_ai = false`

## Cron Expression Reference

Cron expressions use the 6-field format (with seconds):

```
┌───────────── second (0 - 59)
│ ┌───────────── minute (0 - 59)
│ │ ┌───────────── hour (0 - 23)
│ │ │ ┌───────────── day of month (1 - 31)
│ │ │ │ ┌───────────── month (1 - 12)
│ │ │ │ │ ┌───────────── day of week (0 - 6) (Sunday = 0)
│ │ │ │ │ │
* * * * * *
```

### Common Patterns

- `0 0 */6 * * *` - Every 6 hours
- `0 0 */3 * * *` - Every 3 hours
- `0 0 */2 * * *` - Every 2 hours
- `0 0 * * * *` - Every hour
- `0 30 * * * *` - Every hour at 30 minutes past
- `0 0 9 * * *` - Daily at 9 AM
- `0 0 9,17 * * *` - Daily at 9 AM and 5 PM
- `0 0 0 * * *` - Daily at midnight
- `0 0 0 * * 0` - Weekly on Sunday
- `0 0 9 * * 1` - Weekly on Monday at 9 AM
- `0 0 9 * * 1-5` - Weekdays at 9 AM
- `0 0 0 1 * *` - Monthly on the 1st

## Example Configurations

### Tech News Setup

```toml
# global.toml
[ai]
provider = "openai"
model = "gpt-4"
temperature = 0.7
```

```toml
# feeds/tech.toml
[[feed]]
url = "https://hnrss.org/frontpage"
name = "Hacker News"
tags = ["tech", "programming"]
update_interval = "0 0 */2 * * *"

[[feed]]
url = "https://techcrunch.com/feed/"
name = "TechCrunch"
tags = ["startup", "venture"]
update_interval = "0 0 */3 * * *"
```

### Newsletter Setup

```toml
# feeds/newsletters.toml
[[feed]]
url = "https://tldr.tech/tech/rss"
name = "TLDR Newsletter"
tags = ["newsletter", "curated"]
update_interval = "0 0 9 * * *"  # Daily at 9 AM
extract_content = false  # Already summarized

[[feed]]
url = "https://bytes.dev/rss.xml"
name = "Bytes (JavaScript)"
tags = ["newsletter", "javascript"]
update_interval = "0 0 9 * * 1,4"  # Monday and Thursday
```

### Local LLM Setup

```toml
# global.toml
[ai]
provider = "local"
endpoint = "http://localhost:8080/v1"
model = "llama-2-7b-chat"
temperature = 0.8
max_tokens = 400
```

### Privacy-Focused Setup

```toml
# global.toml
[ai]
provider = "local"
endpoint = "http://localhost:8080/v1"
model = "mistral-7b-instruct"

# Don't extract full content (reduces external requests)
[global]
extract_content = false
```

## Environment Variables

### API Keys

```bash
# OpenAI
export OPENAI_API_KEY="sk-..."

# Anthropic
export ANTHROPIC_API_KEY="sk-ant-..."
```

### Override Config Path

```bash
# Custom config directory
export PRESSER_CONFIG_DIR="/custom/path/to/config"

# Custom database path
export PRESSER_DB_PATH="/custom/path/to/presser.db"
```

## Configuration Validation

Presser validates configuration at startup. Common validation errors:

- **Missing API key**: Cloud providers require an API key
- **Invalid URL**: Feed URLs must be valid HTTP(S) URLs
- **Invalid cron**: Update intervals must be valid cron expressions
- **Invalid temperature**: Must be between 0.0 and 2.0
- **Zero concurrent fetches**: Must be at least 1

## Best Practices

### Security

- Store API keys in environment variables, not in config files
- Keep config files readable only by your user (`chmod 600`)
- Don't commit config files with API keys to version control

### Performance

- Use appropriate `max_concurrent_fetches` based on your network
- Set `fetch_timeout_secs` higher for slow feeds
- Enable caching to reduce API costs
- Use less frequent updates for stable feeds

### Organization

- Group related feeds in separate config files
- Use descriptive feed names and consistent tagging
- Document custom prompts with comments
- Keep feed configs under version control (without API keys)

### Cost Optimization

- Enable caching to avoid redundant AI API calls
- Use cheaper models for less critical feeds
- Disable AI for feeds that don't need summarization
- Use longer update intervals for low-priority feeds

## Troubleshooting

### Config not loading

- Check file permissions (must be readable)
- Verify TOML syntax (use a validator)
- Check file location matches platform defaults
- Look for validation errors in logs

### Feed not updating

- Verify `enabled = true`
- Check `update_interval` is valid cron expression
- Ensure scheduler is running (`auto_update = true`)
- Check feed URL is accessible

### AI summaries failing

- Verify API key is set (env var or config)
- Check model name is correct for provider
- Ensure sufficient API credits/quota
- Check network connectivity to API endpoint

### Database errors

- Ensure database directory exists and is writable
- Check disk space
- Verify `max_connections` is appropriate
- Try running migrations: `presser init`
