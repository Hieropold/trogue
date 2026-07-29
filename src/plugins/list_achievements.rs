//! Plugin for listing achievements for a specific game.
//!
//! <purpose-start>
//! This plugin provides the `achievements` command, which allows users to list the achievements for a given game.
//! It supports filtering by achieved status and can include global achievement percentages.
//! <purpose-end>
//!
//! <inputs-start>
//! - `steam`: The Steam client seam, providing game resolution and achievement data.
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

use crate::{
    plugins::Plugin,
    steam_client::{GameMatch, SteamClient},
    ui,
};
use async_trait::async_trait;
use clap::{Arg, Command};
use std::io::Write;

pub struct ListAchievementsPlugin;

#[async_trait]
impl Plugin for ListAchievementsPlugin {
    // Defines the clap command for the `achievements` plugin.
    //
    // <purpose-start>
    // This method provides the command-line interface for the `achievements` plugin,
    // which allows users to list achievements for a specific game.
    // <purpose-end>
    //
    // <inputs-start>
    // - `&self`: A reference to the plugin instance.
    // <inputs-end>
    //
    // <outputs-start>
    // - `clap::Command`: The clap command definition for the `achievements` plugin.
    // <outputs-end>
    //
    // <side-effects-start>
    // - None.
    // <side-effects-end>
    fn command(&self) -> Command {
        Command::new("achievements")
            .about("Displays achievements for a specific game. Game ID or part of game title should be provided as an argument")
            .arg(
                Arg::new("game")
                    .value_name("game")
                    .action(clap::ArgAction::Set)
                    .required(true)
                    .help("The ID of the game or part of game title to list achievements for"),
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

    // Executes the `achievements` plugin's logic.
    //
    // <purpose-start>
    // This method is called by the core application when the `achievements` command is invoked.
    // It resolves the game, fetches its achievements (optionally enriched with global
    // percentages), applies any specified filters, and prints the list to the console.
    // <purpose-end>
    //
    // <inputs-start>
    // - `&self`: A reference to the plugin instance.
    // - `steam`: The Steam client seam.
    // - `matches`: The clap argument matches for the `achievements` subcommand.
    // - `writer`: A mutable reference to a writer for standard output.
    // - `err_writer`: A mutable reference to a writer for standard error.
    // <inputs-end>
    //
    // <outputs-start>
    // - None.
    // <outputs-end>
    //
    // <side-effects-start>
    // - Makes network requests to the Steam API to fetch achievement data.
    // - Writes the list of achievements to the provided writer.
    // <side-effects-end>
    async fn execute(
        &self,
        steam: &dyn SteamClient,
        matches: &clap::ArgMatches,
        writer: &mut (dyn Write + Send),
        err_writer: &mut (dyn Write + Send),
    ) {
        let game_arg = matches.get_one::<String>("game").unwrap();
        let add_global = matches.get_flag("global");
        let remaining = matches.get_flag("remaining");

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

        let achievement_set = if add_global {
            steam.achievements_with_global(game_id).await
        } else {
            steam.achievements(game_id).await
        };

        let achievements = match achievement_set {
            Ok(set) => set.achievements,
            Err(e) => {
                writeln!(err_writer, "{}", e).unwrap();
                return;
            }
        };

        for achievement in achievements {
            if remaining && achievement.achieved > 0 {
                continue;
            }

            let global_percent = achievement.global_percent;
            let displayable_achievement = ui::DisplayableAchievement { achievement };

            let mut title: String;
            if displayable_achievement.achievement.achieved > 0 {
                title = displayable_achievement.format("n - s (t)");
            } else {
                title = displayable_achievement.format("n");
            }

            if add_global {
                title.push_str(&format!(" {}%", global_percent.unwrap_or(0.0)));
            }

            writeln!(writer, "{}", title).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::steam_client::fake::FakeSteam;
    use crate::steam_client::{Achievement, AchievementSet, Game, GlobalAchievement};
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

    fn create_mock_achievement(apiname: &str, name: &str, achieved: u8) -> Achievement {
        Achievement {
            apiname: apiname.to_string(),
            name: name.to_string(),
            description: "Test Description".to_string(),
            achieved,
            unlocktime: 0,
            global_percent: None,
        }
    }

    fn get_matches_for_args(args: &[&str]) -> ArgMatches {
        ListAchievementsPlugin.command().get_matches_from(args)
    }

    #[test]
    fn test_command() {
        let plugin = ListAchievementsPlugin;
        let cmd = plugin.command();
        assert_eq!(cmd.get_name(), "achievements");
        assert!(cmd.get_about().is_some());
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "game"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "global"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "remaining"));
    }

    #[tokio::test]
    async fn test_execute_success() {
        let steam = FakeSteam::new()
            .with_games(vec![create_mock_game(123, "Test Game")])
            .with_achievements(
                123,
                AchievementSet {
                    game_name: "Test Game".to_string(),
                    achievements: vec![
                        create_mock_achievement("ach1", "First Achievement", 1),
                        create_mock_achievement("ach2", "Second Achievement", 0),
                    ],
                },
            );
        let matches = get_matches_for_args(&["achievements", "123"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListAchievementsPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("First Achievement"));
        assert!(output.contains("Second Achievement"));
    }

    #[tokio::test]
    async fn test_execute_game_not_found() {
        let steam = FakeSteam::new().with_games(vec![]);
        let matches = get_matches_for_args(&["achievements", "unknown"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListAchievementsPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(err_writer).unwrap();
        assert!(output.contains("Game not found: unknown"));
    }

    #[tokio::test]
    async fn test_execute_get_achievements_api_error() {
        let steam = FakeSteam::new().with_games(vec![create_mock_game(123, "Test Game")]);
        // No achievements registered for appid 123 -> FakeSteam returns NoStats.
        let matches = get_matches_for_args(&["achievements", "123"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListAchievementsPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(err_writer).unwrap();
        assert!(output.contains("no achievement stats"));
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
        let matches = get_matches_for_args(&["achievements", "123"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListAchievementsPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(writer).unwrap();
        assert_eq!(output.trim(), "");
    }

    #[tokio::test]
    async fn test_execute_with_remaining_filter() {
        let steam = FakeSteam::new()
            .with_games(vec![create_mock_game(123, "Test Game")])
            .with_achievements(
                123,
                AchievementSet {
                    game_name: "Test Game".to_string(),
                    achievements: vec![
                        create_mock_achievement("ach1", "First Achievement", 1),
                        create_mock_achievement("ach2", "Second Achievement", 0),
                    ],
                },
            );
        let matches = get_matches_for_args(&["achievements", "123", "--remaining"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListAchievementsPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(writer).unwrap();
        assert!(!output.contains("First Achievement"));
        assert!(output.contains("Second Achievement"));
    }

    #[tokio::test]
    async fn test_execute_with_global_stats() {
        let steam = FakeSteam::new()
            .with_games(vec![create_mock_game(123, "Test Game")])
            .with_achievements(
                123,
                AchievementSet {
                    game_name: "Test Game".to_string(),
                    achievements: vec![
                        create_mock_achievement("ach1", "First Achievement", 1),
                        create_mock_achievement("ach2", "Second Achievement", 0),
                    ],
                },
            )
            .with_global(
                123,
                vec![
                    GlobalAchievement {
                        name: "ach1".to_string(),
                        percent: 50.5,
                    },
                    GlobalAchievement {
                        name: "ach2".to_string(),
                        percent: 10.2,
                    },
                ],
            );
        let matches = get_matches_for_args(&["achievements", "123", "--global"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListAchievementsPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("First Achievement"));
        assert!(output.contains("50.5%"));
        assert!(output.contains("Second Achievement"));
        assert!(output.contains("10.2%"));
    }

    #[tokio::test]
    async fn test_execute_with_global_stats_fetch_failure_still_shows_achievements() {
        let steam = FakeSteam::new()
            .with_games(vec![create_mock_game(123, "Test Game")])
            .with_achievements(
                123,
                AchievementSet {
                    game_name: "Test Game".to_string(),
                    achievements: vec![create_mock_achievement("ach1", "First Achievement", 1)],
                },
            )
            .with_global_error(
                123,
                crate::steam_client::SteamError::Http {
                    status: Some(500),
                    msg: "boom".to_string(),
                },
            );
        let matches = get_matches_for_args(&["achievements", "123", "--global"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListAchievementsPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        // Best-effort enrichment: a failed global-percentage fetch does not
        // fail the whole command, it just leaves achievements unenriched.
        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("First Achievement"));
        assert!(output.contains(" 0%"));
    }

    #[tokio::test]
    async fn test_execute_substring_success() {
        let steam = FakeSteam::new()
            .with_games(vec![create_mock_game(123, "Specific Game Title")])
            .with_achievements(
                123,
                AchievementSet {
                    game_name: "Specific Game Title".to_string(),
                    achievements: vec![create_mock_achievement("ach1", "Achievement 1", 1)],
                },
            );
        let matches = get_matches_for_args(&["achievements", "specific"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListAchievementsPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Achievement 1"));
    }

    #[tokio::test]
    async fn test_execute_multiple_matches() {
        let steam = FakeSteam::new().with_games(vec![
            create_mock_game(123, "Game One"),
            create_mock_game(456, "Game Two"),
        ]);
        let matches = get_matches_for_args(&["achievements", "Game"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListAchievementsPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Multiple games match 'Game':"));
        assert!(output.contains("Game One"));
        assert!(output.contains("Game Two"));
    }

    #[tokio::test]
    async fn test_execute_numeric_id_not_in_library_fallback() {
        let steam = FakeSteam::new()
            .with_games(vec![create_mock_game(456, "Game 123")])
            .with_achievements(
                456,
                AchievementSet {
                    game_name: "Game 123".to_string(),
                    achievements: vec![create_mock_achievement(
                        "ach1",
                        "Achievement from fallback",
                        1,
                    )],
                },
            );
        let matches = get_matches_for_args(&["achievements", "123"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListAchievementsPlugin
            .execute(&steam, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Achievement from fallback"));
    }
}
