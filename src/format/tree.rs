//! Tree output formatter.

use std::collections::HashMap;

use bole::pm::{GroupedPmInfo, PmInfo};

use crate::format::Formatter;

/// Tree formatter implementation.
pub(super) struct TreeFormatter;

impl Formatter for TreeFormatter {
    fn format_pms(&self, pms: &[PmInfo]) -> String {
        if pms.is_empty() {
            return String::from("No package managers found.");
        }

        let mut out = String::new();
        let mut grouped: HashMap<String, Vec<&PmInfo>> = HashMap::new();

        // Group by name
        for pm in pms {
            grouped.entry(pm.name.clone()).or_default().push(pm);
        }

        // Sort for consistent output
        let mut sorted: Vec<_> = grouped.into_iter().collect();
        sorted.sort_by(|a, b| a.0.cmp(&b.0));

        for (i, (name, instances)) in sorted.iter().enumerate() {
            let is_last = i == sorted.len() - 1;
            let prefix = if is_last { "└──" } else { "├──" };

            // Only show instance count for duplicates
            let count = if instances.len() == 1 {
                String::new()
            } else {
                format!(" ({} instances)", instances.len())
            };

            out.push_str(&format!("{} {}{}\n", prefix, name, count));

            // Render each instance as a child node
            for (j, pm) in instances.iter().enumerate() {
                let is_last_child = j == instances.len() - 1;
                // Proper tree continuation lines
                let continuation = if is_last { "    " } else { "│   " };
                let child_prefix = if is_last_child {
                    "└──"
                } else {
                    "├──"
                };

                let status = if pm.version.is_empty() {
                    "broken"
                } else {
                    "✓"
                };

                out.push_str(&format!(
                    "{}{} {} v{} at {}\n",
                    continuation,
                    child_prefix,
                    status,
                    if pm.version.is_empty() {
                        "?"
                    } else {
                        &pm.version
                    },
                    pm.path
                ));
            }
        }

        out
    }

    fn format_grouped(&self, grouped: &[GroupedPmInfo]) -> String {
        if grouped.is_empty() {
            return String::from("No package managers found.");
        }

        let mut out = String::new();

        for (i, g) in grouped.iter().enumerate() {
            let is_last = i == grouped.len() - 1;
            let prefix = if is_last { "└──" } else { "├──" };

            out.push_str(&format!(
                "{} {} v{} {} [{}]\n",
                prefix, g.name, g.version, g.primary_path, g.install_method
            ));

            // Show duplicate paths as a sub-item
            if g.alternatives != "-" {
                let continuation = if is_last { "    " } else { "│   " };
                out.push_str(&format!("{}└── {}\n", continuation, g.alternatives));
            }
        }

        out
    }
}
