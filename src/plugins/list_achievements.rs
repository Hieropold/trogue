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
use std::io::Write;

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
    /// - `writer`: A mutable reference to a writer for standard output.
    /// - `err_writer`: A mutable reference to a writer for standard error.
    /// <inputs-end>
    ///
    /// <outputs-start>
    /// - None.
    /// <outputs-end>
    ///
    /// <side-effects-start>
    /// - Makes network requests to the Steam API to fetch achievement data.
    /// - Writes the list of achievements to the provided writer.
    /// <side-effects-end>
    async fn execute(
        &self,
        app_context: &AppContext,
        matches: &clap::ArgMatches,
        writer: &mut (dyn Write + Send),
        err_writer: &mut (dyn Write + Send),
    ) {
        let game_id_str = matches.get_one::<String>("game_id").unwrap();
        let add_global = matches.get_flag("global");
        let remaining = matches.get_flag("remaining");

        if let Ok(game_id) = game_id_str.parse::<u32>() {
            let mut achievements = Vec::new();

            match app_context.api.get_game_achievements(game_id).await {
                Ok((_, achs)) => achievements = achs,
                Err(e) => writeln!(err_writer, "Error while trying to get achievements: {}", e).unwrap(),
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
                    Err(e) => writeln!(err_writer, "Error while trying to get global achievements: {}", e).unwrap(),
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

                writeln!(writer, "{}", title).unwrap();
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
    use crate::steam_api::{Api, Achievement, GlobalAchievement};
    use clap::ArgMatches;

    fn create_mock_achievement(apiname: &str, name: &str, achieved: u8) -> Achievement {
        Achievement {
            apiname: apiname.to_string(),
            name: name.to_string(),
            description: "Test Description".to_string(),
            achieved,
            unlocktime: 0,
        }
    }

    fn create_mock_global_achievement(name: &str, percent: f32) -> GlobalAchievement {
        GlobalAchievement {
            name: name.to_string(),
            percent,
        }
    }

    async fn setup_test_env_game_achievements(mock_body: &str, status_code: u16) -> (AppContext, mockito::ServerGuard) {
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

    async fn setup_test_env_with_global(
        game_ach_body: &str, game_ach_status: u16,
        global_ach_body: &str, global_ach_status: u16
    ) -> (AppContext, mockito::ServerGuard) {
        let mut server = mockito::Server::new_async().await;

        server.mock("GET", "/ISteamUserStats/GetPlayerAchievements/v0001/?appid=123&key=test_key&steamid=test_id&l=en")
            .with_status(game_ach_status as usize)
            .with_header("content-type", "application/json")
            .with_body(game_ach_body)
            .create_async().await;

        server.mock("GET", "/ISteamUserStats/GetGlobalAchievementPercentagesForApp/v0002/?gameid=123&format=json&l=en")
            .with_status(global_ach_status as usize)
            .with_header("content-type", "application/json")
            .with_body(global_ach_body)
            .create_async().await;

        let api = Api::new("test_key".to_string(), "test_id".to_string(), server.url());
        let app_context = AppContext { api };
        (app_context, server)
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
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "game_id"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "global"));
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "remaining"));
    }

    #[tokio::test]
    async fn test_execute_success() {
        let achievements = vec![
            create_mock_achievement("ach1", "First Achievement", 1),
            create_mock_achievement("ach2", "Second Achievement", 0),
        ];
        let mock_body = serde_json::to_string(&serde_json::json!({
            "playerstats": {
                "steamID": "test_id",
                "gameName": "Test Game",
                "achievements": achievements,
                "success": true
            }
        })).unwrap();
        let (app_context, _server) = setup_test_env_game_achievements(&mock_body, 200).await;
        let matches = get_matches_for_args(&["achievements", "123"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListAchievementsPlugin.execute(&app_context, &matches, &mut writer, &mut err_writer).await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("First Achievement"));
        assert!(output.contains("Second Achievement"));
    }

    #[tokio::test]
    async fn test_execute_invalid_game_id() {
        let (app_context, _server) = setup_test_env_game_achievements("", 200).await;
        let matches = get_matches_for_args(&["achievements", "invalid"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListAchievementsPlugin.execute(&app_context, &matches, &mut writer, &mut err_writer).await;

        let output = String::from_utf8(err_writer).unwrap();
        assert_eq!(output.trim(), "Invalid game id: invalid");
    }

    #[tokio::test]
    async fn test_execute_get_achievements_api_error() {
        let (app_context, _server) = setup_test_env_game_achievements("", 500).await;
        let matches = get_matches_for_args(&["achievements", "123"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListAchievementsPlugin.execute(&app_context, &matches, &mut writer, &mut err_writer).await;

        let output = String::from_utf8(err_writer).unwrap();
        assert!(output.contains("Error while trying to get achievements"));
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
        let (app_context, _server) = setup_test_env_game_achievements(&mock_body, 200).await;
        let matches = get_matches_for_args(&["achievements", "123"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListAchievementsPlugin.execute(&app_context, &matches, &mut writer, &mut err_writer).await;

        let output = String::from_utf8(writer).unwrap();
        assert_eq!(output.trim(), "");
    }

    #[tokio::test]
    async fn test_execute_with_remaining_filter() {
        let achievements = vec![
            create_mock_achievement("ach1", "First Achievement", 1),
            create_mock_achievement("ach2", "Second Achievement", 0),
        ];
        let mock_body = serde_json::to_string(&serde_json::json!({
            "playerstats": {
                "steamID": "test_id",
                "gameName": "Test Game",
                "achievements": achievements,
                "success": true
            }
        })).unwrap();
        let (app_context, _server) = setup_test_env_game_achievements(&mock_body, 200).await;
        let matches = get_matches_for_args(&["achievements", "123", "--remaining"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListAchievementsPlugin.execute(&app_context, &matches, &mut writer, &mut err_writer).await;

        let output = String::from_utf8(writer).unwrap();
        assert!(!output.contains("First Achievement"));
        assert!(output.contains("Second Achievement"));
    }

    #[tokio::test]
    async fn test_execute_with_global_stats() {
        let game_achievements = vec![
            create_mock_achievement("ach1", "First Achievement", 1),
            create_mock_achievement("ach2", "Second Achievement", 0),
        ];
        let game_ach_body = serde_json::to_string(&serde_json::json!({
            "playerstats": {
                "steamID": "test_id",
                "gameName": "Test Game",
                "achievements": game_achievements,
                "success": true
            }
        })).unwrap();

        let global_achievements = vec![
            create_mock_global_achievement("ach1", 50.5),
            create_mock_global_achievement("ach2", 10.2),
        ];
        let global_ach_body = serde_json::to_string(&serde_json::json!({
            "achievementpercentages": { "achievements": global_achievements }
        })).unwrap();

        let (app_context, _server) = setup_test_env_with_global(&game_ach_body, 200, &global_ach_body, 200).await;
        let matches = get_matches_for_args(&["achievements", "123", "--global"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListAchievementsPlugin.execute(&app_context, &matches, &mut writer, &mut err_writer).await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("First Achievement"));
        assert!(output.contains("50.5%"));
        assert!(output.contains("Second Achievement"));
        assert!(output.contains("10.2%"));
    }

    #[tokio::test]
    async fn test_execute_with_global_stats_api_error() {
        let game_achievements = vec![create_mock_achievement("ach1", "First Achievement", 1)];
        let game_ach_body = serde_json::to_string(&serde_json::json!({
            "playerstats": {
                "steamID": "test_id",
                "gameName": "Test Game",
                "achievements": game_achievements,
                "success": true
            }
        })).unwrap();

        let (app_context, _server) = setup_test_env_with_global(&game_ach_body, 200, "", 500).await;
        let matches = get_matches_for_args(&["achievements", "123", "--global"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        ListAchievementsPlugin.execute(&app_context, &matches, &mut writer, &mut err_writer).await;

        let err_output = String::from_utf8(err_writer).unwrap();
        assert!(err_output.contains("Error while trying to get global achievements"));

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("First Achievement"));
    }
}