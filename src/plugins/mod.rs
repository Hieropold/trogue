//! Manages the plugin system for the application.
//!
//! <purpose-start>
//! This module defines the `Plugin` trait, which provides a common interface for all plugins.
//! It also includes a registration function to collect and provide all available plugins to the application's core.
//! This approach allows for a modular and extensible architecture where features can be added or removed
//! by simply creating or deleting a plugin file.
//! <purpose-end>
//!
//! <inputs-start>
//! - None
//! <inputs-end>
//!
//! <outputs-start>
//! - A vector of `Box<dyn Plugin>`, allowing the core application to interact with all plugins through a common interface.
//! <outputs-end>
//!
//! <side-effects-start>
//! - None
//! <side-effects-end>

use crate::game_library::GameLibrary;
use async_trait::async_trait;
use std::io::Write;

pub mod completions;
pub mod dashboard;
pub mod interactive;
pub mod list_achievements;
pub mod list_games;
pub mod show_progress;

#[async_trait]
pub trait Plugin: Send + Sync {
    // Defines the clap command for the plugin.
    //
    // <purpose-start>
    // This method provides the command-line interface for the plugin,
    // which will be integrated into the main application's CLI.
    // <purpose-end>
    //
    // <inputs-start>
    // - `&self`: A reference to the plugin instance.
    // <inputs-end>
    //
    // <outputs-start>
    // - `clap::Command`: The clap command definition for the plugin.
    // <outputs-end>
    //
    // <side-effects-start>
    // - None.
    // <side-effects-end>
    fn command(&self) -> clap::Command;

    // Executes the plugin's logic.
    //
    // <purpose-start>
    // This method is called by the core application when the plugin's command is invoked.
    // It contains the main logic for the feature provided by the plugin.
    // <purpose-end>
    //
    // <inputs-start>
    // - `&self`: A reference to the plugin instance.
    // - `steam`: The domain-level game library seam (HTTP in production, fake in tests).
    // - `matches`: The clap argument matches for the subcommand.
    // - `writer`: A mutable reference to a writer for standard output.
    // - `err_writer`: A mutable reference to a writer for standard error.
    // <inputs-end>
    //
    // <outputs-start>
    // - None.
    // <outputs-end>
    //
    // <side-effects-start>
    // - Varies by plugin, but can include network requests, file I/O, or writing to the provided writers.
    // <side-effects-end>
    async fn execute(
        &self,
        steam: &dyn GameLibrary,
        matches: &clap::ArgMatches,
        writer: &mut (dyn Write + Send),
        err_writer: &mut (dyn Write + Send),
    );

    // Deepened execute that returns structured domain data (ViewData).
    // This allows decoupling CLI parsing and terminal rendering from the plugin.
    async fn execute_deep(
        &self,
        _steam: &dyn GameLibrary,
        _matches: &clap::ArgMatches,
    ) -> Result<crate::ui::ViewData, String> {
        Ok(crate::ui::ViewData::None)
    }
}

pub fn get_plugins() -> Vec<Box<dyn Plugin>> {
    vec![
        Box::new(list_games::ListGamesPlugin),
        Box::new(dashboard::DashboardPlugin),
        Box::new(list_achievements::ListAchievementsPlugin),
        Box::new(show_progress::ShowProgressPlugin),
        Box::new(completions::CompletionsPlugin),
        Box::new(interactive::InteractivePlugin),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests the `get_plugins` function.
    //
    // <purpose-start>
    // This test verifies that the `get_plugins` function correctly registers and returns all available plugins.
    // It ensures that the plugin discovery mechanism is working as expected and that all core features are included.
    // <purpose-end>
    //
    // <inputs-start>
    // - None
    // <inputs-end>
    //
    // <outputs-start>
    // - None
    // <outputs-end>
    //
    // <side-effects-start>
    // - None
    // <side-effects-end>
    #[test]
    fn test_get_plugins() {
        let plugins = get_plugins();

        // Expected number of plugins.
        assert_eq!(plugins.len(), 6);

        let mut expected_names = vec![
            "list",
            "dashboard",
            "achievements",
            "progress",
            "completions",
            "interactive",
        ];
        expected_names.sort();

        let mut actual_names: Vec<String> = plugins
            .iter()
            .map(|p| p.command().get_name().to_string())
            .collect();
        actual_names.sort();

        assert_eq!(actual_names, expected_names);
    }
}
