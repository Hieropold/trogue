use crate::{cfg::Cfg, steam_api::Api, ui};

pub struct App {
    api: Api,
}

impl App {
    pub fn new(cfg: Cfg) -> App {
        let api = Api::new(cfg.api_key().to_string(), cfg.steam_id().to_string());

        App { api }
    }

    pub fn list_games(&self, filter: Option<String>, optional_pattern: Option<String>) {
        // Load all games
        let mut games = Vec::new();

        match &self.api.get_games_list() {
            Ok(resp) => games = resp.clone(),
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

        let pattern = optional_pattern.unwrap_or("n".to_string());

        for game in games {
            let displayable_game = ui::DisplayableGame { game };
            let formatted_game = displayable_game.format(&pattern);
            println!("{}", formatted_game);
        }
    }

    pub fn list_achievements(&self, game_id: u32, add_global: bool) {
        let mut achievements = Vec::new();

        match &self.api.get_game_achievements(game_id) {
            Ok(resp) => achievements = resp.clone(),
            Err(e) => eprintln!("Error while trying to get achievements: {}", e),
        }

        let mut global_achievement_map = std::collections::HashMap::new();
        if add_global {
            match &self.api.get_global_achievements(game_id) {
                Ok(resp) => {
                    for global_achievement in resp {
                        global_achievement_map
                            .insert(global_achievement.name.clone(), global_achievement.percent);
                    }
                }
                Err(e) => eprintln!("Error while trying to get global achievements: {}", e),
            }
        }

        for achievement in achievements {
            let displayable_achievement = ui::DisplayableAchievement { achievement };
            if displayable_achievement.achievement.achieved > 0 {
                if add_global {
                    let global_percent = global_achievement_map
                        .get(&displayable_achievement.achievement.apiname)
                        .unwrap_or(&0.0);
                    println!(
                        "{} {}%",
                        displayable_achievement.format("n - s (t)"),
                        global_percent
                    );
                } else {
                    println!("{}", displayable_achievement.format("n - s (t)"));
                }
            } else {
                if add_global {
                    let global_percent = global_achievement_map
                        .get(&displayable_achievement.achievement.apiname)
                        .unwrap_or(&0.0);
                    println!("{} {}%", displayable_achievement.format("n"), global_percent);
                } else {
                    println!("{}", displayable_achievement.format("n"));
                }
            }
        }
    }
}
