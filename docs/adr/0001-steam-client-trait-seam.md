# ADR 0001: Steam client becomes a domain-level trait seam

- Status: **Accepted — not yet implemented**
- Date: 2026-07-29
- Deciders: Hieropold (grilling loop with Claude, improve-codebase-arch review)

## Context

`steam_api::Api` is a concrete struct, so no seam exists between plugin logic
and HTTP:

- `reqwest::Error` crosses into all five plugins; users see reqwest prose.
- Wire envelopes (`PlayerStatsResponse`, …) are `pub` but unused outside.
- `base_url` is a test hook in the production constructor, threaded through
  `constants.rs` and `app.rs` solely so tests can inject a mockito URL.
- Every plugin test — including tests of pure logic like name→appid
  resolution — must stand up a live mockito TCP server and hand-write wire
  JSON (a third copy of the envelope shapes).
- Each call uses bare `reqwest::get`: a fresh client per request (~11 per
  `dashboard` run).
- Name→appid resolution exists only inside `list_achievements`;
  `show_progress` rejects names. The global-percentage join by `apiname` is
  inlined in `list_achievements`.

## Decision

Introduce one deep module at a domain-level seam: `trait SteamClient`, with
two adapters — `HttpSteamClient` (production) and `FakeSteam` (tests). Two
adapters make the seam real.

### Interface

```rust
#[async_trait]
pub trait SteamClient: Send + Sync {
    // Required (endpoint-backed) methods
    async fn owned_games(&self) -> Result<Vec<Game>, SteamError>;
    async fn achievements(&self, appid: u32) -> Result<AchievementSet, SteamError>;
    async fn global_percentages(&self, appid: u32) -> Result<Vec<GlobalAchievement>, SteamError>;

    // Default methods — behaviour shared by all adapters, hidden behind the seam
    async fn achievements_with_global(&self, appid: u32) -> Result<AchievementSet, SteamError>;
    async fn resolve(&self, query: &str) -> Result<GameMatch, SteamError>;
}

pub struct AchievementSet {
    pub game_name: String,
    pub achievements: Vec<Achievement>,
}
// Achievement gains: pub global_percent: Option<f32>  (None until enriched)

pub enum GameMatch {
    One(Game),
    Many(Vec<Game>),
    None,
}

pub enum SteamError {
    Config(String),                        // missing/invalid env configuration
    PrivateProfile,                        // Steam success:false / 403
    NoStats { appid: u32 },                // game has no achievement stats
    Http { status: Option<u16>, msg: String },
    Decode(String),
}
```

Invariants and error modes (part of the interface):

- Domain types only cross the seam — no `reqwest` types, no wire envelopes.
- `achievements_with_global` performs the `apiname` join; achievements with no
  global entry keep `global_percent: None`.
- `resolve` precedence: parse the query as `u32` and match it as an app id
  against **owned games** first; otherwise case-insensitive substring match on
  game names. ⚠ Behaviour change: a numeric app id the account does not own
  no longer reaches the achievements endpoint (it falls through to name
  matching, likely `GameMatch::None`).
- Dispatch is dynamic: plugins receive `&dyn SteamClient` (via `async_trait`;
  the trait must stay object-safe). `Plugin::execute` takes `&dyn SteamClient`
  directly.

### Adapters

- `HttpSteamClient::from_env() -> Result<Self, SteamError>` reads
  `TROGUE_STEAM_API_KEY` / `TROGUE_STEAM_ID`; constructed ⇒ valid (no
  two-phase load). Holds one reused `reqwest::Client`. Owns URLs, wire
  envelopes (all private), status→`SteamError` mapping, and the `l=en` locale
  param. `base_url` override becomes a `#[cfg(test)]`-only constructor.
- `FakeSteam` (in `#[cfg(test)]` shared test support): constructed with canned
  games/achievements, returns them without sockets or JSON.

### Consequences for existing modules

- `cfg.rs` (two-phase load), `app.rs` (`AppContext` pass-through), and
  `constants.rs` are deleted; their residue folds into `HttpSteamClient`.
- Plugin tests migrate from mockito to `FakeSteam`; mockito remains only in
  `HttpSteamClient`'s own adapter tests (the single home of wire-format
  knowledge).
- The three duplicated error-prose variants in plugins collapse into
  `SteamError`'s `Display`.

## Consequences

- Leverage: one interface, four caller plugins; `resolve` and the global join
  become free for `show_progress` and `dashboard`.
- Locality: HTTP/JSON/credential knowledge concentrates in one adapter; a
  query-param change breaks one test file, not five.
- The interface is the test surface: pure plugin logic tests need no network.
- Cost: `async_trait` boxing per call (noise next to an HTTP round-trip);
  plugins' `execute` signature changes in all five plugins.

## Follow-ups (out of scope here)

- Candidate 2: single `execute → Result<ViewData, _>` path; `SteamError` is
  the error type it wants.
- Candidate 5: exit-code policy in `main.rs` keyed off `SteamError`.
