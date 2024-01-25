use std::env;

use reqwest;
use serde::{Deserialize, Serialize};
use tokio;

#[derive(Serialize, Deserialize, Debug)]
struct ApiResponse {
    response: GamesList,
}

#[derive(Serialize, Deserialize, Debug)]
struct GamesList {
    game_count: u32,
    games: Vec<Game>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Game {
    appid: u32,
    name: String,
    playtime_forever: u32,
    img_icon_url: String,
    has_community_visible_stats: bool,
    playtime_windows_forever: u32,
    playtime_mac_forever: u32,
    playtime_linux_forever: u32,
    rtime_last_played: u64,
    playtime_disconnected: u32,
}

#[tokio::main]
async fn req(api_key: String, steam_id: String) -> Result<Vec<Game>, reqwest::Error> {
    // List of owned games
    let url = format!("http://api.steampowered.com/IPlayerService/GetOwnedGames/v0001/?key={api_key}&steamid={steam_id}&format=json&include_appinfo=1");

    // Send the GET request
    let response = reqwest::get(url).await?;

    // Check if the request was successful and parse the JSON
    if response.status().is_success() {
        let data: ApiResponse = response.json().await?;
        return Ok(data.response.games);
    } else {
        eprintln!("Failed to fetch data: {}", response.status());
    }

    Ok(Vec::new())
}

fn print_title() {
    let title = r#"                                                                                                                                       
  ****           *                                      *                                                                          
  *  *************                                     **                                                                           
 *     *********                                       **                                                                           
 *     *  *                                            **                                                                           
  **  *  **         ***  ****       ****       ****    **        **   ****      ***  ****       ****       ****                     
     *  ***          **** **** *   * ***  *   * ***  * **  ***    **    ***  *   **** **** *   * ***  *   * ***  * *** **** ****    
    **   **           **   ****   *   ****   *   ****  ** * ***   **     ****     **   ****   *   ****   *   ****   *** **** ***  * 
    **   **           **         **    **   **    **   ***   ***  **      **      **         **    **   **    **     **  **** ****  
    **   **           **         **    **   **    **   **     **  **      **      **         **    **   **    **     **   **   **   
    **   **           **         **    **   **    **   **     **  **      **      **         **    **   **    **     **   **   **   
     **  **           **         **    **   **    **   **     **  **      **      **         **    **   **    **     **   **   **   
      ** *      *     **         **    **   **    **   **     **  **      **      **         **    **   **    **     **   **   **   
       ***     *      ***         ******    *******    **     **   *********      ***         ******     ******      **   **   **   
        *******        ***         ****     ******     **     **     **** ***      ***         ****       ****       ***  ***  ***  
          ***                               **          **    **           ***                                        ***  ***  *** 
                                            **                *     *****   ***                                                     
                                            **               *    ********  **                                                      
                                             **             *    *      ****                                                        
                                                           *                                                                        
"#;

    println!("{title}");
}

fn read_env(key: &str) -> String {
    return env::var(key).expect(&format!("{} must be set", key));
}

fn main() {
    let api_key = read_env("TROPHYROOM_STEAM_API_KEY");
    let steam_id = read_env("TROPHYROOM_STEAM_ID");

    print_title();

    let mut games = Vec::new();
    match req(api_key, steam_id) {
        Ok(resp) => games = resp,
        Err(e) => println!("Error while trying to get Steam data: {}", e),
    }

    println!("Steam games:");
    for game in &games {
        println!("\t- {}", game.name);
    }
}
