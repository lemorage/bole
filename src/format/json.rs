//! JSON output formatter.

use bole::pm::{GroupedPmInfo, PmInfo};

use crate::format::Formatter;

/// JSON formatter implementation.
pub(super) struct JsonFormatter;

impl Formatter for JsonFormatter {
    fn format_pms(&self, pms: &[PmInfo]) -> String {
        serde_json::to_string_pretty(pms)
            .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize: {}\"}}", e))
    }

    fn format_grouped(&self, grouped: &[GroupedPmInfo]) -> String {
        serde_json::to_string_pretty(grouped)
            .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize: {}\"}}", e))
    }
}
