pub mod cfg;
pub mod steam_api;
pub mod ui;

use std::process;
use std::io::{self, Write};

fn main() {
    let mut cfg = cfg::Cfg::new();

    if let Err(e) = cfg.load() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    ui::print_title();

    let api = steam_api::Api::new(cfg.api_key().to_string(), cfg.steam_id().to_string());

    let mut games = Vec::new();
    match api.get_games_list() {
        Ok(resp) => games = resp,
        Err(e) => eprintln!("Error while trying to get Steam data: {}", e),
    }

    let selected_game = select_game(&games).unwrap();

    println!("Selected game: {}", selected_game.name);

    let mut achievements = Vec::new();
    match api.get_game_achievements(selected_game.appid) {
        Ok(resp) => achievements = resp,
        Err(e) => eprintln!("Error while trying to get achievements: {}", e),
    }

    achievements.sort_by(|a, b| a.apiname.cmp(&b.apiname));

    for achievement in achievements {
        println!("{}", achievement.render_card())
    }

    // loop {
    //     print!("Please select achievement [1 - {}]: ", achievements.len());
    //     io::stdout().flush().map_err(|e| e.to_string())?;

    //     let mut input = String::new();
    //     io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;

    //     let selected_achievement = match input.trim().parse::<usize>() {
    //         Ok(idx) => {
    //             if idx > 0 && idx <= achievements.len() {
    //                 &achievements[idx - 1]
    //             } else {
    //                 continue;
    //             }
    //         }
    //         Err(_) => continue,
    //     };

    //     println!("Selected achievement: {}", selected_achievement.name);

    //     match api.get_achievement_progress(selected_achievement.api_name.clone()) {
    //         Ok(resp) => {
    //             let progress = resp;
    //             println!("Progress: {}", progress.progress);
    //         }
    //         Err(e) => eprintln!("Error while trying to get achievement progress: {}", e),
    // }


            // ui::print_achievements(&achievements);

            // let selected_achievement = select_achievement(&achievements).unwrap();

            // println!("Selected achievement: {}", selected_achievement.name);

            // match api.get_achievement_progress(selected_achievement.api_name.clone()) {
            //     Ok(resp) => {
            //         let progress = resp;
            //         println!("Progress: {}", progress.progress);
            //     }
            //     Err(e) => eprintln!("Error while trying to get achievement progress: {}", e),
}

fn select_game(games: &Vec<steam_api::Game>) -> Result<&steam_api::Game, String> {
    println!("Steam games:");
    let mut idx = 0;
    for game in games {
        idx += 1;
        println!("[{}] {}", idx, game.name);
    }

    loop {
        print!("Please select game [1 - {}]: ", idx);
        io::stdout().flush().map_err(|e| e.to_string())?;

        let mut input = String::new();
        io::stdin().read_line(&mut input).map_err(|e| e.to_string())?;

        let entered_idx = match input.trim().parse::<i32>() {
            Ok(num) if num >= 1 && num <= idx => {
                num
            }
            Ok(_) => {
                println!("Error: Number not in range 1-{}.", idx);
                -1
            }
            Err(_) => {
                println!("Error: Invalid input. Please enter an integer.");
                -1
            }
        };

        if entered_idx == -1 {
            continue;
        }

        let game = games.get(entered_idx as usize - 1).ok_or("Invalid game index.")?;

        return Ok(game);
    }

}