//! Output formatting for package manager information.

use std::collections::HashMap;

use bole::pm::{self, GroupedPmInfo, PmInfo};

use crate::color;

/// Output formats for package manager information.
#[derive(Debug, Clone, Copy)]
pub(crate) enum OutputFormat {
    Table,
    Json,
    Csv,
}

impl OutputFormat {
    /// Detect output format from flags.
    pub(crate) fn from_flags(json: bool, csv: bool) -> Self {
        match (json, csv) {
            (true, _) => Self::Json,
            (_, true) => Self::Csv,
            _ => Self::Table,
        }
    }
}

/// Groups package manager instances by name for clean display.
pub(crate) fn group_pm_instances(instances: Vec<PmInfo>) -> Vec<GroupedPmInfo> {
    let mut grouped: HashMap<String, Vec<PmInfo>> = HashMap::new();

    for instance in instances {
        grouped
            .entry(instance.name.clone())
            .or_default()
            .push(instance);
    }

    let mut result: Vec<GroupedPmInfo> = grouped
        .into_iter()
        .map(|(name, instances)| GroupedPmInfo::from_instances(name, instances))
        .collect();

    result.sort_by(|a, b| a.name.cmp(&b.name));
    result
}

/// Displays package manager instances in tree format with full details.
pub(crate) fn display_tree(instances: Vec<PmInfo>) {
    let mut grouped: HashMap<String, Vec<PmInfo>> = HashMap::new();

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

/// Displays grouped package manager information in tree format.
pub(crate) fn display_grouped_tree(grouped: Vec<GroupedPmInfo>) {
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

/// Output package manager instances in JSON format.
pub(crate) fn output_json(instances: &[PmInfo]) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(instances)?);
    Ok(())
}

/// Output grouped package manager instances in JSON format.
pub(crate) fn output_grouped_json(
    grouped: &[GroupedPmInfo],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(grouped)?);
    Ok(())
}

/// Output package manager instances in CSV format.
pub(crate) fn output_csv(instances: &[PmInfo]) {
    println!("Name,Version,Path,Via");
    for pm in instances {
        println!(
            "{},{},{},\"{}\"",
            escape_csv(&pm.name),
            escape_csv(&pm.version),
            escape_csv(&pm.path),
            pm.install_method
        );
    }
}

/// Output grouped package manager instances in CSV format.
pub(crate) fn output_grouped_csv(grouped: &[GroupedPmInfo]) {
    println!("Name,Version,Path,Via,Others");
    for pm in grouped {
        let others = if pm.alternative_paths.is_empty() {
            String::new()
        } else {
            pm.alternative_paths.join(":")
        };
        println!(
            "{},{},{},\"{}\",\"{}\"",
            escape_csv(&pm.name),
            escape_csv(&pm.version),
            escape_csv(&pm.primary_path),
            pm.install_method,
            others
        );
    }
}

/// Escape CSV field if it contains commas or quotes.
fn escape_csv(field: &str) -> String {
    if field.contains(',') || field.contains('"') {
        format!("\"{}\"", field.replace('"', "\"\""))
    } else {
        field.to_string()
    }
}

/// Display detailed PM info with location for check -vv mode.
pub(crate) fn display_check_verbose(pm: &PmInfo) {
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

/// Display checking progress with status for check -v mode.
pub(crate) fn display_check_normal(pm: &PmInfo) {
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

/// Display health summary report for check command.
pub(crate) fn display_check_summary(all_pms: &[PmInfo], spinner: Option<indicatif::ProgressBar>) {
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

    // Clear spinner if present
    if let Some(spinner) = spinner {
        spinner.finish_and_clear();
    }

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
