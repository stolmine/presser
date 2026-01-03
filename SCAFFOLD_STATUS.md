# Presser Project Scaffold - Status Report

This document describes the current state of the Presser project scaffold.

## Compilation Status

✅ **The project compiles successfully** with `cargo check --workspace`

All crates are properly configured and pass compilation, though full implementation is marked with `todo!()` macros.

## Files Created

### Root Files

- `Cargo.toml` - Workspace configuration with all 6 crates
- `README.md` - Complete project overview and usage guide
- `.gitignore` - Rust project ignore patterns
- `SCAFFOLD_STATUS.md` - This file

### Documentation (docs/)

- `ARCHITECTURE.md` - Detailed architecture and design documentation
- `CONFIG.md` - Complete configuration reference guide

### Example Configurations (examples/config/)

- `global.toml` - Comprehensive global configuration with all options
- `feeds/tech-news.toml` - Example tech news feed configurations
- `feeds/newsletters.toml` - Example newsletter feed configurations

### Crate: presser-config

Configuration management crate

**Files:**
- `Cargo.toml` - Dependencies: serde, toml, dirs, regex, url
- `src/lib.rs` - Main config structures and loading logic
- `src/error.rs` - Configuration-specific errors
- `src/validation.rs` - Config validation functions

**Status:** ✅ Complete - Config loading, validation, default AI config

### Crate: presser-feeds

Feed fetching and parsing crate

**Files:**
- `Cargo.toml` - Dependencies: feed-rs, readability, reqwest, scraper, html2text
- `src/lib.rs` - FeedFetcher and main types
- `src/parser.rs` - RSS/Atom parsing
- `src/extractor.rs` - Content extraction with readability
- `src/error.rs` - Feed-specific errors

**Status:** ✅ Complete - HTTP fetching, RSS/Atom parsing, readability extraction

### Crate: presser-scheduler

Task scheduling crate

**Files:**
- `Cargo.toml` - Dependencies: tokio, chrono, cron, async-trait
- `src/lib.rs` - Scheduler implementation
- `src/task.rs` - Task trait and implementations
- `src/error.rs` - Scheduler-specific errors

**Status:** ✅ Complete - Cron scheduling, task execution, concurrency control

### Crate: presser-ai

AI integration crate

**Files:**
- `Cargo.toml` - Dependencies: reqwest, serde_json, sha2
- `src/lib.rs` - AiClient and provider abstraction
- `src/providers.rs` - Provider-specific constants
- `src/error.rs` - AI-specific errors

**Status:** Structure complete, provider implementations marked as TODO
**Features:** local-llm feature available (not yet implemented)

### Crate: presser-db

SQLite database crate

**Files:**
- `Cargo.toml` - Dependencies: sqlx, chrono
- `src/lib.rs` - Database connection and high-level API
- `src/models.rs` - Feed, Entry, Summary models with FromRow
- `src/queries.rs` - Query implementations (TODO stubs)
- `src/error.rs` - Database-specific errors
- `migrations/20240101000001_initial_schema.sql` - Complete schema with FTS

**Status:** ✅ Complete - All 14 queries implemented with FTS5 search
**Note:** Using runtime queries to avoid compile-time DB requirement

### Crate: presser-core

Main application binary

**Files:**
- `Cargo.toml` - Binary crate depending on all other crates
- `src/main.rs` - CLI entry point with clap
- `src/lib.rs` - Library exports
- `src/commands.rs` - CLI command implementations (TODO)
- `src/engine.rs` - Core orchestration engine (TODO)
- `src/ui/mod.rs` - UI module
- `src/ui/app.rs` - TUI application with ratatui
- `src/ui/widgets.rs` - Custom widgets (TODO)

**Status:** ✅ CLI commands working (add, remove, list, update, stats), basic TUI with two-panel layout

## Implementation Status by Feature

### ✅ Fully Implemented

- Project structure and organization
- Cargo workspace configuration
- All Rust module structures
- Type definitions and models
- Error types for all crates
- Database schema (SQL migration)
- Documentation (README, ARCHITECTURE, CONFIG)
- Example configurations
- CLI command structure
- Compilation and type checking
- **Config loading and validation** (presser-config)
- **All database queries** (presser-db) - 14 queries with FTS5 search
- **Feed fetching and parsing** (presser-feeds) - HTTP fetch, RSS/Atom parse, readability extraction
- **Task scheduler** (presser-scheduler) - Cron scheduling with concurrency control
- **Core engine** (presser-core) - Feed management and orchestration
- **CLI commands** - add, remove, list, update, stats
- **Basic TUI** - Two-panel layout with vim-style navigation

### 🔨 Skeleton/TODO

- AI provider API calls (presser-ai) - awaiting user's model server
- Daemon command
- Digest command
- Init command

## Next Steps for Implementation

### Phase 1: AI Integration

1. **AI Providers** (presser-ai) - Awaiting user's model server
   - Implement OpenAI-compatible client
   - Implement Anthropic client
   - Add local LLM support (optional)
   - Implement caching logic

### Phase 2: Additional Commands

2. **Newsboat Import** (presser-core) - PRIORITY
   - [ ] Search for `~/.newsboat/urls` or `~/.config/newsboat/urls`
   - [ ] Parse Newsboat urls format (URL + optional ~tags)
   - [ ] Import feeds on first run if no feeds exist
   - [ ] `presser import --newsboat` command for manual import
   - [ ] Preserve Newsboat tags as presser tags

3. **Init Command** (presser-core)
   - Create default config directory structure
   - Generate sample global.toml
   - Initialize database
   - Auto-import from Newsboat if found

4. **Daemon Command** (presser-core)
   - Start scheduler as background process
   - Handle graceful shutdown
   - Add PID file management

5. **Digest Command** (presser-core)
   - Fetch unread entries
   - Generate AI summaries
   - Format as markdown/HTML
   - Save or output digest

### Phase 3: TUI Enhancements

5. **TUI Navigation** (presser-core) - COMPLETE
   - [x] Two-panel layout (feeds | entries)
   - [x] Tab to switch focus
   - [x] j/k navigation in both panels
   - [x] Enter on feed loads entries
   - [x] Enter on entry opens article reader (full-screen)
   - [x] Esc/backspace/q to go back
   - [x] Mark as read on open
   - [x] 'u' to toggle read/unread

6. **Article Reader View** (presser-core) - COMPLETE
   - [x] Full content display with scrolling
   - [ ] Markdown/HTML rendering (using plain text for now)
   - [x] 'o' to open in browser
   - [ ] 'y' to yank URL
   - [x] j/k or arrows to scroll content
   - [x] g/G to go to top/bottom
   - [x] PageUp/PageDown for fast scrolling
   - [ ] Metadata scrolls with body (not fixed header)
   - [ ] 'n' next unread article (same feed)
   - [ ] 'r' mark as read
   - [ ] 'm' random unread article (any feed)

7. **Visual Polish** (presser-core)
   - [ ] Theming support (config-driven colors)
   - [ ] Status bar with current feed/article info
   - [ ] Unread counts in feed list
   - [ ] Loading indicators for async ops
   - [ ] Error messages in status bar

8. **Advanced TUI Features** (presser-core)
   - [ ] '/' for full-text search
   - [ ] 'a' to add feed (inline prompt)
   - [ ] 'd' to delete feed (with confirm)
   - [ ] AI summary display (when available)
   - [ ] Digest generation from UI

### Phase 4: Polish

6. **Testing**
   - Add unit tests for all crates
   - Add integration tests
   - Create mock providers for testing

7. **Documentation**
   - Add usage examples
   - Create troubleshooting guide
   - Add contribution guidelines

## Build Commands

```bash
# Check compilation
cargo check --workspace

# Build debug
cargo build

# Build release
cargo build --release

# Run tests
cargo test

# Run with local LLM support
cargo build --features local-llm

# Build specific crate
cargo build -p presser-feeds
```

## Configuration Setup

To use this project:

1. Copy example configs to your config directory:
   ```bash
   mkdir -p ~/.config/presser/feeds
   cp examples/config/global.toml ~/.config/presser/
   cp examples/config/feeds/*.toml ~/.config/presser/feeds/
   ```

2. Edit `~/.config/presser/global.toml` and add your API key

3. Customize feed configurations in `~/.config/presser/feeds/`

## Known Limitations

1. **Database Queries**: Using runtime queries instead of compile-time checked queries
   - Can switch to `sqlx::query!` macros once database is set up
   - Could use SQLx offline mode for CI/CD

2. **Local LLM**: Feature flag exists but not implemented
   - Would require llama.cpp bindings
   - Consider using llama-cpp-rs or llm crate

3. **Error Handling**: Basic error types defined but not all edge cases covered

4. **Testing**: No tests yet, all marked as TODO

## File Count Summary

- **Total Rust files**: 23
- **Total TOML files**: 7 (1 workspace + 6 crate configs)
- **Total SQL files**: 1 (database migration)
- **Total Markdown files**: 4 (README, ARCHITECTURE, CONFIG, this file)
- **Example configs**: 3 (global + 2 feed configs)

---

**Created**: 2025-12-13
**Last Updated**: 2026-01-02
**Status**: Core functionality implemented, AI integration pending
**Compiles**: Yes ✅
**Working Commands**: add, remove, list, update, stats, tui ✅
**Pending**: AI integration, daemon, digest, init
