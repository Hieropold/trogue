pub mod plugins;
pub mod steam_api;
pub mod steam_client;
pub mod ui;

use clap::Command;
use steam_api::HttpSteamClient;
use std::io::{stderr, stdout};
use std::process;

// The main entry point of the application.
//
// <purpose-start>
// This function is the main entry point of the application. It parses the command-line arguments,
// builds the production Steam client from environment variables, and runs the appropriate command.
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
// - **Prints to the console**: The output of the commands is printed to the standard output.
// - **Exits the process**: The process is terminated when the command has finished executing, or
//   immediately with a non-zero code if required environment variables are missing.
// <side-effects-end>
#[tokio::main]
async fn main() {
    let steam = match HttpSteamClient::from_env() {
        Ok(client) => client,
        Err(e) => {
            eprintln!("Error: {}", e);
            process::exit(1);
        }
    };
    let plugins = plugins::get_plugins();

    let mut command = Command::new("trogue")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Hieropold <hieropold@gmail.com>")
        .about("A CLI tool for displaying Steam achievements");

    for plugin in &plugins {
        command = command.subcommand(plugin.command());
    }

    let matches = command.get_matches();

    for plugin in &plugins {
        if let Some(sub_matches) = matches.subcommand_matches(plugin.command().get_name()) {
            match plugin.execute_deep(&steam, sub_matches).await {
                Ok(ui::ViewData::None) => {
                    // Fallback to legacy execute if not implemented
                    plugin
                        .execute(&steam, sub_matches, &mut stdout(), &mut stderr())
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
