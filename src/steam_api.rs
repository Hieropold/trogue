//! Production `SteamClient` adapter: talks HTTP/JSON to the real Steam API.
//!
//! <purpose-start>
//! This is the single home of wire-format knowledge (endpoint URLs, JSON
//! envelope shapes, status-code interpretation, credential loading). Plugin
//! code and plugin tests never see any of it — they only see `SteamClient`
//! and its domain types.
//! <purpose-end>

use crate::steam_client::{
    Achievement, AchievementSet, Game, GlobalAchievement, SteamClient, SteamError,
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
            appid: g.appid,
            name: g.name,
            playtime_forever: g.playtime_forever,
            img_icon_url: g.img_icon_url,
            playtime_windows_forever: g.playtime_windows_forever,
            playtime_mac_forever: g.playtime_mac_forever,
            playtime_linux_forever: g.playtime_linux_forever,
            rtime_last_played: g.rtime_last_played,
            playtime_disconnected: g.playtime_disconnected,
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
    percent: f32,
}

impl From<WireGlobalAchievement> for GlobalAchievement {
    fn from(g: WireGlobalAchievement) -> Self {
        GlobalAchievement {
            name: g.name,
            percent: g.percent,
        }
    }
}

// Production `SteamClient` adapter.
pub struct HttpSteamClient {
    api_key: String,
    steam_id: String,
    base_url: String,
    client: reqwest::Client,
}

impl HttpSteamClient {
    // Builds a client from `TROGUE_STEAM_API_KEY` / `TROGUE_STEAM_ID`.
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
    // - `Err(SteamError::Config)` naming the missing variable.
    // <outputs-end>
    //
    // <side-effects-start>
    // - Reads environment variables.
    // <side-effects-end>
    pub fn from_env() -> Result<Self, SteamError> {
        let api_key = env::var("TROGUE_STEAM_API_KEY").map_err(|_| {
            SteamError::Config("Missing TROGUE_STEAM_API_KEY environment variable.".to_string())
        })?;
        let steam_id = env::var("TROGUE_STEAM_ID").map_err(|_| {
            SteamError::Config("Missing TROGUE_STEAM_ID environment variable.".to_string())
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
    fn map_status_error(status: reqwest::StatusCode, appid: Option<u32>) -> SteamError {
        match (status.as_u16(), appid) {
            (403, _) => SteamError::PrivateProfile,
            (400, Some(appid)) => SteamError::NoStats { appid },
            (code, _) => SteamError::Http {
                status: Some(code),
                msg: status
                    .canonical_reason()
                    .unwrap_or("unknown error")
                    .to_string(),
            },
        }
    }

    fn map_transport_error(e: reqwest::Error) -> SteamError {
        SteamError::Http {
            status: e.status().map(|s| s.as_u16()),
            msg: e.to_string(),
        }
    }
}

#[async_trait]
impl SteamClient for HttpSteamClient {
    async fn owned_games(&self) -> Result<Vec<Game>, SteamError> {
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
            .map_err(|e| SteamError::Decode(e.to_string()))?;

        Ok(data.response.games.into_iter().map(Game::from).collect())
    }

    async fn achievements(&self, appid: u32) -> Result<AchievementSet, SteamError> {
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
            .map_err(|e| SteamError::Decode(e.to_string()))?;

        if !data.playerstats.success {
            return Err(SteamError::PrivateProfile);
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

    async fn global_percentages(&self, appid: u32) -> Result<Vec<GlobalAchievement>, SteamError> {
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
            .map_err(|e| SteamError::Decode(e.to_string()))?;

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

        let _m = server.mock("GET", "/IPlayerService/GetOwnedGames/v0001/?key=test_key&steamid=test_id&format=json&include_appinfo=1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "response": {
                    "game_count": 1,
                    "games": [
                        {
                            "appid": 1,
                            "name": "Test Game",
                            "playtime_forever": 100,
                            "img_icon_url": "",
                            "playtime_windows_forever": 100,
                            "playtime_mac_forever": 0,
                            "playtime_linux_forever": 0,
                            "rtime_last_played": 0,
                            "playtime_disconnected": 0
                        }
                    ]
                }
            }"#)
            .create_async().await;

        let client =
            HttpSteamClient::with_base_url("test_key".to_string(), "test_id".to_string(), url);
        let games = client.owned_games().await.unwrap();

        assert_eq!(games.len(), 1);
        assert_eq!(games[0].name, "Test Game");
    }

    #[tokio::test]
    async fn test_owned_games_server_error() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let _m = server.mock("GET", "/IPlayerService/GetOwnedGames/v0001/?key=test_key&steamid=test_id&format=json&include_appinfo=1")
            .with_status(500)
            .create_async().await;

        let client =
            HttpSteamClient::with_base_url("test_key".to_string(), "test_id".to_string(), url);
        let result = client.owned_games().await;

        assert!(matches!(result, Err(SteamError::Http { status: Some(500), .. })));
    }

    #[tokio::test]
    async fn test_achievements_success() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let _m = server.mock("GET", "/ISteamUserStats/GetPlayerAchievements/v0001/?appid=1&key=test_key&steamid=test_id&l=en")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "playerstats": {
                    "steamID": "test_id",
                    "gameName": "Test Game",
                    "achievements": [
                        {
                            "apiname": "test_ach",
                            "achieved": 1,
                            "unlocktime": 0,
                            "name": "Test Achievement",
                            "description": "A test achievement"
                        }
                    ],
                    "success": true
                }
            }"#)
            .create_async().await;

        let client =
            HttpSteamClient::with_base_url("test_key".to_string(), "test_id".to_string(), url);
        let set = client.achievements(1).await.unwrap();

        assert_eq!(set.game_name, "Test Game");
        assert_eq!(set.achievements.len(), 1);
        assert_eq!(set.achievements[0].name, "Test Achievement");
    }

    #[tokio::test]
    async fn test_achievements_private_profile() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let _m = server.mock("GET", "/ISteamUserStats/GetPlayerAchievements/v0001/?appid=1&key=test_key&steamid=test_id&l=en")
            .with_status(403)
            .create_async().await;

        let client =
            HttpSteamClient::with_base_url("test_key".to_string(), "test_id".to_string(), url);
        let result = client.achievements(1).await;

        assert!(matches!(result, Err(SteamError::PrivateProfile)));
    }

    #[tokio::test]
    async fn test_achievements_no_stats() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let _m = server.mock("GET", "/ISteamUserStats/GetPlayerAchievements/v0001/?appid=1&key=test_key&steamid=test_id&l=en")
            .with_status(400)
            .create_async().await;

        let client =
            HttpSteamClient::with_base_url("test_key".to_string(), "test_id".to_string(), url);
        let result = client.achievements(1).await;

        assert!(matches!(result, Err(SteamError::NoStats { appid: 1 })));
    }

    #[tokio::test]
    async fn test_global_percentages_success() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let _m = server.mock("GET", "/ISteamUserStats/GetGlobalAchievementPercentagesForApp/v0002/?gameid=1&format=json&l=en")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{
                "achievementpercentages": {
                    "achievements": [
                        {
                            "name": "test_ach",
                            "percent": 50.5
                        }
                    ]
                }
            }"#)
            .create_async().await;

        let client =
            HttpSteamClient::with_base_url("test_key".to_string(), "test_id".to_string(), url);
        let achievements = client.global_percentages(1).await.unwrap();

        assert_eq!(achievements.len(), 1);
        assert_eq!(achievements[0].name, "test_ach");
        assert_eq!(achievements[0].percent, 50.5);
    }

    #[tokio::test]
    async fn test_global_percentages_server_error() {
        let mut server = mockito::Server::new_async().await;
        let url = server.url();

        let _m = server.mock("GET", "/ISteamUserStats/GetGlobalAchievementPercentagesForApp/v0002/?gameid=1&format=json&l=en")
            .with_status(500)
            .create_async().await;

        let client =
            HttpSteamClient::with_base_url("test_key".to_string(), "test_id".to_string(), url);
        let result = client.global_percentages(1).await;

        assert!(result.is_err());
    }
}
