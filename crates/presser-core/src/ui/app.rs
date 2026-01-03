//! TUI application

use std::io;
use std::sync::Arc;
use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{List, ListItem, ListState, Paragraph, Wrap},
    Frame, Terminal,
};
use presser_db::{Entry, Feed};

use crate::Engine;

/// Current page/view in the TUI
#[derive(Clone, Copy, PartialEq)]
enum Page {
    /// Feed list - shows all feeds with unread/total counts
    Feeds,
    /// Entry list - shows articles for selected feed
    Entries,
    /// Reader - shows article content
    Reader,
}

/// Reader display configuration
struct ReaderConfig {
    /// Left/right margin for body text
    margin: u16,
}

impl Default for ReaderConfig {
    fn default() -> Self {
        Self { margin: 2 }
    }
}

pub struct App {
    engine: Arc<Engine>,
    feeds: Vec<Feed>,
    entries: Vec<Entry>,
    feed_state: ListState,
    entry_state: ListState,
    page: Page,
    should_quit: bool,
    // Reader state
    current_entry: Option<Entry>,
    current_feed_title: String,
    scroll_offset: u16,
    reader_config: ReaderConfig,
}

impl App {
    pub async fn new(engine: Arc<Engine>) -> Result<Self> {
        let feeds = engine.database().get_all_feeds().await?;
        let mut feed_state = ListState::default();
        if !feeds.is_empty() {
            feed_state.select(Some(0));
        }

        Ok(Self {
            engine,
            feeds,
            entries: Vec::new(),
            feed_state,
            entry_state: ListState::default(),
            page: Page::Feeds,
            should_quit: false,
            current_entry: None,
            current_feed_title: String::new(),
            scroll_offset: 0,
            reader_config: ReaderConfig::default(),
        })
    }

    async fn load_entries(&mut self) -> Result<()> {
        if let Some(idx) = self.feed_state.selected() {
            if let Some(feed) = self.feeds.get(idx) {
                self.entries = self.engine.database()
                    .get_entries_for_feed(&feed.id, 100).await?;
                self.entry_state = ListState::default();
                if !self.entries.is_empty() {
                    self.entry_state.select(Some(0));
                }
            }
        }
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        while !self.should_quit {
            terminal.draw(|f| self.render(f))?;

            if event::poll(std::time::Duration::from_millis(100))? {
                if let Event::Key(key) = event::read()? {
                    if key.kind == KeyEventKind::Press {
                        self.handle_key(key.code).await?;
                    }
                }
            }
        }

        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        match self.page {
            Page::Feeds => self.render_feeds_page(frame),
            Page::Entries => self.render_entries_page(frame),
            Page::Reader => self.render_reader(frame),
        }
    }

    fn render_feeds_page(&mut self, frame: &mut Frame) {
        use ratatui::text::{Line, Span};

        let area = frame.size();

        // Layout: title + list + help bar
        let chunks = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ],
        ).split(area);

        // Title bar
        let title = Paragraph::new(Line::from(vec![
            Span::styled(" Presser ", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(format!("({} feeds)", self.feeds.len()), Style::default().fg(Color::DarkGray)),
        ]));
        frame.render_widget(title, chunks[0]);

        // Feed list with unread/total counts
        let feed_items: Vec<ListItem> = self.feeds.iter()
            .map(|f| {
                // TODO: Track unread count properly - for now show entry_count
                let count_str = format!("({})", f.entry_count);
                let line = Line::from(vec![
                    Span::styled(
                        if f.enabled { " " } else { "×" },
                        if f.enabled { Style::default() } else { Style::default().fg(Color::DarkGray) },
                    ),
                    Span::styled(&f.title, Style::default().fg(Color::White)),
                    Span::raw(" "),
                    Span::styled(count_str, Style::default().fg(Color::DarkGray)),
                ]);
                ListItem::new(line)
            })
            .collect();

        let feed_list = List::new(feed_items)
            .highlight_style(Style::default().bg(Color::Rgb(40, 40, 40)).add_modifier(Modifier::BOLD))
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(feed_list, chunks[1], &mut self.feed_state);

        // Help bar
        let help = Paragraph::new(Line::from(vec![
            Span::styled(" Enter", Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" open ", Style::default().fg(Color::Black)),
            Span::styled("│", Style::default().fg(Color::DarkGray)),
            Span::styled(" r", Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" refresh ", Style::default().fg(Color::Black)),
            Span::styled("│", Style::default().fg(Color::DarkGray)),
            Span::styled(" q", Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" quit ", Style::default().fg(Color::Black)),
        ])).style(Style::default().bg(Color::Rgb(80, 80, 80)));

        frame.render_widget(help, chunks[2]);
    }

    fn render_entries_page(&mut self, frame: &mut Frame) {
        use ratatui::text::{Line, Span};

        let area = frame.size();

        // Layout: title + list + help bar
        let chunks = Layout::new(
            Direction::Vertical,
            [
                Constraint::Length(1),
                Constraint::Min(0),
                Constraint::Length(1),
            ],
        ).split(area);

        // Title bar showing current feed
        let title = Paragraph::new(Line::from(vec![
            Span::styled(" ◀ ", Style::default().fg(Color::DarkGray)),
            Span::styled(&self.current_feed_title, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(format!(" ({} articles)", self.entries.len()), Style::default().fg(Color::DarkGray)),
        ]));
        frame.render_widget(title, chunks[0]);

        // Entry list
        let entry_items: Vec<ListItem> = self.entries.iter()
            .map(|e| {
                let read_marker = if e.read { " " } else { "●" };
                let date_str = e.published
                    .map(|d| d.format("%m/%d").to_string())
                    .unwrap_or_default();
                let line = Line::from(vec![
                    Span::styled(
                        read_marker,
                        if e.read { Style::default().fg(Color::DarkGray) } else { Style::default().fg(Color::Green) },
                    ),
                    Span::raw(" "),
                    Span::styled(date_str, Style::default().fg(Color::DarkGray)),
                    Span::raw(" "),
                    Span::styled(
                        &e.title,
                        if e.read { Style::default().fg(Color::DarkGray) } else { Style::default().fg(Color::White) },
                    ),
                ]);
                ListItem::new(line)
            })
            .collect();

        let entry_list = List::new(entry_items)
            .highlight_style(Style::default().bg(Color::Rgb(40, 40, 40)).add_modifier(Modifier::BOLD))
            .highlight_symbol("▶ ");

        frame.render_stateful_widget(entry_list, chunks[1], &mut self.entry_state);

        // Help bar
        let help = Paragraph::new(Line::from(vec![
            Span::styled(" Enter", Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" read ", Style::default().fg(Color::Black)),
            Span::styled("│", Style::default().fg(Color::DarkGray)),
            Span::styled(" r", Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" refresh ", Style::default().fg(Color::Black)),
            Span::styled("│", Style::default().fg(Color::DarkGray)),
            Span::styled(" Esc", Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" back ", Style::default().fg(Color::Black)),
        ])).style(Style::default().bg(Color::Rgb(80, 80, 80)));

        frame.render_widget(help, chunks[2]);
    }

    fn render_reader(&self, frame: &mut Frame) {
        use ratatui::text::{Line, Span};

        let Some(entry) = &self.current_entry else {
            return;
        };

        let area = frame.size();
        let margin = self.reader_config.margin;

        let help_height = 1u16;

        // Layout: scrollable content + help bar at bottom
        let chunks = Layout::new(
            Direction::Vertical,
            [
                Constraint::Min(0),
                Constraint::Length(help_height),
            ],
        ).split(area);

        // Content area with margins
        let content_area = if margin > 0 && area.width > margin * 2 {
            let inner = Layout::new(
                Direction::Horizontal,
                [
                    Constraint::Length(margin),
                    Constraint::Min(0),
                    Constraint::Length(margin),
                ],
            ).split(chunks[0]);
            inner[1]
        } else {
            chunks[0]
        };

        let available_width = content_area.width as usize;
        const LABEL_WIDTH: usize = 8;
        let value_width = if available_width > LABEL_WIDTH {
            available_width - LABEL_WIDTH
        } else {
            available_width.max(20)
        };

        let meta_label_style = Style::default().fg(Color::Cyan);
        let meta_value_style = Style::default().fg(Color::Yellow);
        let indent = " ".repeat(LABEL_WIDTH);

        let date_str = entry.published
            .map(|d| d.format("%a, %d %b %Y %H:%M:%S %z").to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let author_str = entry.author.as_deref().unwrap_or("Unknown");

        let mut all_lines = Vec::with_capacity(10);

        for (i, line) in textwrap::wrap(&self.current_feed_title, value_width).into_iter().enumerate() {
            all_lines.push(if i == 0 {
                Line::from(vec![
                    Span::styled("Feed:   ", meta_label_style),
                    Span::styled(line.into_owned(), meta_value_style),
                ])
            } else {
                Line::from(vec![
                    Span::raw(indent.as_str()),
                    Span::styled(line.into_owned(), meta_value_style),
                ])
            });
        }

        for (i, line) in textwrap::wrap(&entry.title, value_width).into_iter().enumerate() {
            all_lines.push(if i == 0 {
                Line::from(vec![
                    Span::styled("Title:  ", meta_label_style),
                    Span::styled(line.into_owned(), meta_value_style),
                ])
            } else {
                Line::from(vec![
                    Span::raw(indent.as_str()),
                    Span::styled(line.into_owned(), meta_value_style),
                ])
            });
        }

        for (i, line) in textwrap::wrap(author_str, value_width).into_iter().enumerate() {
            all_lines.push(if i == 0 {
                Line::from(vec![
                    Span::styled("Author: ", meta_label_style),
                    Span::styled(line.into_owned(), meta_value_style),
                ])
            } else {
                Line::from(vec![
                    Span::raw(indent.as_str()),
                    Span::styled(line.into_owned(), meta_value_style),
                ])
            });
        }

        for (i, line) in textwrap::wrap(&date_str, value_width).into_iter().enumerate() {
            all_lines.push(if i == 0 {
                Line::from(vec![
                    Span::styled("Date:   ", meta_label_style),
                    Span::styled(line.into_owned(), meta_value_style),
                ])
            } else {
                Line::from(vec![
                    Span::raw(indent.as_str()),
                    Span::styled(line.into_owned(), meta_value_style),
                ])
            });
        }

        for (i, line) in textwrap::wrap(&entry.url, value_width).into_iter().enumerate() {
            all_lines.push(if i == 0 {
                Line::from(vec![
                    Span::styled("Link:   ", meta_label_style),
                    Span::styled(line.into_owned(), Style::default().fg(Color::Blue)),
                ])
            } else {
                Line::from(vec![
                    Span::raw(indent.as_str()),
                    Span::styled(line.into_owned(), Style::default().fg(Color::Blue)),
                ])
            });
        }

        all_lines.push(Line::from(""));

        // Content - prefer content_text, fall back to summary
        let content = entry.content_text.as_deref()
            .or(entry.summary.as_deref())
            .unwrap_or("[No content available]");

        let styled_content = self.style_content(content);
        all_lines.extend(styled_content.lines);

        let paragraph = Paragraph::new(all_lines)
            .wrap(Wrap { trim: false })
            .scroll((self.scroll_offset, 0));

        frame.render_widget(paragraph, content_area);

        // Help bar at bottom with colored background
        let help = Paragraph::new(Line::from(vec![
            Span::styled(" Esc", Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" back ", Style::default().fg(Color::Black)),
            Span::styled("│", Style::default().fg(Color::DarkGray)),
            Span::styled(" j/k", Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" scroll ", Style::default().fg(Color::Black)),
            Span::styled("│", Style::default().fg(Color::DarkGray)),
            Span::styled(" n", Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" next ", Style::default().fg(Color::Black)),
            Span::styled("│", Style::default().fg(Color::DarkGray)),
            Span::styled(" r", Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" read ", Style::default().fg(Color::Black)),
            Span::styled("│", Style::default().fg(Color::DarkGray)),
            Span::styled(" m", Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" random ", Style::default().fg(Color::Black)),
            Span::styled("│", Style::default().fg(Color::DarkGray)),
            Span::styled(" o", Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" open ", Style::default().fg(Color::Black)),
            Span::styled("│", Style::default().fg(Color::DarkGray)),
            Span::styled(" u", Style::default().fg(Color::Black).add_modifier(Modifier::BOLD)),
            Span::styled(" toggle ", Style::default().fg(Color::Black)),
        ])).style(Style::default().bg(Color::Rgb(80, 80, 80)));

        frame.render_widget(help, chunks[1]);
    }

    /// Style plain text content for better readability
    fn style_content<'a>(&self, content: &'a str) -> ratatui::text::Text<'a> {
        use ratatui::text::{Line, Span};

        let body_style = Style::default().fg(Color::White);

        let lines: Vec<Line> = content
            .lines()
            .map(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    Line::from("")
                } else if trimmed.starts_with('#') {
                    // Headers
                    Line::from(Span::styled(
                        line,
                        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
                    ))
                } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("• ") {
                    // Bullet points
                    Line::from(Span::styled(line, body_style))
                } else if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
                    // URLs
                    Line::from(Span::styled(line, Style::default().fg(Color::Blue)))
                } else {
                    // Normal body text
                    Line::from(Span::styled(line, body_style))
                }
            })
            .collect();

        ratatui::text::Text::from(lines)
    }

    async fn handle_key(&mut self, key: KeyCode) -> Result<()> {
        match self.page {
            Page::Feeds => self.handle_feeds_key(key).await?,
            Page::Entries => self.handle_entries_key(key).await?,
            Page::Reader => self.handle_reader_key(key).await?,
        }
        Ok(())
    }

    async fn handle_feeds_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Up | KeyCode::Char('k') => {
                let len = self.feeds.len();
                if len > 0 {
                    let i = self.feed_state.selected().unwrap_or(0);
                    let new_i = if i == 0 { len - 1 } else { i - 1 };
                    self.feed_state.select(Some(new_i));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let len = self.feeds.len();
                if len > 0 {
                    let i = self.feed_state.selected().unwrap_or(0);
                    let new_i = if i >= len - 1 { 0 } else { i + 1 };
                    self.feed_state.select(Some(new_i));
                }
            }
            KeyCode::Enter => {
                // Open feed -> go to entries page
                if let Some(idx) = self.feed_state.selected() {
                    if let Some(feed) = self.feeds.get(idx) {
                        self.current_feed_title = feed.title.clone();
                        self.load_entries().await?;
                        self.page = Page::Entries;
                    }
                }
            }
            KeyCode::Char('r') => self.refresh_current_feed().await?,
            _ => {}
        }
        Ok(())
    }

    async fn handle_entries_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc | KeyCode::Backspace => {
                // Go back to feeds
                self.page = Page::Feeds;
                self.entries.clear();
            }
            KeyCode::Char('q') => {
                // Go back to feeds (not quit)
                self.page = Page::Feeds;
                self.entries.clear();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                let len = self.entries.len();
                if len > 0 {
                    let i = self.entry_state.selected().unwrap_or(0);
                    let new_i = if i == 0 { len - 1 } else { i - 1 };
                    self.entry_state.select(Some(new_i));
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                let len = self.entries.len();
                if len > 0 {
                    let i = self.entry_state.selected().unwrap_or(0);
                    let new_i = if i >= len - 1 { 0 } else { i + 1 };
                    self.entry_state.select(Some(new_i));
                }
            }
            KeyCode::Enter => {
                if let Some(idx) = self.entry_state.selected() {
                    if let Some(entry) = self.entries.get(idx) {
                        let entry_id = entry.id.clone();
                        let needs_mark = !entry.read;
                        if needs_mark {
                            self.mark_entry_as_read(&entry_id).await?;
                        }
                        self.current_entry = Some(self.entries[idx].clone());
                        self.scroll_offset = 0;
                        self.page = Page::Reader;
                    }
                }
            }
            KeyCode::Char('r') => self.refresh_current_feed().await?,
            _ => {}
        }
        Ok(())
    }

    async fn handle_reader_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('q') => {
                // Go back to entries
                self.page = Page::Entries;
                self.current_entry = None;
                self.scroll_offset = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
            }
            KeyCode::PageUp => {
                self.scroll_offset = self.scroll_offset.saturating_sub(20);
            }
            KeyCode::PageDown => {
                self.scroll_offset = self.scroll_offset.saturating_add(20);
            }
            KeyCode::Char('g') => {
                self.scroll_offset = 0;
            }
            KeyCode::Char('G') => {
                self.scroll_offset = u16::MAX;
            }
            KeyCode::Char('o') => {
                if let Some(entry) = &self.current_entry {
                    let _ = open::that(&entry.url);
                }
            }
            KeyCode::Char('u') => {
                if let Some(entry) = &self.current_entry {
                    let entry_id = entry.id.clone();
                    let was_read = entry.read;
                    if was_read {
                        self.engine.database().mark_unread(&entry_id).await?;
                    } else {
                        self.engine.database().mark_read(&entry_id).await?;
                    }
                    if let Some(e) = self.current_entry.as_mut() {
                        e.read = !was_read;
                    }
                    if let Some(list_entry) = self.entries.iter_mut().find(|e| e.id == entry_id) {
                        list_entry.read = !was_read;
                    }
                }
            }
            KeyCode::Char('r') => {
                if let Some(entry) = &self.current_entry {
                    let entry_id = entry.id.clone();
                    if !entry.read {
                        self.mark_entry_as_read(&entry_id).await?;
                    }
                }
            }
            KeyCode::Char('n') => {
                self.load_next_unread_in_feed().await?;
            }
            KeyCode::Char('m') => {
                self.load_random_unread().await?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn load_next_unread_in_feed(&mut self) -> Result<()> {
        if let Some(current) = &self.current_entry {
            let current_id = current.id.clone();
            let mut found_current = false;
            let mut next_id: Option<String> = None;

            for entry in &self.entries {
                if found_current && !entry.read {
                    next_id = Some(entry.id.clone());
                    break;
                }
                if entry.id == current_id {
                    found_current = true;
                }
            }

            if let Some(id) = next_id {
                self.load_entry_by_id(&id).await?;
            }
        }
        Ok(())
    }

    async fn load_random_unread(&mut self) -> Result<()> {
        let unread_entries = self.engine.database().get_unread_entries(1000).await?;

        if unread_entries.is_empty() {
            return Ok(());
        }

        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        if let Some(entry) = unread_entries.choose(&mut rng) {
            let entry_id = entry.id.clone();
            let feed_id = entry.feed_id.clone();

            let feed = self.engine.database().get_feed(&feed_id).await?;
            if let Some(feed) = feed {
                self.current_feed_title = feed.title;
                self.entries = self.engine.database()
                    .get_entries_for_feed(&feed.id, 100).await?;
                self.load_entry_by_id(&entry_id).await?;
            }
        }
        Ok(())
    }

    async fn load_entry_by_id(&mut self, entry_id: &str) -> Result<()> {
        if let Some(entry) = self.engine.database().get_entry(entry_id).await? {
            if !entry.read {
                self.mark_entry_as_read(&entry.id).await?;
            }
            self.current_entry = Some(entry);
            self.scroll_offset = 0;
        }
        Ok(())
    }

    async fn mark_entry_as_read(&mut self, entry_id: &str) -> Result<()> {
        self.engine.database().mark_read(entry_id).await?;

        if let Some(entry) = self.current_entry.as_mut() {
            if entry.id == entry_id {
                entry.read = true;
            }
        }

        if let Some(list_entry) = self.entries.iter_mut().find(|e| e.id == entry_id) {
            list_entry.read = true;
        }

        Ok(())
    }

    async fn refresh_current_feed(&mut self) -> Result<()> {
        if let Some(idx) = self.feed_state.selected() {
            if let Some(feed) = self.feeds.get(idx) {
                self.engine.update_feed(&feed.id).await?;
                // Reload feeds to get updated counts
                self.feeds = self.engine.database().get_all_feeds().await?;
                // If on entries page, reload entries too
                if self.page == Page::Entries {
                    self.load_entries().await?;
                }
            }
        }
        Ok(())
    }
}
