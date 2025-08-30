use bole::{find::Find, pm};
use clap::Parser;
use tabled::{Table, settings::Style};

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

/// A CLI to manage your package manager.
#[derive(Parser, Debug)]
#[command(
    version,
    about = "Bole is a unified CLI for managing all package managers on your system.",
    long_about = None,
    before_help = BOLE_BANNER,
    before_long_help = BOLE_BANNER
)]
struct Bole {}

fn main() {
    // Parse CLI flags (currently none; enables --help/--version)
    let _ = Bole::parse();

    let mut found_pms = Vec::new();

    for detector in pm::all_package_managers() {
        let instances = detector.find();
        found_pms.extend(instances);
    }

    if found_pms.is_empty() {
        println!("No supported package managers found.");
        return;
    }

    let mut table = Table::new(found_pms);
    println!("{}", table.with(Style::modern()));
}
