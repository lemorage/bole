mod cli;
mod display;

use clap::{CommandFactory, Parser, Subcommand};
use cli::handle_show_command;

const BOLE_BANNER: &str = r#"
           .-.
          /   \
         /  .  \
        /__/ \__\
          || ||
        __||_||__
       /  BOLE   \
      /___________\
         \  |  /
          \ | /
           \|/
            V
    "#;

/// A CLI to manage your package managers.
#[derive(Parser, Debug)]
#[command(
    version,
    about = "Bole is a unified CLI for managing all package managers on your system.",
    long_about = None,
    before_help = BOLE_BANNER,
    before_long_help = BOLE_BANNER
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
    },
}

fn main() {
    let args = Bole::parse();

    match args.command {
        Some(Commands::Show {
            category,
            all,
            tree,
        }) => {
            handle_show_command(category, all, tree);
        },
        None => {
            // We show help when no subcommand provided
            let mut app = Bole::command();
            app.print_help().unwrap();
            std::process::exit(1);
        },
    }
}
