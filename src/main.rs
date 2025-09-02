use bole::{find::Find, pm};
use clap::Parser;
use std::collections::HashMap;
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
struct Bole {
    /// Show all package manager instances
    #[arg(long, help = "Show all package manager instances with detailed paths")]
    all: bool,
}

fn main() {
    let args = Bole::parse();

    let mut found_pms = Vec::new();

    for detector in pm::all_package_managers() {
        let instances = detector.find();
        found_pms.extend(instances);
    }

    if found_pms.is_empty() {
        println!("No supported package managers found.");
        return;
    }

    if args.all {
        let mut table = Table::new(found_pms);
        println!("{}", table.with(Style::modern()));
    } else {
        let mut table = Table::new(group_pm_instances(found_pms));
        println!("{}", table.with(Style::modern()));
        println!("\n💡 Use --all to see all individual installations");
    }
}

fn group_pm_instances(instances: Vec<pm::PmInfo>) -> Vec<pm::GroupedPmInfo> {
    let mut grouped: HashMap<String, Vec<pm::PmInfo>> = HashMap::new();

    for instance in instances {
        grouped
            .entry(instance.name.clone())
            .or_default()
            .push(instance);
    }

    let mut result: Vec<pm::GroupedPmInfo> = grouped
        .into_iter()
        .map(|(name, instances)| pm::GroupedPmInfo::from_instances(name, instances))
        .collect();

    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}
