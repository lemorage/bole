pub mod path;
pub mod query;

use std::path::Path;

use crate::pm::types::InstallMethod;

/// Main entry point for installation source attribution
///
/// Strategy: Query authoritative sources first, fall back to path analysis
pub fn determine(path: &Path) -> InstallMethod {
    let tool_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");

    if let Some(method) = query::resolve(tool_name, path) {
        return method;
    }

    path::detect(path)
}
