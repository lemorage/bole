//! Installation source attribution system.
//!
//! Determines how package managers were installed by combining authoritative
//! querying with path-based heuristics.

pub mod path;
pub mod query;

use std::path::Path;

use crate::pm::core::types::InstallMethod;

/// Determines installation source using query-first, path-fallback strategy.
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
