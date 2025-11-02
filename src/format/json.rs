//! JSON output formatter.

use std::collections::HashMap;

use bole::pm::{GroupedPmInfo, PmInfo, Tool};

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

    fn format_tools(&self, tools: &HashMap<String, Vec<Tool>>) -> String {
        serde_json::to_string_pretty(tools)
            .unwrap_or_else(|e| format!("{{\"error\": \"Failed to serialize: {}\"}}", e))
    }
}
