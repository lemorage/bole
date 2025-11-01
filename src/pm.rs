//! Package manager discovery and attribution.
//!
//! Core registry of supported package managers with installation source
//! detection.

mod core;
mod gleam;
mod go;
mod haskell;
mod js;
mod php;
mod python;
mod ruby;
mod rust;
mod system;
mod wrappers;
mod zig;

pub(crate) use core::search_paths::{get_search_locations, scan_common_directories};
pub use core::types::{Categorizable, Category, Detector, GroupedPmInfo, InstallMethod, PmInfo};
use std::{collections::HashSet, hash::Hash, path::Path, process::Command};

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
use which::which_all;
pub use wrappers::{Asdf, Corepack, Mise, Phpbrew, Pyenv, Volta};
pub use zig::Zig;

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

/// Determine how a package manager was installed.
#[inline]
pub fn determine_install_method(path: &Path) -> InstallMethod {
    core::attribution::determine(path)
}

/// Find all package manager instances.
#[must_use]
pub fn find_all_pms(name: &str) -> Vec<PmInfo> {
    find_all_pms_with_args(name, &["--version"])
}

/// Find all instances with custom version args.
#[must_use]
pub(super) fn find_all_pms_with_args(name: &str, version_args: &[&str]) -> Vec<PmInfo> {
    let cache = PM_DISCOVERY_CACHE.get_or_init(DashMap::new);
    let cache_key = CacheKey::new(name, version_args);

    if let Some(cached_results) = cache.get(&cache_key) {
        return cached_results.clone();
    }

    let results = find_all_installations(name, version_args);
    cache.insert(cache_key, results.clone());
    results
}

fn find_all_installations(name: &str, version_args: &[&str]) -> Vec<PmInfo> {
    let mut instances = Vec::new();
    let mut seen_canonical_paths = HashSet::new();

    // PATH discovery
    if let Ok(paths) = which_all(name) {
        for path in paths {
            if let Some(pm_info) = try_detect_at_path(&path, name, version_args)
                && let Ok(canonical) = std::fs::canonicalize(&path)
                && !seen_canonical_paths.contains(&canonical)
            {
                seen_canonical_paths.insert(canonical);
                instances.push(pm_info);
            }
        }
    }

    // Known installation locations
    let search_locations = get_search_locations(name);
    for location in search_locations {
        if location.exists()
            && let Ok(canonical_location) = std::fs::canonicalize(&location)
        {
            if seen_canonical_paths.contains(&canonical_location) {
                continue;
            }
            if let Some(pm_info) = try_detect_at_path(&location, name, version_args) {
                seen_canonical_paths.insert(canonical_location);
                instances.push(pm_info);
            }
        }
    }

    if instances.is_empty() {
        instances.extend(scan_common_directories(name, &mut seen_canonical_paths));
    }

    instances
}

/// Detect a package manager at a path.
fn try_detect_at_path(path: &std::path::Path, name: &str, version_args: &[&str]) -> Option<PmInfo> {
    let output = Command::new(path).args(version_args).output().ok()?;

    if !output.status.success() {
        return None;
    }

    let version = String::from_utf8(output.stdout).ok()?.trim().to_string();

    let install_method = determine_install_method(path);

    Some(PmInfo {
        name: name.to_string(),
        version,
        install_method,
        path: path.display().to_string(),
        latest_version: None,
    })
}

/// Get package manager names by category.
pub fn get_package_managers_in_category(category: Category) -> Vec<&'static str> {
    all_package_managers()
        .into_iter()
        .filter(|detector| detector.category() == category)
        .map(|detector| detector.name())
        .collect()
}

/// All supported package manager detectors.
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
        // Zig
        Box::new(Zig),
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
