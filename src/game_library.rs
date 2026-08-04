//! Domain-level seam between plugins and platform API transport.
//!
//! <purpose-start>
//! Plugins previously called a concrete HTTP struct directly, so wire-format JSON
//! leaked into every plugin and every plugin test had to stand up a live mockito server.
//! This module defines `GameLibrary`, a trait-based seam with domain-only inputs/outputs,
//! so adapters (Steam HTTP, PSN HTTP, fake-for-tests) can be swapped without touching
//! plugin logic.
//! <purpose-end>

use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;

// The platform a game belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    Steam,
    Psn,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::Steam => write!(f, "steam"),
            Platform::Psn => write!(f, "psn"),
        }
    }
}

impl FromStr for Platform {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "steam" => Ok(Platform::Steam),
            "psn" => Ok(Platform::Psn),
            _ => Err(format!("Unknown platform: {}", s)),
        }
    }
}

// A game identifier tagged by platform.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum GameId {
    Steam(u32),
    Psn(String),
}

impl GameId {
    // Returns the platform for this game identifier.
    //
    // <purpose-start>
    // Allows routing operations to the corresponding platform adapter.
    // <purpose-end>
    //
    // <inputs-start>
    // - `&self`: The game identifier.
    // <inputs-end>
    //
    // <outputs-start>
    // - `Platform`: The platform enum variant.
    // <outputs-end>
    //
    // <side-effects-start>
    // - None.
    // <side-effects-end>
    pub fn platform(&self) -> Platform {
        match self {
            GameId::Steam(_) => Platform::Steam,
            GameId::Psn(_) => Platform::Psn,
        }
    }
}

impl fmt::Display for GameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GameId::Steam(appid) => write!(f, "steam:{}", appid),
            GameId::Psn(id) => write!(f, "psn:{}", id),
        }
    }
}

impl FromStr for GameId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(rest) = s.strip_prefix("steam:") {
            rest.parse::<u32>()
                .map(GameId::Steam)
                .map_err(|_| format!("Invalid Steam appid: {}", rest))
        } else if let Some(rest) = s.strip_prefix("psn:") {
            if rest.is_empty() {
                Err("Empty PSN id".to_string())
            } else {
                Ok(GameId::Psn(rest.to_string()))
            }
        } else if let Ok(appid) = s.parse::<u32>() {
            Ok(GameId::Steam(appid))
        } else {
            Err(format!("Invalid game id: {}", s))
        }
    }
}

impl From<u32> for GameId {
    fn from(appid: u32) -> Self {
        GameId::Steam(appid)
    }
}

// PSN trophy grade tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TrophyGrade {
    Bronze,
    Silver,
    Gold,
    Platinum,
}

impl fmt::Display for TrophyGrade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TrophyGrade::Bronze => write!(f, "Bronze"),
            TrophyGrade::Silver => write!(f, "Silver"),
            TrophyGrade::Gold => write!(f, "Gold"),
            TrophyGrade::Platinum => write!(f, "Platinum"),
        }
    }
}

impl FromStr for TrophyGrade {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bronze" => Ok(TrophyGrade::Bronze),
            "silver" => Ok(TrophyGrade::Silver),
            "gold" => Ok(TrophyGrade::Gold),
            "platinum" => Ok(TrophyGrade::Platinum),
            _ => Err(format!("Unknown trophy grade: {}", s)),
        }
    }
}

// A game owned by the user across any platform.
#[derive(Debug, Clone, PartialEq)]
pub struct Game {
    pub id: GameId,
    pub platform: Platform,
    pub name: String,
    pub playtime_forever: Option<u32>,
    pub img_icon_url: Option<String>,
    pub rtime_last_played: u64,
}

// A single achievement or trophy for a game, optionally enriched with global unlock percentage.
#[derive(Debug, Clone, PartialEq)]
pub struct Achievement {
    pub apiname: String,
    pub achieved: u8,
    pub unlocktime: u64,
    pub name: String,
    pub description: String,
    pub global_percent: Option<f32>,
    pub grade: Option<TrophyGrade>,
}

// The global unlock percentage for one achievement across players.
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

// The result of resolving a user-supplied game query (game id or name substring).
#[derive(Debug, Clone, PartialEq)]
pub enum GameMatch {
    One(Game),
    Many(Vec<Game>),
    None,
}

// Domain-level errors a `GameLibrary` adapter can produce.
//
// <purpose-start>
// Callers across every plugin need one error type they can display directly,
// instead of each plugin hand-writing its own error prose around transport failures.
// <purpose-end>
#[derive(Debug, Clone)]
pub enum PlatformError {
    Config(String),
    PrivateProfile,
    NoStats { id: GameId },
    Http { status: Option<u16>, msg: String },
    Decode(String),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformError::Config(msg) => write!(f, "Configuration error: {}", msg),
            PlatformError::PrivateProfile => {
                write!(f, "Profile is private or has no public data")
            }
            PlatformError::NoStats { id } => {
                write!(f, "Game {} has no achievement stats", id)
            }
            PlatformError::Http {
                status: Some(status),
                msg,
            } => write!(
                f,
                "Platform API request failed (status {}): {}",
                status, msg
            ),
            PlatformError::Http { status: None, msg } => {
                write!(f, "Platform API request failed: {}", msg)
            }
            PlatformError::Decode(msg) => {
                write!(f, "Failed to parse platform API response: {}", msg)
            }
        }
    }
}

impl std::error::Error for PlatformError {}

// The domain-level seam plugins call through instead of concrete HTTP clients.
//
// <purpose-start>
// Adapters (Steam HTTP, PSN HTTP, MultiSource, fake for tests) implement this trait so
// plugin logic and plugin tests never depend on transport details or wire JSON shapes.
// <purpose-end>
#[async_trait]
pub trait GameLibrary: Send + Sync {
    // Returns the primary platform of this library source.
    //
    // <purpose-start>
    // Identifies which platform source this implementation belongs to.
    // <purpose-end>
    //
    // <inputs-start>
    // - `&self`: Reference to the library adapter.
    // <inputs-end>
    //
    // <outputs-start>
    // - `Platform`: The platform enum variant.
    // <outputs-end>
    //
    // <side-effects-start>
    // - None.
    // <side-effects-end>
    fn platform(&self) -> Platform {
        Platform::Steam
    }

    async fn owned_games(&self) -> Result<Vec<Game>, PlatformError>;
    async fn achievements(&self, id: &GameId) -> Result<AchievementSet, PlatformError>;
    async fn global_percentages(
        &self,
        id: &GameId,
    ) -> Result<Vec<GlobalAchievement>, PlatformError>;

    // Fetches a game's achievements enriched with global unlock percentages.
    //
    // <purpose-start>
    // Centralizes the apiname join that plugins use so they get enriched achievements.
    // A failure to fetch global percentages is not fatal: achievements are still returned, just
    // without enrichment (`global_percent: None`).
    // <purpose-end>
    //
    // <inputs-start>
    // - `id`: Reference to the game identifier.
    // <inputs-end>
    //
    // <outputs-start>
    // - `Result<AchievementSet, PlatformError>`: Enriched achievement set or domain error.
    // <outputs-end>
    //
    // <side-effects-start>
    // - Makes asynchronous library requests to fetch achievements and global percentages.
    // <side-effects-end>
    async fn achievements_with_global(&self, id: &GameId) -> Result<AchievementSet, PlatformError> {
        let mut set = self.achievements(id).await?;

        if let Ok(globals) = self.global_percentages(id).await {
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
    // Centralizes name and game ID resolution across plugins.
    // <purpose-end>
    //
    // <inputs-start>
    // - `query`: User-supplied string query.
    // <inputs-end>
    //
    // <outputs-start>
    // - `Result<GameMatch, PlatformError>`: Matching game result or domain error.
    // <outputs-end>
    //
    // <side-effects-start>
    // - Asynchronously retrieves owned games from the library.
    // <side-effects-end>
    async fn resolve(&self, query: &str) -> Result<GameMatch, PlatformError> {
        let games = self.owned_games().await?;

        if let Ok(id) = GameId::from_str(query)
            && let Some(game) = games.iter().find(|g| g.id == id)
        {
            return Ok(GameMatch::One(game.clone()));
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

    // A canned `GameLibrary` for plugin tests: returns pre-set data or errors
    // without opening sockets or hand-writing wire JSON.
    #[derive(Default)]
    pub struct FakeLibrary {
        pub platform: Option<Platform>,
        pub games: Option<Result<Vec<Game>, PlatformError>>,
        pub achievements: HashMap<GameId, Result<AchievementSet, PlatformError>>,
        pub global: HashMap<GameId, Result<Vec<GlobalAchievement>, PlatformError>>,
    }

    impl FakeLibrary {
        pub fn new() -> Self {
            Self::default()
        }

        pub fn with_platform(mut self, platform: Platform) -> Self {
            self.platform = Some(platform);
            self
        }

        pub fn with_games(mut self, games: Vec<Game>) -> Self {
            self.games = Some(Ok(games));
            self
        }

        pub fn with_games_error(mut self, err: PlatformError) -> Self {
            self.games = Some(Err(err));
            self
        }

        pub fn with_achievements(mut self, id: impl Into<GameId>, set: AchievementSet) -> Self {
            self.achievements.insert(id.into(), Ok(set));
            self
        }

        pub fn with_achievements_error(
            mut self,
            id: impl Into<GameId>,
            err: PlatformError,
        ) -> Self {
            self.achievements.insert(id.into(), Err(err));
            self
        }

        pub fn with_global(
            mut self,
            id: impl Into<GameId>,
            achievements: Vec<GlobalAchievement>,
        ) -> Self {
            self.global.insert(id.into(), Ok(achievements));
            self
        }

        pub fn with_global_error(mut self, id: impl Into<GameId>, err: PlatformError) -> Self {
            self.global.insert(id.into(), Err(err));
            self
        }
    }

    #[async_trait]
    impl GameLibrary for FakeLibrary {
        fn platform(&self) -> Platform {
            self.platform.unwrap_or(Platform::Steam)
        }

        async fn owned_games(&self) -> Result<Vec<Game>, PlatformError> {
            self.games.clone().unwrap_or(Ok(Vec::new()))
        }

        async fn achievements(&self, id: &GameId) -> Result<AchievementSet, PlatformError> {
            self.achievements
                .get(id)
                .cloned()
                .unwrap_or_else(|| Err(PlatformError::NoStats { id: id.clone() }))
        }

        async fn global_percentages(
            &self,
            id: &GameId,
        ) -> Result<Vec<GlobalAchievement>, PlatformError> {
            self.global.get(id).cloned().unwrap_or(Ok(Vec::new()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fake::FakeLibrary;
    use super::*;

    fn game(appid: u32, name: &str) -> Game {
        Game {
            id: GameId::Steam(appid),
            platform: Platform::Steam,
            name: name.to_string(),
            playtime_forever: Some(0),
            img_icon_url: Some(String::new()),
            rtime_last_played: 0,
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
            grade: None,
        }
    }

    #[tokio::test]
    async fn resolve_matches_owned_appid_before_name() {
        let client = FakeLibrary::new().with_games(vec![game(123, "Game 123")]);

        let result = client.resolve("123").await.unwrap();

        assert_eq!(result, GameMatch::One(game(123, "Game 123")));
    }

    #[tokio::test]
    async fn resolve_falls_back_to_name_when_appid_not_owned() {
        let client = FakeLibrary::new().with_games(vec![game(456, "Game 123")]);

        let result = client.resolve("123").await.unwrap();

        assert_eq!(result, GameMatch::One(game(456, "Game 123")));
    }

    #[tokio::test]
    async fn resolve_returns_many_on_ambiguous_name() {
        let client = FakeLibrary::new().with_games(vec![game(1, "Foo One"), game(2, "Foo Two")]);

        let result = client.resolve("foo").await.unwrap();

        match result {
            GameMatch::Many(games) => assert_eq!(games.len(), 2),
            other => panic!("expected Many, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn resolve_returns_none_when_nothing_matches() {
        let client = FakeLibrary::new().with_games(vec![game(1, "Foo")]);

        let result = client.resolve("bar").await.unwrap();

        assert_eq!(result, GameMatch::None);
    }

    #[tokio::test]
    async fn achievements_with_global_joins_by_apiname() {
        let client = FakeLibrary::new()
            .with_achievements(
                123,
                AchievementSet {
                    game_name: "Test Game".to_string(),
                    achievements: vec![achievement("ach1", 1), achievement("ach2", 0)],
                },
            )
            .with_global(
                123,
                vec![GlobalAchievement {
                    name: "ach1".to_string(),
                    percent: 50.5,
                }],
            );

        let set = client
            .achievements_with_global(&GameId::Steam(123))
            .await
            .unwrap();

        assert_eq!(set.achievements[0].global_percent, Some(50.5));
        assert_eq!(set.achievements[1].global_percent, None);
    }

    #[tokio::test]
    async fn achievements_with_global_keeps_achievements_when_global_fetch_fails() {
        let client = FakeLibrary::new()
            .with_achievements(
                123,
                AchievementSet {
                    game_name: "Test Game".to_string(),
                    achievements: vec![achievement("ach1", 1)],
                },
            )
            .with_global_error(
                123,
                PlatformError::Http {
                    status: Some(500),
                    msg: "boom".to_string(),
                },
            );

        let set = client
            .achievements_with_global(&GameId::Steam(123))
            .await
            .unwrap();

        assert_eq!(set.achievements[0].global_percent, None);
    }

    #[test]
    fn test_game_id_parsing() {
        assert_eq!("steam:440".parse::<GameId>().unwrap(), GameId::Steam(440));
        assert_eq!(
            "psn:NPWR12345_00".parse::<GameId>().unwrap(),
            GameId::Psn("NPWR12345_00".to_string())
        );
        assert_eq!("440".parse::<GameId>().unwrap(), GameId::Steam(440));
        assert!("invalid".parse::<GameId>().is_err());
    }

    #[test]
    fn test_platform_parsing() {
        assert_eq!("steam".parse::<Platform>().unwrap(), Platform::Steam);
        assert_eq!("PSN".parse::<Platform>().unwrap(), Platform::Psn);
        assert!("xbox".parse::<Platform>().is_err());
    }
}
