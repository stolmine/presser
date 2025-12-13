# Presser Architecture

This document describes the high-level architecture and design decisions for Presser.

## Overview

Presser is designed as a modular Rust workspace with clear separation of concerns. Each crate has a specific responsibility and minimal coupling with others.

## System Architecture

```
┌─────────────────────────────────────────────────────────┐
│                     presser-core                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐   │
│  │   CLI   │  │   TUI   │  │ Engine  │  │ Commands │   │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘   │
└───────┼───────────┼────────────┼────────────┼──────────┘
        │           │            │            │
        └───────────┴────────────┴────────────┘
                    │
        ┌───────────┴───────────┐
        │                       │
┌───────▼────────┐      ┌───────▼────────┐
│ presser-config │      │  presser-db    │
└────────────────┘      └────────────────┘
        │                       │
┌───────▼────────┐      ┌───────▼────────┐
│ presser-feeds  │      │ presser-ai     │
└────────────────┘      └────────────────┘
        │                       │
┌───────▼────────┐              │
│presser-scheduler│             │
└────────────────┘              │
                                │
                    ┌───────────▼────────────┐
                    │  External AI APIs      │
                    │  (OpenAI, Anthropic)   │
                    └────────────────────────┘
```

## Crate Responsibilities

### presser-core

**Purpose**: Main application binary and orchestration

**Key Components**:
- `main.rs`: Entry point, CLI parsing with clap
- `commands.rs`: CLI command implementations
- `engine.rs`: Core engine that orchestrates all components
- `ui/`: Terminal UI implementation with ratatui

**Dependencies**: All other presser-* crates

**Key Types**:
- `Engine`: Coordinates between config, database, feeds, AI, and scheduler
- `App`: TUI application state and event loop

### presser-config

**Purpose**: Configuration management and validation

**Key Components**:
- `lib.rs`: Config loading, merging, and validation
- `error.rs`: Configuration-specific errors
- `validation.rs`: Config validation logic

**Dependencies**: None (only external crates)

**Key Types**:
- `Config`: Root configuration structure
- `GlobalConfig`: Application-wide settings
- `AiConfig`: AI provider configuration
- `FeedConfig`: Per-feed configuration

**Design Decisions**:
- Hierarchical config: Global settings with feed-level overrides
- TOML format for human-readable configuration
- Validation happens at load time to fail fast
- Environment variable support for API keys

### presser-feeds

**Purpose**: Feed fetching, parsing, and content extraction

**Key Components**:
- `lib.rs`: Main feed fetcher API
- `parser.rs`: RSS/Atom parsing using feed-rs
- `extractor.rs`: Content extraction using readability
- `error.rs`: Feed-specific errors

**Dependencies**: None (only external crates)

**Key Types**:
- `FeedFetcher`: HTTP client for fetching feeds
- `FeedParser`: Parses RSS/Atom into our types
- `ContentExtractor`: Extracts main content from HTML
- `FeedEntry`: Represents a single feed item

**Design Decisions**:
- Async-first design with tokio
- Separate parsing from fetching for testability
- Content extraction is optional per-feed
- HTML to clean text conversion

### presser-scheduler

**Purpose**: Task scheduling for periodic feed updates

**Key Components**:
- `lib.rs`: Scheduler implementation
- `task.rs`: Task trait and implementations
- `error.rs`: Scheduler-specific errors

**Dependencies**: None (only external crates)

**Key Types**:
- `Scheduler`: Manages scheduled tasks
- `Task`: Trait for executable tasks
- `ScheduledTask`: Task with cron schedule

**Design Decisions**:
- Cron expressions for flexible scheduling
- Configurable concurrency limits
- Graceful shutdown support
- Task cancellation

### presser-ai

**Purpose**: AI integration for summarization

**Key Components**:
- `lib.rs`: Main AI client
- `providers.rs`: Provider-specific implementations
- `error.rs`: AI-specific errors

**Dependencies**: None (only external crates)

**Key Types**:
- `AiClient`: Unified interface for all providers
- `AiProvider`: Enum of supported providers
- `Summary`: Summary response with metadata

**Design Decisions**:
- Provider abstraction for easy switching
- Content-hash based caching to avoid redundant API calls
- Streaming support (future enhancement)
- Local LLM support via feature flag

**Provider Support**:
- OpenAI: GPT-4, GPT-3.5-turbo
- Anthropic: Claude 3 family
- Local: Via llama.cpp bindings (optional feature)

### presser-db

**Purpose**: SQLite database layer

**Key Components**:
- `lib.rs`: Database connection and high-level API
- `models.rs`: Database models (Feed, Entry, Summary)
- `queries.rs`: SQL query implementations
- `migrations/`: SQLx migrations

**Dependencies**: None (only external crates)

**Key Types**:
- `Database`: Connection pool and operations
- `Feed`: Feed metadata model
- `Entry`: Article/entry model
- `Summary`: AI-generated summary model

**Design Decisions**:
- SQLite for simplicity and portability
- SQLx for compile-time checked queries
- WAL mode for better concurrency
- Full-text search using FTS5
- Foreign keys for referential integrity

**Schema**:
- `feeds`: Feed metadata and status
- `entries`: Individual articles with content
- `summaries`: AI-generated summaries (cached)
- `entries_fts`: Full-text search virtual table

## Data Flow

### Feed Update Flow

1. **Trigger**: User command or scheduler
2. **Fetch**: presser-feeds downloads RSS/Atom feed
3. **Parse**: Convert feed XML to structured data
4. **Extract**: (Optional) Fetch full article content
5. **Store**: Save entries to database (presser-db)
6. **Summarize**: Generate AI summary (presser-ai)
7. **Cache**: Store summary with content hash

### Digest Generation Flow

1. **Query**: Fetch unread entries from database
2. **Filter**: Apply time range and feed filters
3. **Summarize**: Ensure all entries have summaries
4. **Format**: Generate digest in requested format
5. **Output**: Display or save digest

### Configuration Loading Flow

1. **Read**: Load global.toml from config directory
2. **Read**: Load all feed configs from feeds/ directory
3. **Merge**: Apply feed-level overrides to global settings
4. **Validate**: Check all values, URLs, cron expressions
5. **Return**: Ready-to-use Config object

## Concurrency Model

- **Async Runtime**: Tokio for all async operations
- **Feed Fetching**: Concurrent with configurable limits
- **Database**: Connection pool (max 5 connections)
- **AI Requests**: Sequential per-entry, but multiple entries in parallel
- **Scheduler**: Independent task execution

## Error Handling

- Each crate defines its own error types using thiserror
- Errors bubble up to presser-core
- CLI commands convert to user-friendly messages
- TUI shows errors in status bar
- Partial failures are allowed (one feed failure doesn't stop others)

## Performance Considerations

### Caching

- **AI Summaries**: Cached by content hash in database
- **Feed Metadata**: Cached in database with last-updated times
- **Content Extraction**: Cached in entry.content_text field

### Optimization

- Connection pooling for database
- Concurrent feed fetching
- Incremental updates (only new entries)
- FTS5 for fast search
- WAL mode for better SQLite concurrency

## Security Considerations

- API keys stored in config or environment variables
- No plaintext passwords (use environment variables)
- Input validation for URLs and user input
- SQL injection prevention via SQLx prepared statements
- Rate limiting for AI APIs (future enhancement)

## Testing Strategy

- **Unit Tests**: Each crate has its own tests
- **Integration Tests**: presser-core tests full workflows
- **Mock Providers**: For testing without external APIs
- **Test Fixtures**: Sample RSS feeds and responses

## Future Enhancements

### Planned Features

- [ ] Full-text search in TUI
- [ ] Custom digest templates
- [ ] Email delivery of digests
- [ ] OPML import/export
- [ ] Podcast support
- [ ] Read-it-later integration (Pocket, Instapaper)
- [ ] Collaborative filtering/recommendations
- [ ] Web UI (optional)

### Scalability

- Current design targets single-user, local usage
- For multi-user: Add authentication, use PostgreSQL
- For high volume: Add Redis cache, queue workers

## Development Guidelines

### Code Style

- Use Rust standard formatting (rustfmt)
- Document all public APIs
- Add examples to documentation
- Use descriptive variable names
- Prefer explicit over implicit

### Adding a New AI Provider

1. Add variant to `AiProvider` enum in presser-ai
2. Implement API client in `providers.rs`
3. Add summarization method to `AiClient`
4. Update config validation
5. Add tests with mock responses
6. Update documentation

### Adding a New Command

1. Add variant to `Commands` enum in presser-core
2. Implement handler in `commands.rs`
3. Add to `Engine` if needed
4. Update CLI help text
5. Add to README

## References

- [Rust Async Book](https://rust-lang.github.io/async-book/)
- [Ratatui Documentation](https://ratatui.rs/)
- [SQLx Documentation](https://github.com/launchbadge/sqlx)
- [feed-rs Documentation](https://github.com/feed-rs/feed-rs)
