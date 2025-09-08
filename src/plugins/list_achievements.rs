//! Plugin for listing achievements for a specific game.
//!
//! <purpose-start>
//! This plugin provides the `achievements` command, which allows users to list the achievements for a given game.
//! It supports filtering by achieved status and can include global achievement percentages.
//! <purpose-end>
//!
//! <inputs-start>
//! - `app_context`: The shared application context, providing access to the Steam API client.
//! - `matches`: The command-line arguments parsed by `clap`.
//! <inputs-end>
//!
//! <outputs-start>
//! - A list of achievements printed to the console.
//! <outputs-end>
//!
//! <side-effects-start>
//! - Makes network requests to the Steam API to fetch achievement data.
//! <side-effects-end>

use crate::{app::AppContext, plugins::Plugin, ui};
use async_trait::async_trait;
use clap::{Arg, Command};

pub struct ListAchievementsPlugin;

#[async_trait]
impl Plugin for ListAchievementsPlugin {
    /// Defines the clap command for the `achievements` plugin.
    ///
    /// <purpose-start>
    /// This method provides the command-line interface for the `achievements` plugin,
    /// which allows users to list achievements for a specific game.
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `&self`: A reference to the plugin instance.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - `clap::Command`: The clap command definition for the `achievements` plugin.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - None.
    /// <side-effects-end>
    fn command(&self) -> Command {
        Command::new("achievements")
            .about("Displays achievements for a specific game. Game id should be provided as an argument")
            .arg(
                Arg::new("game_id")
                    .value_name("game_id")
                    .action(clap::ArgAction::Set)
                    .required(true)
                    .help("The ID of the game to list achievements for"),
            )
            .arg(
                Arg::new("global")
                    .short('g')
                    .long("global")
                    .action(clap::ArgAction::SetTrue)
                    .help("Adds global achievement percentages for the output of game achievements."),
            )
            .arg(
                Arg::new("remaining")
                    .short('r')
                    .long("remaining")
                    .action(clap::ArgAction::SetTrue)
                    .help("Displays only remaining locked achievements."),
            )
    }

    /// Executes the `achievements` plugin's logic.
    ///
    /// <purpose-start>
    /// This method is called by the core application when the `achievements` command is invoked.
    /// It fetches the list of achievements for a given game, applies any specified filters, and prints the list to the console.
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `&self`: A reference to the plugin instance.
    /// - `app_context`: The shared application context.
    /// - `matches`: The clap argument matches for the `achievements` subcommand.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - None.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - Makes network requests to the Steam API to fetch achievement data.
    /// - Prints the list of achievements to the console.
    /// <side-effects-end>
    async fn execute(&self, app_context: &AppContext, matches: &clap::ArgMatches) {
        let game_id_str = matches.get_one::<String>("game_id").unwrap();
        let add_global = matches.get_flag("global");
        let remaining = matches.get_flag("remaining");

        if let Ok(game_id) = game_id_str.parse::<u32>() {
            let mut achievements = Vec::new();

            match app_context.api.get_game_achievements(game_id).await {
                Ok((_, achs)) => achievements = achs,
                Err(e) => eprintln!("Error while trying to get achievements: {}", e),
            }

            let mut global_achievement_map = std::collections::HashMap::new();
            if add_global {
                match app_context.api.get_global_achievements(game_id).await {
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

                if add_global {
                    let global_percent = global_achievement_map
                        .get(&displayable_achievement.achievement.apiname)
                        .unwrap_or(&0.0);

                    title.push_str(&format!(" {}%", global_percent));
                }

                println!("{}", title);
            }
        } else {
            eprintln!("Invalid game id: {}", game_id_str);
        }
    }
}
