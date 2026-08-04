# Task: Add PSN trophy support

## Origin

Prompt-based task (no source ticket): "Are there any feasible way to add support for PSN
trophies?" Refined via `/refine-task` on 2026-08-04.

## Context

trogue is Steam-only today: `trait SteamClient` (`src/steam_client.rs:108`) is the single
seam where wire knowledge lives, and every plugin receives `&dyn SteamClient`. The question
was whether PSN trophies can be supported at all, and if so how.

**Feasibility verdict: yes, with a caveat.** Sony publishes no trophy API for individuals.
The only working route is the reverse-engineered PSN web API — NPSSO cookie → OAuth code →
access/refresh token → `m.np.playstation.com/api/trophy/v1/...`. This is what `psn-api` (JS),
`psnawp` (Python) and `psn_api_rs` (Rust, unmaintained since ~2020) all do. It is
undocumented, unversioned, ToS-grey, and heavy use has reportedly triggered PSN account
restrictions. **Decision: accept this**, document the risk in an ADR, keep request volume
minimal, and isolate the blast radius so a Sony-side change breaks PSN without breaking Steam.

Intended outcome: `trogue list`, `dashboard`, `achievements`, `progress` and interactive mode
show Steam games and PSN games **in one merged library**, tagged by platform, with PSN trophy
grades visible.

Two facts discovered during refinement that shape the design:

- PSN's `trophyTitles` endpoint already returns per-title `progress` and
  `earnedTrophies`/`definedTrophies` by grade. **Dashboard needs zero per-game PSN calls** —
  it needs N per-game calls for Steam only. This is the main rate-limit mitigation.
- `trophyEarnedRate` ships inside the per-title trophy list, so PSN gets global-percentage
  enrichment for free (1 call, where Steam needs 2).
- PSN exposes **no playtime**. `lastUpdatedDateTime` is the only temporal field.
- The NPSSO is a **bootstrap** credential, not a standing one: it mints a refresh token good
  for ~2 months, after which the user must repeat a manual browser step. See "PSN
  authentication" below.

## PSN authentication: how the NPSSO is obtained and how long it lasts

**Obtaining it is inherently manual and cannot be automated.** Sony's sign-in has 2FA and
CAPTCHA, so there is no headless path. The user must:

1. Sign in at `https://www.playstation.com/` in a browser.
2. In the *same* browser, open `https://ca.account.sony.com/api/v1/ssocookie`.
3. Copy the 64-character value from `{"npsso":"<token>"}`.

trogue then performs the standard three-step exchange (constants are the PlayStation Android
app's, as used by every existing unofficial PSN client):

| Step | Request |
|---|---|
| 1. Authorization code | `GET /authz/v3/oauth/authorize` with header `Cookie: npsso=<token>`, `client_id=09515159-7237-4370-9b40-3806e67c0891`, `redirect_uri=com.scee.psxandroid.scecompcall://redirect`, `scope=psn:mobile.v2.core psn:clientapp`, `response_type=code`, `access_type=offline`. Code comes back in the redirect's query string. |
| 2. Token exchange | `POST /authz/v3/oauth/token`, `grant_type=authorization_code`, HTTP Basic auth with the app's client id/secret. Returns `access_token`, `refresh_token`, `expires_in`, `refresh_token_expires_in`. |
| 3. Refresh | `POST /authz/v3/oauth/token`, `grant_type=refresh_token`. |

**Lifetimes:**

| Token | Lifetime | Consequence |
|---|---|---|
| Access token | ~1 hour (`expires_in`) | Refreshed transparently on 401; user never sees this. |
| Refresh token | **~2 months** (`refresh_token_expires_in`) | The real cadence — no user interaction needed within this window. |
| NPSSO | ~2 months, **invalidated immediately if a new NPSSO is generated for the same account** | Authenticating any other PSN tool with the same account silently breaks trogue's *bootstrap* (not an already-issued refresh token). |

This is why the NPSSO is treated as a one-time bootstrap rather than a permanent env var like
`TROGUE_STEAM_API_KEY`: it's needed once per ~60 days, and parking a full account session
credential in `.bashrc` would outlive its own usefulness. `trogue psn login` reads it
interactively from stdin with echo disabled (reusing `ratatui::crossterm` raw mode, already a
dependency — no new crate). `TROGUE_PSN_NPSSO` remains as a non-interactive/CI fallback.

Because re-auth requires a manual browser round-trip, every auth failure message inlines the
two URLs above plus the `trogue psn login` command, and the refresh-token expiry is persisted
so trogue can warn on stderr once under 7 days remain — before the user hits a surprise
failure mid-command.

## Key decisions reached

| Area | Decision |
|---|---|
| API route | Unofficial NPSSO/OAuth web API; risk accepted and recorded in an ADR |
| UX | Unified merged library (not `--platform`, not separate `psn-*` commands) |
| Game identity | `enum GameId { Steam(u32), Psn(String) }`, `Display` → `steam:440` / `psn:NPWR12345_00`; bare numeric input still resolves to Steam for back-compat |
| Merge wiring | Rename the trait neutral; add `MultiSource` that **implements the same trait**. Plugins keep taking one `&dyn` and contain no merge logic |
| Partial failure | Unconfigured source → silently omitted. Configured-but-failing → omitted + stderr warning naming the platform. Exit 0 if any source succeeded; exit 1 only if all failed |
| Trophy grade | `grade: Option<TrophyGrade>` on the shared `Achievement`; `None` for Steam |
| Steam-only fields | `playtime_forever: Option<u32>`, `img_icon_url: Option<String>`; drop the three per-OS playtime variants and `playtime_disconnected`; PSN `lastUpdatedDateTime` → epoch `rtime_last_played` so dashboard's sort works across both |
| Credentials | `trogue psn login` prompts for the NPSSO on stdin (no echo, not argv); `TROGUE_PSN_NPSSO` env var is a non-interactive fallback. Access+refresh token cached at `$XDG_CACHE_HOME/trogue/psn.json`, mode `0600`; refresh on 401 |
| Expiry handling | Persist the absolute refresh-token expiry; warn on stderr when under 7 days remain, naming the date and the `trogue psn login` fix |
| Request policy | PSN strictly serial with a small fixed delay; no concurrency; no response cache |
| Slicing | Three sequential PRs (below) |
| New dependencies | **None.** `reqwest` (JSON + rustls), `serde`, `chrono`, `tokio` already cover OAuth-over-HTTPS, ISO-8601 parsing, and the token cache. `psn_api_rs` (the one existing Rust crate) was rejected — unmaintained since ~2020. |

## Proposed slicing

### Slice 1 — Platform-neutral refactor (no PSN yet)
Pure refactor. Tests stay green, no new deps, no network change. Shippable on its own.

- `src/steam_client.rs` → `src/game_library.rs`: `SteamClient` → `GameLibrary`,
  `SteamError` → `PlatformError`; de-Steam the five `Display` arms (`steam_client.rs:78-98`,
  note `dashboard.rs:246` asserts on current strings); `NoStats { appid: u32 }` →
  `NoStats { id: GameId }`.
- Add `enum Platform { Steam, Psn }` and `enum GameId { Steam(u32), Psn(String) }` with
  `Display`/`FromStr`.
- `Game.appid: u32` → `id: GameId` + `platform: Platform`; Option-ise Steam-only fields per
  the table above; `Achievement` gains `grade: Option<TrophyGrade>`.
- Trait methods take `&GameId`. Keep both default methods (`achievements_with_global`,
  `resolve` at `steam_client.rs:122,149`) — genuinely reusable cross-platform logic. In
  `resolve`, `GameId::from_str` replaces the bare `query.parse::<u32>()`.
- `pub mod fake`'s `FakeSteam` → `FakeLibrary` (`steam_client.rs:173`).
- New `src/multi_source.rs`: `MultiSource { sources: Vec<Box<dyn GameLibrary>> }`, itself
  `impl GameLibrary`; concatenates `owned_games()`, routes `achievements`/
  `global_percentages` on the `GameId` variant.
- `src/main.rs`: move source construction to **after** clap parsing so
  `trogue completions bash` runs with zero credentials (fails today at `main.rs:38`).
- `src/ui.rs`: `'i'` token emits `GameId::Display`; add grade/platform tokens; fix the
  `.unwrap()`/`.expect()` at `ui.rs:262-266` while touching this function.
- `src/plugins/interactive.rs`: `achievements_cache: HashMap<u32, _>` → keyed by `GameId`
  (`interactive.rs:80`), `Effect::FetchAchievements(GameId)`, `SteamError::NoStats` match at
  `interactive.rs:418`.
- `plugins/mod.rs:134`'s `assert_eq!(plugins.len(), 6)`; `completions.rs:129`'s duplicated
  `.about(...)`.

### Slice 2 — PSN adapter
New `src/psn_api.rs`, mirroring `steam_api.rs`'s proven shape (private wire structs +
`From<Wire…>`).

- Auth: the three-step NPSSO/OAuth exchange (see "PSN authentication" above). Cache at
  `$XDG_CACHE_HOME/trogue/psn.json`, mode `0600` set at creation (no chmod race), holding
  `access_token`, `refresh_token`, and both absolute expiry timestamps; refresh on 401, once,
  then fail with re-auth instructions. `&self` trait methods need interior mutability
  (`tokio::sync::RwLock` around token state) — no precedent for this in `steam_api.rs`. The
  NPSSO itself is never written to disk and is zeroed from memory after the exchange.
- `owned_games()` → `trophyTitles` (maps `lastUpdatedDateTime` → epoch, retains grade
  breakdown + `progress`); `achievements(GameId::Psn)` → per-title trophy list, with
  `trophyEarnedRate` populating `global_percent` in the same pass.
- Mirror and extend `map_transport_error`'s `e.without_url()` discipline
  (`steam_api.rs:271`) so the bearer token never reaches an error string and the NPSSO is
  never logged — copy the pattern of `test_transport_error_never_leaks_api_key`
  (`steam_api.rs:586`).
- Serial requests, small fixed inter-request delay.
- **New `src/plugins/psn_auth.rs`**: a `psn` plugin with `login`/`logout`/`status`
  subcommands. `login` prints the two acquisition URLs, then reads the NPSSO from stdin with
  echo disabled (reusing `ratatui::crossterm` raw mode — no new crate), falling back to
  `TROGUE_PSN_NPSSO` when stdin isn't a TTY. On success it prints the refresh-token expiry.
  `logout` deletes the cache; `status` reports validity and expiry. Registering this plugin
  bumps `plugins/mod.rs:134`'s `assert_eq!(plugins.len(), 6)` to 7 and needs a
  `completions.rs:129` update.
- **Expiry warning**: any command checks the cached refresh-token expiry and prints a
  one-line stderr warning once under 7 days remain, naming the date and `trogue psn login`.
  Pure timestamp comparison — no extra API call.
- mockito tests confined to `psn_api.rs` (ADR 0001 pattern): authorize-code step, token
  exchange, refresh-on-401, expired-refresh-token produces the re-auth message, invalid NPSSO
  rejected, all four grade mappings, ISO-8601 conversion, token-leak test. Expiry-warning
  threshold is a pure function, unit-tested with no I/O.

### Slice 3 — Merge UX
- Partial-failure warnings from `MultiSource` to stderr; exit-code policy per the table
  (closes ADR 0001's open "Candidate 5").
- Platform column/tag in `list`/`dashboard`; dashboard reads PSN progress from cached
  `trophyTitles` data, issuing zero per-game PSN calls.
- Interactive mode: grade in detail view; sort-by-grade alongside existing
  sort-by-global-percent (`interactive.rs:154`).

## Proposed GLOSSARY.md updates

`docs/GLOSSARY.md` currently has no platform-neutral vocabulary — every term in its first two
sections is Steam-scoped by construction. Add:

- **Platform** — the game/trophy source a game belongs to (`Steam` or `Psn`).
- **Game library** — the trait (`GameLibrary`, formerly `SteamClient`) plugins call through;
  implementations include the HTTP adapters, `MultiSource`, and the test fake.
- **Game id** — `GameId`, an id tagged by platform (`Steam(u32)` app id, `Psn(String)`
  `npCommunicationId`); renders as `steam:440` / `psn:NPWR12345_00`.
- **Trophy** — PSN's term for an achievement; represented via the existing `Achievement`
  type with `grade: Option<TrophyGrade>` set.
- **Trophy grade** — PSN's bronze/silver/gold/platinum tier; `None` for Steam achievements.
- **Source** — one platform's `GameLibrary` implementation, as held by `MultiSource`.
- **Merged library** — the concatenated view of all configured sources' owned games.

Also revise the existing Steam-scoped entries (*App id*, *Steam client*) to point at these
neutral terms rather than being redefined.

## Proposed ADRs

**ADR: "Support PSN trophies via the unofficial web API."** Meets all three criteria: hard to
reverse (auth model, on-disk secret, domain model changes ripple through the whole plugin
layer), surprising without context (why an undocumented reverse-engineered endpoint is the
implementation, not a bug), and a real trade-off existed (no official API exists; a
local-export alternative was considered and rejected for having no live sync). Body: records
the NPSSO/OAuth flow, the account-restriction risk, and the decision to accept it with a
serial/no-concurrency request policy as mitigation.

**ADR: "Unified merged library over per-platform commands."** Hard to reverse (touches the
trait, every plugin, and `interactive.rs`), surprising (a `--platform` flag or separate
`psn-*` commands were the cheaper alternatives and were explicitly rejected), real trade-off
(UX quality vs. implementation cost). Body: records why `Game` lost its per-OS playtime
fields, why `GameId` is an enum rather than an opaque string, and why `MultiSource`
implements the same trait rather than plugins merging sources themselves.

## Proposed security considerations

- **New trust boundary + credential class.** The NPSSO is a *full PSN account session
  credential*, not a scoped API key like Steam's — anyone holding it has the user's PSN
  account. It must never be written to disk, logged, or included in an error string; only
  the derived access/refresh tokens are cached.
- **Credential entry path.** No-echo stdin (via `trogue psn login`) rather than argv or a
  permanent env var is deliberate: argv is world-readable via `ps`, and a permanent env var
  would park a session credential in `.bashrc` long past its ~60-day usefulness. The
  `TROGUE_PSN_NPSSO` env var remains only as a documented non-interactive/CI fallback.
- **New secret at rest**: `$XDG_CACHE_HOME/trogue/psn.json`, mode `0600`, set at file
  creation (not chmod'ed after — avoids a race window). Needs a documented deletion path
  (`trogue psn logout`).
- **Token rotation is bounded and Sony-enforced** (~60 days) — a leaked cache file grants at
  most two months of access, and re-auth requires an interactive browser sign-in an attacker
  holding only the file cannot perform.
- **New outbound hosts**: `ca.account.sony.com`, `m.np.playstation.com` (both HTTPS — note
  existing Steam adapter uses plain **http**, `steam_api.rs:17`, worth revisiting separately).
- **Account-restriction risk** from PSN's undocumented rate limits is the reason for the
  serial/no-concurrency request policy — no bounded-concurrency or response-cache slice was
  approved for v1.
- **PII**: PSN account id and trophy history become personal data cached locally. No
  server-side store exists, so the deletion pathway is deleting the cache file — must be
  documented explicitly, not left implicit.
- **No new dependencies**, so nothing to clear through the `implement-feature` Dependency Gate.

## Deferred / not yet decided (raise at implementation time)

- Whether `global_percentages` should remain a required trait method now that PSN gets the
  data for free in the trophy call, or become a default method returning empty.
  **Recommended:** make it a default method once a second implementation exists to confirm
  the pattern; keep required for now since only Steam implements it today.
- Whether PSN's `platinum` trophy should count toward completion percentage or be excluded
  (it's auto-awarded for earning all others, so including it is arguably double-counting).
  **Recommended:** exclude platinum from the completion-percentage denominator; show it
  separately as "100% + platinum".
- Whether multiple PSN accounts / a friend's public profile should be supported.
  **Recommended:** no — own account only for v1, matching Steam's current single-account scope.
- `reqwest` is pinned at 0.11 (current is 0.12). Not required by this work, but implementing
  PSN auth is a natural moment to consider the bump. **Recommended:** separate PR, not
  bundled into slice 2.

## Verification plan

Per slice: `cargo fmt --check`, `cargo clippy --all-targets -- -D clippy::all`,
`cargo test`, `cargo llvm-cov --fail-under-lines 79`.

End-to-end, after slice 2:
- Steam-only run unchanged apart from id token format (regression check for slice 1).
- `trogue completions bash` succeeds with zero credentials set (fails today).
- PSN-only run: `list`, `dashboard`, `achievements <name>`, `progress <id>`.
- Both configured: merged list shows both platforms tagged; PSN rows show grades.
- Break Steam creds only → PSN results still render, stderr names Steam, exit 0. Break
  both → exit 1.
- `trogue psn login` with a real NPSSO → succeeds, reports expiry, creates cache at mode
  `600`; confirm the typed NPSSO is **not echoed**. `trogue psn status`/`logout` work.
- Delete token cache and unset `TROGUE_PSN_NPSSO` → Steam results still render; PSN warning
  names `trogue psn login` with both acquisition URLs.
- Hand-edit the cached expiry to 3 days out → under-7-days warning fires on an ordinary
  command; set it to the past → re-auth instructions, not a raw HTTP error.
- `echo $NPSSO | trogue psn login` (non-TTY) → uses the fallback without hanging on a prompt.
- Grep all output, the cache file, and error paths for the NPSSO and bearer token — the
  NPSSO must appear nowhere; the bearer token only in the cache file. Confirm `history` and
  `ps aux` during login contain neither.

## Recommended next step

No deferred questions block the design — the "Deferred / not yet decided" items above are
recommendations, not open blockers, since they only affect implementation-time detail, not
architecture. **Recommend proceeding straight to implementation**, starting with Slice 1
(hand off to `implement-feature`, referencing this file). Slices 2 and 3 depend on Slice 1
landing first.
