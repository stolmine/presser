//! Presser - AI-powered RSS feed processor and digest generator
//!
//! Presser is a terminal-based RSS feed reader that uses AI to summarize articles
//! and generate daily digests. It supports multiple AI providers (OpenAI, Anthropic,
//! local LLMs) and provides both a TUI and CLI interface.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod commands;
mod engine;
mod ui;

use commands::*;

/// Presser - AI-powered RSS feed processor
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Enable debug logging
    #[arg(short, long, global = true)]
    debug: bool,

    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Add a new feed
    Add {
        /// Feed URL
        url: String,

        /// Feed name/title
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Remove a feed
    Remove {
        /// Feed ID
        id: String,
    },

    /// List all feeds
    List,

    /// Update feeds (fetch new entries)
    Update {
        /// Update a specific feed (omit to update all)
        feed_id: Option<String>,
    },

    /// Generate digest
    Digest {
        /// Number of days to include
        #[arg(short, long, default_value = "1")]
        days: u32,

        /// Output format (text, html, markdown)
        #[arg(short, long, default_value = "text")]
        format: String,
    },

    /// Start the interactive TUI
    Tui,

    /// Start the scheduler daemon
    Daemon,

    /// Show database statistics
    Stats,

    /// Initialize configuration
    Init,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let log_level = if cli.debug {
        Level::DEBUG
    } else if cli.verbose {
        Level::INFO
    } else {
        Level::WARN
    };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(log_level)
        .with_target(false)
        .finish();

    tracing::subscriber::set_global_default(subscriber)
        .context("Failed to set tracing subscriber")?;

    // Execute command
    match cli.command {
        Commands::Add { url, name } => {
            add_feed(&url, name.as_deref()).await?;
        }
        Commands::Remove { id } => {
            remove_feed(&id).await?;
        }
        Commands::List => {
            list_feeds().await?;
        }
        Commands::Update { feed_id } => {
            update_feeds(feed_id.as_deref()).await?;
        }
        Commands::Digest { days, format } => {
            generate_digest(days, &format).await?;
        }
        Commands::Tui => {
            start_tui().await?;
        }
        Commands::Daemon => {
            start_daemon().await?;
        }
        Commands::Stats => {
            show_stats().await?;
        }
        Commands::Init => {
            init_config().await?;
        }
    }

    Ok(())
}
