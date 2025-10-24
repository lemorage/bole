//! CSV output formatter (RFC 4180 compliant).

use bole::pm::{GroupedPmInfo, PmInfo};

use crate::format::Formatter;

/// CSV formatter implementation.
pub(super) struct CsvFormatter;

impl CsvFormatter {
    /// Escape CSV field according to RFC 4180.
    fn escape(field: &str) -> String {
        if field.contains(',') || field.contains('"') || field.contains('\n') {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }
}

impl Formatter for CsvFormatter {
    fn format_pms(&self, pms: &[PmInfo]) -> String {
        let mut out = String::from("Name,Version,Path,Via\n");

        for pm in pms {
            out.push_str(&format!(
                "{},{},{},\"{}\"\n",
                Self::escape(&pm.name),
                Self::escape(&pm.version),
                Self::escape(&pm.path),
                pm.install_method
            ));
        }

        out
    }

    fn format_grouped(&self, grouped: &[GroupedPmInfo]) -> String {
        let mut out = String::from("Name,Version,Active Path,Via,Others\n");

        for g in grouped {
            let others = if g.alternative_paths.is_empty() {
                String::new()
            } else {
                g.alternative_paths.join(":")
            };

            out.push_str(&format!(
                "{},{},{},\"{}\",\"{}\"\n",
                Self::escape(&g.name),
                Self::escape(&g.version),
                Self::escape(&g.primary_path),
                g.install_method,
                others
            ));
        }

        out
    }
}
