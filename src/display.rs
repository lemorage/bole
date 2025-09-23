//! Output formatting for package manager information.

use std::collections::HashMap;

use bole::pm::{GroupedPmInfo, PmInfo};

/// Output formats for package manager information.
#[derive(Debug, Clone, Copy)]
pub(super) enum OutputFormat {
    Table,
    Json,
    Csv,
}

impl OutputFormat {
    /// Detect output format from flags.
    pub(super) fn from_flags(json: bool, csv: bool) -> Self {
        match (json, csv) {
            (true, _) => Self::Json,
            (_, true) => Self::Csv,
            _ => Self::Table,
        }
    }
}

/// Groups package manager instances by name for clean display.
pub(super) fn group_pm_instances(instances: Vec<PmInfo>) -> Vec<GroupedPmInfo> {
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
pub(super) fn display_tree(instances: Vec<PmInfo>) {
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
pub(super) fn display_grouped_tree(grouped: Vec<GroupedPmInfo>) {
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
pub(super) fn output_json(instances: &[PmInfo]) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(instances)?);
    Ok(())
}

/// Output grouped package manager instances in JSON format.
pub(super) fn output_grouped_json(
    grouped: &[GroupedPmInfo],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(grouped)?);
    Ok(())
}

/// Output package manager instances in CSV format.
pub(super) fn output_csv(instances: &[PmInfo]) {
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
pub(super) fn output_grouped_csv(grouped: &[GroupedPmInfo]) {
    println!("Name,Version,Path,Via,Others");
    for pm in grouped {
        println!(
            "{},{},{},\"{}\",\"{}\"",
            escape_csv(&pm.name),
            escape_csv(&pm.version),
            escape_csv(&pm.primary_path),
            pm.install_method,
            pm.alternatives
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
