//! Plugin for listing all games owned by the user.
//!
//! <purpose-start>
//! This plugin provides the `list` command, which allows users to see a list of their games.
//! It supports filtering by name and custom output formatting.
//! <purpose-end>
//!
//! <inputs-start>
//! - `app_context`: The shared application context, providing access to the Steam API client.
//! - `matches`: The command-line arguments parsed by `clap`.
//! <inputs-end>
//!
//! <outputs-start>
//! - A list of games printed to the console.
//! <outputs-end>
//!
//! <side-effects-start>
//! - Makes a network request to the Steam API to fetch the list of games.
//! <side-effects-end>

use crate::{app::AppContext, plugins::Plugin, ui};
use async_trait::async_trait;
use clap::{Arg, Command};

pub struct ListGamesPlugin;

#[async_trait]
impl Plugin for ListGamesPlugin {
    /// Defines the clap command for the `list` plugin.
    ///
    /// <purpose-start>
    /// This method provides the command-line interface for the `list` plugin,
    /// which allows users to list their games with optional filtering and formatting.
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `&self`: A reference to the plugin instance.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - `clap::Command`: The clap command definition for the `list` plugin.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - None.
    /// <side-effects-end>
    fn command(&self) -> Command {
        Command::new("list")
            .about("Displays a list of all games on account set in environment variables")
            .arg(
                Arg::new("filter")
                    .short('f')
                    .long("filter")
                    .value_name("filter")
                    .action(clap::ArgAction::Set)
                    .num_args(0..=1)
                    .help("Displays a list of all games on account set in environment variables"),
            )
            .arg(
                Arg::new("pattern")
                    .short('p')
                    .long("pattern")
                    .help(
                        r#"Specifies the output format for the list command. It can be used only with --list command. By default, the game id and name are displayed.
Possible tokens are:
    n - game name
    i - game id
E.g.: -p "i: n""#,
                    )
                    .requires("filter")
                    .value_name("pattern"),
            )
    }

    /// Executes the `list` plugin's logic.
    ///
    /// <purpose-start>
    /// This method is called by the core application when the `list` command is invoked.
    /// It fetches the list of games, applies any specified filters, and prints the formatted list to the console.
    /// <purpose-end>
    ///
    /// <inputs-start>
    /// - `&self`: A reference to the plugin instance.
    /// - `app_context`: The shared application context.
    /// - `matches`: The clap argument matches for the `list` subcommand.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - None.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - Makes a network request to the Steam API to fetch the list of games.
    /// - Prints the list of games to the console.
    /// <side-effects-end>
    async fn execute(&self, app_context: &AppContext, matches: &clap::ArgMatches) {
        let filter = matches.get_one::<String>("filter").cloned();
        let pattern = matches.get_one::<String>("pattern").cloned();

        let mut games = Vec::new();
        match app_context.api.get_games_list().await {
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

        let pattern = pattern.unwrap_or("[i] n".to_string());

        for game in games {
            let displayable_game = ui::DisplayableGame { game };
            let formatted_game = displayable_game.format(&pattern);
            println!("{}", formatted_game);
        }
    }
}
