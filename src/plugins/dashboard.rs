//! Plugin for displaying a dashboard of recently played games.
//!
//! <purpose-start>
//! This plugin provides the `dashboard` command, which shows the 10 most recently played games
//! and their achievement progress.
//! <purpose-end>
//!
//! <inputs-start>
//! - `app_context`: The shared application context, providing access to the Steam API client.
//! - `_matches`: The command-line arguments parsed by `clap` (unused in this plugin).
//! <inputs-end>
//!
//! <outputs-start>
//! - A dashboard of recently played games printed to the console.
//! <outputs-end>
//!
//! <side-effects-start>
//! - Makes multiple network requests to the Steam API to fetch game lists and achievement data.
//! <side-effects-end>

use crate::{app::AppContext, plugins::Plugin};
use clap::Command;

pub struct DashboardPlugin;

impl Plugin for DashboardPlugin {
    /// Defines the clap command for the `dashboard` plugin.
    ///
    /// <purpose-start>
    /// This method provides the command-line interface for the `dashboard` plugin,
    /// which displays a dashboard of recently played games.
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `&self`: A reference to the plugin instance.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - `clap::Command`: The clap command definition for the `dashboard` plugin.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - None.
    /// <side-effects-end>
    fn command(&self) -> Command {
        Command::new("dashboard")
            .about("Displays a dashboard with 10 last played games and their achievement progress")
    }

    /// Executes the `dashboard` plugin's logic.
    ///
    /// <purpose-start>
    /// This method is called by the core application when the `dashboard` command is invoked.
    /// It fetches the list of recently played games and their achievement progress, and prints the dashboard to the console.
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `&self`: A reference to the plugin instance.
    /// - `app_context`: The shared application context.
    /// - `_matches`: The clap argument matches for the `dashboard` subcommand (unused).
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - None.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - Makes multiple network requests to the Steam API to fetch game and achievement data.
    /// - Prints the dashboard to the console.
    /// <side-effects-end>
    fn execute(&self, app_context: &AppContext, _matches: &clap::ArgMatches) {
        let mut games = Vec::new();
        match &app_context.api.get_games_list() {
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
            let mut achievements = Vec::new();
            let mut game_name = String::new();

            match &app_context.api.get_game_achievements(game.appid) {
                Ok((name, achs)) => {
                    game_name = name.clone();
                    achievements = achs.clone();
                }
                Err(e) => eprintln!("Error while trying to get achievements: {}", e),
            }

            println!("{}", game_name);

            if achievements.is_empty() {
                println!("No achievements found for this game");
                continue;
            }

            let total = achievements.len();
            let completed = achievements.iter().filter(|a| a.achieved > 0).count();
            let percentage = (completed as f32 / total as f32) * 100.0;

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
    }
}