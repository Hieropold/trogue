# Trogue — AI Agent Instructions

## Project Overview

`trogue` is a CLI tool written in **Rust** that interacts with the Steam API. It lets users view owned games, achievements, achievement progress, and a dashboard of recently played games.

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

## Safe Rust & Enforcement

`unsafe_code` is `forbid`den crate-wide (see `[lints]` in `Cargo.toml`). If a future
dependency genuinely requires `unsafe`, that is an architectural decision — raise it,
don't work around the lint.

The `[lints]` table in `Cargo.toml` is the single source of truth for lint levels. It
applies identically whether clippy is run by hand, by `.githooks/pre-commit`, or by CI —
no flags to remember. It splits lints into two tiers:

- **Deny (always fails):** `clippy::todo`, `clippy::unimplemented`, `clippy::dbg_macro`,
  `clippy::mem_forget`. These have no legitimate use in committed code.
- **Warn (a ratchet, not a wall):** `clippy::unwrap_used`, `clippy::expect_used`,
  `clippy::panic`, `missing_debug_implementations`, `unused_qualifications`. They surface
  in `cargo clippy` output without blocking every commit, because the codebase does not
  yet satisfy them everywhere (e.g. `enable_raw_mode().unwrap()` in
  `plugins/interactive.rs`'s terminal setup). Tests are exempt from the panic-path trio
  via `#![cfg_attr(test, allow(...))]` in `main.rs` — asserting via `unwrap()` in a test
  is the point, not a smell.

**Enforcement gate:** `.githooks/pre-commit` runs `cargo fmt --check` and
`cargo clippy --all-targets -- -D clippy::all` — deliberately `clippy::all`, not
`-D warnings`, so the warn-tier ratchet lints above stay advisory locally while the
default clippy correctness/style/complexity/perf categories still hard-fail.
`.githooks/pre-push` runs `cargo llvm-cov --fail-under-lines 79` (a coverage floor,
raised as coverage improves) and an advisory `cargo audit`. Enable the hooks once per
clone:

```
git config core.hooksPath .githooks
```

`.github/workflows/ci.yml` runs the same fmt/clippy checks plus a blocking
`cargo audit`, so the gate survives `git commit --no-verify` or a hook someone forgot
to enable. The toolchain is pinned in `rust-toolchain.toml` so lint output is
reproducible across machines and CI.

Every third-party action in `ci.yml` is pinned to a commit SHA, not a version tag —
tags are mutable, so a compromised action maintainer could otherwise repoint one at
malicious code with no visible diff here. `.github/dependabot.yml` keeps those pins
from going stale by opening PRs that bump the SHA (and its version comment) on a
weekly schedule.

### `debug_assert!` policy

`debug_assert!`/`debug_assert_eq!` are compiled out under `--release` — that makes them
free in production, and unsafe to lean on for anything production needs to actually
check.

- **Use them** for the crate's own internal invariants: post-conditions of pure logic
  (e.g. `GameMatch::One` only being constructed when exactly one match exists in
  `steam_client.rs`), index/bounds reasoning in the rendering path, percentage math
  staying within `0..=100`.
- **Never use them** to validate Steam API responses or CLI input — those are untrusted
  and must fail via `Result` in both debug and release builds, not silently pass in
  release because the check vanished. Also avoid them for anything with side effects,
  and anything load-bearing for memory or data safety.

Rule of thumb: *if the condition can be false because of something outside this crate,
it's an error to handle, not an assert.* Give every `debug_assert!` a message naming the
invariant, and cover it in the enclosing block's semantic markup doc comment.

---

## Architecture

Plugin-based architecture. Do **not** put feature logic in core modules.

```
src/
├── main.rs            # Entry point: load plugins, dispatch commands
├── steam_api.rs       # HTTP client for Steam API; data structures for responses
├── steam_client.rs    # Domain interfaces for Steam operations
├── ui.rs              # Shared UI utilities for consistent formatted output
└── plugins/
    ├── mod.rs             # Plugin trait definition + plugin registry
    ├── interactive.rs     # TUI interactive mode plugin
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
| `ratatui` | Terminal UI rendering |
| `mockito` (dev) | HTTP mocking in tests |
| `gag` (dev) | Capture stdout/stderr in tests |

**Important:** `reqwest` uses `rustls-tls` (not `native-tls`) to avoid OpenSSL incompatibilities on Ubuntu 22.04.

---

## Git Command Restrictions (Non-Negotiable)

**Never run any git command that changes repository state** — `git commit`, `git push`,
`git checkout -b`, `git merge`, `git rebase`, `git reset`, `git stash`, `git tag`, etc. are all
**FORBIDDEN**, including via hooks or scripts. Only **read-only** git commands are permitted:
`git status`, `git diff`, `git log`, `git show`, `git blame`, and similar. The user handles all
git state changes themselves. If a task seems to call for a commit or other state change,
make the file changes and stop — tell the user what's ready, and let them run the git command.

---

## Development Rules

- Language: **Rust only** — no other languages in the implementation.
- All changes must go through pull requests with at least one approval.
- Before submitting changes, run: `cargo fmt --check`,
  `cargo clippy --all-targets -- -D clippy::all`, `cargo test`, and
  `cargo llvm-cov --fail-under-lines 79`. See "Safe Rust & Enforcement" above.
- Enable the local git hooks once per clone with
  `git config core.hooksPath .githooks`, and do not skip them (`--no-verify`) or bypass
  commit signing. CI (`.github/workflows/ci.yml`) re-runs the same checks as the
  backstop for anyone who hasn't enabled the hooks.
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
- `docs/adr/` — architecture decision records for hard-to-reverse, non-obvious choices.
