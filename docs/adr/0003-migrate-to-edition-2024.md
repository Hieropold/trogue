**ADR 0003 — Migrate to Rust edition 2024**

The toolchain pin moved from 1.90.0 to 1.97.1 (`rust-toolchain.toml`), and with it we adopt
edition 2024 (stabilized in 1.85), migrating from 2021 via `cargo fix --edition` followed by
flipping `edition` in `Cargo.toml` and `rustfmt.toml`. Trade-off accepted: edition 2024 changes
the drop order of temporaries in tail expressions (`rust-2024-compatibility` flagged one site in
`main.rs::main`, around the per-plugin dispatch loop) — verified harmless since no type in this
crate implements `Drop`, so nothing here depends on the old ordering. In exchange, `let_chains`
stabilize, letting clippy collapse three nested `if let` pairs (`steam_client.rs::resolve`,
`interactive.rs::run`'s event loop and its achievement-loaded handler) into single conditions,
which is a real readability win, not just churn.

Meets the ADR bar: hard to reverse (edition is a per-crate, crate-wide switch, not something to
flip per-PR), surprising without context (why drop order came up in a routine version bump), and
a real trade-off existed (edition 2021 remained available and required no action).
