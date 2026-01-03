# Progress Log

Development notes and session progress. Keep entries brief.

---

## 2026-01-02 (Reader Polish)

**Reader navigation & display improvements:**
- Metadata now scrolls with body (not fixed header)
- `n` next unread article (same feed)
- `r` mark as read
- `m` random unread article (any feed)
- Added `rand` crate for random selection

**Text wrapping fixes:**
- Added `textwrap` crate with unicode-width
- Metadata fields wrap properly with aligned continuation lines
- Margins applied to metadata (same as body)
- Fixed double-wrap issue: store unwrapped (width=10000), let ratatui wrap at display

**Code quality:**
- Extracted `mark_entry_as_read()` helper (DRY)
- Extracted `load_entry_by_id()` helper
- Pre-computed indent strings to reduce allocations
- Minimum width fallback for narrow terminals

**Files changed:**
- `crates/presser-core/src/ui/app.rs` - Reader rendering, new keybindings
- `crates/presser-feeds/src/parser.rs` - Unwrapped text storage
- `crates/presser-feeds/src/extractor.rs` - Width parameter for html_to_text

---

## 2026-01-02 (Roadmap Update)

**Decision:** Continuing with Presser vs forking eilmeldung
- Analyzed eilmeldung architecture (news-flash backend, 40+ deps)
- AI would be bolted-on there; it's architectural here
- Key differentiator: AI summarization as core design principle

**TUI Roadmap added to SCAFFOLD_STATUS.md:**
- Phase 3 expanded: Navigation → Reader → Polish → Advanced
- Priority: Article reader view (Enter on entry)
- Current gap: Can navigate entries but can't read them

**Newsboat import added:**
- Search `~/.newsboat/urls` or `~/.config/newsboat/urls`
- Parse URL + ~tags format
- Auto-import on first run, manual `presser import --newsboat`

---

## 2026-01-02 (Article Reader)

**TUI Article Reader implemented:**
- Full-screen reader view with header (title, date, author)
- Content display with word wrap
- j/k scroll, g/G top/bottom, PageUp/PageDown
- 'o' opens article URL in browser (via `open` crate)
- 'u' toggles read/unread status
- Esc/Backspace/q returns to entry list
- Mark as read on open

**Files changed:**
- `crates/presser-core/src/ui/app.rs` - Reader state, rendering, key handling
- `crates/presser-core/Cargo.toml` - Added `open` crate

**Content fallback chain:** content_text → summary → "[No content]"

**39 tests passing**

---

## 2026-01-02 (MVP Complete)

**Session:** CLI + Basic TUI implemented

**CLI Commands working:**
- `presser add <url>` - fetches feed metadata, adds to DB
- `presser remove <id>` - removes feed
- `presser list` - shows all feeds with entry counts
- `presser update [id]` - updates single or all feeds
- `presser stats` - shows database statistics
- `presser tui` - launches TUI

**TUI features:**
- Two-panel layout (30% feeds, 70% entries)
- j/k or arrow keys for navigation
- Tab to switch panels
- Enter to load entries for selected feed
- r to refresh current feed
- q to quit

**Config changes:**
- AI config now optional (defaults to Local provider)
- Works out of box without config files

**Files changed:**
- `crates/presser-core/src/engine.rs` - Engine::new(), fetcher()
- `crates/presser-core/src/commands.rs` - all CLI commands
- `crates/presser-core/src/main.rs` - command wiring
- `crates/presser-core/src/ui/app.rs` - complete TUI
- `crates/presser-config/src/lib.rs` - default AI config

**Total workspace tests: 39 passing**

---

## 2026-01-02 (Integration)

**Session:** Feed update integration complete

**presser-core integration:**
- `Engine::with_config()` - initializes DB, FeedFetcher, AiClient
- `Engine::update_feed()` - fetches feed, stores entries in DB
- `Engine::update_all_feeds()` - updates all enabled feeds
- `FeedUpdateTask` - scheduler task wrapper for Engine
- 3 tests passing

**Type conversions:**
- `FeedEntry.categories: Vec<String>` → `Entry.categories: Option<String>` (JSON)
- `FeedMetadata` → `Feed` (partial update with timestamps)

**Files changed:**
- `crates/presser-core/src/engine.rs` - full implementation
- `crates/presser-core/src/tasks.rs` - new FeedUpdateTask
- `crates/presser-core/src/lib.rs` - exports tasks module

**Total workspace tests: 39 passing**

**Next:** TUI implementation or CLI commands

---

## 2026-01-02 (Phase 3)

**Session:** presser-scheduler complete

**presser-scheduler complete:**
- Scheduler main loop with tokio::select! (tick/shutdown)
- Semaphore-based concurrency limiting
- Broadcast channel for graceful shutdown
- Proper handle tracking for task completion
- 5 tests passing (4 unit + 1 doc)

**Code review fixes:**
- Removed unused `max_concurrent` field (semaphore handles it)
- Fixed lock ordering: collect tasks first, spawn outside lock
- Added handle storage for stop() waiting
- Added debug logging for concurrency limit hits

**Files changed:**
- `crates/presser-scheduler/src/lib.rs` - full implementation

**Next:** presser-core TUI or integration testing

---

## 2026-01-02 (Phase 2)

**Session:** presser-feeds complete

**presser-feeds complete:**
- FeedParser: RSS/Atom parsing via feed-rs, stable ID generation
- ContentExtractor: readability-based content extraction
- FeedFetcher: HTTP fetching with proper error handling
- 10 tests passing (9 unit + 1 doc)

**Code review fixes:**
- HTTP status checking for all non-2xx responses
- Logging for content extraction failures
- Removed panicking Default impl
- Improved ID generation (URL + title + date hash)

**Other fixes:**
- Fixed doc examples in presser-ai, presser-scheduler
- All workspace tests: 31 passing

**Files changed:**
- `crates/presser-feeds/src/lib.rs` - FeedFetcher implementation
- `crates/presser-feeds/src/parser.rs` - stable ID generation
- `crates/presser-feeds/src/extractor.rs` - readability integration
- `crates/presser-feeds/src/error.rs` - HttpStatus error variant

**Next:** Phase 3 = presser-ai or presser-scheduler

---

## 2026-01-02

**Session:** Config + DB implementation

**presser-db complete:**
- Implemented all 14 query functions in `queries.rs`
- Feed CRUD, Entry CRUD, Summary ops, FTS5 search, stats
- 6 tests passing (feed_crud, entry_ops, summary_ops, stats, fts_search, db_open)
- Uses runtime queries (no compile-time DB required)

**presser-config complete:**
- Implemented `Config::load_from_dir()`
- Added cron validation (6-field format)
- 7 tests passing

**Files changed:**
- `crates/presser-db/src/queries.rs` - all 14 query implementations
- `crates/presser-db/src/lib.rs` - fixed imports, added 6 tests
- `crates/presser-config/src/lib.rs` - load_from_dir + tests
- `crates/presser-config/src/validation.rs` - cron validation
- `examples/config/*.toml` + `docs/CONFIG.md` - 6-field cron format

**Next:** Phase 1 complete. Phase 2 = presser-feeds implementation

---

**Earlier:** Initial orientation
- Created `documentation_index.md` and `progress.md`
- Project status: scaffold complete, compiles, all logic is `todo!()`

---

## Pre-history

- `c384266` - Complete project scaffold created
- `2716818` - Initial commit
