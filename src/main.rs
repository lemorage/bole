mod banner;
mod color;
mod commands;
mod discovery;
mod filters;
mod format;
mod network;
mod pipeline;

use banner::stream_banner;
use clap::{CommandFactory, Parser, Subcommand};
use commands::{CheckCommand, OwnCommand, ShowCommand};

/// A CLI to manage your package managers.
#[derive(Parser, Debug)]
#[command(
    version,
    about = "Bole is a unified CLI for managing all package managers on your system.",
    long_about = None
)]
struct Bole {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show package managers by category
    Show {
        /// Category to show (system, language-specific, tools)
        category: Option<String>,

        /// Show all matching executables in PATH (including inactive
        /// duplicates)
        #[arg(
            short = 'a',
            long = "all-paths",
            visible_alias = "all",
            help = "Show all matching executables in PATH"
        )]
        all: bool,

        /// Display output in tree format
        #[arg(short, long, help = "Display output in tree format")]
        tree: bool,

        /// Output in JSON format
        #[arg(long, conflicts_with = "csv", help = "Output in JSON format")]
        json: bool,

        /// Output in CSV format
        #[arg(long, conflicts_with = "json", help = "Output in CSV format")]
        csv: bool,
    },
    /// Check package manager health and status
    Check {
        /// Category to check (system, language-specific, tools)
        category: Option<String>,

        /// Check all matching executables in PATH (including inactive
        /// duplicates)
        #[arg(
            short = 'a',
            long = "all-paths",
            visible_alias = "all",
            help = "Check all matching executables in PATH"
        )]
        all: bool,

        /// Increase verbosity level (use -v for categories, -vv for detailed)
        #[arg(short, long, action = clap::ArgAction::Count, help = "Increase verbosity")]
        verbose: u8,

        /// Show only broken package managers
        #[arg(short, long, help = "Show only broken package managers")]
        broken: bool,

        /// Show only outdated package managers
        #[arg(short, long, help = "Show only outdated package managers")]
        outdated: bool,
    },
    /// Show which package managers own a specific tool
    Own {
        /// Tool name to search for (e.g., typescript, ripgrep, uv)
        tool: String,

        /// Display output in tree format
        #[arg(short, long, help = "Display output in tree format")]
        tree: bool,

        /// Output in JSON format
        #[arg(long, conflicts_with = "csv", help = "Output in JSON format")]
        json: bool,

        /// Output in CSV format
        #[arg(long, conflicts_with = "json", help = "Output in CSV format")]
        csv: bool,
    },
}

fn main() {
    let raw_args: Vec<String> = std::env::args().skip(1).collect();

    if raw_args.is_empty()
        || (raw_args.len() == 1 && matches!(raw_args[0].as_str(), "--help" | "-h" | "help"))
    {
        // Bare `bole` or explicit top-level help
        stream_banner();
        println!();
        if raw_args.is_empty() {
            Bole::command().print_help().unwrap();
            std::process::exit(1);
        }
    }

    let args = Bole::parse();

    match args.command {
        Some(Commands::Show {
            category,
            all,
            tree,
            json,
            csv,
        }) => {
            ShowCommand::from_args(category, all, tree, json, csv).execute();
        },
        Some(Commands::Check {
            category,
            all,
            verbose,
            broken,
            outdated,
        }) => {
            CheckCommand::from_args(category, all, verbose, broken, outdated).execute();
        },
        Some(Commands::Own {
            tool,
            tree,
            json,
            csv,
        }) => {
            OwnCommand::from_args(tool, tree, json, csv).execute();
        },
        None => {
            Bole::command().print_help().unwrap();
        },
    }
}
