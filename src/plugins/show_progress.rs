//! Plugin for showing the achievement progress for a specific game.
//!
//! <purpose-start>
//! This plugin provides the `progress` command, which displays a progress bar
//! representing the achievement completion for a given game.
//! <purpose-end>
//!
//! <inputs-start>
//! - `app_context`: The shared application context, providing access to the Steam API client.
//! - `matches`: The command-line arguments parsed by `clap`.
//! <inputs-end>
//!
//! <outputs-start>
//! - A progress bar and completion statistics printed to the console.
//! <outputs-end>
//!
//! <side-effects-start>
//! - Makes a network request to the Steam API to fetch achievement data.
//! <side-effects-end>

use crate::{app::AppContext, plugins::Plugin};
use async_trait::async_trait;
use clap::{Arg, Command};

pub struct ShowProgressPlugin;

#[async_trait]
impl Plugin for ShowProgressPlugin {
    /// Defines the clap command for the `progress` plugin.
    ///
    /// <purpose-start>
    /// This method provides the command-line interface for the `progress` plugin,
    /// which displays the achievement progress for a specific game.
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `&self`: A reference to the plugin instance.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - `clap::Command`: The clap command definition for the `progress` plugin.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - None.
    /// <side-effects-end>
    fn command(&self) -> Command {
        Command::new("progress")
            .about("Displays game achievements progress.")
            .arg(
                Arg::new("game_id")
                    .value_name("game_id")
                    .action(clap::ArgAction::Set)
                    .required(true)
                    .help("The ID of the game to show progress for"),
            )
    }

    /// Executes the `progress` plugin's logic.
    ///
    /// <purpose-start>
    /// This method is called by the core application when the `progress` command is invoked.
    /// It fetches the achievement data for a given game and displays a progress bar in the console.
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `&self`: A reference to the plugin instance.
    /// - `app_context`: The shared application context.
    /// - `matches`: The clap argument matches for the `progress` subcommand.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - None.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - Makes a network request to the Steam API to fetch achievement data.
    /// - Prints the progress bar to the console.
    /// <side-effects-end>
    async fn execute(&self, app_context: &AppContext, matches: &clap::ArgMatches) {
        let game_id_str = matches.get_one::<String>("game_id").unwrap();

        if let Ok(game_id) = game_id_str.parse::<u32>() {
            let mut achievements = Vec::new();
            let mut game_name = String::new();

            match app_context.api.get_game_achievements(game_id).await {
                Ok((name, achs)) => {
                    game_name = name;
                    achievements = achs;
                }
                Err(e) => eprintln!("Error while trying to get achievements: {}", e),
            }

            println!("{}", game_name);

            if achievements.is_empty() {
                println!("No achievements found for this game");
                return;
            }

            let total = achievements.len();
            let completed = achievements.iter().filter(|a| a.achieved > 0).count();
            let percentage = (completed as f32 / total as f32) * 100.0;

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
        } else {
            eprintln!("Invalid game id: {}", game_id_str);
        }
    }
}
