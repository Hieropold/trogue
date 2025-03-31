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

        let pattern = optional_pattern.unwrap_or("[i] n".to_string());

        for game in games {
            let displayable_game = ui::DisplayableGame { game };
            let formatted_game = displayable_game.format(&pattern);
            println!("{}", formatted_game);
        }
    }

    pub fn show_progress(&self, game_id: u32) {
        let mut achievements = Vec::new();
        let mut game_name = String::new();

        match &self.api.get_game_achievements(game_id) {
            Ok((name, achs)) => {
                game_name = name.clone();
                achievements = achs.clone();
            },
            Err(e) => eprintln!("Error while trying to get achievements: {}", e)
        }

        println!("{}", game_name);

        if achievements.is_empty() {
            println!("No achievements found for this game");
            return;
        }

        let total = achievements.len();
        let completed = achievements.iter().filter(|a| a.achieved > 0).count();
        let percentage = (completed as f32 / total as f32) * 100.0;

        // Get terminal width and use 50% of it for the progress bar
        let terminal_width = crossterm::terminal::size().unwrap_or((80, 24)).0 as usize;
        let bar_width = terminal_width / 2;
        
        let filled_chars = ((percentage / 100.0) * bar_width as f32).round() as usize;
        let empty_chars = bar_width - filled_chars;

        print!("[");
        for _ in 0..filled_chars {
            print!("█");
        }
        for _ in 0..empty_chars {
            print!(" ");
        }
        println!("] {:.1}% ({}/{})", percentage, completed, total);
    }

    pub fn list_achievements(&self, game_id: u32, add_global: bool, remaining: bool) {
        let mut achievements = Vec::new();

        match &self.api.get_game_achievements(game_id) {
            Ok((_, achs)) => achievements = achs.clone(),
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
            // Skip achieved achievements if we only want to display remaining ones
            if remaining && achievement.achieved > 0 {
                continue;
            }

            let displayable_achievement = ui::DisplayableAchievement { achievement };

            let mut title: String;
            if displayable_achievement.achievement.achieved > 0 {
                title = displayable_achievement.format("n - s (t)");
            } else {
                title = displayable_achievement.format("n");
            }

            // Add global percentage to the output if requested
            if add_global {
                let global_percent = global_achievement_map
                        .get(&displayable_achievement.achievement.apiname)
                        .unwrap_or(&0.0);

                title.push_str(&format!(" {}%", global_percent));
            }

            println!("{}", title);
        }
    }

    pub fn show_dashboard(&self) {
        let mut games = Vec::new();
        match &self.api.get_games_list() {
            Ok(resp) => games = resp.clone(),
            Err(e) => eprintln!("Error while trying to get Steam data: {}", e),
        }

        // Sort games by last played time (most recent first)
        games.sort_by(|a, b| b.rtime_last_played.cmp(&a.rtime_last_played));

        // Take only the 10 most recently played games
        let recent_games: Vec<_> = games.iter().take(10).collect();

        // Output title
        let terminal_width = crossterm::terminal::size().unwrap_or((80, 24)).0 as usize;
        let box_width = terminal_width / 2;
        let title = "Recently Played Games Dashboard";
        let padding = (box_width - title.len()) / 2;
        
        println!("{}", "=".repeat(box_width));
        println!("{}{}{}", " ".repeat(padding), title, " ".repeat(padding));
        println!("{}", "=".repeat(box_width));

        for game in recent_games {
            self.show_progress(game.appid);                
        }
    }
}
