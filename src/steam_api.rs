use reqwest;
use serde::{Deserialize, Serialize};
use tokio;
use chrono::{TimeZone, Utc};

#[derive(Serialize, Deserialize, Debug)]
struct GamesListResponse {
    response: GamesList,
}

#[derive(Serialize, Deserialize, Debug)]
struct GamesList {
    game_count: u32,
    games: Vec<Game>,
}

#[derive(Serialize, Deserialize, Debug)]
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
    pub steamID: String,
    pub gameName: String,
    pub achievements: Vec<Achievement>,
    pub success: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Achievement {
    pub apiname: String,
    pub achieved: u8,
    pub unlocktime: u64,
}

impl Achievement {
    pub fn new(apiname: String, achieved: u8, unlocktime: u64) -> Achievement {
        Achievement { apiname, achieved, unlocktime }
    }

    pub fn render_card(&self) -> String {
        let mut card = String::new();
        let achieved = if self.achieved == 1 { "Y" } else { "N" };
        let unlock_date = self.formatted_unlocktime();

        let apiname_length = self.apiname.len();
        let unlock_length = unlock_date.len();

        let longest_length = if apiname_length > unlock_length { apiname_length } else { unlock_length };

        // Generate top ┌──────┐
        card.push_str("┌");
        let horizontal_line_width = longest_length + 8;
        for _ in 0..horizontal_line_width {
            card.push_str("─");
        }
        card.push_str("┐\n");

        card.push_str(&format!("│ Name: {:>longest_length$} │\n", self.apiname));

        let achieved_width = longest_length - 4;
        card.push_str(&format!("│ Achieved: {:>achieved_width$} │\n", achieved, achieved_width = achieved_width));
        
        card.push_str(&format!("│ Date: {:>longest_length$} │\n", self.formatted_unlocktime()));

        // Lower └─────────┘
        card.push_str("└");
        for i in 0..horizontal_line_width {
            card.push_str("─");
        }
        card.push_str("┘\n");

        card
    }

    fn formatted_unlocktime(&self) -> String {
        let ts = self.unlocktime.try_into().unwrap();
        let datetime = Utc.timestamp_opt(ts, 0)
            .single()
            .expect("Invalid Unix timestamp");
        
        // Format the NaiveDateTime into a human-readable string
        datetime.format("%Y-%m-%d %H:%M:%S").to_string()
    }
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
        let url = format!("http://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v0001/?appid={appid}&key={api_key}&steamid={steam_id}");

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