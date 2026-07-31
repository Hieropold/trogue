//! Plugin for showing the achievement progress for a specific game.
//!
//! <purpose-start>
//! This plugin provides the `progress` command, which displays a progress bar
//! representing the achievement completion for a given game.
//! <purpose-end>
//!
//! <inputs-start>
//! - `steam`: The Steam client seam, providing game resolution and achievement data.
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

use crate::{
    plugins::Plugin,
    steam_client::{GameMatch, SteamClient},
};
use async_trait::async_trait;
use clap::{Arg, Command};
use std::io::Write;

pub struct ShowProgressPlugin;

#[async_trait]
impl Plugin for ShowProgressPlugin {
    // Defines the clap command for the `progress` plugin.
    //
    // <purpose-start>
    // This method provides the command-line interface for the `progress` plugin,
    // which displays the achievement progress for a specific game.
    // <purpose-end>
    //
    // <inputs-start>
    // - `&self`: A reference to the plugin instance.
    // <inputs-end>
    //
    // <outputs-start>
    // - `clap::Command`: The clap command definition for the `progress` plugin.
    // <outputs-end>
    //
    // <side-effects-start>
    // - None.
    // <side-effects-end>
    fn command(&self) -> Command {
        Command::new("progress")
            .about("Displays game achievements progress.")
            .arg(
                Arg::new("game_id")
                    .value_name("game_id")
                    .action(clap::ArgAction::Set)
                    .required(true)
                    .help("The ID of the game or part of game title to show progress for"),
            )
    }

    // Executes the `progress` plugin's logic.
    //
    // <purpose-start>
    // This method is called by the core application when the `progress` command is invoked.
    // It resolves the game (by id or name), fetches its achievement data, and displays a
    // progress bar in the console.
    // <purpose-end>
    //
    // <inputs-start>
    // - `&self`: A reference to the plugin instance.
    // - `steam`: The Steam client seam.
    // - `matches`: The clap argument matches for the `progress` subcommand.
    // - `writer`: A mutable reference to a writer for standard output.
    // - `err_writer`: A mutable reference to a writer for standard error.
    // <inputs-end>
    //
    // <outputs-start>
    // - None.
    // <outputs-end>
    //
    // <side-effects-start>
    // - Makes a network request to the Steam API to fetch achievement data.
    // - Writes the progress bar to the provided writer.
    // <side-effects-end>
    async fn execute(
        &self,
        steam: &dyn SteamClient,
        matches: &clap::ArgMatches,
        writer: &mut (dyn Write + Send),
        err_writer: &mut (dyn Write + Send),
    ) {
        let game_arg = matches.get_one::<String>("game_id").unwrap();

        let game_id = match steam.resolve(game_arg).await {
            Ok(GameMatch::One(game)) => game.appid,
            Ok(GameMatch::None) => {
                writeln!(err_writer, "Game not found: {}", game_arg).unwrap();
                return;
            }
            Ok(GameMatch::Many(games)) => {
                writeln!(writer, "Multiple games match '{}':", game_arg).unwrap();
                for m in games {
                    writeln!(writer, " - {}", m.name).unwrap();
                }
                return;
            }
            Err(e) => {
                writeln!(err_writer, "{}", e).unwrap();
                return;
            }
        };

        match steam.achievements(game_id).await {
            Ok(set) => {
                writeln!(writer, "{}", set.game_name).unwrap();

                if set.achievements.is_empty() {
                    writeln!(writer, "No achievements found for this game").unwrap();
                    return;
                }

                let total = set.achievements.len();
                let completed = set.achievements.iter().filter(|a| a.achieved > 0).count();
                let percentage = (completed as f32 / total as f32) * 100.0;

                let terminal_width =
                    ratatui::crossterm::terminal::size().unwrap_or((80, 24)).0 as usize;
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
            Err(e) => {
                writeln!(err_writer, "{}", e).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::steam_client::fake::FakeSteam;
    use crate::steam_client::{Achievement, AchievementSet, Game};
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

    fn create_mock_achievement(achieved: u8) -> Achievement {
        Achievement {
            apiname: "test_api".to_string(),
            name: "Test Achievement".to_string(),
            description: "Test Description".to_string(),
            achieved,
            unlocktime: 0,
            global_percent: None,
        }
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
        let steam = FakeSteam::new()
            .with_games(vec![create_mock_game(123, "Test Game")])
            .with_achievements(
                123,
                AchievementSet {
                    game_name: "Test Game".to_string(),
                    achievements: vec![create_mock_achievement(1), create_mock_achievement(0)],
                },
            );
        let matches = get_matches_for_args(&["progress", "123"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ShowProgressPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.starts_with("Test Game"));
        assert!(output.contains("50.0% (1/2)"));
    }

    #[tokio::test]
    async fn test_execute_by_name() {
        let steam = FakeSteam::new()
            .with_games(vec![create_mock_game(123, "Specific Game Title")])
            .with_achievements(
                123,
                AchievementSet {
                    game_name: "Specific Game Title".to_string(),
                    achievements: vec![create_mock_achievement(1)],
                },
            );
        let matches = get_matches_for_args(&["progress", "specific"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ShowProgressPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.starts_with("Specific Game Title"));
        assert!(output.contains("100.0% (1/1)"));
    }

    #[tokio::test]
    async fn test_execute_no_achievements() {
        let steam = FakeSteam::new()
            .with_games(vec![create_mock_game(123, "Test Game")])
            .with_achievements(
                123,
                AchievementSet {
                    game_name: "Test Game".to_string(),
                    achievements: vec![],
                },
            );
        let matches = get_matches_for_args(&["progress", "123"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ShowProgressPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.starts_with("Test Game"));
        assert!(output.contains("No achievements found for this game"));
    }

    #[tokio::test]
    async fn test_execute_api_error() {
        let steam = FakeSteam::new().with_games(vec![create_mock_game(123, "Test Game")]);
        // No achievements registered -> FakeSteam returns NoStats.
        let matches = get_matches_for_args(&["progress", "123"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ShowProgressPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(err_writer).unwrap();
        assert!(output.contains("no achievement stats"));
    }

    #[tokio::test]
    async fn test_execute_game_not_found() {
        let steam = FakeSteam::new().with_games(vec![]);
        let matches = get_matches_for_args(&["progress", "invalid"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ShowProgressPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(err_writer).unwrap();
        assert_eq!(output.trim(), "Game not found: invalid");
    }
}
