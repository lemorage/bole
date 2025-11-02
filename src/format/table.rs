//! Table output formatter using tabled.

use std::{collections::HashMap, fmt::Write};

use bole::pm::{GroupedPmInfo, PmInfo, Tool};
use tabled::{Table, settings::Style};

use crate::format::Formatter;

/// Table formatter implementation.
pub(super) struct TableFormatter;

impl Formatter for TableFormatter {
    fn format_pms(&self, pms: &[PmInfo]) -> String {
        if pms.is_empty() {
            return String::from("No package managers found.");
        }

        let mut table = Table::new(pms);
        table.with(Style::modern()).to_string()
    }

    fn format_grouped(&self, grouped: &[GroupedPmInfo]) -> String {
        if grouped.is_empty() {
            return String::from("No package managers found.");
        }

        let mut table = Table::new(grouped);
        table.with(Style::modern()).to_string()
    }

    fn format_tools(&self, tools: &HashMap<String, Vec<Tool>>) -> String {
        if tools.is_empty() {
            return String::from("No installed tools found.");
        }

        let mut output = String::new();
        writeln!(output, "{:<30} {:<15} {:<15}", "TOOL", "VERSION", "MANAGER").unwrap();
        writeln!(output, "{}", "-".repeat(60)).unwrap();

        for (manager, tool_list) in tools {
            for tool in tool_list {
                writeln!(
                    output,
                    "{:<30} {:<15} {:<15}",
                    tool.name, tool.version, manager
                )
                .unwrap();
            }
        }

        output
    }
}
