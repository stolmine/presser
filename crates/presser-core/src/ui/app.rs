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
    widgets::{Block, Borders, List, ListItem, ListState},
    Frame, Terminal,
};
use presser_db::{Entry, Feed};

use crate::Engine;

#[derive(Clone, Copy, PartialEq)]
enum Focus {
    Feeds,
    Entries,
}

pub struct App {
    engine: Arc<Engine>,
    feeds: Vec<Feed>,
    entries: Vec<Entry>,
    feed_state: ListState,
    entry_state: ListState,
    focus: Focus,
    should_quit: bool,
}

impl App {
    pub async fn new(engine: Arc<Engine>) -> Result<Self> {
        let feeds = engine.database().get_all_feeds().await?;
        let mut feed_state = ListState::default();
        if !feeds.is_empty() {
            feed_state.select(Some(0));
        }

        let mut app = Self {
            engine,
            feeds,
            entries: Vec::new(),
            feed_state,
            entry_state: ListState::default(),
            focus: Focus::Feeds,
            should_quit: false,
        };

        app.load_entries().await?;
        Ok(app)
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
        let chunks = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Percentage(30),
                Constraint::Percentage(70),
            ],
        ).split(frame.size());

        let feed_items: Vec<ListItem> = self.feeds.iter()
            .map(|f| {
                let style = if f.enabled { Style::default() } else { Style::default().fg(Color::DarkGray) };
                ListItem::new(format!("{} ({})", f.title, f.entry_count)).style(style)
            })
            .collect();

        let feed_block = Block::default()
            .borders(Borders::ALL)
            .title("Feeds")
            .border_style(if self.focus == Focus::Feeds {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });

        let feed_list = List::new(feed_items)
            .block(feed_block)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(feed_list, chunks[0], &mut self.feed_state);

        let entry_items: Vec<ListItem> = self.entries.iter()
            .map(|e| {
                let style = if e.read {
                    Style::default().fg(Color::DarkGray)
                } else {
                    Style::default()
                };
                ListItem::new(e.title.clone()).style(style)
            })
            .collect();

        let entry_block = Block::default()
            .borders(Borders::ALL)
            .title("Entries")
            .border_style(if self.focus == Focus::Entries {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default()
            });

        let entry_list = List::new(entry_items)
            .block(entry_block)
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(entry_list, chunks[1], &mut self.entry_state);
    }

    async fn handle_key(&mut self, key: KeyCode) -> Result<()> {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Tab => self.toggle_focus(),
            KeyCode::Up | KeyCode::Char('k') => self.move_up(),
            KeyCode::Down | KeyCode::Char('j') => self.move_down(),
            KeyCode::Enter => self.select().await?,
            KeyCode::Char('r') => self.refresh().await?,
            _ => {}
        }
        Ok(())
    }

    fn toggle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Feeds => Focus::Entries,
            Focus::Entries => Focus::Feeds,
        };
    }

    fn move_up(&mut self) {
        let state = match self.focus {
            Focus::Feeds => &mut self.feed_state,
            Focus::Entries => &mut self.entry_state,
        };
        let len = match self.focus {
            Focus::Feeds => self.feeds.len(),
            Focus::Entries => self.entries.len(),
        };
        if len == 0 { return; }

        let i = state.selected().unwrap_or(0);
        let new_i = if i == 0 { len - 1 } else { i - 1 };
        state.select(Some(new_i));
    }

    fn move_down(&mut self) {
        let state = match self.focus {
            Focus::Feeds => &mut self.feed_state,
            Focus::Entries => &mut self.entry_state,
        };
        let len = match self.focus {
            Focus::Feeds => self.feeds.len(),
            Focus::Entries => self.entries.len(),
        };
        if len == 0 { return; }

        let i = state.selected().unwrap_or(0);
        let new_i = if i >= len - 1 { 0 } else { i + 1 };
        state.select(Some(new_i));
    }

    async fn select(&mut self) -> Result<()> {
        if self.focus == Focus::Feeds {
            self.load_entries().await?;
        }
        Ok(())
    }

    async fn refresh(&mut self) -> Result<()> {
        if self.focus == Focus::Feeds {
            if let Some(idx) = self.feed_state.selected() {
                if let Some(feed) = self.feeds.get(idx) {
                    self.engine.update_feed(&feed.id).await?;
                    self.load_entries().await?;
                }
            }
        }
        Ok(())
    }
}
