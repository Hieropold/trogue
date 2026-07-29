**ADR 0002 — Adopt ratatui and retire hand-rolled crossterm rendering**

Interactive mode needs stateful list selection, scrolling viewports, a progress gauge,
a column-aligned table, and flicker-free redraw on resize. Hand-rolling these on
crossterm 0.23 (as the dead `src/tui.rs` began to) meant hundreds of lines of viewport
and cursor math with no test seam. We adopt `ratatui` 0.30.x and take crossterm through
its re-export (`ratatui::crossterm`), dropping the direct `crossterm = "0.23"`
dependency so no duplicate versions exist. Trade-off accepted: one new dependency and a
crossterm 0.23 → ~0.29 bump that touches existing call sites, in exchange for the
widget set, double buffering, and a `TestBackend` that makes rendering assertable —
which is what lets interactive mode satisfy the TDD principle at all.

Meets the ADR bar: hard to reverse (the whole TUI is written against ratatui's widget
and buffer model), surprising without context (why the crossterm version jumped and why
the direct dependency vanished), and a real trade-off existed (hand-rolling was viable).
