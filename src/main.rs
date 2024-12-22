pub mod cfg;
pub mod steam_api;
pub mod ui;

use clap::{Arg, Command};
use crossterm::{cursor, event::Event, event::KeyCode, execute, terminal};
use std::io::{self, stdout, Read, Write};
use std::process;

fn load_cfg() -> cfg::Cfg {
    let mut cfg = cfg::Cfg::new();

    if let Err(e) = cfg.load() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    cfg
}

fn main() {
    let cli_matches = Command::new("trophyroom")
        .version("1.0")
        .author("Hieropold <unsolicited.pcholler@gmail.com>")
        .about("A CLI tool for displaying Steam achievements")
        .arg(
            Arg::new("banner")
                .short('b')
                .long("banner")
                .action(clap::ArgAction::SetTrue)
                .help("Displays a banner with the name of the program"),
        )
        .arg(
            Arg::new("list")
                .short('l')
                .long("list")
                .value_name("filter")
                .action(clap::ArgAction::Set)
                .num_args(0..=1)
                .help("Displays a list of all games on account set in environment variables"),
        )
        .get_matches();

    if cli_matches.get_flag("banner") {
        ui::print_title();
        return;
    }

    if cli_matches.contains_id("list") {
        let filter = cli_matches.get_one::<String>("list").cloned();
        list_games(filter);
    }

    return;

    ui::print_title();

    let cfg = load_cfg();
    let api = steam_api::Api::new(cfg.api_key().to_string(), cfg.steam_id().to_string());

    let mut games = Vec::new();
    match api.get_games_list() {
        Ok(resp) => games = resp,
        Err(e) => eprintln!("Error while trying to get Steam data: {}", e),
    }

    // let selected_game = select_game(&games).unwrap();
    select_game(&games);

    /*println!("Selected game: {}", selected_game.name);

    let mut achievements = Vec::new();
    match api.get_game_achievements(selected_game.appid) {
        Ok(resp) => achievements = resp,
        Err(e) => eprintln!("Error while trying to get achievements: {}", e),
    }

    achievements.sort_by(|a, b| a.apiname.cmp(&b.apiname));

    for achievement in achievements {
        println!("{}", achievement.render_card())
    }*/

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

fn list_games(filter: Option<String>) {
    // Load all games
    let cfg = load_cfg();
    let api = steam_api::Api::new(cfg.api_key().to_string(), cfg.steam_id().to_string());
    let mut games = Vec::new();
    match api.get_games_list() {
        Ok(resp) => games = resp,
        Err(e) => eprintln!("Error while trying to get Steam data: {}", e),
    }

    match filter {
        Some(f) => {
            println!("Displaying games filtered by: {}", f);
            games.retain(|entry| entry.name.to_lowercase().contains(&f.to_lowercase()));
        }
        None => {
            println!("Displaying all games:");
        }
    }

    for game in games {
        ui::print_game_title(&game);
    }
}

// fn select_game(games: &Vec<steam_api::Game>) -> Result<&steam_api::Game, String> {
fn select_game(games: &Vec<steam_api::Game>) {
    // let mut idx = 0;
    // for game in games {
    //     idx += 1;
    //     println!("[{}] {}", idx, game.name);
    // }

    let mut name_filter = String::new();

    // Initialize term to enter raw mode
    terminal::enable_raw_mode().expect("Failed to enable terminal raw mode");

    // Clear terminal screen
    execute!(
        stdout(),
        cursor::MoveTo(0, 0),
        terminal::Clear(terminal::ClearType::All)
    )
    .unwrap();

    loop {
        // io::stdout().flush().map_err(|e| e.to_string())?;

        // io::stdin().read_line(&mut name_filter).map_err(|e| e.to_string())?;

        // io::stdin().read_to_string(name_filter).map_err(|e| e.to_string());

        // name_filter = name_filter.trim().to_string();

        /*let mut filtered_games = games.iter().filter(|game| {
            if name_filter.len() == 0 {
                return true;
            }
            return game.name.to_lowercase().contains(&name_filter.to_lowercase());
        }).collect::<Vec<&steam_api::Game>>();

        if filtered_games.len() == 0 {
            println!("No games found.");
            continue;
        }*/

        // Read the next event from the terminal
        if let Event::Key(key_event) = crossterm::event::read().expect("Failed to read key event") {
            match key_event.code {
                KeyCode::Char(c) => {
                    // Append the character to the filter
                    name_filter.push(c);
                }
                KeyCode::Backspace => {
                    // Remove the last character from the filter
                    name_filter.pop();
                }
                KeyCode::Esc | KeyCode::Enter => {
                    break;
                }
                _ => {}
            }
        }

        execute!(
            stdout(),
            cursor::MoveTo(0, 0),
            terminal::Clear(terminal::ClearType::All)
        )
        .unwrap();
        print!("{}\n", name_filter);

        // Filter the games based on the current filter input
        let mut filtered_games = games.clone();
        filtered_games.retain(|entry| {
            entry
                .name
                .to_lowercase()
                .contains(&name_filter.to_lowercase())
        });

        // Print out the filtered list
        let mut idx = 0;
        for game in filtered_games {
            idx += 1;
            execute!(stdout(), cursor::MoveTo(0, idx)).unwrap();
            println!("{}", game.name);
        }

        // Move the cursor to end of first line
        let name_length: u16 = name_filter
            .len()
            .try_into()
            .expect("Name length too long to fit into u16");
        execute!(stdout(), cursor::MoveTo(name_length, 0)).unwrap();
    }

    // Reset terminal mode
    terminal::disable_raw_mode().expect("Failed to disable the raw mode");
}

/*print!("Please select game [1 - {}]: ", filtered_games.len());
io::stdout().flush().map_err(|e| e.to_string())?;

let game = games.get(entered_idx as usize - 1).ok_or("Invalid game index.")?;

return Ok(game);*/
