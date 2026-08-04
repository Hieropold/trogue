//! Plugin for displaying a dashboard of recently played games.
//!
//! <purpose-start>
//! This plugin provides the `dashboard` command, which shows the 10 most recently played games
//! and their achievement progress.
//! <purpose-end>
//!
//! <inputs-start>
//! - `steam`: The `GameLibrary` seam, providing access to owned games and achievement data.
//! - `_matches`: The command-line arguments parsed by `clap` (unused in this plugin).
//! <inputs-end>
//!
//! <outputs-start>
//! - A dashboard of recently played games printed to the console.
//! <outputs-end>
//!
//! <side-effects-start>
//! - Makes multiple network requests to platform APIs to fetch game lists and achievement data.
//! <side-effects-end>

use crate::game_library::GameLibrary;
use crate::plugins::Plugin;
use async_trait::async_trait;
use clap::Command;
use std::io::Write;

pub struct DashboardPlugin;

#[async_trait]
impl Plugin for DashboardPlugin {
    // Defines the clap command for the `dashboard` plugin.
    //
    // <purpose-start>
    // This method provides the command-line interface for the `dashboard` plugin,
    // which displays a dashboard of recently played games.
    // <purpose-end>
    //
    // <inputs-start>
    // - `&self`: A reference to the plugin instance.
    // <inputs-end>
    //
    // <outputs-start>
    // - `clap::Command`: The clap command definition for the `dashboard` plugin.
    // <outputs-end>
    //
    // <side-effects-start>
    // - None.
    // <side-effects-end>
    fn command(&self) -> Command {
        Command::new("dashboard")
            .about("Displays a dashboard with 10 last played games and their achievement progress")
    }

    // Executes the `dashboard` plugin's logic.
    //
    // <purpose-start>
    // This method is called by the core application when the `dashboard` command is invoked.
    // It fetches the list of recently played games and their achievement progress, and prints the dashboard to the console.
    // <purpose-end>
    //
    // <inputs-start>
    // - `&self`: A reference to the plugin instance.
    // - `steam`: The `GameLibrary` seam.
    // - `_matches`: The clap argument matches for the `dashboard` subcommand (unused).
    // - `writer`: A mutable reference to a writer for standard output.
    // - `err_writer`: A mutable reference to a writer for standard error.
    // <inputs-end>
    //
    // <outputs-start>
    // - None.
    // <outputs-end>
    //
    // <side-effects-start>
    // - Makes multiple network requests to platform APIs to fetch game and achievement data.
    // - Writes the dashboard to the provided writer.
    // <side-effects-end>
    async fn execute(
        &self,
        steam: &dyn GameLibrary,
        _matches: &clap::ArgMatches,
        writer: &mut (dyn Write + Send),
        err_writer: &mut (dyn Write + Send),
    ) {
        let mut games = Vec::new();
        match steam.owned_games().await {
            Ok(resp) => games = resp,
            Err(e) => writeln!(err_writer, "{}", e).unwrap(),
        }

        // Sort games by last played time (most recent first)
        games.sort_by_key(|g| std::cmp::Reverse(g.rtime_last_played));

        // Take only the 10 most recently played games
        let recent_games: Vec<_> = games.iter().take(10).collect();

        // Output title
        let terminal_width = ratatui::crossterm::terminal::size().unwrap_or((80, 24)).0 as usize;
        let box_width = terminal_width / 2;
        let title = "Recently Played Games Dashboard";
        let padding = (box_width - title.len()) / 2;

        writeln!(writer, "{}", "=".repeat(box_width)).unwrap();
        writeln!(
            writer,
            "{}{}{}",
            " ".repeat(padding),
            title,
            " ".repeat(padding)
        )
        .unwrap();
        writeln!(writer, "{}", "=".repeat(box_width)).unwrap();

        for game in recent_games {
            let achievements = match steam.achievements(&game.id).await {
                Ok(set) => {
                    writeln!(writer, "{}", set.game_name).unwrap();
                    set.achievements
                }
                Err(e) => {
                    writeln!(err_writer, "{}", e).unwrap();
                    continue;
                }
            };

            if achievements.is_empty() {
                writeln!(writer, "No achievements found for this game").unwrap();
                continue;
            }

            let total = achievements.len();
            let completed = achievements.iter().filter(|a| a.achieved > 0).count();
            let percentage = (completed as f32 / total as f32) * 100.0;

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
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_library::fake::FakeLibrary;
    use crate::game_library::{Achievement, AchievementSet, Game, GameId, Platform, PlatformError};
    use clap::ArgMatches;

    fn create_mock_game(appid: u32, name: &str, rtime_last_played: u64) -> Game {
        Game {
            id: GameId::Steam(appid),
            platform: Platform::Steam,
            name: name.to_string(),
            playtime_forever: Some(0),
            img_icon_url: None,
            rtime_last_played,
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
            grade: None,
        }
    }

    fn get_matches_for_args(args: &[&str]) -> ArgMatches {
        DashboardPlugin.command().get_matches_from(args)
    }

    #[test]
    fn test_command() {
        let plugin = DashboardPlugin;
        let cmd = plugin.command();
        assert_eq!(cmd.get_name(), "dashboard");
        assert!(cmd.get_about().is_some());
    }

    #[tokio::test]
    async fn test_execute_success() {
        let steam = FakeLibrary::new()
            .with_games(vec![
                create_mock_game(1, "Game 1", 100),
                create_mock_game(2, "Game 2", 200),
            ])
            .with_achievements(
                1,
                AchievementSet {
                    game_name: "Game 1".to_string(),
                    achievements: vec![create_mock_achievement(1), create_mock_achievement(1)],
                },
            )
            .with_achievements(
                2,
                AchievementSet {
                    game_name: "Game 2".to_string(),
                    achievements: vec![create_mock_achievement(1), create_mock_achievement(0)],
                },
            );
        let matches = get_matches_for_args(&["dashboard"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        DashboardPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Recently Played Games Dashboard"));
        assert!(output.contains("Game 1"));
        assert!(output.contains("100.0% (2/2)"));
        assert!(output.contains("Game 2"));
        assert!(output.contains("50.0% (1/2)"));
    }

    #[tokio::test]
    async fn test_execute_get_games_list_api_error() {
        let steam = FakeLibrary::new().with_games_error(PlatformError::Http {
            status: Some(500),
            msg: "boom".to_string(),
        });
        let matches = get_matches_for_args(&["dashboard"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        DashboardPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let err_output = String::from_utf8(err_writer).unwrap();
        assert!(err_output.contains("API request failed"));
    }

    #[tokio::test]
    async fn test_execute_get_game_achievements_api_error() {
        let steam = FakeLibrary::new().with_games(vec![create_mock_game(1, "Game 1", 100)]);
        // No achievements registered for appid 1 -> FakeLibrary returns NoStats.
        let matches = get_matches_for_args(&["dashboard"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        DashboardPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let err_output = String::from_utf8(err_writer).unwrap();
        assert!(err_output.contains("no achievement stats"));
    }

    #[tokio::test]
    async fn test_execute_no_games() {
        let steam = FakeLibrary::new().with_games(vec![]);
        let matches = get_matches_for_args(&["dashboard"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        DashboardPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Recently Played Games Dashboard"));
        assert!(!output.contains("[")); // No progress bars
    }

    #[tokio::test]
    async fn test_execute_game_with_no_achievements() {
        let steam = FakeLibrary::new()
            .with_games(vec![create_mock_game(1, "Game 1", 100)])
            .with_achievements(
                1,
                AchievementSet {
                    game_name: "Game 1".to_string(),
                    achievements: vec![],
                },
            );
        let matches = get_matches_for_args(&["dashboard"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        DashboardPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Game 1"));
        assert!(output.contains("No achievements found for this game"));
    }
}
