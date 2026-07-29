//! Plugin for listing all games owned by the user.
//!
//! <purpose-start>
//! This plugin provides the `list` command, which allows users to see a list of their games.
//! It supports filtering by name and custom output formatting.
//! <purpose-end>
//!
//! <inputs-start>
//! - `steam`: The Steam client seam, providing access to the user's owned games.
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

use crate::{plugins::Plugin, steam_client::SteamClient, ui};
use async_trait::async_trait;
use clap::{Arg, Command};
use std::io::Write;

pub struct ListGamesPlugin;

#[async_trait]
impl Plugin for ListGamesPlugin {
    // Defines the clap command for the `list` plugin.
    //
    // <purpose-start>
    // This method provides the command-line interface for the `list` plugin,
    // which allows users to list their games with optional filtering and formatting.
    // <purpose-end>
    //
    // <inputs-start>
    // - `&self`: A reference to the plugin instance.
    // <inputs-end>
    //
    // <outputs-start>
    // - `clap::Command`: The clap command definition for the `list` plugin.
    // <outputs-end>
    //
    // <side-effects-start>
    // - None.
    // <side-effects-end>
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
                    .help("Filters the list of games by name"),
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

    // Executes the `list` plugin's logic.
    //
    // <purpose-start>
    // This method is called by the core application when the `list` command is invoked.
    // It fetches the list of games, applies any specified filters, and prints the formatted list to the console.
    // <purpose-end>
    //
    // <inputs-start>
    // - `&self`: A reference to the plugin instance.
    // - `steam`: The Steam client seam.
    // - `matches`: The clap argument matches for the `list` subcommand.
    // - `writer`: A mutable reference to a writer for standard output.
    // - `err_writer`: A mutable reference to a writer for standard error.
    // <inputs-end>
    //
    // <outputs-start>
    // - None.
    // <outputs-end>
    //
    // <side-effects-start>
    // - Makes a network request to the Steam API to fetch the list of games.
    // - Writes the list of games to the provided writer.
    // <side-effects-end>
    async fn execute(
        &self,
        steam: &dyn SteamClient,
        matches: &clap::ArgMatches,
        writer: &mut (dyn Write + Send),
        err_writer: &mut (dyn Write + Send),
    ) {
        let filter = matches.get_one::<String>("filter").cloned();
        let pattern = matches.get_one::<String>("pattern").cloned();

        let mut games = Vec::new();
        match steam.owned_games().await {
            Ok(resp) => games = resp,
            Err(e) => writeln!(err_writer, "{}", e).unwrap(),
        }

        match filter {
            Some(f) => {
                writeln!(writer, "Displaying games filtered by: {}", f).unwrap();
                games.retain(|entry| entry.name.to_lowercase().contains(&f.to_lowercase()));
            }
            None => {
                writeln!(writer, "Displaying all games:").unwrap();
            }
        }

        let pattern = pattern.unwrap_or("[i] n".to_string());

        for game in games {
            let displayable_game = ui::DisplayableGame { game };
            let formatted_game = displayable_game.format(&pattern);
            writeln!(writer, "{}", formatted_game).unwrap();
        }
    }

    // Executes the `list` plugin's logic and returns structured data.
    async fn execute_deep(
        &self,
        steam: &dyn SteamClient,
        matches: &clap::ArgMatches,
    ) -> Result<crate::ui::ViewData, String> {
        let filter = matches.get_one::<String>("filter").cloned();
        let pattern = matches.get_one::<String>("pattern").cloned();

        let mut games = match steam.owned_games().await {
            Ok(resp) => resp,
            Err(e) => return Err(e.to_string()),
        };

        if let Some(ref f) = filter {
            games.retain(|entry| entry.name.to_lowercase().contains(&f.to_lowercase()));
        }

        let pattern = pattern.unwrap_or("[i] n".to_string());

        Ok(crate::ui::ViewData::ListGames(games, filter, pattern))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::steam_client::fake::FakeSteam;
    use crate::steam_client::{Game, SteamError};
    use clap::ArgMatches;

    fn create_mock_game(appid: u32, name: &str) -> Game {
        Game {
            appid,
            name: name.to_string(),
            playtime_forever: 0,
            img_icon_url: "".to_string(),
            playtime_windows_forever: 0,
            playtime_mac_forever: 0,
            playtime_linux_forever: 0,
            rtime_last_played: 0,
            playtime_disconnected: 0,
        }
    }

    fn get_matches_for_args(args: &[&str]) -> ArgMatches {
        ListGamesPlugin.command().get_matches_from(args)
    }

    #[test]
    fn test_command() {
        let plugin = ListGamesPlugin;
        let cmd = plugin.command();
        assert_eq!(cmd.get_name(), "list");
        assert!(cmd.get_about().is_some());
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "filter"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "pattern"));
    }

    #[tokio::test]
    async fn test_execute_success_no_filter() {
        let steam =
            FakeSteam::new().with_games(vec![create_mock_game(1, "Game 1"), create_mock_game(2, "Game 2")]);
        let matches = get_matches_for_args(&["list"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListGamesPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Displaying all games:"));
        assert!(output.contains("[1] Game 1"));
        assert!(output.contains("[2] Game 2"));
    }

    #[tokio::test]
    async fn test_execute_success_with_filter() {
        let steam = FakeSteam::new().with_games(vec![
            create_mock_game(1, "Awesome Game"),
            create_mock_game(2, "Another Game"),
        ]);
        let matches = get_matches_for_args(&["list", "--filter", "Awesome"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListGamesPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Displaying games filtered by: Awesome"));
        assert!(output.contains("[1] Awesome Game"));
        assert!(!output.contains("[2] Another Game"));
    }

    #[tokio::test]
    async fn test_execute_success_with_filter_and_pattern() {
        let steam = FakeSteam::new().with_games(vec![create_mock_game(1, "Awesome Game")]);
        let matches = get_matches_for_args(&["list", "--filter", "Awesome", "--pattern", "i - n"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListGamesPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Displaying games filtered by: Awesome"));
        assert!(output.contains("1 - Awesome Game"));
    }

    #[tokio::test]
    async fn test_execute_api_error() {
        let steam = FakeSteam::new().with_games_error(SteamError::Http {
            status: Some(500),
            msg: "boom".to_string(),
        });
        let matches = get_matches_for_args(&["list"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListGamesPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(err_writer).unwrap();
        assert!(output.contains("Steam API request failed"));
    }

    #[tokio::test]
    async fn test_execute_no_games() {
        let steam = FakeSteam::new().with_games(vec![]);
        let matches = get_matches_for_args(&["list"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListGamesPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Displaying all games:"));
        assert!(!output.contains("[")); // No games should be listed
    }

    #[tokio::test]
    async fn test_execute_deep_success_no_filter() {
        let steam =
            FakeSteam::new().with_games(vec![create_mock_game(1, "Game 1"), create_mock_game(2, "Game 2")]);
        let matches = get_matches_for_args(&["list"]);

        let result = ListGamesPlugin
            .execute_deep(&steam, &matches)
            .await
            .unwrap();

        match result {
            crate::ui::ViewData::ListGames(games, filter, pattern) => {
                assert_eq!(games.len(), 2);
                assert_eq!(filter, None);
                assert_eq!(pattern, "[i] n");
            }
            _ => panic!("Expected ListGames ViewData"),
        }
    }
}
