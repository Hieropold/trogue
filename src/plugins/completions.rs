//! Plugin for generating shell completion scripts.
//!
//! <purpose-start>
//! This plugin provides the `completions` command, which generates shell completion scripts
//! for bash and zsh. These scripts enable tab completion for trogue commands and subcommands.
//! <purpose-end>
//!
//! <inputs-start>
//! - `app_context`: The shared application context (not used by this plugin).
//! - `matches`: The command-line arguments parsed by `clap`, containing the shell type.
//! <inputs-end>
//!
//! <outputs-start>
//! - A shell completion script printed to stdout.
//! <outputs-end>
//!
//! <side-effects-start>
//! - Writes the completion script to the provided writer (stdout).
//! <side-effects-end>

use crate::{app::AppContext, plugins::Plugin};
use async_trait::async_trait;
use clap::{Arg, Command, ValueEnum};
use clap_complete::{generate, Shell};
use std::io::Write;

pub struct CompletionsPlugin;

// Represents the supported shell types for completion generation.
//
// <purpose-start>
// This enum defines the shells for which completion scripts can be generated.
// It implements clap's ValueEnum trait to enable command-line parsing.
// <purpose-end>
#[derive(Debug, Clone, Copy, ValueEnum)]
enum ShellType {
    // Bash shell
    Bash,
    // Zsh shell
    Zsh,
    // Fish shell
    Fish,
    // PowerShell
    PowerShell,
}

// No need for CommandFactory - we'll build the command structure manually

#[async_trait]
impl Plugin for CompletionsPlugin {
    // Defines the clap command for the `completions` plugin.
    //
    // <purpose-start>
    // This method provides the command-line interface for the `completions` plugin,
    // which allows users to generate shell completion scripts for various shells.
    // <purpose-end>
    //
    // <inputs-start>
    // - `&self`: A reference to the plugin instance.
    // <inputs-end>
    //
    // <outputs-start>
    // - `clap::Command`: The clap command definition for the `completions` plugin.
    // <outputs-end>
    //
    // <side-effects-start>
    // - None.
    // <side-effects-end>
    fn command(&self) -> Command {
        Command::new("completions")
            .about("Generate shell completion scripts")
            .long_about(
                "Generate shell completion scripts for trogue.\n\n\
                To install completions:\n\n\
                Bash:\n  \
                trogue completions bash >> ~/.bashrc\n  \
                source ~/.bashrc\n\n\
                Zsh:\n  \
                trogue completions zsh > ~/.zsh/completions/_trogue\n  \
                Add 'fpath=(~/.zsh/completions $fpath)' to ~/.zshrc\n  \
                source ~/.zshrc"
            )
            .arg(
                Arg::new("shell")
                    .value_name("SHELL")
                    .required(true)
                    .value_parser(clap::value_parser!(ShellType))
                    .help("The shell to generate completions for (bash, zsh, fish, powershell)"),
            )
    }

    // Executes the `completions` plugin's logic.
    //
    // <purpose-start>
    // This method is called by the core application when the `completions` command is invoked.
    // It generates a shell completion script for the specified shell and outputs it to stdout.
    // The generated script includes completions for all registered plugins and their subcommands.
    // <purpose-end>
    //
    // <inputs-start>
    // - `&self`: A reference to the plugin instance.
    // - `app_context`: The shared application context (unused by this plugin).
    // - `matches`: The clap argument matches for the `completions` subcommand.
    // - `writer`: A mutable reference to a writer for standard output.
    // - `err_writer`: A mutable reference to a writer for standard error (unused).
    // <inputs-end>
    //
    // <outputs-start>
    // - None.
    // <outputs-end>
    //
    // <side-effects-start>
    // - Writes the completion script to the provided writer.
    // - The script must be redirected to a file and sourced by the shell to enable completions.
    // <side-effects-end>
    async fn execute(
        &self,
        _app_context: &AppContext,
        matches: &clap::ArgMatches,
        writer: &mut (dyn Write + Send),
        _err_writer: &mut (dyn Write + Send),
    ) {
        let shell_type = matches.get_one::<ShellType>("shell").unwrap();

        // Build the complete command structure with all subcommands
        let mut cmd = Command::new("trogue")
            .version("1.0")
            .author("Hieropold <unsolicited.pcholler@gmail.com>")
            .about("A CLI tool for displaying Steam achievements");

        // Add all plugin commands
        for plugin in crate::plugins::get_plugins() {
            cmd = cmd.subcommand(plugin.command());
        }

        // Generate the completion script using clap_complete
        let shell = match shell_type {
            ShellType::Bash => Shell::Bash,
            ShellType::Zsh => Shell::Zsh,
            ShellType::Fish => Shell::Fish,
            ShellType::PowerShell => Shell::PowerShell,
        };

        generate(shell, &mut cmd, "trogue", writer);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::AppContext;
    use crate::steam_api::Api;
    use clap::ArgMatches;

    fn get_matches_for_args(args: &[&str]) -> ArgMatches {
        CompletionsPlugin.command().get_matches_from(args)
    }

    #[test]
    fn test_command() {
        let plugin = CompletionsPlugin;
        let cmd = plugin.command();
        assert_eq!(cmd.get_name(), "completions");
        assert!(cmd.get_about().is_some());
        assert!(cmd.get_arguments().any(|arg| arg.get_id() == "shell"));
    }

    #[tokio::test]
    async fn test_execute_bash() {
        let api = Api::new(
            "test_key".to_string(),
            "test_id".to_string(),
            "http://localhost".to_string(),
        );
        let app_context = AppContext { api };
        let matches = get_matches_for_args(&["completions", "bash"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        CompletionsPlugin
            .execute(&app_context, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(writer).unwrap();
        // Verify that the output contains bash completion script markers
        assert!(output.contains("_trogue()") || output.contains("complete"));
    }

    #[tokio::test]
    async fn test_execute_zsh() {
        let api = Api::new(
            "test_key".to_string(),
            "test_id".to_string(),
            "http://localhost".to_string(),
        );
        let app_context = AppContext { api };
        let matches = get_matches_for_args(&["completions", "zsh"]);
        let mut writer = Vec::new();
        let mut err_writer = Vec::new();

        CompletionsPlugin
            .execute(&app_context, &matches, &mut writer, &mut err_writer)
            .await;

        let output = String::from_utf8(writer).unwrap();
        // Verify that the output contains zsh completion script markers
        assert!(output.contains("#compdef") || output.contains("_trogue"));
    }

    #[test]
    fn test_shell_type_conversion() {
        let bash: Shell = match ShellType::Bash {
            ShellType::Bash => Shell::Bash,
            ShellType::Zsh => Shell::Zsh,
            ShellType::Fish => Shell::Fish,
            ShellType::PowerShell => Shell::PowerShell,
        };
        assert!(matches!(bash, Shell::Bash));

        let zsh: Shell = match ShellType::Zsh {
            ShellType::Bash => Shell::Bash,
            ShellType::Zsh => Shell::Zsh,
            ShellType::Fish => Shell::Fish,
            ShellType::PowerShell => Shell::PowerShell,
        };
        assert!(matches!(zsh, Shell::Zsh));

        let fish: Shell = match ShellType::Fish {
            ShellType::Bash => Shell::Bash,
            ShellType::Zsh => Shell::Zsh,
            ShellType::Fish => Shell::Fish,
            ShellType::PowerShell => Shell::PowerShell,
        };
        assert!(matches!(fish, Shell::Fish));

        let powershell: Shell = match ShellType::PowerShell {
            ShellType::Bash => Shell::Bash,
            ShellType::Zsh => Shell::Zsh,
            ShellType::Fish => Shell::Fish,
            ShellType::PowerShell => Shell::PowerShell,
        };
        assert!(matches!(powershell, Shell::PowerShell));
    }
}