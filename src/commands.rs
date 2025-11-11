//! Command implementations for show, check, and own operations.

use std::collections::HashMap;

use bole::pm::{Category, PmInfo, Tool, all_package_managers, scan_managed_tools};

use crate::{
    discovery::Discovery,
    filters::{Filter, Grouper, Sorter},
    format::{Format, Formatter},
    network::{NetworkStatus, check_network_status},
    pipeline::pipe,
};

/// Configuration for the show command.
pub(crate) struct ShowCommand {
    category: Option<String>,
    all_paths: bool,
    format: Format,
}

impl ShowCommand {
    /// Create from CLI arguments.
    pub(crate) fn from_args(
        category: Option<String>,
        all: bool,
        tree: bool,
        json: bool,
        csv: bool,
    ) -> Self {
        Self {
            category,
            all_paths: all,
            format: Format::from_flags(json, csv, tree),
        }
    }

    /// Execute the show command.
    pub(crate) fn execute(self) {
        // Bail early on invalid category
        if let Some(ref cat) = self.category
            && !Discovery::is_valid_category(cat)
        {
            print_category_help(cat);
            return;
        }

        // Pipeline: discover -> sort -> format -> output
        let result = pipe(Discovery::from_optional_category(self.category))
            .then(|d| d.discover())
            .then(|pms| {
                if pms.is_empty() {
                    println!("No package managers found.");
                    vec![]
                } else {
                    Sorter::by_name(pms)
                }
            })
            .then(|pms| {
                if self.all_paths {
                    self.format.format_pms(&pms) // Show all instances
                } else {
                    let grouped = Grouper::by_name(pms); // Group by name
                    self.format.format_grouped(&grouped)
                }
            })
            .finish();

        if !result.is_empty() {
            println!("{}", result);
        }
    }
}

/// Verbosity level for check command.
#[derive(Debug, Clone, Copy)]
pub(crate) enum CheckMode {
    Summary,      // Quiet mode - just totals
    Progress,     // Normal mode - show progress
    Detailed,     // Verbose mode - full details
    BrokenOnly,   // Show only broken PMs
    OutdatedOnly, // Show only outdated PMs
}

impl CheckMode {
    /// Convert CLI flags to check mode.
    pub(crate) fn from_flags(verbose: u8, broken: bool, outdated: bool) -> Self {
        if broken {
            return Self::BrokenOnly;
        }
        if outdated {
            return Self::OutdatedOnly;
        }
        match verbose {
            0 => Self::Summary,
            1 => Self::Progress,
            _ => Self::Detailed,
        }
    }
}

/// Configuration for the check command.
pub(crate) struct CheckCommand {
    category: Option<String>,
    all_paths: bool,
    mode: CheckMode,
}

impl CheckCommand {
    /// Create from CLI arguments.
    pub(crate) fn from_args(
        category: Option<String>,
        all: bool,
        verbose: u8,
        broken: bool,
        outdated: bool,
    ) -> Self {
        Self {
            category,
            all_paths: all,
            mode: CheckMode::from_flags(verbose, broken, outdated),
        }
    }

    /// Execute the check command.
    /// Returns true if any broken PMs found.
    pub(crate) fn execute(self) -> bool {
        // Validate category if provided
        if let Some(ref cat) = self.category
            && !Discovery::is_valid_category(cat)
        {
            print_category_help(cat);
            return false;
        }

        // Discover package managers
        let pms = Discovery::from_optional_category(self.category.clone()).discover();
        if pms.is_empty() {
            println!("No package managers found.");
            return false;
        }

        // Filter to primary only unless showing all
        let pms = if self.all_paths {
            pms
        } else {
            Grouper::primary_only(pms)
        };

        // Execute based on check mode
        match self.mode {
            CheckMode::Summary => self.check_summary(&pms),
            CheckMode::Progress => self.check_with_progress(&pms),
            CheckMode::Detailed => self.check_detailed(&pms),
            CheckMode::BrokenOnly => self.check_broken_only(&pms),
            CheckMode::OutdatedOnly => {
                self.check_outdated_only(&pms);
                false // Outdated doesn't cause exit failure
            },
        }
    }

    /// Execute quiet check (summary only).
    fn check_summary(&self, pms: &[PmInfo]) -> bool {
        use std::time::Duration;

        use indicatif::{ProgressBar, ProgressStyle};

        // Check network first with spinner
        let network = check_network_with_spinner();

        // Now check package managers
        let spinner = ProgressBar::new_spinner();
        spinner.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg}")
                .unwrap(),
        );
        spinner.enable_steady_tick(Duration::from_millis(80));

        // Set message based on network status
        let message = match &network {
            NetworkStatus::Good => "Checking package managers...",
            NetworkStatus::Slow(_) => "Checking package managers (slow network)...",
            NetworkStatus::Offline => "Checking package managers (offline)...",
        };
        spinner.set_message(message);

        let stats = compute_stats(pms, &network);
        spinner.finish_and_clear();

        print_stats(&stats);
        stats.broken > 0
    }

    /// Execute normal check (with progress).
    fn check_with_progress(&self, pms: &[PmInfo]) -> bool {
        let network = check_network_with_spinner();

        println!("Checking package manager health...\n");

        // Display progress for each PM
        for pm in pms {
            print!("Checking {}... ", pm.name);

            if pm.version.trim().is_empty() {
                println!("{} broken", crate::color::cross_mark());
            } else if matches!(network, NetworkStatus::Offline) {
                println!(
                    "{} {} (version check skipped)",
                    crate::color::check_mark(),
                    pm.version
                );
            } else if let Some(bump) = get_bump_info(pm) {
                if bump.latest != pm.version {
                    println!(
                        "{} {} [outdated: {}]",
                        crate::color::check_mark(),
                        pm.version,
                        crate::color::warning(&bump.latest)
                    );
                } else {
                    println!("{} {}", crate::color::check_mark(), pm.version);
                }
            } else {
                println!("{} {}", crate::color::check_mark(), pm.version);
            }
        }

        println!();
        let stats = compute_stats(pms, &network);
        print_stats(&stats);
        stats.broken > 0
    }

    /// Execute verbose check (detailed output).
    fn check_detailed(&self, pms: &[PmInfo]) -> bool {
        use std::collections::HashMap;

        // Check network first with spinner
        let network = check_network_with_spinner();

        println!("Checking package manager health...\n");

        // Build category map
        let mut by_category: HashMap<Category, Vec<&PmInfo>> = HashMap::new();
        for pm in pms {
            let category = get_pm_category(&pm.name);
            by_category.entry(category).or_default().push(pm);
        }

        // Display in proper order
        for category in Category::all() {
            if let Some(cat_pms) = by_category.get(category) {
                println!("{}:", crate::color::bold(category.name()));
                for pm in cat_pms {
                    display_pm_detailed(pm);
                }
                println!();
            }
        }

        let stats = compute_stats(pms, &network);
        print_stats(&stats);
        stats.broken > 0
    }

    /// Display broken package managers.
    /// Returns true if any were found.
    fn check_broken_only(&self, pms: &[PmInfo]) -> bool {
        let broken = Filter::broken(pms.to_vec());

        if broken.is_empty() {
            println!(
                "{} No broken package managers found.",
                crate::color::check_mark()
            );
            return false;
        }

        println!(
            "{} Broken package managers detected:\n",
            crate::color::cross_mark()
        );

        for pm in &broken {
            println!(
                "{} {} at {}",
                crate::color::error("BROKEN:"),
                crate::color::bold(&pm.name),
                crate::color::dim(&pm.path)
            );
            diagnose_broken_pm(pm); // Diagnose the specific issue
            println!();
        }

        println!("Total broken: {}", broken.len());
        true
    }

    /// Display outdated package managers.
    fn check_outdated_only(&self, pms: &[PmInfo]) {
        let network = check_network_with_spinner();

        // Exit if offline
        if matches!(network, NetworkStatus::Offline) {
            return;
        }

        let outdated = Filter::outdated(pms.to_vec());

        // Get outdated with bump info
        let with_bumps: Vec<(&PmInfo, bole::find::Bump)> = outdated
            .iter()
            .filter_map(|pm| get_bump_info(pm).map(|b| (pm, b)))
            .filter(|(pm, bump)| bump.latest != pm.version)
            .collect();

        if with_bumps.is_empty() {
            println!(
                "{} All package managers are up to date.",
                crate::color::check_mark()
            );
            return;
        }

        println!(
            "{} Outdated package managers:\n",
            crate::color::warning_mark()
        );

        for (pm, bump) in &with_bumps {
            println!(
                "{}: {} → {}",
                crate::color::bold(&pm.name),
                crate::color::dim(&pm.version),
                crate::color::success(&bump.latest)
            );
            println!("  {} {}", crate::color::bold("Update:"), bump.cmd);
            println!();
        }

        println!(
            "Total outdated: {}",
            crate::color::warning(&with_bumps.len().to_string())
        );
    }
}

/// Check network with spinner feedback.
/// Shows spinner during check, updates message based on result.
fn check_network_with_spinner() -> NetworkStatus {
    use std::time::Duration;

    use indicatif::{ProgressBar, ProgressStyle};

    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    spinner.enable_steady_tick(Duration::from_millis(80));

    // Check network
    spinner.set_message("Checking network connectivity...");
    let network = check_network_status();

    // Update based on result
    match &network {
        NetworkStatus::Good => {
            spinner.finish_and_clear(); // Clear spinner, no message
        },
        NetworkStatus::Slow(duration) => {
            spinner.finish_and_clear();
            println!(
                "{} Network slow ({:.1}s)",
                crate::color::warning_mark(),
                duration.as_secs_f32()
            );
        },
        NetworkStatus::Offline => {
            spinner.finish_and_clear();
            println!(
                "{} Version checks will be skipped because you're offline.",
                crate::color::warning_mark()
            );
        },
    }

    network
}

/// Check result statistics.
struct Stats {
    total: usize,
    healthy: usize,
    broken: usize,
    outdated: usize,
}

/// Compute statistics for package managers.
/// Handles offline mode by skipping version checks.
fn compute_stats(pms: &[PmInfo], network: &NetworkStatus) -> Stats {
    // Count broken (no version)
    let broken = pms.iter().filter(|pm| pm.version.trim().is_empty()).count();

    // Count outdated (version differs from latest)
    let outdated = if matches!(network, NetworkStatus::Offline) {
        0 // Can't check updates when offline
    } else {
        pms.iter()
            .filter(|pm| !pm.version.trim().is_empty())
            .filter(|pm| {
                get_bump_info(pm)
                    .map(|bump| bump.latest != pm.version)
                    .unwrap_or(false)
            })
            .count()
    };

    Stats {
        total: pms.len(),
        healthy: pms.len() - broken - outdated,
        broken,
        outdated,
    }
}

/// Display summary statistics.
fn print_stats(stats: &Stats) {
    println!(
        "Summary: {} total, {} healthy, {} broken, {} outdated",
        stats.total,
        crate::color::success(&stats.healthy.to_string()),
        if stats.broken > 0 {
            crate::color::error(&stats.broken.to_string())
        } else {
            stats.broken.to_string()
        },
        if stats.outdated > 0 {
            crate::color::warning(&stats.outdated.to_string())
        } else {
            stats.outdated.to_string()
        }
    );

    // Show hints if issues found
    if stats.broken > 0 || stats.outdated > 0 {
        println!();
        if stats.broken > 0 {
            println!("Run 'bole check -b' for broken PM diagnostics");
        }
        if stats.outdated > 0 {
            println!("Run 'bole check -o' for update information");
        }
    }
}

/// Get update info for a package manager.
fn get_bump_info(pm: &PmInfo) -> Option<bole::find::Bump> {
    all_package_managers()
        .into_iter()
        .find(|d| d.name() == pm.name)
        .and_then(|d| d.check_bump(pm))
}

/// Get category for a package manager by name.
fn get_pm_category(name: &str) -> Category {
    all_package_managers()
        .into_iter()
        .find(|d| d.name() == name)
        .map(|d| d.category())
        .unwrap_or(Category::System)
}

/// Display verbose info for a package manager.
fn display_pm_detailed(pm: &PmInfo) {
    if pm.version.trim().is_empty() {
        // Broken PM
        println!(
            "  {} {} (broken) at {}",
            crate::color::cross_mark(),
            pm.name,
            crate::color::dim(&pm.path)
        );
    } else if let Some(bump) = get_bump_info(pm) {
        if bump.latest != pm.version {
            // Outdated PM
            println!(
                "  {} {} ({}) [outdated: {}] at {}",
                crate::color::check_mark(),
                pm.name,
                pm.version,
                crate::color::warning(&bump.latest),
                crate::color::dim(&pm.path)
            );
        } else {
            // Up-to-date PM
            println!(
                "  {} {} ({}) at {}",
                crate::color::check_mark(),
                pm.name,
                pm.version,
                crate::color::dim(&pm.path)
            );
        }
    } else {
        // No bump info available
        println!(
            "  {} {} ({}) at {}",
            crate::color::check_mark(),
            pm.name,
            pm.version,
            crate::color::dim(&pm.path)
        );
    }
}

/// Configuration for the own command.
/// Shows which package managers own which tools.
#[allow(unused)]
pub(crate) struct OwnCommand {
    tool_filter: String,
    format: Format,
}

impl OwnCommand {
    /// Create from CLI arguments.
    pub(crate) fn from_args(tool: String, tree: bool, json: bool, csv: bool) -> Self {
        Self {
            tool_filter: tool,
            format: Format::from_flags(json, csv, tree),
        }
    }

    /// Execute the own command.
    pub(crate) fn execute(self) {
        // Scan all managed tools
        let all_tools = scan_managed_tools();

        if all_tools.is_empty() {
            println!("No managed tools found on this system");
            return;
        }

        // Transform to tool-centric view
        let by_tool = group_by_tool_name(all_tools);

        // Apply filter by tool name
        let tools_to_show: HashMap<String, Vec<Tool>> = by_tool
            .into_iter()
            .filter(|(tool_name, _)| {
                tool_name
                    .to_lowercase()
                    .contains(&self.tool_filter.to_lowercase())
            })
            .collect();

        if tools_to_show.is_empty() {
            println!("No tools matching '{}' found", self.tool_filter);
            return;
        }

        display_tool_ownership(&tools_to_show);
    }
}

/// Transform PM-centric data into tool-centric data.
fn group_by_tool_name(pm_tools: HashMap<String, Vec<Tool>>) -> HashMap<String, Vec<Tool>> {
    let mut by_tool: HashMap<String, Vec<Tool>> = HashMap::new();

    for (_manager, tools) in pm_tools {
        for tool in tools {
            by_tool.entry(tool.name.clone()).or_default().push(tool);
        }
    }

    by_tool
}

/// Display tools with their owning package managers.
fn display_tool_ownership(tools: &HashMap<String, Vec<Tool>>) {
    let mut sorted_tools: Vec<_> = tools.iter().collect();
    sorted_tools.sort_by_key(|(name, _)| name.as_str());

    // Table header
    println!("{:<30} {:<15} {:<15}", "TOOL", "VERSION", "MANAGER");
    println!("{}", "-".repeat(60));

    for (tool_name, instances) in sorted_tools {
        for tool in instances {
            println!(
                "{:<30} {:<15} {:<15}",
                tool_name, tool.version, tool.manager
            );
        }
    }
}

/// Diagnose why a PM is broken.
fn diagnose_broken_pm(pm: &PmInfo) {
    use std::path::Path;

    if !Path::new(&pm.path).exists() {
        println!("  Issue: Binary not found at expected path");
        println!("  Fix: Reinstall {} or update PATH", pm.name);
    } else {
        println!("  Issue: Binary exists but version check failed");
        println!(
            "  Fix: Check if {} is properly installed or corrupted",
            pm.name
        );

        // Unix-specific: check execute permissions
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
}

/// Print category help.
fn print_category_help(unknown: &str) {
    use rayon::prelude::*;
    use tabled::{Table, Tabled, settings::Style};

    #[derive(Tabled)]
    struct CategoryRow {
        #[tabled(rename = "Category")]
        category: String,
        #[tabled(rename = "Managers")]
        managers: String,
        #[tabled(rename = "Aliases")]
        aliases: String,
    }

    println!("Unknown category '{}'.", unknown);
    println!("\nAvailable categories:");

    let rows: Vec<CategoryRow> = Category::all()
        .par_iter()
        .map(|&category| {
            let mut tools: Vec<&str> = all_package_managers()
                .iter()
                .filter(|d| d.category() == category)
                .map(|d| d.name())
                .collect();
            tools.sort_unstable();

            CategoryRow {
                category: category.name().to_string(),
                managers: if tools.is_empty() {
                    "-".to_string()
                } else {
                    tools.join(", ")
                },
                aliases: if category.aliases().is_empty() {
                    "-".to_string()
                } else {
                    category.aliases().join(", ")
                },
            }
        })
        .collect();

    let mut table = Table::new(rows);
    println!("{}", table.with(Style::modern()));
    println!("\nHint: use 'bole show <category>' to filter.");
}
