pub mod cfg;
pub mod steam_api;
pub mod ui;

use std::process;

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
        Err(e) => println!("Error while trying to get Steam data: {}", e),
    }

    println!("Steam games:");
    for game in &games {
        println!("\t- {}", game.name);
    }
}
