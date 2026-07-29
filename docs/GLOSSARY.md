# Trogue Domain Glossary

Domain language for trogue. Architecture terms (module, interface, seam, adapter,
depth) come from the improve-codebase-arch skill's LANGUAGE.md and are not
repeated here.

## Steam domain

**Game**
A game in the user's Steam library, identified by its **app id**. Carries
playtime and last-played metadata (`steam_api::Game` today).

**App id**
Steam's numeric identifier for a game (`u32`). The only identifier the Steam
achievement endpoints accept.

**Owned games**
The list of games belonging to the configured Steam account
(`IPlayerService/GetOwnedGames`).

**Achievement**
A per-player achievement record for one game: api name, unlocked flag, unlock
time, display name, description. `achieved > 0` means unlocked.

**Global achievement percentage**
The percentage of all Steam players who unlocked a given achievement
(`GetGlobalAchievementPercentagesForApp`). Joined to the player's achievements
by **api name**.

**Api name**
Steam's internal string key for an achievement (`apiname`). The join key
between player achievements and global percentages. Distinct from the
achievement's display name.

## Seam vocabulary (decided in [ADR 0001](adr/0001-steam-client-trait-seam.md))

**Steam client**
The deep module fronting all Steam interaction: `trait SteamClient`. The only
seam through which plugins reach Steam. Speaks domain types only — no HTTP,
URLs, or wire envelopes cross it.

**Achievement set**
`AchievementSet { game_name, achievements }` — a game's name together with the
player's achievements for it. Replaces the unlabelled
`(String, Vec<Achievement>)` tuple.

**Resolution / game match**
Turning user input (numeric app id or name fragment) into an owned game:
`resolve(query) → GameMatch::{One, Many, None}`. Numeric input is tried as an
app id against owned games first, then case-insensitive substring match on
names.

**HTTP adapter**
`HttpSteamClient` — the production adapter at the Steam client seam. Owns
credentials (from `TROGUE_STEAM_API_KEY` / `TROGUE_STEAM_ID`), a reused
`reqwest::Client`, URL construction, and the wire envelopes.

**Fake adapter**
`FakeSteam` — in-memory test adapter at the same seam, constructed with canned
games/achievements. No sockets, no JSON.

## Interactive mode (TUI)

**Interactive mode**
The full-screen terminal UI trogue enters when run with no arguments (or via
`trogue interactive`). Two screens: the game list and the game detail view.
Read-only — it reaches Steam only through the **Steam client** seam.

**Game detail view**
The per-game screen opened with Enter from the game list: one game's completion
progress bar plus its achievement table. Distinct from **dashboard**, which is
the existing multi-game recently-played overview.

**Achievement view mode**
Which subset of a game's achievements the detail view shows: `All`,
`Remaining` (locked, `achieved == 0`), or `Unlocked` (`achieved > 0`).
Orthogonal to **achievement sort mode**.

**Achievement sort mode**
The ordering of the achievement table: by name, by unlock time, or by **global
achievement percentage**, each with a direction. Orthogonal to **achievement
view mode**.

**Effect**
A data value returned by a pure state transition describing work the event loop
must perform (`FetchAchievements(appid)`, `Quit`, …). Keeps I/O out of the state
model so transitions are unit-testable without a terminal or a network.
