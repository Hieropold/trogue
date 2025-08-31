use crate::{cfg::Cfg, steam_api::Api, ui};

/// The main application structure.
///
/// <purpose-start>
/// This struct holds the state of the application, including the Steam API client.
/// <purpose-end>
pub struct App {
    api: Api,
}

impl App {
    /// Creates a new `App` instance.
    ///
    /// <purpose-start>
    /// This function initializes the `App` struct, creating a new `Api` instance with the provided configuration.
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `cfg`: The application configuration, containing the API key and Steam ID.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - `App`: A new `App` instance.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - None.
    /// <side-effects-end>
    pub fn new(cfg: Cfg) -> App {
        let api = Api::new(cfg.api_key().to_string(), cfg.steam_id().to_string());

        App { api }
    }

    /// Lists all games owned by the user, with optional filtering.
    ///
    /// <purpose-start>
    /// This function retrieves the list of games owned by the user from the Steam API,
    /// optionally filters them by name, and prints them to the console.
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `filter`: An optional string to filter the games by name.
    /// - `optional_pattern`: An optional string to format the output.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - None.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - **Prints to the console**: The list of games is printed to the standard output.
    /// - **Network request**: Fetches the list of games from the Steam API.
    /// <side-effects-end>
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

    /// Shows the achievement progress for a specific game.
    ///
    /// <purpose-start>
    /// This function retrieves the achievement progress for a specific game from the Steam API
    /// and displays a progress bar in the console.
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `game_id`: The ID of the game to show the progress for.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - None.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - **Prints to the console**: The achievement progress is printed to the standard output.
    /// - **Network request**: Fetches the achievement data from the Steam API.
    /// <side-effects-end>
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

    /// Lists the achievements for a specific game.
    ///
    /// <purpose-start>
    /// This function retrieves the achievements for a specific game from the Steam API
    /// and prints them to the console.
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `game_id`: The ID of the game to list the achievements for.
    /// - `add_global`: Whether to include the global achievement percentage.
    /// - `remaining`: Whether to only show remaining (unachieved) achievements.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - None.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - **Prints to the console**: The list of achievements is printed to the standard output.
    /// - **Network request**: Fetches the achievement data from the Steam API.
    /// <side-effects-end>
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

    /// Shows a dashboard of recently played games.
    ///
    /// <purpose-start>
    /// This function retrieves the user's recently played games from the Steam API
    /// and displays a dashboard with their achievement progress.
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - None.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - None.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - **Prints to the console**: The dashboard is printed to the standard output.
    /// - **Network request**: Fetches the list of games and achievement data from the Steam API.
    /// <side-effects-end>
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
