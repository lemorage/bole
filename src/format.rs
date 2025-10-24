//! Output formatting.

use bole::pm::{GroupedPmInfo, PmInfo};

pub(crate) mod csv;
pub(crate) mod json;
pub(crate) mod table;
pub(crate) mod tree;

/// Supported output formats.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Format {
    Json,
    Csv,
    Table,
    Tree,
}

/// Common interface for all formatters.
pub(crate) trait Formatter {
    /// Format a list of package managers.
    fn format_pms(&self, pms: &[PmInfo]) -> String;

    /// Format grouped package managers.
    fn format_grouped(&self, grouped: &[GroupedPmInfo]) -> String;
}

impl Format {
    /// Determine format from CLI flags.
    pub(crate) fn from_flags(json: bool, csv: bool, tree: bool) -> Self {
        match (json, csv, tree) {
            (true, _, _) => Self::Json,
            (_, true, _) => Self::Csv,
            (_, _, true) => Self::Tree,
            _ => Self::Table,
        }
    }
}

impl Formatter for Format {
    fn format_pms(&self, pms: &[PmInfo]) -> String {
        match self {
            Self::Json => json::JsonFormatter.format_pms(pms),
            Self::Csv => csv::CsvFormatter.format_pms(pms),
            Self::Table => table::TableFormatter.format_pms(pms),
            Self::Tree => tree::TreeFormatter.format_pms(pms),
        }
    }

    fn format_grouped(&self, grouped: &[GroupedPmInfo]) -> String {
        match self {
            Self::Json => json::JsonFormatter.format_grouped(grouped),
            Self::Csv => csv::CsvFormatter.format_grouped(grouped),
            Self::Table => table::TableFormatter.format_grouped(grouped),
            Self::Tree => tree::TreeFormatter.format_grouped(grouped),
        }
    }
}
