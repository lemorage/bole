mod banner;
mod cli;
mod display;

use banner::stream_banner;
use clap::{CommandFactory, Parser, Subcommand};
use cli::handle_show_command;

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

    /// Show all package manager instances
    #[arg(short, long, help = "Show all individual package manager instances")]
    all: bool,

    /// Display output in tree format
    #[arg(short, long, help = "Display output in tree format")]
    tree: bool,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Show package managers by category
    Show {
        /// Category to show (system, language-specific, tools)
        category: Option<String>,

        /// Show all package manager instances
        #[arg(short, long, help = "Show all individual package manager instances")]
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
}

#[inline]
fn show_help_and_exit(exit_code: i32) -> ! {
    stream_banner();
    println!();
    let mut app = Bole::command();
    app.print_help().unwrap();
    std::process::exit(exit_code);
}

fn main() {
    // Check if user wants help before parsing
    let help_requested = std::env::args().any(|arg| arg == "--help" || arg == "-h");

    if help_requested {
        show_help_and_exit(0);
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
            handle_show_command(category, all, tree, json, csv);
        },
        None => {
            show_help_and_exit(1);
        },
    }
}
