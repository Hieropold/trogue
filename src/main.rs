pub mod app;
pub mod cfg;
pub mod steam_api;
pub mod ui;

use cfg::Cfg;
use clap::{Arg, Command};
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
fn main() {
    let cli_matches = Command::new("trophyroom")
        .version("1.0")
        .author("Hieropold <unsolicited.pcholler@gmail.com>")
        .about("A CLI tool for displaying Steam achievements")
        .arg(
            Arg::new("banner")
                .short('b')
                .long("banner")
                .action(clap::ArgAction::SetTrue)
                .help("Displays a banner with the name of the program"),
        )
        .arg(
            Arg::new("list")
                .short('l')
                .long("list")
                .value_name("filter")
                .action(clap::ArgAction::Set)
                .num_args(0..=1)
                .help("Displays a list of all games on account set in environment variables"),
        )
        .arg(
            Arg::new("pattern")
                .short('p')
                .long("pattern")
                .help(r#"Specifies the output format for the list command. It can be used only with --list command. By default, the game id and name are displayed.
Possible tokens are:
    n - game name
    i - game id
E.g.: -p "i: n""#)
                .requires("list")
                .value_name("pattern"),
        )
        .arg(
            Arg::new("achievements")
                .short('a')
                .long("achievements")
                .value_name("achievements")
                .action(clap::ArgAction::Set)
                .num_args(0..=1)
                .help("Displays achievements for a specific game. Game id should be provided as an argument"),
        )
        .arg(
            Arg::new("global")
                .short('g')
                .long("global")
                .value_name("global")
                .requires("achievements")
                .action(clap::ArgAction::SetTrue)
                .help("Adds global achievement percentages for the output of game achievements. This flag can be used only with --achievements command"),
        )
        .arg(
            Arg::new("remaining")
                .short('r')
                .long("remaining")
                .value_name("remaining")
                .requires("achievements")
                .action(clap::ArgAction::SetTrue)
                .help("Displays only remaining locked achievements. This flag can be used only with --achievements command"),
        )
        .arg(
            Arg::new("progress-summary")
                .short('s')
                .long("progress-summary")
                .value_name("progress_summary")
                .requires("achievements")
                .action(clap::ArgAction::SetTrue)
                .help("Displays game achievements progress. This flag can be used only with --achievements command"),
        )
        .arg(
            Arg::new("dashboard")
                .short('d')
                .long("dashboard")
                .action(clap::ArgAction::SetTrue)
                .help("Displays a dashboard with 10 last played games and their achievement progress"),
        )
        .get_matches();

    let cfg = load_cfg();
    let app = app::App::new(cfg);

    if cli_matches.get_flag("banner") {
        ui::print_title();
        return;
    }

    if cli_matches.contains_id("list") {
        let filter = cli_matches.get_one::<String>("list").cloned();
        let format = cli_matches.get_one::<String>("pattern").cloned();
        app.list_games(filter, format);
        return;
    }

    if cli_matches.contains_id("achievements") {
        let game_id_str = cli_matches.get_one::<String>("achievements").unwrap();
        let add_global = cli_matches.get_flag("global");
        let remaining = cli_matches.get_flag("remaining");
        let progress = cli_matches.get_flag("progress-summary");
        if let Ok(game_id) = game_id_str.parse::<u32>() {
            if progress {
                app.show_progress(game_id);
            } else {
                app.list_achievements(game_id, add_global, remaining);
            }
        } else {
            eprintln!("Invalid game id: {}", game_id_str);
        }
        return;
    }

    if cli_matches.get_flag("dashboard") {
        app.show_dashboard();
        return;
    }

    return;
}
