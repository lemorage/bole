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
pub(super) fn handle_check_command(verbose: u8, broken: bool, outdated: bool) {
    use std::collections::HashMap;

    // Discover all package managers in parallel and group by category
    let all_detectors = pm::all_package_managers();
    let all_pms_raw: Vec<(Category, pm::PmInfo)> = all_detectors
        .into_par_iter()
        .flat_map(|detector| {
            let pms: Vec<(Category, pm::PmInfo)> = detector
                .find()
                .into_iter()
                .map(move |pm| (detector.category(), pm))
                .collect();
            pms
        })
        .collect();

    // Group PMs by category
    let mut pms_by_category: HashMap<Category, Vec<pm::PmInfo>> = HashMap::new();
    for (category, pm) in all_pms_raw {
        pms_by_category.entry(category).or_default().push(pm);
    }

    // Flatten to get all PMs
    let flat_pms: Vec<_> = pms_by_category
        .values()
        .flat_map(|pms| pms.iter())
        .collect();

    if flat_pms.is_empty() {
        println!("No package managers found.");
        return;
    }

    // Find broken PMs
    let broken_pms: Vec<_> = flat_pms
        .iter()
        .filter(|pm| pm.version.trim().is_empty())
        .collect();

    // Check for outdated PMs if requested
    let outdated_pms: Vec<(&pm::PmInfo, bole::find::Bump)> = if outdated {
        flat_pms
            .iter()
            .filter_map(|pm| {
                // Skip broken PMs
                if pm.version.trim().is_empty() {
                    return None;
                }

                // Get the detector for this PM
                let detector = pm::all_package_managers()
                    .into_iter()
                    .find(|d| d.name() == pm.name)?;

                // Check if outdated
                if let Some(bump) = detector.check_bump(&pm.version)
                    && bump.latest != pm.version
                {
                    return Some((*pm, bump));
                }
                None
            })
            .collect()
    } else {
        Vec::new()
    };

    // Handle --outdated flag: show only outdated PMs
    if outdated {
        if outdated_pms.is_empty() {
            println!("All package managers are up to date.");
            std::process::exit(0);
        }

        println!("Outdated package managers:\n");
        for (pm, bump) in &outdated_pms {
            println!("OUTDATED: {} at {}", pm.name, pm.path);
            println!("  Current: {}", pm.version);
            println!("  Latest:  {}", bump.latest);
            println!("  Update: {}", bump.cmd);
            println!();
        }

        println!("Total outdated: {}", outdated_pms.len());
        std::process::exit(0);
    }

    // Handle --broken flag: show only broken PMs with diagnostic info
    if broken {
        if broken_pms.is_empty() {
            println!("No broken package managers found.");
            std::process::exit(0);
        }

        println!("Broken package managers detected:\n");
        for pm in &broken_pms {
            println!("BROKEN: {} at {}", pm.name, pm.path);

            // Provide diagnostic information
            if !std::path::Path::new(&pm.path).exists() {
                println!("  Issue: Binary not found at expected path");
                println!("  Fix: Reinstall {} or update PATH", pm.name);
            } else {
                println!("  Issue: Binary exists but version check failed");
                println!(
                    "  Fix: Check if {} is properly installed or corrupted",
                    pm.name
                );

                // Check file permissions
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    if let Ok(metadata) = std::fs::metadata(&pm.path) {
                        let perms = metadata.permissions();
                        if perms.mode() & 0o111 == 0 {
                            println!("  Note: File is not executable");
                        }
                    }
                }
            }
            println!();
        }

        println!("Total broken: {}", broken_pms.len());
        std::process::exit(1);
    }

    println!("Checking package manager health...\n");

    match verbose {
        0 => {
            for pm in &broken_pms {
                println!("BROKEN: {} at {}", pm.name, pm.path);
            }

            let healthy_count = flat_pms.len() - broken_pms.len();
            println!("\nTotal: {} package managers", flat_pms.len());
            println!("Healthy: {}", healthy_count);
            if !broken_pms.is_empty() {
                println!("Broken: {}", broken_pms.len());
            }
        },
        1 => {
            // Display by category
            for category in Category::all() {
                if let Some(pms) = pms_by_category.get(category) {
                    // Count occurrences of each PM name
                    let mut name_counts: HashMap<&str, usize> = HashMap::new();
                    for pm in pms {
                        *name_counts.entry(pm.name.as_str()).or_insert(0) += 1;
                    }

                    // Sort PM names for consistent display
                    let mut sorted_names: Vec<_> = name_counts.keys().copied().collect();
                    sorted_names.sort();

                    // Build display string with counts
                    let display_names: Vec<String> = sorted_names
                        .iter()
                        .map(|&name| {
                            let count = name_counts[name];
                            if count > 1 {
                                format!("{}({})", name, count)
                            } else {
                                name.to_string()
                            }
                        })
                        .collect();

                    println!(
                        "{}: {} checked ({})",
                        category.name(),
                        pms.len(),
                        display_names.join(", ")
                    );
                }
            }

            if !broken_pms.is_empty() {
                println!("\nBROKEN:");
                for pm in &broken_pms {
                    println!("  {} at {}", pm.name, pm.path);
                }
            }

            let healthy_count = flat_pms.len() - broken_pms.len();
            println!("\nTotal: {} package managers", flat_pms.len());
            println!("Healthy: {}", healthy_count);
            if !broken_pms.is_empty() {
                println!("Broken: {}", broken_pms.len());
            }
        },
        _ => {
            for category in Category::all() {
                if let Some(pms) = pms_by_category.get(category) {
                    println!("Checking {} package managers:", category.name());
                    for pm in pms {
                        let status = if pm.version.trim().is_empty() {
                            "BROKEN"
                        } else {
                            "OK"
                        };
                        println!("  [{}] {} v{} at {}", status, pm.name, pm.version, pm.path);
                    }
                }
            }

            if !broken_pms.is_empty() {
                println!("\nBROKEN SUMMARY:");
                for pm in &broken_pms {
                    println!("  {} at {}", pm.name, pm.path);
                }
            }

            let healthy_count = flat_pms.len() - broken_pms.len();
            println!("\nTotal: {} package managers", flat_pms.len());
            println!("Healthy: {}", healthy_count);
            if !broken_pms.is_empty() {
                println!("Broken: {}", broken_pms.len());
            }
        },
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
