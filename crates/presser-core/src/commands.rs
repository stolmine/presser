//! CLI command implementations

use anyhow::Result;
use presser_db::Feed;

fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub async fn add_feed(engine: &crate::Engine, url: &str, name: Option<&str>) -> Result<()> {
    println!("Fetching feed: {}", url);
    let (metadata, _) = engine.fetcher().fetch(url).await?;

    let title = name.map(String::from).unwrap_or_else(|| metadata.title.clone());
    let feed = Feed {
        id: slugify(&title),
        url: url.to_string(),
        title,
        description: metadata.description,
        site_url: metadata.site_url,
        ..Default::default()
    };

    engine.database().upsert_feed(&feed).await?;
    println!("Added feed: {} ({})", feed.title, feed.id);
    Ok(())
}

pub async fn remove_feed(engine: &crate::Engine, id: &str) -> Result<()> {
    engine.database().delete_feed(id).await?;
    println!("Removed feed: {}", id);
    Ok(())
}

pub async fn list_feeds(engine: &crate::Engine) -> Result<()> {
    let feeds = engine.database().get_all_feeds().await?;
    if feeds.is_empty() {
        println!("No feeds configured. Use 'presser add <url>' to add one.");
    } else {
        for feed in feeds {
            let status = if feed.enabled { "" } else { " [disabled]" };
            println!("{}: {} ({} entries){}", feed.id, feed.title, feed.entry_count, status);
        }
    }
    Ok(())
}

/// Update feeds
pub async fn update_feeds(engine: &crate::Engine, feed_id: Option<&str>) -> Result<()> {
    match feed_id {
        Some(id) => {
            println!("Updating feed: {}", id);
            engine.update_feed(id).await?;
            println!("Feed updated successfully");
        }
        None => {
            println!("Updating all feeds...");
            engine.update_all_feeds().await?;
            println!("All feeds updated");
        }
    }
    Ok(())
}

/// Generate digest
pub async fn generate_digest(days: u32, format: &str) -> Result<()> {
    println!("Generating {}-day digest in {} format...", days, format);

    // TODO: Implement digest generation
    // 1. Load config
    // 2. Open database
    // 3. Query entries from last N days
    // 4. Group by feed/category
    // 5. Format output
    // 6. Display or save

    todo!("Implement generate_digest")
}

/// Start scheduler daemon
pub async fn start_daemon() -> Result<()> {
    println!("Starting daemon...");

    // TODO: Implement daemon
    // 1. Load config
    // 2. Open database
    // 3. Initialize scheduler
    // 4. Schedule feed updates
    // 5. Run until interrupted

    todo!("Implement start_daemon")
}

/// Show database statistics
pub async fn show_stats(engine: &crate::Engine) -> Result<()> {
    let stats = engine.database().get_stats().await?;
    println!("Database Statistics:");
    println!("  Feeds:     {}", stats.total_feeds);
    println!("  Entries:   {} ({} unread)", stats.total_entries, stats.unread_entries);
    println!("  Summaries: {}", stats.total_summaries);
    Ok(())
}

/// Start interactive TUI
pub async fn run_tui(engine: std::sync::Arc<crate::Engine>) -> Result<()> {
    let mut app = crate::ui::App::new(engine).await?;
    app.run().await
}

/// Initialize configuration
pub async fn init_config() -> Result<()> {
    println!("Initializing configuration...");

    // TODO: Implement config initialization
    // 1. Create config directories
    // 2. Create default config files
    // 3. Prompt for AI provider settings
    // 4. Initialize database

    todo!("Implement init_config")
}
