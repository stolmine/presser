# Documentation Index

Quick reference for navigating Presser documentation.

## Core Documentation

| File | Purpose |
|------|---------|
| `README.md` | Project overview, quick start, usage |
| `docs/ARCHITECTURE.md` | System design, crate responsibilities, data flow |
| `docs/CONFIG.md` | Configuration reference (TOML options) |
| `SCAFFOLD_STATUS.md` | Current implementation status, next steps |
| `progress.md` | Development log, session notes |

## Example Configs

Located in `examples/config/`:
- `global.toml` - Full global config example
- `feeds/tech-news.toml` - Tech feed examples
- `feeds/newsletters.toml` - Newsletter feed examples

## Crate Structure

```
crates/
├── presser-core/      # CLI + TUI (main binary)
├── presser-config/    # Config loading/validation
├── presser-feeds/     # RSS/Atom fetching + readability
├── presser-scheduler/ # Cron-based task scheduling
├── presser-ai/        # AI summarization providers
└── presser-db/        # SQLite + FTS5
```

## Key Entry Points

- CLI: `crates/presser-core/src/main.rs`
- Config types: `crates/presser-config/src/lib.rs`
- DB schema: `crates/presser-db/migrations/`
