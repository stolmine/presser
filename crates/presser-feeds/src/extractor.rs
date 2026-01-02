//! Content extraction using readability algorithms

use crate::FeedError;
use anyhow::Result;
use std::io::Cursor;
use url::Url;

/// Content extractor that extracts main article content from HTML
pub struct ContentExtractor;

impl ContentExtractor {
    /// Create a new content extractor
    pub fn new() -> Self {
        Self
    }

    /// Extract main content from HTML
    pub fn extract(&self, html: &str, url: &str) -> Result<String, FeedError> {
        let parsed_url = Url::parse(url)
            .map_err(|e| FeedError::InvalidUrl(e.to_string()))?;

        let mut cursor = Cursor::new(html.as_bytes());

        let product = readability::extractor::extract(&mut cursor, &parsed_url)
            .map_err(|e| FeedError::ExtractionError(e.to_string()))?;

        Ok(product.text)
    }

    /// Convert HTML to plain text
    pub fn html_to_text(&self, html: &str) -> String {
        // TODO: Use html2text to convert HTML to readable plain text
        html2text::from_read(html.as_bytes(), 80)
    }
}

impl Default for ContentExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extractor_creation() {
        let _extractor = ContentExtractor::new();
        // Extractor created successfully
    }

    #[test]
    fn test_html_to_text() {
        let extractor = ContentExtractor::new();
        let html = "<p>Hello <strong>world</strong>!</p>";
        let text = extractor.html_to_text(html);
        assert!(text.contains("Hello"));
        assert!(text.contains("world"));
    }

    #[test]
    fn test_extract_simple_html() {
        let extractor = ContentExtractor::new();
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head><title>Test Article</title></head>
            <body>
                <article>
                    <h1>Article Title</h1>
                    <p>This is the main content of the article.</p>
                    <p>It has multiple paragraphs with important information.</p>
                </article>
            </body>
            </html>
        "#;
        let url = "https://example.com/article";

        let result = extractor.extract(html, url);
        assert!(result.is_ok());

        let text = result.unwrap();
        assert!(!text.is_empty());
        assert!(text.contains("main content"));
        assert!(text.contains("important information"));
    }

    #[test]
    fn test_extract_invalid_url() {
        let extractor = ContentExtractor::new();
        let html = "<p>Some content</p>";
        let invalid_url = "not a valid url";

        let result = extractor.extract(html, invalid_url);
        assert!(result.is_err());

        match result {
            Err(FeedError::InvalidUrl(_)) => {},
            _ => panic!("Expected InvalidUrl error"),
        }
    }
}
