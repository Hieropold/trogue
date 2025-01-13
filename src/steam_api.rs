use reqwest;
use serde::{Deserialize, Serialize};
use tokio;

#[derive(Serialize, Deserialize, Debug)]
struct GamesListResponse {
    response: GamesList,
}

#[derive(Serialize, Deserialize, Debug)]
struct GamesList {
    game_count: u32,
    games: Vec<Game>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Game {
    pub appid: u32,
    pub name: String,
    pub playtime_forever: u32,
    pub img_icon_url: String,
    // pub has_community_visible_stats: bool,
    pub playtime_windows_forever: u32,
    pub playtime_mac_forever: u32,
    pub playtime_linux_forever: u32,
    pub rtime_last_played: u64,
    pub playtime_disconnected: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerStatsResponse {
    playerstats: PlayerStats,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerStats {
    #[serde(rename = "steamID")]
    pub steam_id: String,
    #[serde(rename = "gameName")]
    pub game_name: String,
    pub achievements: Vec<Achievement>,
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Achievement {
    pub apiname: String,
    pub achieved: u8,
    pub unlocktime: u64,
    pub name: String,
    pub description: String,
}

pub struct Api {
    api_key: String,
    steam_id: String,
}

impl Api {
    pub fn new(api_key: String, steam_id: String) -> Api {
        Api { api_key, steam_id }
    }

    #[tokio::main]
    pub async fn get_games_list(&self) -> Result<Vec<Game>, reqwest::Error> {
        let api_key = self.api_key.clone();
        let steam_id = self.steam_id.clone();
        
        // List of owned games
        let url = format!("http://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={api_key}&steamid={steam_id}&format=json&include_appinfo=1");

        // Send the GET request
        let response = reqwest::get(url).await?;

        // Check if the request was successful and parse the JSON
        if response.status().is_success() {
            let data: GamesListResponse = response.json().await?;
            return Ok(data.response.games);
        } else {
            eprintln!("Failed to fetch data: {}", response.status());
        }

        Ok(Vec::new())
    }

    #[tokio::main]
    pub async fn get_game_achievements(&self, appid: u32) -> Result<Vec<Achievement>, reqwest::Error> {
        let api_key = self.api_key.clone();
        let steam_id = self.steam_id.clone();

        // Game achievements
        let url = format!("http://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v0001/?appid={appid}&key={api_key}&steamid={steam_id}&l=en");

        // Send the GET request
        let response = reqwest::get(url).await?;

        // Check if the request was successful and parse the JSON
        if response.status().is_success() {
            let data: PlayerStatsResponse = response.json().await?;
            return Ok(data.playerstats.achievements);
        } else {
            eprintln!("Failed to fetch data: {}", response.status());
        }

        Ok(Vec::new())
    }
}