pub mod app;
pub mod cfg;
pub mod constants;
pub mod steam_api;
pub mod ui;
pub mod plugins;

use cfg::Cfg;
use clap::Command;
use std::io::{stdout, stderr};
use std::process;

/// Loads the application configuration.
///
/// <purpose-start>
/// This function is responsible for loading the application configuration from environment variables.
/// If the configuration cannot be loaded, it prints an error message and exits the process.
/// <purpose-end>
///
/// <inputs-start>
/// - None.
/// <inputs-end>
///
/// <outputs-start>
/// - `Cfg`: The loaded application configuration.
/// <outputs-end>
///
/// <side-effects-start>
/// - **Exits the process**: If the configuration cannot be loaded, the process is terminated with a non-zero exit code.
/// <side-effects-end>
fn load_cfg() -> Cfg {
    let mut cfg = Cfg::new();

    if let Err(e) = cfg.load() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }

    cfg
}

/// The main entry point of the application.
///
/// <purpose-start>
/// This function is the main entry point of the application. It parses the command-line arguments,
/// loads the configuration, and runs the appropriate command.
/// <purpose-end>
///
/// <inputs-start>
/// - None.
/// <inputs-end>
///
/// <outputs-start>
/// - None.
/// <outputs-end>
///
/// <side-effects-start>
/// - **Prints to the console**: The output of the commands is printed to the standard output.
/// - **Exits the process**: The process is terminated when the command has finished executing.
/// <side-effects-end>
#[tokio::main]
async fn main() {
    let cfg = load_cfg();
    let app_context = app::AppContext::new(cfg);
    let plugins = plugins::get_plugins();

    let mut command = Command::new("trogue")
        .version("1.0")
        .author("Hieropold <unsolicited.pcholler@gmail.com>")
        .about("A CLI tool for displaying Steam achievements");

    for plugin in &plugins {
        command = command.subcommand(plugin.command());
    }

    let matches = command.get_matches();

    for plugin in &plugins {
        if let Some(sub_matches) = matches.subcommand_matches(plugin.command().get_name()) {
            plugin.execute(
                &app_context,
                sub_matches,
                &mut stdout(),
                &mut stderr(),
            ).await;
            return;
        }
    }
}
