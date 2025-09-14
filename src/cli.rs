//! Command-line interface handling for the show command.

use bole::pm::{self, Category};
use tabled::{Table, Tabled, settings::Style};

use crate::display::{display_grouped_tree, display_tree, group_pm_instances};

/// Handles the show command with filtering and output format options.
pub(super) fn handle_show_command(category: Option<String>, all: bool, tree: bool) {
    // Filter by category if specified
    let filtered_pms = if let Some(ref category_str) = category {
        let target_category = match parse_category(category_str) {
            Some(cat) => cat,
            None => {
                print_category_help(category_str);
                return;
            },
        };

        filter_by_category(target_category)
    // No category filter, return all found package managers
    } else {
        let mut found_pms = Vec::new();

        for detector in pm::all_package_managers() {
            let instances = detector.find();
            found_pms.extend(instances);
        }

        found_pms
    };

    if filtered_pms.is_empty() {
        if let Some(cat_str) = category {
            println!("No {} package managers found.", cat_str);
        }
        return;
    }

    match (all, tree) {
        (false, false) => {
            // Default: grouped table
            let mut table = Table::new(group_pm_instances(filtered_pms));
            println!("{}", table.with(Style::modern()));
            if !all {
                println!("\nTip: Use --all to see all the other locations");
            }
        },
        (true, false) => {
            // All instances table
            let mut table = Table::new(filtered_pms);
            println!("{}", table.with(Style::modern()));
        },
        (false, true) => {
            // Grouped tree
            display_grouped_tree(group_pm_instances(filtered_pms));
            if !all {
                println!("\nTip: Use --all to see all the other locations");
            }
        },
        (true, true) => {
            // All instances tree
            display_tree(filtered_pms);
        },
    }
}

/// Prints help for available categories with dynamically generated package
/// lists.
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

    let mut rows: Vec<CategoryRow> = Vec::new();

    for &category in Category::all() {
        let mut tools = pm::get_package_managers_in_category(category);
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

        rows.push(CategoryRow {
            category: category.name(),
            managers,
            aliases,
        });
    }

    let mut table = Table::new(rows);
    println!("{}", table.with(Style::modern()));

    println!("\nHint: use 'bole show <category>' to filter.");
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

/// Filters package managers by category and returns all matching instances.
fn filter_by_category(target_category: Category) -> Vec<pm::PmInfo> {
    let mut filtered = Vec::new();

    for detector in pm::all_package_managers() {
        if detector.category() == target_category {
            let instances = detector.find();
            filtered.extend(instances);
        }
    }

    filtered
}
