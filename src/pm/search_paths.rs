use std::{collections::HashSet, path::PathBuf};

use crate::pm::{PmInfo, try_detect_at_path};

/// Standard fallback patterns for unknown package managers.
const FALLBACK_PATTERNS: &[&str] = &[
    "/usr/bin/{name}",
    "/opt/homebrew/bin/{name}",
    "/usr/local/bin/{name}",
    "~/.local/bin/{name}",
    "~/.cargo/bin/{name}",
    "~/.{name}/bin/{name}",
];

/// Expand home directory in path.
fn expand_home(path: &str) -> PathBuf {
    if path.starts_with("~/") {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home + &path[1..])
        } else {
            PathBuf::from(path)
        }
    } else {
        PathBuf::from(path)
    }
}

/// Find detector by name, fallback to generic paths.
pub(super) fn get_search_locations(name: &str) -> Vec<PathBuf> {
    // Try to find the detector by name (O(n))
    if let Some(detector) = crate::pm::all_package_managers()
        .iter()
        .find(|detector| detector.name() == name)
    {
        return detector
            .search_paths()
            .iter()
            .map(|&path| expand_home(path))
            .collect();
    }

    // Fallback to generic patterns for unknown tools
    FALLBACK_PATTERNS
        .iter()
        .map(|pattern| expand_home(&pattern.replace("{name}", name)))
        .collect()
}

/// Scan common directories for `name` and detect installations.
///
/// Skips paths already present in `seen_paths` (by canonical path).
pub(super) fn scan_common_directories(
    name: &str,
    seen_paths: &mut HashSet<PathBuf>,
) -> Vec<PmInfo> {
    let mut results = Vec::new();

    for path in get_search_locations(name) {
        if path.exists()
            && let Ok(canonical) = std::fs::canonicalize(&path)
            && !seen_paths.contains(&canonical)
            && let Some(pm_info) = try_detect_at_path(&path, name, &["--version"])
        {
            seen_paths.insert(canonical);
            results.push(pm_info);
        }
    }
    results
}
