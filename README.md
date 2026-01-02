# Presser

**Presser** is an AI-powered RSS feed reader and digest generator for the terminal. It fetches your favorite feeds, uses AI to summarize articles, and generates daily digests so you can stay informed without information overload.

## Features

- **Multi-source feed aggregation**: Subscribe to RSS and Atom feeds from any source
- **AI-powered summarization**: Automatic article summaries using OpenAI, Anthropic, or local LLMs
- **Smart content extraction**: Uses readability algorithms to extract clean article text
- **Scheduled updates**: Cron-based scheduling for automatic feed updates
- **Terminal UI**: Beautiful, keyboard-driven interface built with Ratatui
- **SQLite storage**: Efficient local storage with full-text search
- **Customizable**: Feed-level configuration overrides and custom AI prompts
- **Daily digests**: Generate comprehensive digests of your unread content

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/presser.git
cd presser

# Build the project
cargo build --release

# Run Presser
./target/release/presser --help
```

### Initial Setup

```bash
# Add your first feed
presser add https://hnrss.org/frontpage

# Update feeds
presser update

# View in TUI
presser tui
```

## Configuration

Presser uses a hierarchical configuration system:

1. **Global config**: `~/.config/presser/global.toml` - Default settings for all feeds
2. **Feed configs**: `~/.config/presser/feeds/*.toml` - Feed-specific overrides

See [Configuration Guide](docs/CONFIG.md) for detailed documentation.

### Example Global Config

```toml
[ai]
provider = "openai"
model = "gpt-4"
system_prompt = "Create concise summaries of articles..."

[scheduler]
default_interval = "0 */6 * * *"  # Every 6 hours
auto_update = true
```

### Example Feed Config

```toml
[[feed]]
url = "https://hnrss.org/frontpage"
name = "Hacker News"
tags = ["tech", "programming"]
update_interval = "0 */2 * * *"  # Every 2 hours
custom_prompt = "Focus on technical insights..."
```

## Usage

### CLI Commands

```bash
# Add a feed
presser add <url>

# Remove a feed
presser remove <id>

# List all feeds
presser list

# Update all feeds
presser update

# Update a specific feed
presser update <id>

# Show statistics
presser stats

# Start the TUI
presser tui

# Generate a digest (not yet implemented)
presser digest --days 1 --format markdown

# Start the scheduler daemon (not yet implemented)
presser daemon
```

### Terminal UI

The TUI provides an interactive interface for browsing feeds and reading articles:

- **Tab**: Switch between panels (feeds, entries)
- **j/k or ↑/↓**: Navigate lists
- **Enter**: Select feed/entry
- **r**: Refresh current feed
- **q**: Quit

## Architecture

Presser is built as a modular Rust workspace with six crates:

- **presser-core**: Main binary and CLI
- **presser-config**: Configuration management
- **presser-feeds**: Feed fetching and parsing
- **presser-scheduler**: Task scheduling
- **presser-ai**: AI integration (OpenAI, Anthropic, local)
- **presser-db**: SQLite database layer

See [Architecture Guide](docs/ARCHITECTURE.md) for detailed design documentation.

## AI Providers

Presser supports multiple AI providers:

### OpenAI

```toml
[ai]
provider = "openai"
api_key = "sk-..."  # Or set OPENAI_API_KEY env var
model = "gpt-4"
```

### Anthropic

```toml
[ai]
provider = "anthropic"
api_key = "sk-ant-..."  # Or set ANTHROPIC_API_KEY env var
model = "claude-3-sonnet-20240229"
```

### Local LLM

```toml
[ai]
provider = "local"
endpoint = "http://localhost:8080/v1"
model = "llama-2-7b"
```

Build with local LLM support:

```bash
cargo build --release --features local-llm
```

## Development

### Building

```bash
# Development build
cargo build

# Release build
cargo build --release

# With local LLM support
cargo build --release --features local-llm
```

### Testing

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p presser-feeds

# Run with logging
RUST_LOG=debug cargo test
```

### Project Structure

```
presser/
├── crates/
│   ├── presser-core/      # Main application
│   ├── presser-config/    # Configuration
│   ├── presser-feeds/     # Feed fetching
│   ├── presser-scheduler/ # Task scheduling
│   ├── presser-ai/        # AI integration
│   └── presser-db/        # Database layer
├── docs/                  # Documentation
├── examples/              # Example configs
└── Cargo.toml            # Workspace manifest
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Acknowledgments

- [feed-rs](https://github.com/feed-rs/feed-rs) - RSS/Atom parsing
- [readability](https://github.com/kumabook/readability) - Content extraction
- [ratatui](https://github.com/ratatui-org/ratatui) - Terminal UI
- [SQLx](https://github.com/launchbadge/sqlx) - Database toolkit
