# Trogue — Claude Agent Instructions

## Project Overview

`trogue` is a CLI tool written in **Rust** that interacts with the Steam API. It lets users view owned games, achievements, achievement progress, and a dashboard of recently played games.

**Current version:** 0.2.2
**License:** Apache

---

## Constitution Principles (Non-Negotiable)

1. **CLI-First** — all functionality must be exposed via CLI.
2. **Performance-Focused** — minimize CPU and memory usage without sacrificing correctness.
3. **Modular / Extensible** — new features go into self-contained plugins; never touch core logic.
4. **Test-Driven Development** — all new features require comprehensive automated tests.
5. **Clear Code** — follow Rust best practices and community style guides.
6. **Check Docs Before Using Libraries** — always verify up-to-date documentation before introducing or updating a dependency.
7. **Semantic Markup Doc Comments** — mandatory on every new or modified function/class/logical block (see below).

---

## Semantic Markup Doc Comments (Mandatory)

Every new or modified function, method, or logical block **must** have a doc comment with the following XML-like sections. Focus on **WHY**, not HOW.

```rust
// Short one-line summary of what the function does.
//
// <purpose-start>
// Why this exists and what problem it solves.
// <purpose-end>
//
// <inputs-start>
// - `param`: description
// <inputs-end>
//
// <outputs-start>
// - Return value and meaning.
// <outputs-end>
//
// <side-effects-start>
// - Any mutations, I/O, or observable state changes.
// <side-effects-end>
```

Rules:
- Comments must be in **English**.
- When editing existing code, read and preserve existing doc comments; update them only if behavior changes.
- Do **not** remove existing doc comments unless they are being replaced.

---

## Architecture

Plugin-based architecture. Do **not** put feature logic in core modules.

```
src/
├── main.rs            # Entry point: init App, load plugins, dispatch commands
├── app.rs             # App struct — shared context (cfg, steam_api) passed to plugins
├── cfg.rs             # Loads config from environment variables (API key, Steam ID)
├── steam_api.rs       # HTTP client for Steam API; data structures for responses
├── ui.rs              # Shared UI utilities for consistent formatted output
├── tui.rs             # TUI game-selector (currently unused)
├── constants.rs       # Shared constants
└── plugins/
    ├── mod.rs             # Plugin trait definition + plugin registry
    ├── list_games.rs
    ├── list_achievements.rs
    ├── show_progress.rs
    ├── dashboard.rs
    └── completions.rs
```

**Adding a new feature:** create a new file under `src/plugins/`, implement the `Plugin` trait, and register it in `plugins/mod.rs`. Do not modify other core files.

---

## Key Dependencies

| Crate | Purpose |
|---|---|
| `clap` 4 + `clap_complete` 4 | CLI argument parsing + shell completions |
| `reqwest` 0.11 (rustls-tls) | Async HTTP — uses `rustls` (no OpenSSL dependency) |
| `tokio` 1 | Async runtime |
| `serde` / `serde_json` | JSON serialization |
| `chrono` | Date/time formatting |
| `crossterm` | Terminal control |
| `mockito` (dev) | HTTP mocking in tests |
| `gag` (dev) | Capture stdout/stderr in tests |

**Important:** `reqwest` uses `rustls-tls` (not `native-tls`) to avoid OpenSSL incompatibilities on Ubuntu 22.04.

---

## Development Rules

- Language: **Rust only** — no other languages in the implementation.
- All changes must go through pull requests with at least one approval.
- Run `cargo test` before submitting changes.
- Do not skip pre-commit hooks or bypass signing.
- Do not add features beyond what is explicitly requested.
- Do not add error handling for scenarios that cannot happen.
- Use Context7 MCP to check docs and examples.

---

## Shell Completions

Generated via `clap_complete`. Usage: `trogue completions <bash|zsh|fish|powershell>`.

---

## Reference Files

- `architecture.md` — detailed module descriptions and shell completion docs.
- `instructions.md` — semantic markup doc comment rules (mirrors the section above).
- `.specify/memory/constitution.md` — full project constitution (version 1.2.0).
