//! Feed parsing implementation

use crate::{FeedEntry, FeedError, FeedMetadata};
use anyhow::Result;
use feed_rs::parser;
use sha2::{Digest, Sha256};

/// Feed parser that handles RSS and Atom feeds
pub struct FeedParser;

impl FeedParser {
    /// Create a new feed parser
    pub fn new() -> Self {
        Self
    }

    /// Parse feed XML/content into metadata and entries
    pub fn parse(&self, content: &[u8]) -> Result<(FeedMetadata, Vec<FeedEntry>), FeedError> {
        let feed = parser::parse(content)
            .map_err(|e| FeedError::ParseError(e.to_string()))?;

        let metadata = FeedMetadata {
            title: feed.title.map(|t| t.content).unwrap_or_default(),
            description: feed.description.map(|t| t.content),
            url: feed.links.first().map(|l| l.href.clone()).unwrap_or_default(),
            site_url: feed.links.iter()
                .find(|l| l.rel.as_deref() == Some("alternate"))
                .map(|l| l.href.clone()),
            last_updated: feed.updated,
        };

        let entries = feed.entries.into_iter().map(|entry| {
            let id = if entry.id.is_empty() {
                // Generate stable ID from URL, title, and published date
                let url = entry.links.first().map(|l| l.href.as_str()).unwrap_or("");
                let title = entry.title.as_ref().map(|t| t.content.as_str()).unwrap_or("");
                let published = entry.published.map(|d| d.to_rfc3339()).unwrap_or_default();
                let mut hasher = Sha256::new();
                hasher.update(format!("{}|{}|{}", url, title, published).as_bytes());
                format!("{:x}", hasher.finalize())
            } else {
                entry.id
            };

            FeedEntry {
                id,
                title: entry.title.map(|t| t.content).unwrap_or_default(),
                url: entry.links.first().map(|l| l.href.clone()).unwrap_or_default(),
                published: entry.published,
                updated: entry.updated,
                summary: entry.summary.map(|t| t.content),
                content_html: entry.content.and_then(|c| c.body),
                content_text: None,
                author: entry.authors.first().map(|p| p.name.clone()),
                categories: entry.categories.iter().map(|c| c.term.clone()).collect(),
            }
        }).collect();

        Ok((metadata, entries))
    }
}

impl Default for FeedParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let _parser = FeedParser::new();
    }

    #[test]
    fn test_parse_rss() {
        let rss = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Test Feed</title>
    <link>https://example.com</link>
    <description>A test feed</description>
    <item>
      <title>Test Entry</title>
      <link>https://example.com/entry1</link>
      <guid>entry-1</guid>
      <pubDate>Mon, 01 Jan 2024 12:00:00 GMT</pubDate>
      <description>Entry summary</description>
    </item>
  </channel>
</rss>"#;

        let parser = FeedParser::new();
        let result = parser.parse(rss.as_bytes());
        assert!(result.is_ok());

        let (metadata, entries) = result.unwrap();
        assert_eq!(metadata.title, "Test Feed");
        assert_eq!(metadata.description, Some("A test feed".to_string()));
        assert_eq!(metadata.url, "https://example.com/");

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Test Entry");
        assert_eq!(entries[0].url, "https://example.com/entry1");
        assert_eq!(entries[0].id, "entry-1");
    }

    #[test]
    fn test_parse_atom() {
        let atom = r#"<?xml version="1.0" encoding="UTF-8"?>
<feed xmlns="http://www.w3.org/2005/Atom">
  <title>Test Atom Feed</title>
  <link href="https://example.com/feed" rel="self"/>
  <link href="https://example.com" rel="alternate"/>
  <updated>2024-01-01T12:00:00Z</updated>
  <entry>
    <title>Atom Entry</title>
    <link href="https://example.com/atom-entry"/>
    <id>atom-entry-1</id>
    <updated>2024-01-01T12:00:00Z</updated>
    <summary>Atom summary</summary>
    <author>
      <name>Test Author</name>
    </author>
  </entry>
</feed>"#;

        let parser = FeedParser::new();
        let result = parser.parse(atom.as_bytes());
        assert!(result.is_ok());

        let (metadata, entries) = result.unwrap();
        assert_eq!(metadata.title, "Test Atom Feed");
        assert_eq!(metadata.site_url, Some("https://example.com/".to_string()));

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "Atom Entry");
        assert_eq!(entries[0].url, "https://example.com/atom-entry");
        assert_eq!(entries[0].id, "atom-entry-1");
        assert_eq!(entries[0].author, Some("Test Author".to_string()));
    }

    #[test]
    fn test_parse_missing_fields() {
        let minimal_rss = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Minimal Feed</title>
    <item>
      <link>https://example.com/minimal</link>
    </item>
  </channel>
</rss>"#;

        let parser = FeedParser::new();
        let result = parser.parse(minimal_rss.as_bytes());
        assert!(result.is_ok());

        let (metadata, entries) = result.unwrap();
        assert_eq!(metadata.title, "Minimal Feed");
        assert_eq!(metadata.description, None);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].url, "https://example.com/minimal");
        assert_eq!(entries[0].title, "");
        assert!(entries[0].published.is_none());
        assert!(entries[0].author.is_none());
        assert_eq!(entries[0].categories.len(), 0);
    }
}
