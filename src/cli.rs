//! Command-line interface handling for the show command.

use bole::pm::{self, Category};
use rayon::prelude::*;
use tabled::{Table, Tabled, settings::Style};

use crate::display::{
    OutputFormat, display_grouped_tree, display_tree, group_pm_instances, output_csv,
    output_grouped_csv, output_grouped_json, output_json,
};

/// Handles the show command with filtering and output format options.
pub(super) fn handle_show_command(
    category: Option<String>,
    all: bool,
    tree: bool,
    json: bool,
    csv: bool,
) {
    // Determine target categories, either specific or all
    let target_categories = if let Some(ref category_str) = category {
        let target_category = match parse_category(category_str) {
            Some(cat) => cat,
            None => {
                print_category_help(category_str);
                return;
            },
        };
        vec![target_category]
    } else {
        Category::all().to_vec()
    };

    // Filter detectors by target categories, then parallelize their execution
    let filtered_pms: Vec<_> = pm::all_package_managers()
        .into_iter()
        .filter(|detector| {
            target_categories
                .iter()
                .any(|&cat| detector.category() == cat)
        })
        .collect::<Vec<_>>()
        .par_iter()
        .flat_map(|detector| detector.find())
        .collect();

    if filtered_pms.is_empty() {
        if let Some(cat_str) = category {
            println!("No {} package managers found.", cat_str);
        }
        return;
    }

    let format = OutputFormat::from_flags(json, csv);

    match format {
        OutputFormat::Json => {
            if all {
                if let Err(e) = output_json(&filtered_pms) {
                    eprintln!("Error outputting JSON: {}", e);
                }
            } else {
                let grouped = group_pm_instances(filtered_pms);
                if let Err(e) = output_grouped_json(&grouped) {
                    eprintln!("Error outputting JSON: {}", e);
                }
            }
        },
        OutputFormat::Csv => {
            if all {
                output_csv(&filtered_pms);
            } else {
                let grouped = group_pm_instances(filtered_pms);
                output_grouped_csv(&grouped);
            }
        },
        OutputFormat::Table => {
            match (all, tree) {
                (false, false) => {
                    // Default: grouped table
                    let mut table = Table::new(group_pm_instances(filtered_pms));
                    println!("{}", table.with(Style::modern()));
                    println!("\nTip: Use --all to see all the other locations");
                },
                (true, false) => {
                    // All instances table
                    let mut table = Table::new(filtered_pms);
                    println!("{}", table.with(Style::modern()));
                },
                (false, true) => {
                    // Grouped tree
                    display_grouped_tree(group_pm_instances(filtered_pms));
                    println!("\nTip: Use --all to see all the other locations");
                },
                (true, true) => {
                    // All instances tree
                    display_tree(filtered_pms);
                },
            }
        },
    }
}

/// Handles the check command for package manager health analysis.
pub(super) fn handle_check_command() {
    println!("Checking package manager health...\n");

    // Discover all package managers in parallel
    let all_pms: Vec<_> = pm::all_package_managers()
        .into_par_iter()
        .flat_map(|detector| detector.find())
        .collect();

    if all_pms.is_empty() {
        println!("No package managers found.");
        return;
    }

    // If discovery succeeded and got version, PM is healthy
    // If version is empty, PM is broken
    let broken: Vec<_> = all_pms
        .iter()
        .filter(|pm| pm.version.trim().is_empty())
        .collect();

    for pm in &broken {
        println!("BROKEN: {} at {}", pm.name, pm.path);
    }

    let names = all_pms
        .iter()
        .map(|pm| pm.name.clone())
        .collect::<Vec<String>>();

    // Summary
    let healthy_count = all_pms.len() - broken.len();
    println!(
        "\nTotal: {} package managers: {}",
        all_pms.len(),
        names.join(", ")
    );
    println!("Healthy: {}", healthy_count);
    if !broken.is_empty() {
        println!("Broken: {}", broken.len());
    }
}

/// Parses category string into Category enum, supporting aliases.
fn parse_category(category_str: &str) -> Option<Category> {
    let input = category_str.to_lowercase();

    for &category in Category::all() {
        // Check primary name
        if input == category.name() {
            return Some(category);
        }

        // Check aliases
        for &alias in category.aliases() {
            if input == alias {
                return Some(category);
            }
        }
    }

    None
}

/// Prints help for available categories with package manager lists.
fn print_category_help(unknown_category: &str) {
    #[derive(Tabled)]
    struct CategoryRow<'a> {
        #[tabled(rename = "Category")]
        category: &'a str,
        #[tabled(rename = "Managers")]
        managers: String,
        #[tabled(rename = "Aliases")]
        aliases: String,
    }

    println!("Unknown category '{}'.", unknown_category);
    println!("\nAvailable categories:");

    // Parallel category help generation
    let rows: Vec<CategoryRow> = Category::all()
        .par_iter()
        .map(|&category| {
            let mut tools: Vec<&str> = pm::all_package_managers()
                .iter()
                .filter(|detector| detector.category() == category)
                .map(|detector| detector.name())
                .collect();
            tools.sort_unstable();

            let managers = if tools.is_empty() {
                String::from("-")
            } else {
                tools.join(", ")
            };

            let aliases = if category.aliases().is_empty() {
                String::from("-")
            } else {
                category.aliases().join(", ")
            };

            CategoryRow {
                category: category.name(),
                managers,
                aliases,
            }
        })
        .collect();

    let mut table = Table::new(rows);
    println!("{}", table.with(Style::modern()));

    println!("\nHint: use 'bole show <category>' to filter.");
}
