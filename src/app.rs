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

    pub fn list_achievements(&self, game_id: u32) {
        let mut achievements = Vec::new();

        match &self.api.get_game_achievements(game_id) {
            Ok(resp) => achievements = resp.clone(),
            Err(e) => eprintln!("Error while trying to get achievements: {}", e),
        }

        for achievement in achievements {
            let displayable_achievement = ui::DisplayableAchievement { achievement };
            if displayable_achievement.achievement.achieved > 0 {
                println!("{}", displayable_achievement.format("n - s (t)"));
            } else {
                println!("{}", displayable_achievement.format("n"));
            }
        }
    }
}
