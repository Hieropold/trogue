pub mod cfg;
pub mod steam_api;
pub mod ui;

fn main() {
    let api_key = cfg::read_env("TROPHYROOM_STEAM_API_KEY");
    let steam_id = cfg::read_env("TROPHYROOM_STEAM_ID");

    ui::print_title();

    let api = steam_api::Api::new(api_key, steam_id);

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
