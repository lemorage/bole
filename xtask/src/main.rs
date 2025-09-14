mod changelog;
mod release;

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "bole project automation tasks")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new release
    Release {
        /// Version bump type or explicit version
        version: String,
        /// Show what would be done without making changes
        #[arg(long)]
        dry_run: bool,
        /// Skip pushing to remote
        #[arg(long)]
        no_push: bool,
        /// Skip code quality checks
        #[arg(long)]
        skip_checks: bool,
        /// Skip all confirmation prompts
        #[arg(short, long)]
        yes: bool,
    },
    /// Generate changelog preview
    Changelog,
    /// Run all development checks
    Check,
    /// Format code
    Fmt,
    /// Run clippy lints
    Clippy,
    /// Run tests
    Test,
    /// Build the project
    Build,
    /// Clean build artifacts
    Clean,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Release {
            version,
            dry_run,
            no_push,
            skip_checks,
            yes,
        } => release::run(release::ReleaseArgs {
            version,
            dry_run,
            no_push,
            skip_checks,
            yes,
        }),
        Commands::Changelog => changelog::run(),
        Commands::Check => run_check(),
        Commands::Fmt => run_fmt(),
        Commands::Clippy => run_clippy(),
        Commands::Test => run_test(),
        Commands::Build => run_build(),
        Commands::Clean => run_clean(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run_fmt() -> Result<()> {
    println!("{}", "𓇢 Running cargo fmt...".cyan());
    std::process::Command::new("cargo")
        .args(["fmt", "--all"])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run cargo fmt: {}", e))?;
    println!("{}", "✓ Code formatting complete".green());
    Ok(())
}

fn run_clippy() -> Result<()> {
    println!("{}", "𓇢 Running cargo clippy...".cyan());
    let status = std::process::Command::new("cargo")
        .args([
            "clippy",
            "--all-targets",
            "--all-features",
            "--",
            "-D",
            "warnings",
        ])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run cargo clippy: {}", e))?;

    if !status.success() {
        anyhow::bail!("Clippy found issues");
    }
    println!("{}", "✓ Clippy checks passed".green());
    Ok(())
}

fn run_test() -> Result<()> {
    println!("{}", "𓇢 Running cargo test...".cyan());
    let status = std::process::Command::new("cargo")
        .args(["test", "--all-features"])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run cargo test: {}", e))?;

    if !status.success() {
        anyhow::bail!("Tests failed");
    }
    println!("{}", "✓ All tests passed".green());
    Ok(())
}

fn run_build() -> Result<()> {
    println!("{}", "𓇢 Running cargo build...".cyan());
    let status = std::process::Command::new("cargo")
        .args(["build", "--all-features"])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run cargo build: {}", e))?;

    if !status.success() {
        anyhow::bail!("Build failed");
    }
    println!("{}", "✓ Build successful".green());
    Ok(())
}

fn run_clean() -> Result<()> {
    println!("{}", "𓇢 Running cargo clean...".cyan());
    std::process::Command::new("cargo")
        .args(["clean"])
        .status()
        .map_err(|e| anyhow::anyhow!("Failed to run cargo clean: {}", e))?;
    println!("{}", "✓ Clean complete".green());
    Ok(())
}

fn run_check() -> Result<()> {
    println!("{}", "Woo, running all development checks...".cyan().bold());
    println!();

    println!("{}", "Step 1/4: Building project".blue().bold());
    run_build()?;
    println!();

    println!("{}", "Step 2/4: Running tests".blue().bold());
    run_test()?;
    println!();

    println!("{}", "Step 3/4: Code formatting".blue().bold());
    run_fmt()?;
    println!();

    println!("{}", "Step 4/4: Linting with clippy".blue().bold());
    run_clippy()?;
    println!();

    println!(
        "{}",
        "Good! All development checks passed successfully!"
            .green()
            .bold()
    );
    println!("{}", "   ✓ Project builds cleanly".green());
    println!("{}", "   ✓ All tests passing".green());
    println!("{}", "   ✓ Code formatted".green());
    println!("{}", "   ✓ No clippy warnings".green());

    Ok(())
}
