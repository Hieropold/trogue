use reqwest;
use serde::{Deserialize, Serialize};
use tokio;

/// Represents the response from the GetGamesList API endpoint.
#[derive(Serialize, Deserialize, Debug)]
struct GamesListResponse {
    response: GamesList,
}

/// Represents the list of games in the GamesListResponse.
#[derive(Serialize, Deserialize, Debug)]
struct GamesList {
    game_count: u32,
    games: Vec<Game>,
}

/// Represents a game owned by the user.
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

/// Represents the response from the GetPlayerAchievements API endpoint.
#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerStatsResponse {
    playerstats: PlayerStats,
}

/// Represents the player stats in the PlayerStatsResponse.
#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerStats {
    #[serde(rename = "steamID")]
    pub steam_id: String,
    #[serde(rename = "gameName")]
    pub game_name: String,
    pub achievements: Vec<Achievement>,
    pub success: bool,
}

/// Represents an achievement for a game.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Achievement {
    pub apiname: String,
    pub achieved: u8,
    pub unlocktime: u64,
    pub name: String,
    pub description: String,
}

/// Represents the response from the GetGlobalAchievementPercentagesForApp API endpoint.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalAchievementsResponse {
    pub achievementpercentages: GlobalAchievements,
}

/// Represents the global achievements in the GlobalAchievementsResponse.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalAchievements {
    pub achievements: Vec<GlobalAchievement>,
}

/// Represents a global achievement.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GlobalAchievement {
    pub name: String,
    pub percent: f32,
}

/// A client for interacting with the Steam API.
pub struct Api {
    api_key: String,
    steam_id: String,
}

impl Api {
    /// Creates a new `Api` instance.
    ///
    /// <purpose-start>
    /// This function initializes a new `Api` instance with the provided API key and Steam ID.
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `api_key`: The Steam API key.
    /// - `steam_id`: The user's Steam ID.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - `Api`: A new `Api` instance.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - None.
    /// <side-effects-end>
    pub fn new(api_key: String, steam_id: String) -> Api {
        Api { api_key, steam_id }
    }

    /// Retrieves the list of games owned by the user.
    ///
    /// <purpose-start>
    /// This function sends a request to the Steam API to retrieve the list of games owned by the user.
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - None.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - `Ok(Vec<Game>)`: A vector of `Game` structs representing the owned games.
    /// - `Err(reqwest::Error)`: An error if the request fails.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - **Network request**: Sends a GET request to the Steam API.
    /// <side-effects-end>
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

    /// Retrieves the achievements for a specific game.
    ///
    /// <purpose-start>
    /// This function sends a request to the Steam API to retrieve the achievements for a specific game.
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `appid`: The ID of the game.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - `Ok((String, Vec<Achievement>))`: A tuple containing the game name and a vector of `Achievement` structs.
    /// - `Err(reqwest::Error)`: An error if the request fails.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - **Network request**: Sends a GET request to the Steam API.
    /// <side-effects-end>
    #[tokio::main]
    pub async fn get_game_achievements(&self, appid: u32) -> Result<(String, Vec<Achievement>), reqwest::Error> {
        let api_key = self.api_key.clone();
        let steam_id = self.steam_id.clone();

        // Game achievements
        let url = format!("http://api.steampowered.com/ISteamUserStats/GetPlayerAchievements/v0001/?appid={appid}&key={api_key}&steamid={steam_id}&l=en");

        // Send the GET request
        let response = reqwest::get(url).await?;

        // Check if the request was successful and parse the JSON
        if response.status().is_success() {
            let data: PlayerStatsResponse = response.json().await?;
            return Ok((data.playerstats.game_name, data.playerstats.achievements));
        } else {
            eprintln!("Failed to fetch data: {}", response.status());
        }

        Ok((String::new(), Vec::new()))
    }

    /// Retrieves the global achievement percentages for a specific game.
    ///
    /// <purpose-start>
    /// This function sends a request to the Steam API to retrieve the global achievement percentages for a specific game.
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `appid`: The ID of the game.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - `Ok(Vec<GlobalAchievement>)`: A vector of `GlobalAchievement` structs.
    /// - `Err(reqwest::Error)`: An error if the request fails.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - **Network request**: Sends a GET request to the Steam API.
    /// <side-effects-end>
    #[tokio::main]
    pub async fn get_global_achievements(&self, appid: u32) -> Result<Vec<GlobalAchievement>, reqwest::Error> {
        // Global achievements
        let url = format!("http://api.steampowered.com/ISteamUserStats/GetGlobalAchievementPercentagesForApp/v0002/?gameid={appid}&format=json&l=en");

        // Send the GET request
        let response = reqwest::get(url).await?;

        // Check if the request was successful and parse the JSON
        if response.status().is_success() {
            let data: GlobalAchievementsResponse = response.json().await?;
            return Ok(data.achievementpercentages.achievements);
        } else {
            eprintln!("Failed to fetch data: {}", response.status());
        }

        Ok(Vec::new())
    }
}