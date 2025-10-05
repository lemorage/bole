//! Command-line interface handling.

use std::{sync::mpsc, thread};

use bole::pm::{self, Category};
use rayon::prelude::*;
use tabled::{Table, Tabled, settings::Style};

use crate::{
    color,
    display::{
        OutputFormat, display_grouped_tree, display_tree, group_pm_instances, output_csv,
        output_grouped_csv, output_grouped_json, output_json,
    },
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
    // Determine target categories
    let target_categories = match category.as_ref() {
        Some(cat_str) => match parse_category(cat_str) {
            Some(cat) => vec![cat],
            None => {
                print_category_help(cat_str);
                return;
            },
        },
        None => Category::all().to_vec(),
    };

    // Discover package managers for target categories
    let filtered_pms: Vec<pm::PmInfo> = pm::all_package_managers()
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

    match OutputFormat::from_flags(json, csv) {
        OutputFormat::Json => handle_json_output(filtered_pms, all),
        OutputFormat::Csv => handle_csv_output(filtered_pms, all),
        OutputFormat::Table => handle_table_output(filtered_pms, all, tree),
    }
}

/// Handles JSON output format
fn handle_json_output(pms: Vec<pm::PmInfo>, all: bool) {
    if all {
        if let Err(e) = output_json(&pms) {
            eprintln!("Error outputting JSON: {}", e);
        }
    } else {
        let grouped = group_pm_instances(pms);
        if let Err(e) = output_grouped_json(&grouped) {
            eprintln!("Error outputting JSON: {}", e);
        }
    }
}

/// Handles CSV output format
fn handle_csv_output(pms: Vec<pm::PmInfo>, all: bool) {
    if all {
        output_csv(&pms);
    } else {
        let grouped = group_pm_instances(pms);
        output_grouped_csv(&grouped);
    }
}

/// Handles table/tree output format
fn handle_table_output(pms: Vec<pm::PmInfo>, all: bool, tree: bool) {
    match (all, tree) {
        (false, false) => {
            // Default: grouped table
            let mut table = Table::new(group_pm_instances(pms));
            println!("{}", table.with(Style::modern()));
            println!("\nTip: Use --all to see all the other locations");
        },
        (true, false) => {
            // All instances table
            let mut table = Table::new(pms);
            println!("{}", table.with(Style::modern()));
        },
        (false, true) => {
            // Grouped tree
            display_grouped_tree(group_pm_instances(pms));
            println!("\nTip: Use --all to see all the other locations");
        },
        (true, true) => {
            // All instances tree
            display_tree(pms);
        },
    }
}

/// Handles the check command for package manager health analysis.
pub(super) fn handle_check_command(verbose: u8, broken: bool, outdated: bool) {
    // Handle special check modes
    match (broken, outdated) {
        (false, false) => { /* fall through to verbosity handling */ },
        _ => {
            // For --broken and --outdated, we need all PMs first
            let all_pms: Vec<pm::PmInfo> = pm::all_package_managers()
                .into_par_iter()
                .flat_map(|detector| detector.find())
                .collect();

            if all_pms.is_empty() {
                println!("No package managers found.");
                return;
            }

            match (broken, outdated) {
                (true, false) => {
                    if handle_broken_check(&all_pms) {
                        std::process::exit(1);
                    }
                },
                (false, true) => handle_outdated_check(&all_pms),
                (true, true) => {
                    let has_broken = handle_broken_check(&all_pms);
                    println!();
                    handle_outdated_check(&all_pms);
                    if has_broken {
                        std::process::exit(1);
                    }
                },
                _ => unreachable!(),
            }
            return;
        },
    }

    // Regular check with verbosity
    println!("Checking package manager health...\n");

    match Verbosity::from(verbose) {
        Verbosity::Quiet => handle_quiet_check(),
        Verbosity::Normal => handle_normal_check(),
        Verbosity::Verbose => handle_verbose_check(),
    }
}

fn handle_quiet_check() {
    let all_pms: Vec<pm::PmInfo> = pm::all_package_managers()
        .into_par_iter()
        .flat_map(|detector| detector.find())
        .collect();

    display_summary(&all_pms);
}

fn handle_normal_check() {
    let (tx, rx) = mpsc::channel();
    let detectors = pm::all_package_managers();

    let handle = thread::spawn(move || {
        detectors.into_par_iter().for_each(|detector| {
            let results = detector.find();
            for pm in results {
                let _ = tx.send(pm);
            }
        });
    });

    let mut all_pms = Vec::new();
    for pm in rx {
        display_pm_progress(&pm);
        all_pms.push(pm);
    }

    handle.join().unwrap();
    println!();
    display_summary(&all_pms);
}

fn handle_verbose_check() {
    // Simple solution: collect all, then display by category
    // This is NOT progressive, but it's CORRECT and SIMPLE
    let all_pms: Vec<pm::PmInfo> = pm::all_package_managers()
        .into_par_iter()
        .flat_map(|detector| detector.find())
        .collect();

    // Build category map
    let category_map: std::collections::HashMap<String, Category> = pm::all_package_managers()
        .into_iter()
        .map(|d| (d.name().to_string(), d.category()))
        .collect();

    // Group by category
    let mut by_category: std::collections::HashMap<Category, Vec<&pm::PmInfo>> =
        std::collections::HashMap::new();
    for pm in &all_pms {
        let category = category_map
            .get(&pm.name)
            .copied()
            .unwrap_or(Category::System);
        by_category.entry(category).or_default().push(pm);
    }

    // Display in proper order
    for category in Category::all() {
        if let Some(pms) = by_category.get(category) {
            println!("{}:", color::bold(category.name()));
            for pm in pms {
                display_pm_verbose(pm);
            }
            println!();
        }
    }

    display_summary(&all_pms);
}

fn display_pm_verbose(pm: &pm::PmInfo) {
    if pm.version.trim().is_empty() {
        println!(
            "  {} {} (broken) at {}",
            color::cross_mark(),
            pm.name,
            color::dim(&pm.path)
        );
    } else {
        // Check if outdated
        let detector = pm::all_package_managers()
            .into_iter()
            .find(|d| d.name() == pm.name);

        if let Some(d) = detector
            && let Some(bump) = d.check_bump(&pm.version)
            && bump.latest != pm.version
        {
            println!(
                "  {} {} ({}) [outdated: {}] at {}",
                color::check_mark(),
                pm.name,
                pm.version,
                color::warning(&bump.latest),
                color::dim(&pm.path)
            );
            return;
        }

        println!(
            "  {} {} ({}) at {}",
            color::check_mark(),
            pm.name,
            pm.version,
            color::dim(&pm.path)
        );
    }
}

fn display_pm_progress(pm: &pm::PmInfo) {
    print!("Checking {}... ", pm.name);

    if pm.version.trim().is_empty() {
        println!("{} broken", color::cross_mark());
    } else {
        // Check if outdated
        let detector = pm::all_package_managers()
            .into_iter()
            .find(|d| d.name() == pm.name);

        if let Some(d) = detector
            && let Some(bump) = d.check_bump(&pm.version)
            && bump.latest != pm.version
        {
            println!(
                "{} {} [outdated: {}]",
                color::check_mark(),
                pm.version,
                color::warning(&bump.latest)
            );
            return;
        }

        println!("{} {}", color::check_mark(), pm.version);
    }
}

fn display_summary(all_pms: &[pm::PmInfo]) {
    // Count broken and outdated
    let broken_count = all_pms
        .iter()
        .filter(|pm| pm.version.trim().is_empty())
        .count();

    let outdated_count = all_pms
        .iter()
        .filter(|pm| !pm.version.trim().is_empty())
        .filter(|pm| {
            pm::all_package_managers()
                .into_iter()
                .find(|d| d.name() == pm.name)
                .and_then(|d| d.check_bump(&pm.version))
                .map(|bump| bump.latest != pm.version)
                .unwrap_or(false)
        })
        .count();

    let healthy_count = all_pms.len() - broken_count - outdated_count;

    println!(
        "Summary: {} total, {} healthy, {} broken, {} outdated",
        all_pms.len(),
        color::success(&healthy_count.to_string()),
        if broken_count > 0 {
            color::error(&broken_count.to_string())
        } else {
            broken_count.to_string()
        },
        if outdated_count > 0 {
            color::warning(&outdated_count.to_string())
        } else {
            outdated_count.to_string()
        }
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

/// Shows detailed diagnostics for broken package managers.
/// Returns true if any broken PMs were found.
fn handle_broken_check(all_pms: &[pm::PmInfo]) -> bool {
    let broken_pms: Vec<_> = all_pms
        .iter()
        .filter(|pm| pm.version.trim().is_empty())
        .collect();

    if broken_pms.is_empty() {
        println!("{} No broken package managers found.", color::check_mark());
        return false;
    }

    println!(
        "{} Broken package managers detected:\n",
        color::cross_mark()
    );

    for pm in &broken_pms {
        println!(
            "{} {} at {}",
            color::error("BROKEN:"),
            color::bold(&pm.name),
            color::dim(&pm.path)
        );

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
    !broken_pms.is_empty()
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
        println!(
            "{} All package managers are up to date.",
            color::check_mark()
        );
        std::process::exit(0);
    }

    println!("{} Outdated package managers:\n", color::warning_mark());

    for (pm, bump) in &outdated_pms {
        println!(
            "{}: {} → {}",
            color::bold(&pm.name),
            color::dim(&pm.version),
            color::success(&bump.latest)
        );
        println!("  {} {}", color::bold("Update:"), bump.cmd);
        println!();
    }

    println!(
        "Total outdated: {}",
        color::warning(&outdated_pms.len().to_string())
    );
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
