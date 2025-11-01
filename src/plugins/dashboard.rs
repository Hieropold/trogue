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
    // - `app_context`: The shared application context.
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
    // - Makes multiple network requests to the Steam API to fetch game and achievement data.
    // - Writes the dashboard to the provided writer.
    // <side-effects-end>
    async fn execute(
        &self,
        app_context: &AppContext,
        _matches: &clap::ArgMatches,
        writer: &mut (dyn Write + Send),
        err_writer: &mut (dyn Write + Send),
    ) {
        let mut games = Vec::new();
        match app_context.api.get_games_list().await {
            Ok(resp) => games = resp,
            Err(e) => writeln!(err_writer, "Error while trying to get Steam data: {}", e).unwrap(),
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

        writeln!(writer, "{}", "=".repeat(box_width)).unwrap();
        writeln!(writer, "{}{}{}", " ".repeat(padding), title, " ".repeat(padding)).unwrap();
        writeln!(writer, "{}", "=".repeat(box_width)).unwrap();

        for game in recent_games {
            let mut achievements = Vec::new();
            let mut game_name = String::new();

            match app_context.api.get_game_achievements(game.appid).await {
                Ok((name, achs)) => {
                    game_name = name;
                    achievements = achs;
                }
                Err(e) => writeln!(err_writer, "Error while trying to get achievements: {}", e).unwrap(),
            }

            writeln!(writer, "{}", game_name).unwrap();

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
    use crate::app::AppContext;
    use crate::steam_api::{Api, Achievement, Game};
    use clap::ArgMatches;

    fn create_mock_game(appid: u32, name: &str, rtime_last_played: u64) -> Game {
        Game {
            appid,
            name: name.to_string(),
            playtime_forever: 0,
            img_icon_url: "".to_string(),
            playtime_windows_forever: 0,
            playtime_mac_forever: 0,
            playtime_linux_forever: 0,
            rtime_last_played,
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
        }
    }

    struct MockGameAchievements {
        appid: u32,
        body: String,
        status: u16,
    }

    async fn setup_test_env(
        games_list_body: &str,
        games_list_status: u16,
        achievements_mocks: &[MockGameAchievements],
    ) -> (AppContext, mockito::ServerGuard) {
        let mut server = mockito::Server::new_async().await;

        server.mock("GET", "/IPlayerService/GetOwnedGames/v0001/?key=test_key&steamid=test_id&format=json&include_appinfo=1")
            .with_status(games_list_status as usize)
            .with_header("content-type", "application/json")
            .with_body(games_list_body)
            .create_async().await;

        for mock in achievements_mocks {
            let url = format!("/ISteamUserStats/GetPlayerAchievements/v0001/?appid={}&key=test_key&steamid=test_id&l=en", mock.appid);
            server.mock("GET", url.as_str())
                .with_status(mock.status as usize)
                .with_header("content-type", "application/json")
                .with_body(&mock.body)
                .create_async().await;
        }

        let api = Api::new("test_key".to_string(), "test_id".to_string(), server.url());
        let app_context = AppContext { api };
        (app_context, server)
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
        let games = vec![
            create_mock_game(1, "Game 1", 100),
            create_mock_game(2, "Game 2", 200),
        ];
        let games_list_body = serde_json::to_string(&serde_json::json!({
            "response": { "game_count": 2, "games": games }
        })).unwrap();

        let achievements1 = vec![create_mock_achievement(1), create_mock_achievement(0)];
        let achievements_body1 = serde_json::to_string(&serde_json::json!({
            "playerstats": { "steamID": "test_id", "gameName": "Game 2", "achievements": achievements1, "success": true }
        })).unwrap();

        let achievements2 = vec![create_mock_achievement(1), create_mock_achievement(1)];
        let achievements_body2 = serde_json::to_string(&serde_json::json!({
            "playerstats": { "steamID": "test_id", "gameName": "Game 1", "achievements": achievements2, "success": true }
        })).unwrap();

        let achievements_mocks = vec![
            MockGameAchievements { appid: 2, body: achievements_body1, status: 200 },
            MockGameAchievements { appid: 1, body: achievements_body2, status: 200 },
        ];

        let (app_context, _server) = setup_test_env(&games_list_body, 200, &achievements_mocks).await;
        let matches = get_matches_for_args(&["dashboard"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        DashboardPlugin.execute(&app_context, &matches, &mut writer, &mut err_writer).await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Recently Played Games Dashboard"));
        assert!(output.contains("Game 1"));
        assert!(output.contains("100.0% (2/2)"));
        assert!(output.contains("Game 2"));
        assert!(output.contains("50.0% (1/2)"));
    }

    #[tokio::test]
    async fn test_execute_get_games_list_api_error() {
        let (app_context, _server) = setup_test_env("", 500, &[]).await;
        let matches = get_matches_for_args(&["dashboard"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        DashboardPlugin.execute(&app_context, &matches, &mut writer, &mut err_writer).await;

        let err_output = String::from_utf8(err_writer).unwrap();
        assert!(err_output.contains("Error while trying to get Steam data"));
    }

    #[tokio::test]
    async fn test_execute_get_game_achievements_api_error() {
        let games = vec![create_mock_game(1, "Game 1", 100)];
        let games_list_body = serde_json::to_string(&serde_json::json!({
            "response": { "game_count": 1, "games": games }
        })).unwrap();

        let achievements_mocks = vec![
            MockGameAchievements { appid: 1, body: "".to_string(), status: 500 },
        ];

        let (app_context, _server) = setup_test_env(&games_list_body, 200, &achievements_mocks).await;
        let matches = get_matches_for_args(&["dashboard"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        DashboardPlugin.execute(&app_context, &matches, &mut writer, &mut err_writer).await;

        let err_output = String::from_utf8(err_writer).unwrap();
        assert!(err_output.contains("Error while trying to get achievements"));
    }

    #[tokio::test]
    async fn test_execute_no_games() {
        let games_list_body = serde_json::to_string(&serde_json::json!({
            "response": { "game_count": 0, "games": [] }
        })).unwrap();

        let (app_context, _server) = setup_test_env(&games_list_body, 200, &[]).await;
        let matches = get_matches_for_args(&["dashboard"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        DashboardPlugin.execute(&app_context, &matches, &mut writer, &mut err_writer).await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Recently Played Games Dashboard"));
        assert!(!output.contains("[")); // No progress bars
    }

    #[tokio::test]
    async fn test_execute_game_with_no_achievements() {
        let games = vec![create_mock_game(1, "Game 1", 100)];
        let games_list_body = serde_json::to_string(&serde_json::json!({
            "response": { "game_count": 1, "games": games }
        })).unwrap();

        let achievements_body = serde_json::to_string(&serde_json::json!({
            "playerstats": { "steamID": "test_id", "gameName": "Game 1", "achievements": [], "success": true }
        })).unwrap();

        let achievements_mocks = vec![
            MockGameAchievements { appid: 1, body: achievements_body, status: 200 },
        ];

        let (app_context, _server) = setup_test_env(&games_list_body, 200, &achievements_mocks).await;
        let matches = get_matches_for_args(&["dashboard"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        DashboardPlugin.execute(&app_context, &matches, &mut writer, &mut err_writer).await;

        let output = String::from_utf8(writer).unwrap();
        assert!(output.contains("Game 1"));
        assert!(output.contains("No achievements found for this game"));
    }
}