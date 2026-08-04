//! Aggregates multiple platform library sources into a single unified `GameLibrary`.
//!
//! <purpose-start>
//! Combines owned games across configured sources (Steam, PSN) and routes per-game queries
//! (`achievements`, `global_percentages`) to the appropriate platform adapter based on `GameId`.
//! <purpose-end>

use crate::game_library::{
    AchievementSet, Game, GameId, GameLibrary, GlobalAchievement, PlatformError,
};
use async_trait::async_trait;

// Aggregates multiple `GameLibrary` sources.
pub struct MultiSource {
    sources: Vec<Box<dyn GameLibrary>>,
}

impl std::fmt::Debug for MultiSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiSource")
            .field("sources_count", &self.sources.len())
            .finish()
    }
}

impl MultiSource {
    // Creates a new `MultiSource` instance containing the provided sources.
    //
    // <purpose-start>
    // Wraps a list of platform adapters into a single composite library.
    // <purpose-end>
    //
    // <inputs-start>
    // - `sources`: Vector of boxed `GameLibrary` trait objects.
    // <inputs-end>
    //
    // <outputs-start>
    // - `MultiSource`: The aggregated library instance.
    // <outputs-end>
    //
    // <side-effects-start>
    // - None.
    // <side-effects-end>
    pub fn new(sources: Vec<Box<dyn GameLibrary>>) -> Self {
        Self { sources }
    }
}

#[async_trait]
impl GameLibrary for MultiSource {
    // Concatenates owned games across all configured sources.
    //
    // <purpose-start>
    // Retrieves games from each platform adapter and returns the merged library list.
    // <purpose-end>
    //
    // <inputs-start>
    // - `&self`: Reference to `MultiSource`.
    // <inputs-end>
    //
    // <outputs-start>
    // - `Result<Vec<Game>, PlatformError>`: Combined list of games or platform error if all fail.
    // <outputs-end>
    //
    // <side-effects-start>
    // - Issues requests to all child sources.
    // <side-effects-end>
    async fn owned_games(&self) -> Result<Vec<Game>, PlatformError> {
        let mut all_games = Vec::new();
        let mut last_err = None;
        let mut any_success = false;

        for source in &self.sources {
            match source.owned_games().await {
                Ok(games) => {
                    any_success = true;
                    all_games.extend(games);
                }
                Err(e) => {
                    last_err = Some(e);
                }
            }
        }

        if any_success || self.sources.is_empty() {
            Ok(all_games)
        } else {
            Err(last_err.unwrap_or_else(|| {
                PlatformError::Config("No game library sources configured".to_string())
            }))
        }
    }

    // Routes achievement queries to the source matching `id.platform()`.
    //
    // <purpose-start>
    // Delegates achievement retrieval to the platform source that owns `id`.
    // <purpose-end>
    //
    // <inputs-start>
    // - `id`: The game identifier.
    // <inputs-end>
    //
    // <outputs-start>
    // - `Result<AchievementSet, PlatformError>`: Achievement set or error.
    // <outputs-end>
    //
    // <side-effects-start>
    // - Issues a request to the matching child source.
    // <side-effects-end>
    async fn achievements(&self, id: &GameId) -> Result<AchievementSet, PlatformError> {
        let target_platform = id.platform();
        for source in &self.sources {
            if source.platform() == target_platform {
                return source.achievements(id).await;
            }
        }
        Err(PlatformError::NoStats { id: id.clone() })
    }

    // Routes global percentages queries to the source matching `id.platform()`.
    //
    // <purpose-start>
    // Delegates global percentage retrieval to the platform source that owns `id`.
    // <purpose-end>
    //
    // <inputs-start>
    // - `id`: The game identifier.
    // <inputs-end>
    //
    // <outputs-start>
    // - `Result<Vec<GlobalAchievement>, PlatformError>`: Global percentages list or error.
    // <outputs-end>
    //
    // <side-effects-start>
    // - Issues a request to the matching child source.
    // <side-effects-end>
    async fn global_percentages(
        &self,
        id: &GameId,
    ) -> Result<Vec<GlobalAchievement>, PlatformError> {
        let target_platform = id.platform();
        for source in &self.sources {
            if source.platform() == target_platform {
                return source.global_percentages(id).await;
            }
        }
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_library::Platform;
    use crate::game_library::fake::FakeLibrary;

    #[tokio::test]
    async fn multi_source_concatenates_owned_games() {
        let s1 = FakeLibrary::new()
            .with_platform(Platform::Steam)
            .with_games(vec![Game {
                id: GameId::Steam(1),
                platform: Platform::Steam,
                name: "Steam Game".to_string(),
                playtime_forever: Some(10),
                img_icon_url: None,
                rtime_last_played: 0,
            }]);

        let multi = MultiSource::new(vec![Box::new(s1)]);
        let games = multi.owned_games().await.unwrap();
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].name, "Steam Game");
    }

    #[tokio::test]
    async fn multi_source_routes_achievements_by_platform() {
        let s1 = FakeLibrary::new()
            .with_platform(Platform::Steam)
            .with_achievements(
                GameId::Steam(1),
                AchievementSet {
                    game_name: "Steam Game".to_string(),
                    achievements: vec![],
                },
            );

        let multi = MultiSource::new(vec![Box::new(s1)]);
        let set = multi.achievements(&GameId::Steam(1)).await.unwrap();
        assert_eq!(set.game_name, "Steam Game");

        let err = multi.achievements(&GameId::Psn("123".to_string())).await;
        assert!(matches!(err, Err(PlatformError::NoStats { .. })));
    }
}
