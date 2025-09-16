//! Package manager discovery and attribution.
//!
//! This module provides the core registry of all supported package managers and
//! determines how each tool was installed on the system.

mod attribution;
mod gleam;
mod go;
mod haskell;
mod js;
mod php;
mod python;
mod ruby;
mod rust;
mod search_paths;
mod system;
mod types;
mod wrappers;

use std::{collections::HashSet, hash::Hash, path::Path, process::Command};

pub use attribution::query::{Querier, Resolver, Tool};
use dashmap::DashMap;
pub use gleam::Gleam;
pub use go::Go;
pub use haskell::{Cabal, Stack};
pub use js::{Bun, Deno, Ni, Npm, Pnpm, Yarn};
pub use php::{Composer, Pecl};
pub use python::{Conda, Pdm, Pip, Pipenv, Pipx, Poetry, Uv};
pub use ruby::{Bundle, Bundler, Gem, Rbenv, Rvm};
pub use rust::Cargo;
pub use system::{Homebrew, Macports, Nix};
pub use types::{Categorizable, Category, Detector, GroupedPmInfo, InstallMethod, PmInfo};
use which::which;
pub use wrappers::{Asdf, Corepack, Mise, Phpbrew, Pyenv, Volta};

use crate::pm::search_paths::{get_search_locations, scan_common_directories};

/// Cache key for discovery results.
#[derive(Hash, Eq, PartialEq, Clone)]
struct CacheKey {
    name: String,
    version_args: Vec<String>,
}

impl CacheKey {
    fn new(name: &str, version_args: &[&str]) -> Self {
        Self {
            name: name.to_string(),
            version_args: version_args.iter().map(|s| s.to_string()).collect(),
        }
    }
}

static PM_DISCOVERY_CACHE: std::sync::OnceLock<DashMap<CacheKey, Vec<PmInfo>>> =
    std::sync::OnceLock::new();

/// Infer install method from an executable path.
#[inline]
pub fn determine_install_method(path: &Path) -> InstallMethod {
    attribution::determine(path)
}

/// Find all installations of a package manager.
///
/// Searches PATH and known locations; deduplicates by canonical path.
#[must_use]
pub fn find_all_pms(name: &str) -> Vec<PmInfo> {
    find_all_pms_with_args(name, &["--version"])
}

/// Find all installations using custom version arguments.
#[must_use]
pub(crate) fn find_all_pms_with_args(name: &str, version_args: &[&str]) -> Vec<PmInfo> {
    let cache = PM_DISCOVERY_CACHE.get_or_init(DashMap::new);
    let cache_key = CacheKey::new(name, version_args);

    if let Some(cached_results) = cache.get(&cache_key) {
        return cached_results.clone();
    }

    let results = exhaustive_discovery(name, version_args);
    cache.insert(cache_key, results.clone());
    results
}

fn exhaustive_discovery(name: &str, version_args: &[&str]) -> Vec<PmInfo> {
    let mut instances = Vec::new();
    let mut seen_canonical_paths = HashSet::new();

    // PATH discovery
    if let Ok(path) = which(name) {
        if let Some(pm_info) = try_detect_at_path(&path, name, version_args) {
            if let Ok(canonical) = std::fs::canonicalize(&path) {
                seen_canonical_paths.insert(canonical);
                instances.push(pm_info);
            }
        }
    }

    // Known installation locations
    let search_locations = get_search_locations(name);
    for location in search_locations {
        if location.exists() {
            if let Ok(canonical_location) = std::fs::canonicalize(&location) {
                if seen_canonical_paths.contains(&canonical_location) {
                    continue;
                }
                if let Some(pm_info) = try_detect_at_path(&location, name, version_args) {
                    seen_canonical_paths.insert(canonical_location);
                    instances.push(pm_info);
                }
            }
        }
    }

    if instances.is_empty() {
        instances.extend(scan_common_directories(name, &mut seen_canonical_paths));
    }

    instances
}

/// Detect a package manager at a path with timeout protection.
fn try_detect_at_path(path: &std::path::Path, name: &str, version_args: &[&str]) -> Option<PmInfo> {
    use std::{sync::mpsc, thread, time::Duration};

    let (tx, rx) = mpsc::channel();
    let path_clone = path.to_path_buf();
    let args_clone: Vec<String> = version_args.iter().map(|s| s.to_string()).collect();

    // Spawn command in separate thread to enable timeout
    thread::spawn(move || {
        let result = Command::new(&path_clone)
            .args(&args_clone)
            .output()
            .map_err(|_| ())
            .and_then(|output| {
                String::from_utf8(output.stdout)
                    .map(|s| s.trim().to_string())
                    .map_err(|_| ())
            });
        let _ = tx.send(result);
    });

    // Wait for result with 3-second timeout
    let version = match rx.recv_timeout(Duration::from_secs(3)) {
        Ok(Ok(version)) => version,
        Ok(Err(_)) | Err(_) => return None, // Command failed or timed out
    };

    let install_method = determine_install_method(path);

    Some(PmInfo {
        name: name.to_string(),
        version,
        install_method,
        // Use original path for display (before canonicalization)
        path: path.display().to_string(),
    })
}

/// List package manager names in a category.
pub fn get_package_managers_in_category(category: Category) -> Vec<&'static str> {
    all_package_managers()
        .into_iter()
        .filter(|detector| detector.category() == category)
        .map(|detector| detector.name())
        .collect()
}

/// Return detectors for all supported package managers.
pub fn all_package_managers() -> Vec<Box<dyn Detector>> {
    vec![
        // System
        Box::new(Homebrew),
        Box::new(Macports),
        Box::new(Nix),
        // Gleam
        Box::new(Gleam),
        // Go
        Box::new(Go),
        // Haskell
        Box::new(Cabal),
        Box::new(Stack),
        // JavaScript/TypeScript
        Box::new(Bun),
        Box::new(Deno),
        Box::new(Ni),
        Box::new(Npm),
        Box::new(Pnpm),
        Box::new(Yarn),
        // PHP
        Box::new(Composer),
        Box::new(Pecl),
        // Python
        Box::new(Conda),
        Box::new(Pdm),
        Box::new(Pip),
        Box::new(Pipenv),
        Box::new(Poetry),
        Box::new(Uv),
        // Ruby
        Box::new(Bundle),
        Box::new(Bundler),
        Box::new(Gem),
        // Rust
        Box::new(Cargo),
        // Universal Wrappers
        Box::new(Asdf),
        Box::new(Volta),
        Box::new(Mise),
        Box::new(Corepack),
        Box::new(Phpbrew),
        Box::new(Pyenv),
        Box::new(Rbenv),
        Box::new(Rvm),
    ]
}
