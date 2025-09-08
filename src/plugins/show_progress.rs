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
use std::io::Write;

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
    /// - `writer`: A mutable reference to a writer for standard output.
    /// - `err_writer`: A mutable reference to a writer for standard error.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - None.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - Makes a network request to the Steam API to fetch achievement data.
    /// - Writes the progress bar to the provided writer.
    /// <side-effects-end>
    async fn execute(
        &self,
        app_context: &AppContext,
        matches: &clap::ArgMatches,
        writer: &mut (dyn Write + Send),
        err_writer: &mut (dyn Write + Send),
    ) {
        let game_id_str = matches.get_one::<String>("game_id").unwrap();

        if let Ok(game_id) = game_id_str.parse::<u32>() {
            match app_context.api.get_game_achievements(game_id).await {
                Ok((game_name, achievements)) => {
                    writeln!(writer, "{}", game_name).unwrap();

                    if achievements.is_empty() {
                        writeln!(writer, "No achievements found for this game").unwrap();
                        return;
                    }

                    let total = achievements.len();
                    let completed = achievements.iter().filter(|a| a.achieved > 0).count();
                    let percentage = (completed as f32 / total as f32) * 100.0;

                    let terminal_width = crossterm::terminal::size().unwrap_or((80, 24)).0 as usize;
                    let bar_width = terminal_width / 2;

                    let filled_chars = ((percentage / 100.0) * bar_width as f32).round() as usize;
                    let empty_chars = bar_width - filled_chars;

                    write!(writer, "[").unwrap();
                    for _ in 0..filled_chars {
                        write!(writer, "█").unwrap();
                    }
                    for _ in 0..empty_chars {
                        write!(writer, " ").unwrap();
                    }
                    writeln!(writer, "] {:.1}% ({}/{})", percentage, completed, total).unwrap();
                }
                Err(e) => writeln!(err_writer, "Error while trying to get achievements: {}", e).unwrap(),
            }
        } else {
            writeln!(err_writer, "Invalid game id: {}", game_id_str).unwrap();
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppContext;
    use crate::steam_api::{Api, Achievement};
    use clap::ArgMatches;

    fn create_mock_achievement(achieved: u8) -> Achievement {
        Achievement {
            apiname: "test_api".to_string(),
            name: "Test Achievement".to_string(),
            description: "Test Description".to_string(),
            achieved,
            unlocktime: 0,
        }
    }

    async fn setup_test_env(mock_body: &str, status_code: u16) -> (AppContext, mockito::ServerGuard) {
        let mut server = mockito::Server::new_async().await;
        server.mock("GET", "/ISteamUserStats/GetPlayerAchievements/v0001/?appid=123&key=test_key&steamid=test_id&l=en")
            .with_status(status_code as usize)
            .with_header("content-type", "application/json")
            .with_body(mock_body)
            .create_async().await;

        let api = Api::new("test_key".to_string(), "test_id".to_string(), server.url());
        let app_context = AppContext { api };
        (app_context, server)
    }

    fn get_matches_for_args(args: &[&str]) -> ArgMatches {
        ShowProgressPlugin.command().get_matches_from(args)
    }

    #[test]
    fn test_command() {
        let plugin = ShowProgressPlugin;
        let cmd = plugin.command();
        assert_eq!(cmd.get_name(), "progress");
        assert!(cmd.get_about().is_some());
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "game_id"));
    }

    #[tokio::test]
    async fn test_execute_success() {
        let achievements = vec![create_mock_achievement(1), create_mock_achievement(0)];
        let mock_body = serde_json::to_string(&serde_json::json!({
            "playerstats": {
                "steamID": "test_id",
                "gameName": "Test Game",
                "achievements": achievements,
                "success": true
            }
        })).unwrap();
        let (app_context, _server) = setup_test_env(&mock_body, 200).await;
        let matches = get_matches_for_args(&["progress", "123"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ShowProgressPlugin.execute(&app_context, &matches, &mut writer, &mut err_writer).await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.starts_with("Test Game"));
        assert!(output.contains("50.0% (1/2)"));
    }

    #[tokio::test]
    async fn test_execute_no_achievements() {
        let mock_body = serde_json::to_string(&serde_json::json!({
            "playerstats": {
                "steamID": "test_id",
                "gameName": "Test Game",
                "achievements": [],
                "success": true
            }
        })).unwrap();
        let (app_context, _server) = setup_test_env(&mock_body, 200).await;
        let matches = get_matches_for_args(&["progress", "123"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ShowProgressPlugin.execute(&app_context, &matches, &mut writer, &mut err_writer).await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.starts_with("Test Game"));
        assert!(output.contains("No achievements found for this game"));
    }

    #[tokio::test]
    async fn test_execute_api_error() {
        let (app_context, _server) = setup_test_env("", 500).await;
        let matches = get_matches_for_args(&["progress", "123"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ShowProgressPlugin.execute(&app_context, &matches, &mut writer, &mut err_writer).await;

        let output = String::from_utf8(err_writer).unwrap();
        assert!(output.contains("Error while trying to get achievements"));
    }

    #[tokio::test]
    async fn test_execute_invalid_game_id() {
        let (app_context, _server) = setup_test_env("", 200).await;
        let matches = get_matches_for_args(&["progress", "invalid"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ShowProgressPlugin.execute(&app_context, &matches, &mut writer, &mut err_writer).await;

        let output = String::from_utf8(err_writer).unwrap();
        assert_eq!(output.trim(), "Invalid game id: invalid");
    }
}
