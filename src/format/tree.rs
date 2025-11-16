//! Tree output formatter.

use std::collections::HashMap;

use bole::pm::{GroupedPmInfo, PmInfo, Tool};

use crate::{color, format::Formatter};

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

            // Show instance count
            let count = if instances.len() == 1 {
                " (1 installation)".to_string()
            } else {
                format!(" ({} installations)", instances.len())
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

                // First instance is active (first in PATH)
                let is_active = j == 0;
                let indicator = if is_active { "✓" } else { "-" };

                // Format path info with colors
                let path_info = format!("{} v{} [{}]", pm.path, pm.version, pm.install_method);
                let formatted_path = if is_active {
                    color::success(&path_info)
                } else {
                    color::dim(&path_info)
                };

                out.push_str(&format!(
                    "{} {} {} {}\n",
                    continuation, child_prefix, indicator, formatted_path
                ));
            }
        }

        out
    }

    fn format_grouped(&self, grouped: &[GroupedPmInfo]) -> String {
        if grouped.is_empty() {
            return String::from("No package managers found.");
        }

        let mut out =
            String::from("Showing active package managers (use --all-paths to see duplicates)\n\n");

        for (i, g) in grouped.iter().enumerate() {
            let is_last = i == grouped.len() - 1;
            let prefix = if is_last { "└──" } else { "├──" };

            out.push_str(&format!(
                "{} {} v{} {} [{}]\n",
                prefix, g.name, g.version, g.active_path, g.install_method
            ));

            // Show count of alternatives as a sub-item
            if g.alternatives != "-" {
                let continuation = if is_last { "    " } else { "│   " };
                out.push_str(&format!(
                    "{}└── {}\n",
                    continuation,
                    color::dim(&g.alternatives)
                ));
            }
        }

        out
    }

    fn format_tools(&self, tools: &HashMap<String, Vec<Tool>>) -> String {
        if tools.is_empty() {
            return String::from("No installed tools found.");
        }

        let mut out = String::new();
        let mut sorted: Vec<_> = tools.iter().collect();
        sorted.sort_by_key(|(name, _)| name.as_str());

        for (idx, (tool_name, instances)) in sorted.iter().enumerate() {
            out.push_str(&format!("{}\n", tool_name));

            // List versions with their package managers
            for (i, tool) in instances.iter().enumerate() {
                let is_last = i == instances.len() - 1;
                let prefix = if is_last { "└──" } else { "├──" };
                out.push_str(&format!(
                    "  {} v{} via {}\n",
                    prefix, tool.version, tool.manager
                ));
            }

            if idx < sorted.len() - 1 {
                out.push('\n');
            }
        }

        out
    }
}
