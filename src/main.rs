use std::collections::HashMap;

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
struct Bole {
    /// Show all package manager instances
    #[arg(short, long, help = "Show all individual package manager instances")]
    all: bool,

    /// Display output in tree format
    #[arg(short, long, help = "Display output in tree format")]
    tree: bool,
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

    match (args.all, args.tree) {
        (false, false) => {
            // Default: grouped table
            let mut table = Table::new(group_pm_instances(found_pms));
            println!("{}", table.with(Style::modern()));
            println!("\nTip: Use -a to see all the other locations");
        },
        (true, false) => {
            // All instances table
            let mut table = Table::new(found_pms);
            println!("{}", table.with(Style::modern()));
        },
        (false, true) => {
            // Grouped tree
            display_grouped_tree(group_pm_instances(found_pms));
            println!("\nTip: Use -a to see all the other locations");
        },
        (true, true) => {
            // All instances tree
            display_tree(found_pms);
        },
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

fn display_tree(instances: Vec<pm::PmInfo>) {
    let mut grouped: HashMap<String, Vec<pm::PmInfo>> = HashMap::new();

    for instance in instances {
        grouped
            .entry(instance.name.clone())
            .or_default()
            .push(instance);
    }

    let mut sorted: Vec<_> = grouped.into_iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(&b.0));

    for (i, (name, instances)) in sorted.iter().enumerate() {
        let is_last_group = i == sorted.len() - 1;
        let group_prefix = if is_last_group {
            "└──"
        } else {
            "├──"
        };
        let count = instances.len();

        println!(
            "{} {} ({})",
            group_prefix,
            name,
            if count == 1 {
                "1 installation".to_string()
            } else {
                format!("{} installations", count)
            }
        );

        for (j, instance) in instances.iter().enumerate() {
            let is_last_instance = j == instances.len() - 1;
            let continuation = if is_last_group { "    " } else { "│   " };
            let instance_prefix = if is_last_instance {
                "└──"
            } else {
                "├──"
            };
            let indicator = if j == 0 { "*" } else { "-" };

            println!(
                "{} {} {} {} v{} [{}]",
                continuation,
                instance_prefix,
                indicator,
                instance.path,
                instance.version,
                instance.install_method
            );
        }
    }
}

fn display_grouped_tree(grouped: Vec<pm::GroupedPmInfo>) {
    for (i, group) in grouped.iter().enumerate() {
        let is_last = i == grouped.len() - 1;
        let prefix = if is_last { "└──" } else { "├──" };

        println!(
            "{} {} v{} [{}]",
            prefix, group.name, group.version, group.install_method
        );

        if group.alternatives != "-" {
            let continuation = if is_last { "    " } else { "│   " };
            println!("{}└── {}", continuation, group.alternatives);
        }
    }
}
