//! Production `GameLibrary` adapter: talks HTTP/JSON to the real Steam API.
//!
//! <purpose-start>
//! This is the single home of wire-format knowledge for Steam (endpoint URLs, JSON
//! envelope shapes, status-code interpretation, credential loading). Plugin
//! code and plugin tests never see any of it — they only see `GameLibrary`
//! and its domain types.
//! <purpose-end>

use crate::game_library::{
    Achievement, AchievementSet, Game, GameId, GameLibrary, GlobalAchievement, Platform,
    PlatformError,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::env;

const STEAM_API_BASE_URL: &str = "http://api.steampowered.com";

// Wire envelope for GetOwnedGames. Private: no caller outside this file
// should ever see Steam's on-the-wire shape.
#[derive(Serialize, Deserialize, Debug)]
struct GamesListResponse {
    response: GamesList,
}

#[derive(Serialize, Deserialize, Debug)]
struct GamesList {
    #[serde(default)]
    games: Vec<WireGame>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WireGame {
    appid: u32,
    name: String,
    playtime_forever: u32,
    #[serde(default)]
    img_icon_url: String,
    #[serde(default)]
    playtime_windows_forever: u32,
    #[serde(default)]
    playtime_mac_forever: u32,
    #[serde(default)]
    playtime_linux_forever: u32,
    #[serde(default)]
    rtime_last_played: u64,
    #[serde(default)]
    playtime_disconnected: u32,
}

impl From<WireGame> for Game {
    fn from(g: WireGame) -> Self {
        Game {
            id: GameId::Steam(g.appid),
            platform: Platform::Steam,
            name: g.name,
            playtime_forever: Some(g.playtime_forever),
            img_icon_url: if g.img_icon_url.is_empty() {
                None
            } else {
                Some(g.img_icon_url)
            },
            rtime_last_played: g.rtime_last_played,
        }
    }
}

// Wire envelope for GetPlayerAchievements.
#[derive(Serialize, Deserialize, Debug)]
struct PlayerStatsResponse {
    playerstats: PlayerStats,
}

#[derive(Serialize, Deserialize, Debug)]
struct PlayerStats {
    #[serde(rename = "gameName", default)]
    game_name: String,
    #[serde(default)]
    achievements: Vec<WireAchievement>,
    success: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WireAchievement {
    apiname: String,
    achieved: u8,
    unlocktime: u64,
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
}

impl From<WireAchievement> for Achievement {
    fn from(a: WireAchievement) -> Self {
        Achievement {
            apiname: a.apiname,
            achieved: a.achieved,
            unlocktime: a.unlocktime,
            name: a.name,
            description: a.description,
            global_percent: None,
            grade: None,
        }
    }
}

// Wire envelope for GetGlobalAchievementPercentagesForApp.
#[derive(Serialize, Deserialize, Debug)]
struct GlobalAchievementsResponse {
    achievementpercentages: GlobalAchievements,
}

#[derive(Serialize, Deserialize, Debug)]
struct GlobalAchievements {
    #[serde(default)]
    achievements: Vec<WireGlobalAchievement>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct WireGlobalAchievement {
    name: String,
    #[serde(deserialize_with = "deserialize_percent")]
    percent: f32,
}

// Deserializes the `percent` field from either a JSON string or number.
//
// <purpose-start>
// The Steam API `GetGlobalAchievementPercentagesForApp/v0002` returns
// `percent` as a JSON string (e.g., `"0.4"`) rather than a bare number.
// serde's default `f32` deserialization rejects strings, causing a silent
// `Decode` error that leaves every achievement's `global_percent` as `None`.
// This handles both string and numeric representations for robustness.
// <purpose-end>
//
// <inputs-start>
// - `deserializer`: A serde `Deserializer` positioned at the `percent` field.
// <inputs-end>
//
// <outputs-start>
// - `Result<f32, D::Error>`: The parsed floating point percentage value.
// <outputs-end>
//
// <side-effects-start>
// - None.
// <side-effects-end>
fn deserialize_percent<'de, D>(deserializer: D) -> Result<f32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;

    struct PercentVisitor;

    impl<'de> de::Visitor<'de> for PercentVisitor {
        type Value = f32;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a number or numeric string")
        }

        fn visit_f64<E: de::Error>(self, v: f64) -> Result<f32, E> {
            Ok(v as f32)
        }

        fn visit_u64<E: de::Error>(self, v: u64) -> Result<f32, E> {
            Ok(v as f32)
        }

        fn visit_i64<E: de::Error>(self, v: i64) -> Result<f32, E> {
            Ok(v as f32)
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<f32, E> {
            v.parse::<f32>().map_err(de::Error::custom)
        }
    }

    deserializer.deserialize_any(PercentVisitor)
}

impl From<WireGlobalAchievement> for GlobalAchievement {
    fn from(g: WireGlobalAchievement) -> Self {
        GlobalAchievement {
            name: g.name,
            percent: g.percent,
        }
    }
}

// Production `GameLibrary` adapter for Steam.
pub struct HttpSteamClient {
    api_key: String,
    steam_id: String,
    base_url: String,
    client: reqwest::Client,
}

impl HttpSteamClient {
    // Loads credentials from environment variables and constructs a client.
    //
    // <purpose-start>
    // Folds in what used to be `Cfg`'s two-phase env load: a `HttpSteamClient`
    // that exists is guaranteed to hold valid credentials, so callers never
    // need a separate "did config load?" check.
    // <purpose-end>
    //
    // <inputs-start>
    // - None: reads process environment variables directly.
    // <inputs-end>
    //
    // <outputs-start>
    // - `Ok(Self)` when both environment variables are present.
    // - `Err(PlatformError::Config)` naming the missing variable.
    // <outputs-end>
    //
    // <side-effects-start>
    // - Reads environment variables.
    // <side-effects-end>
    pub fn from_env() -> Result<Self, PlatformError> {
        let api_key = env::var("TROGUE_STEAM_API_KEY").map_err(|_| {
            PlatformError::Config("Missing TROGUE_STEAM_API_KEY environment variable.".to_string())
        })?;
        let steam_id = env::var("TROGUE_STEAM_ID").map_err(|_| {
            PlatformError::Config("Missing TROGUE_STEAM_ID environment variable.".to_string())
        })?;

        Ok(Self {
            api_key,
            steam_id,
            base_url: STEAM_API_BASE_URL.to_string(),
            client: reqwest::Client::new(),
        })
    }

    // Test-only constructor letting tests point this adapter at a mockito server.
    #[cfg(test)]
    fn with_base_url(api_key: String, steam_id: String, base_url: String) -> Self {
        Self {
            api_key,
            steam_id,
            base_url,
            client: reqwest::Client::new(),
        }
    }

    // Maps a non-success HTTP response into the right domain error.
    //
    // <purpose-start>
    // Steam's status codes carry meaning specific to this API (400 means "no
    // stats for this app", 403 means "profile is private") that a generic
    // `reqwest::Error` would otherwise flatten into unreadable prose.
    // <purpose-end>
    fn map_status_error(status: reqwest::StatusCode, appid: Option<u32>) -> PlatformError {
        match (status.as_u16(), appid) {
            (403, _) => PlatformError::PrivateProfile,
            (400, Some(appid)) => PlatformError::NoStats {
                id: GameId::Steam(appid),
            },
            (code, _) => PlatformError::Http {
                status: Some(code),
                msg: status
                    .canonical_reason()
                    .unwrap_or("unknown error")
                    .to_string(),
            },
        }
    }

    // Strips the request URL from the error before stringifying it, since the
    // URL's query string carries `key=<TROGUE_STEAM_API_KEY>` and this
    // message is printed verbatim by every plugin and by the interactive
    // TUI's error area.
    fn map_transport_error(e: reqwest::Error) -> PlatformError {
        let status = e.status().map(|s| s.as_u16());
        PlatformError::Http {
            status,
            msg: e.without_url().to_string(),
        }
    }
}

#[async_trait]
impl GameLibrary for HttpSteamClient {
    fn platform(&self) -> Platform {
        Platform::Steam
    }

    async fn owned_games(&self) -> Result<Vec<Game>, PlatformError> {
        let url = format!(
            "{}/IPlayerService/GetOwnedGames/v0001/?key={}&steamid={}&format=json&include_appinfo=1",
            self.base_url, self.api_key, self.steam_id
        );

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(Self::map_transport_error)?;

        let status = response.status();
        if !status.is_success() {
            return Err(Self::map_status_error(status, None));
        }

        let data: GamesListResponse = response
            .json()
            .await
            .map_err(|e| PlatformError::Decode(e.to_string()))?;

        Ok(data.response.games.into_iter().map(Game::from).collect())
    }

    async fn achievements(&self, id: &GameId) -> Result<AchievementSet, PlatformError> {
        let appid = match id {
            GameId::Steam(appid) => *appid,
            GameId::Psn(_) => return Err(PlatformError::NoStats { id: id.clone() }),
        };

        let url = format!(
            "{}/ISteamUserStats/GetPlayerAchievements/v0001/?appid={appid}&key={}&steamid={}&l=en",
            self.base_url, self.api_key, self.steam_id
        );

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(Self::map_transport_error)?;

        let status = response.status();
        if !status.is_success() {
            return Err(Self::map_status_error(status, Some(appid)));
        }

        let data: PlayerStatsResponse = response
            .json()
            .await
            .map_err(|e| PlatformError::Decode(e.to_string()))?;

        if !data.playerstats.success {
            return Err(PlatformError::PrivateProfile);
        }

        Ok(AchievementSet {
            game_name: data.playerstats.game_name,
            achievements: data
                .playerstats
                .achievements
                .into_iter()
                .map(Achievement::from)
                .collect(),
        })
    }

    async fn global_percentages(
        &self,
        id: &GameId,
    ) -> Result<Vec<GlobalAchievement>, PlatformError> {
        let appid = match id {
            GameId::Steam(appid) => *appid,
            GameId::Psn(_) => return Ok(Vec::new()),
        };

        let url = format!(
            "{}/ISteamUserStats/GetGlobalAchievementPercentagesForApp/v0002/?gameid={appid}&format=json&l=en",
            self.base_url
        );

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(Self::map_transport_error)?;

        let status = response.status();
        if !status.is_success() {
            return Err(Self::map_status_error(status, Some(appid)));
        }

        let data: GlobalAchievementsResponse = response
            .json()
            .await
            .map_err(|e| PlatformError::Decode(e.to_string()))?;

        Ok(data
            .achievementpercentages
            .achievements
            .into_iter()
            .map(GlobalAchievement::from)
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_owned_games_success() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let json_body = serde_json::json!({
            "response": {
                "game_count": 1,
                "games": [
                    {
                        "appid": 440,
                        "name": "Team Fortress 2",
                        "playtime_forever": 100,
                        "img_icon_url": "icon_url",
                        "playtime_windows_forever": 100,
                        "playtime_mac_forever": 0,
                        "playtime_linux_forever": 0,
                        "rtime_last_played": 1600000000u64,
                        "playtime_disconnected": 0
                    }
                ]
            }
        });

        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json_body.to_string())
            .create_async()
            .await;

        let client =
            HttpSteamClient::with_base_url("test_key".to_string(), "test_id".to_string(), url);
        let games = client.owned_games().await.unwrap();

        mock.assert_async().await;
        assert_eq!(games.len(), 1);
        assert_eq!(games[0].id, GameId::Steam(440));
        assert_eq!(games[0].platform, Platform::Steam);
        assert_eq!(games[0].name, "Team Fortress 2");
        assert_eq!(games[0].playtime_forever, Some(100));
        assert_eq!(games[0].img_icon_url, Some("icon_url".to_string()));
        assert_eq!(games[0].rtime_last_played, 1600000000);
    }

    #[tokio::test]
    async fn test_owned_games_server_error() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(500)
            .create_async()
            .await;

        let client =
            HttpSteamClient::with_base_url("test_key".to_string(), "test_id".to_string(), url);
        let result = client.owned_games().await;

        mock.assert_async().await;
        assert!(result.is_err());
        assert!(matches!(
            result,
            Err(PlatformError::Http {
                status: Some(500),
                ..
            })
        ));
    }

    #[tokio::test]
    async fn test_achievements_success() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let json_body = serde_json::json!({
            "playerstats": {
                "steamID": "76561198000000000",
                "gameName": "Team Fortress 2",
                "achievements": [
                    {
                        "apiname": "TF_GET_HEADS",
                        "achieved": 1,
                        "unlocktime": 1600000000u64,
                        "name": "Headhunter",
                        "description": "Decapitate 50 enemies."
                    }
                ],
                "success": true
            }
        });

        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json_body.to_string())
            .create_async()
            .await;

        let client =
            HttpSteamClient::with_base_url("test_key".to_string(), "test_id".to_string(), url);
        let set = client.achievements(&GameId::Steam(440)).await.unwrap();

        mock.assert_async().await;
        assert_eq!(set.game_name, "Team Fortress 2");
        assert_eq!(set.achievements.len(), 1);
        assert_eq!(set.achievements[0].apiname, "TF_GET_HEADS");
        assert_eq!(set.achievements[0].achieved, 1);
        assert_eq!(set.achievements[0].name, "Headhunter");
        assert_eq!(set.achievements[0].description, "Decapitate 50 enemies.");
    }

    #[tokio::test]
    async fn test_achievements_private_profile() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(403)
            .create_async()
            .await;

        let client =
            HttpSteamClient::with_base_url("test_key".to_string(), "test_id".to_string(), url);
        let result = client.achievements(&GameId::Steam(440)).await;

        mock.assert_async().await;
        assert!(matches!(result, Err(PlatformError::PrivateProfile)));
    }

    #[tokio::test]
    async fn test_achievements_no_stats() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(400)
            .create_async()
            .await;

        let client =
            HttpSteamClient::with_base_url("test_key".to_string(), "test_id".to_string(), url);
        let result = client.achievements(&GameId::Steam(1)).await;

        mock.assert_async().await;
        assert!(matches!(
            result,
            Err(PlatformError::NoStats { id }) if id == GameId::Steam(1)
        ));
    }

    #[tokio::test]
    async fn test_global_percentages_success() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let json_body = serde_json::json!({
            "achievementpercentages": {
                "achievements": [
                    {
                        "name": "TF_GET_HEADS",
                        "percent": 12.5
                    }
                ]
            }
        });

        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json_body.to_string())
            .create_async()
            .await;

        let client =
            HttpSteamClient::with_base_url("test_key".to_string(), "test_id".to_string(), url);
        let globals = client
            .global_percentages(&GameId::Steam(440))
            .await
            .unwrap();

        mock.assert_async().await;
        assert_eq!(globals.len(), 1);
        assert_eq!(globals[0].name, "TF_GET_HEADS");
        assert_eq!(globals[0].percent, 12.5);
    }

    #[tokio::test]
    async fn test_global_percentages_numeric_format() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let json_body = serde_json::json!({
            "achievementpercentages": {
                "achievements": [
                    {
                        "name": "TF_GET_HEADS",
                        "percent": "12.5"
                    }
                ]
            }
        });

        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(json_body.to_string())
            .create_async()
            .await;

        let client =
            HttpSteamClient::with_base_url("test_key".to_string(), "test_id".to_string(), url);
        let globals = client
            .global_percentages(&GameId::Steam(440))
            .await
            .unwrap();

        mock.assert_async().await;
        assert_eq!(globals.len(), 1);
        assert_eq!(globals[0].percent, 12.5);
    }

    #[tokio::test]
    async fn test_global_percentages_server_error() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let mock = server
            .mock("GET", mockito::Matcher::Any)
            .with_status(500)
            .create_async()
            .await;

        let client =
            HttpSteamClient::with_base_url("test_key".to_string(), "test_id".to_string(), url);
        let result = client.global_percentages(&GameId::Steam(440)).await;

        mock.assert_async().await;
        assert!(matches!(
            result,
            Err(PlatformError::Http {
                status: Some(500),
                ..
            })
        ));
    }

    #[tokio::test]
    async fn test_transport_error_never_leaks_api_key() {
        let secret_key = "super_secret_steam_api_key_12345";
        let client = HttpSteamClient::with_base_url(
            secret_key.to_string(),
            "test_id".to_string(),
            "http://127.0.0.1:1".to_string(),
        );

        let err = client.owned_games().await.unwrap_err();
        let err_msg = err.to_string();

        assert!(
            !err_msg.contains(secret_key),
            "Error message leaked API key: {}",
            err_msg
        );
    }
}
