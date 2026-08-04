// Test code legitimately reaches for `unwrap`/`expect`/`panic!` to fail fast
// on unexpected fixtures; the warn-level lints in Cargo.toml's [lints.clippy]
// exist to catch those same calls in production code paths, so exempt tests.
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod game_library;
pub mod multi_source;
pub mod plugins;
pub mod steam_api;
pub mod ui;

use clap::Command;
use multi_source::MultiSource;
use std::io::{stderr, stdout};
use std::process;
use steam_api::HttpSteamClient;

// The main entry point of the application.
//
// <purpose-start>
// This function is the main entry point of the application. It parses command-line arguments
// and dispatches execution to plugins, initializing game library sources after argument parsing.
// <purpose-end>
//
// <inputs-start>
// - None.
// <inputs-end>
//
// <outputs-start>
// - None.
// <outputs-end>
//
// <side-effects-start>
// - **Prints to the console**: The output of the commands is printed to standard output or error.
// - **Exits the process**: The process is terminated when the command finishes or fails.
// <side-effects-end>
#[tokio::main]
async fn main() {
    let plugins = plugins::get_plugins();

    let mut command = Command::new("trogue")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Hieropold <hieropold@gmail.com>")
        .about("A CLI tool for displaying game achievements");

    for plugin in &plugins {
        command = command.subcommand(plugin.command());
    }

    let matches = command.get_matches();

    // If subcommand is `completions`, run it directly before credential loading
    // so shell completions can be generated without environment variables.
    if let Some(("completions", sub_matches)) = matches.subcommand() {
        for plugin in &plugins {
            if plugin.command().get_name() == "completions" {
                let empty_library = MultiSource::new(vec![]);
                plugin
                    .execute(&empty_library, sub_matches, &mut stdout(), &mut stderr())
                    .await;
                return;
            }
        }
    }

    // Build platform sources after clap parsing.
    let sources: Vec<Box<dyn game_library::GameLibrary>> = match HttpSteamClient::from_env() {
        Ok(client) => vec![Box::new(client)],
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };
    let library: std::sync::Arc<dyn game_library::GameLibrary> =
        std::sync::Arc::new(MultiSource::new(sources));

    if matches.subcommand().is_none() {
        use std::io::IsTerminal;
        if stdout().is_terminal() {
            plugins::interactive::run(library.clone()).await;
            return;
        } else {
            eprintln!("No subcommand provided. See --help.");
            process::exit(1);
        }
    }

    if let Some(("interactive", _)) = matches.subcommand() {
        plugins::interactive::run(library.clone()).await;
        return;
    }

    for plugin in &plugins {
        if let Some(sub_matches) = matches.subcommand_matches(plugin.command().get_name()) {
            match plugin.execute_deep(&*library, sub_matches).await {
                Ok(ui::ViewData::None) => {
                    // Fallback to legacy execute if not implemented
                    plugin
                        .execute(&*library, sub_matches, &mut stdout(), &mut stderr())
                        .await;
                }
                Ok(view_data) => {
                    if let Err(e) = ui::Renderer::render(view_data, &mut stdout()) {
                        eprintln!("Error rendering output: {}", e);
                    }
                }
                Err(e) => {
                    eprintln!("{}", e);
                }
            }
            return;
        }
    }
}
