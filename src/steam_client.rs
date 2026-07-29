//! Domain-level seam between plugins and the Steam API transport.
//!
//! <purpose-start>
//! Plugins previously called a concrete HTTP struct directly, so `reqwest::Error`
//! and wire-format JSON leaked into every plugin and every plugin test had to
//! stand up a live mockito server. This module defines `SteamClient`, a
//! trait-based seam with domain-only inputs/outputs, so adapters (HTTP,
//! fake-for-tests) can be swapped without touching plugin logic.
//! <purpose-end>

use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt;

// A game owned by the user.
#[derive(Debug, Clone, PartialEq)]
pub struct Game {
    pub appid: u32,
    pub name: String,
    pub playtime_forever: u32,
    pub img_icon_url: String,
    pub playtime_windows_forever: u32,
    pub playtime_mac_forever: u32,
    pub playtime_linux_forever: u32,
    pub rtime_last_played: u64,
    pub playtime_disconnected: u32,
}

// A single achievement for a game, optionally enriched with its global unlock percentage.
#[derive(Debug, Clone, PartialEq)]
pub struct Achievement {
    pub apiname: String,
    pub achieved: u8,
    pub unlocktime: u64,
    pub name: String,
    pub description: String,
    pub global_percent: Option<f32>,
}

// The global unlock percentage for one achievement, across all Steam players.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalAchievement {
    pub name: String,
    pub percent: f32,
}

// The achievements for one game, plus the game's display name.
#[derive(Debug, Clone, PartialEq)]
pub struct AchievementSet {
    pub game_name: String,
    pub achievements: Vec<Achievement>,
}

// The result of resolving a user-supplied game query (numeric app id or name substring).
#[derive(Debug, Clone, PartialEq)]
pub enum GameMatch {
    One(Game),
    Many(Vec<Game>),
    None,
}

// Domain-level errors a `SteamClient` adapter can produce.
//
// <purpose-start>
// Callers across every plugin need one error type they can display directly,
// instead of each plugin hand-writing its own "Error while trying to X: {e}"
// prose around a leaked `reqwest::Error`.
// <purpose-end>
#[derive(Debug, Clone)]
pub enum SteamError {
    Config(String),
    PrivateProfile,
    NoStats { appid: u32 },
    Http { status: Option<u16>, msg: String },
    Decode(String),
}

impl fmt::Display for SteamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SteamError::Config(msg) => write!(f, "Configuration error: {}", msg),
            SteamError::PrivateProfile => {
                write!(f, "Steam profile is private or has no public data")
            }
            SteamError::NoStats { appid } => {
                write!(f, "Game {} has no achievement stats", appid)
            }
            SteamError::Http {
                status: Some(status),
                msg,
            } => write!(f, "Steam API request failed (status {}): {}", status, msg),
            SteamError::Http { status: None, msg } => {
                write!(f, "Steam API request failed: {}", msg)
            }
            SteamError::Decode(msg) => write!(f, "Failed to parse Steam API response: {}", msg),
        }
    }
}

impl std::error::Error for SteamError {}

// The domain-level seam plugins call through instead of a concrete HTTP client.
//
// <purpose-start>
// Two adapters (HTTP for production, fake for tests) implement this trait so
// plugin logic and plugin tests never depend on `reqwest` or wire JSON shapes.
// <purpose-end>
#[async_trait]
pub trait SteamClient: Send + Sync {
    async fn owned_games(&self) -> Result<Vec<Game>, SteamError>;
    async fn achievements(&self, appid: u32) -> Result<AchievementSet, SteamError>;
    async fn global_percentages(&self, appid: u32) -> Result<Vec<GlobalAchievement>, SteamError>;

    // Fetches a game's achievements enriched with global unlock percentages.
    //
    // <purpose-start>
    // Centralizes the apiname join that `list_achievements` used to inline,
    // so `show_progress` and `dashboard` get it for free. A failure to fetch
    // global percentages is not fatal: achievements are still returned, just
    // without enrichment (`global_percent: None`).
    // <purpose-end>
    async fn achievements_with_global(&self, appid: u32) -> Result<AchievementSet, SteamError> {
        let mut set = self.achievements(appid).await?;

        if let Ok(globals) = self.global_percentages(appid).await {
            let percent_by_apiname: HashMap<String, f32> =
                globals.into_iter().map(|g| (g.name, g.percent)).collect();

            for achievement in set.achievements.iter_mut() {
                achievement.global_percent = percent_by_apiname.get(&achievement.apiname).copied();
            }
        }

        Ok(set)
    }

    // Resolves a user-supplied query to an owned game.
    //
    // <purpose-start>
    // Centralizes the name/appid resolution `list_achievements` used to own
    // exclusively, so `show_progress` and `dashboard` can accept names too.
    // <purpose-end>
    //
    // <side-effects-start>
    // - Behaviour change from the old `list_achievements`-only resolver: a
    //   numeric app id the account does not own no longer reaches the
    //   achievements endpoint; it falls through to name matching.
    // <side-effects-end>
    async fn resolve(&self, query: &str) -> Result<GameMatch, SteamError> {
        let games = self.owned_games().await?;

        if let Ok(appid) = query.parse::<u32>() {
            if let Some(game) = games.iter().find(|g| g.appid == appid) {
                return Ok(GameMatch::One(game.clone()));
            }
        }

        let query_lower = query.to_lowercase();
        let matches: Vec<Game> = games
            .into_iter()
            .filter(|g| g.name.to_lowercase().contains(&query_lower))
            .collect();

        Ok(match matches.len() {
            0 => GameMatch::None,
            1 => GameMatch::One(matches.into_iter().next().unwrap()),
            _ => GameMatch::Many(matches),
        })
    }
}

#[cfg(test)]
pub mod fake {
    use super::*;
    use std::collections::HashMap;

    // A canned `SteamClient` for plugin tests: returns pre-set data or errors
    // without opening a socket or hand-writing wire JSON.
    #[derive(Default)]
    pub struct FakeSteam {
        games: Option<Result<Vec<Game>, SteamError>>,
        achievements: HashMap<u32, Result<AchievementSet, SteamError>>,
        global: HashMap<u32, Result<Vec<GlobalAchievement>, SteamError>>,
    }

    impl FakeSteam {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn with_games(mut self, games: Vec<Game>) -> Self {
            self.games = Some(Ok(games));
            self
        }

        pub fn with_games_error(mut self, err: SteamError) -> Self {
            self.games = Some(Err(err));
            self
        }

        pub fn with_achievements(mut self, appid: u32, set: AchievementSet) -> Self {
            self.achievements.insert(appid, Ok(set));
            self
        }

        pub fn with_achievements_error(mut self, appid: u32, err: SteamError) -> Self {
            self.achievements.insert(appid, Err(err));
            self
        }

        pub fn with_global(mut self, appid: u32, achievements: Vec<GlobalAchievement>) -> Self {
            self.global.insert(appid, Ok(achievements));
            self
        }

        pub fn with_global_error(mut self, appid: u32, err: SteamError) -> Self {
            self.global.insert(appid, Err(err));
            self
        }
    }

    #[async_trait]
    impl SteamClient for FakeSteam {
        async fn owned_games(&self) -> Result<Vec<Game>, SteamError> {
            self.games.clone().unwrap_or(Ok(Vec::new()))
        }

        async fn achievements(&self, appid: u32) -> Result<AchievementSet, SteamError> {
            self.achievements
                .get(&appid)
                .cloned()
                .unwrap_or(Err(SteamError::NoStats { appid }))
        }

        async fn global_percentages(
            &self,
            appid: u32,
        ) -> Result<Vec<GlobalAchievement>, SteamError> {
            self.global
                .get(&appid)
                .cloned()
                .unwrap_or(Ok(Vec::new()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fake::FakeSteam;
    use super::*;

    fn game(appid: u32, name: &str) -> Game {
        Game {
            appid,
            name: name.to_string(),
            playtime_forever: 0,
            img_icon_url: String::new(),
            playtime_windows_forever: 0,
            playtime_mac_forever: 0,
            playtime_linux_forever: 0,
            rtime_last_played: 0,
            playtime_disconnected: 0,
        }
    }

    fn achievement(apiname: &str, achieved: u8) -> Achievement {
        Achievement {
            apiname: apiname.to_string(),
            achieved,
            unlocktime: 0,
            name: apiname.to_string(),
            description: String::new(),
            global_percent: None,
        }
    }

    #[tokio::test]
    async fn resolve_matches_owned_appid_before_name() {
        let client = FakeSteam::new().with_games(vec![game(123, "Game 123")]);

        let result = client.resolve("123").await.unwrap();

        assert_eq!(result, GameMatch::One(game(123, "Game 123")));
    }

    #[tokio::test]
    async fn resolve_falls_back_to_name_when_appid_not_owned() {
        let client = FakeSteam::new().with_games(vec![game(456, "Game 123")]);

        let result = client.resolve("123").await.unwrap();

        // "123" parses as a u32 but isn't owned, so it falls through to a
        // substring match on the name, per ADR 0001's documented behaviour
        // change.
        assert_eq!(result, GameMatch::One(game(456, "Game 123")));
    }

    #[tokio::test]
    async fn resolve_returns_many_on_ambiguous_name() {
        let client =
            FakeSteam::new().with_games(vec![game(1, "Foo One"), game(2, "Foo Two")]);

        let result = client.resolve("foo").await.unwrap();

        match result {
            GameMatch::Many(games) => assert_eq!(games.len(), 2),
            other => panic!("expected Many, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn resolve_returns_none_when_nothing_matches() {
        let client = FakeSteam::new().with_games(vec![game(1, "Foo")]);

        let result = client.resolve("bar").await.unwrap();

        assert_eq!(result, GameMatch::None);
    }

    #[tokio::test]
    async fn achievements_with_global_joins_by_apiname() {
        let client = FakeSteam::new()
            .with_achievements(
                123,
                AchievementSet {
                    game_name: "Test Game".to_string(),
                    achievements: vec![achievement("ach1", 1), achievement("ach2", 0)],
                },
            )
            .with_global(
                123,
                vec![
                    GlobalAchievement {
                        name: "ach1".to_string(),
                        percent: 50.5,
                    },
                ],
            );

        let set = client.achievements_with_global(123).await.unwrap();

        assert_eq!(set.achievements[0].global_percent, Some(50.5));
        assert_eq!(set.achievements[1].global_percent, None);
    }

    #[tokio::test]
    async fn achievements_with_global_keeps_achievements_when_global_fetch_fails() {
        let client = FakeSteam::new()
            .with_achievements(
                123,
                AchievementSet {
                    game_name: "Test Game".to_string(),
                    achievements: vec![achievement("ach1", 1)],
                },
            )
            .with_global_error(123, SteamError::Http { status: Some(500), msg: "boom".to_string() });

        let set = client.achievements_with_global(123).await.unwrap();

        assert_eq!(set.achievements[0].global_percent, None);
    }
}
