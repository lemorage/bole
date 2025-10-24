//! Table output formatter using tabled.

use bole::pm::{GroupedPmInfo, PmInfo};
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
}
