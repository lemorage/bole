//! Command-line interface handling for the show command.

use bole::pm::{self, Category};
use rayon::prelude::*;
use tabled::{Table, Tabled, settings::Style};

use crate::display::{
    OutputFormat, display_grouped_tree, display_tree, group_pm_instances, output_csv,
    output_grouped_csv, output_grouped_json, output_json,
};

/// Verbosity level for check command output.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Verbosity {
    /// Minimal output (summary)
    Quiet,
    /// Normal output (progress list)
    Normal,
    /// Verbose output (full details)
    Verbose,
}

impl From<u8> for Verbosity {
    fn from(count: u8) -> Self {
        match count {
            0 => Self::Quiet,
            1 => Self::Normal,
            _ => Self::Verbose,
        }
    }
}

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
    let all_pms: Vec<pm::PmInfo> = pm::all_package_managers()
        .into_par_iter()
        .flat_map(|detector| detector.find())
        .collect();

    if all_pms.is_empty() {
        println!("No package managers found.");
        return;
    }

    match (broken, outdated) {
        (true, false) => handle_broken_check(&all_pms),
        (false, true) => handle_outdated_check(&all_pms),
        (false, false) => handle_overview_check(&all_pms, Verbosity::from(verbose)),
        (true, true) => {
            eprintln!("Error: Cannot use --broken and --outdated together");
            std::process::exit(1);
        },
    }
}

/// Shows overview dashboard of all package managers
fn handle_overview_check(all_pms: &[pm::PmInfo], verbosity: Verbosity) {
    use std::collections::HashMap;

    println!("Checking package manager health...\n");

    // Find broken PMs
    let broken_pms: Vec<_> = all_pms
        .iter()
        .filter(|pm| pm.version.trim().is_empty())
        .collect();

    // Check for outdated PMs
    let outdated_map: HashMap<String, String> = all_pms
        .iter()
        .filter_map(|pm| {
            if pm.version.trim().is_empty() {
                return None;
            }

            let detector = pm::all_package_managers()
                .into_iter()
                .find(|d| d.name() == pm.name)?;

            if let Some(bump) = detector.check_bump(&pm.version)
                && bump.latest != pm.version
            {
                return Some((format!("{}:{}", pm.name, pm.path), bump.latest));
            }
            None
        })
        .collect();

    match verbosity {
        Verbosity::Quiet => {
            // Just show summary
        },
        Verbosity::Normal => {
            // Simple progress list
            for pm in all_pms {
                if pm.version.trim().is_empty() {
                    println!("✗ {} (broken)", pm.name);
                } else if let Some(latest) = outdated_map.get(&format!("{}:{}", pm.name, pm.path)) {
                    println!("✓ {} ({}) [outdated: {}]", pm.name, pm.version, latest);
                } else {
                    println!("✓ {} ({})", pm.name, pm.version);
                }
            }
            println!();
        },
        Verbosity::Verbose => {
            // Full categorized details
            let mut by_category: HashMap<Category, Vec<&pm::PmInfo>> = HashMap::new();
            for pm in all_pms {
                let category = pm::all_package_managers()
                    .into_iter()
                    .find(|d| d.name() == pm.name)
                    .map(|d| d.category())
                    .unwrap_or(Category::System);
                by_category.entry(category).or_default().push(pm);
            }

            for category in Category::all() {
                if let Some(pms) = by_category.get(category) {
                    println!("{}:", category.name());
                    for pm in pms {
                        if pm.version.trim().is_empty() {
                            println!("  ✗ {} (broken) at {}", pm.name, pm.path);
                        } else if let Some(latest) =
                            outdated_map.get(&format!("{}:{}", pm.name, pm.path))
                        {
                            println!(
                                "  ✓ {} ({}) [outdated: {}] at {}",
                                pm.name, pm.version, latest, pm.path
                            );
                        } else {
                            println!("  ✓ {} ({}) at {}", pm.name, pm.version, pm.path);
                        }
                    }
                    println!();
                }
            }
        },
    }

    // Summary
    let broken_count = broken_pms.len();
    let outdated_count = outdated_map.len();
    let healthy_count = all_pms.len() - broken_count - outdated_count;

    println!(
        "\nSummary: {} total, {} healthy, {} broken, {} outdated",
        all_pms.len(),
        healthy_count,
        broken_count,
        outdated_count
    );

    if broken_count > 0 || outdated_count > 0 {
        println!();
        if broken_count > 0 {
            println!("Run 'bole check -b' for broken PM diagnostics");
        }
        if outdated_count > 0 {
            println!("Run 'bole check -o' for update information");
        }
    }

    if broken_count > 0 {
        std::process::exit(1);
    }
}

/// Shows detailed diagnostics for broken package managers
fn handle_broken_check(all_pms: &[pm::PmInfo]) {
    let broken_pms: Vec<_> = all_pms
        .iter()
        .filter(|pm| pm.version.trim().is_empty())
        .collect();

    if broken_pms.is_empty() {
        println!("No broken package managers found.");
        std::process::exit(0);
    }

    println!("Broken package managers detected:\n");

    for pm in &broken_pms {
        println!("BROKEN: {} at {}", pm.name, pm.path);

        // Show diagnostics
        if !std::path::Path::new(&pm.path).exists() {
            println!("  Issue: Binary not found at expected path");
            println!("  Fix: Reinstall {} or update PATH", pm.name);
        } else {
            println!("  Issue: Binary exists but version check failed");
            println!(
                "  Fix: Check if {} is properly installed or corrupted",
                pm.name
            );

            // Check file permissions on Unix
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

/// Shows update information for outdated package managers
fn handle_outdated_check(all_pms: &[pm::PmInfo]) {
    let outdated_pms: Vec<(&pm::PmInfo, bole::find::Bump)> = all_pms
        .iter()
        .filter_map(|pm| {
            if pm.version.trim().is_empty() {
                return None;
            }

            let detector = pm::all_package_managers()
                .into_iter()
                .find(|d| d.name() == pm.name)?;

            if let Some(bump) = detector.check_bump(&pm.version)
                && bump.latest != pm.version
            {
                return Some((pm, bump));
            }
            None
        })
        .collect();

    if outdated_pms.is_empty() {
        println!("All package managers are up to date.");
        std::process::exit(0);
    }

    println!("Outdated package managers:\n");

    for (pm, bump) in &outdated_pms {
        println!("{}: {} → {}", pm.name, pm.version, bump.latest);
        println!("  Update: {}", bump.cmd);
        println!();
    }

    println!("Total outdated: {}", outdated_pms.len());
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
