//! Package manager discovery and attribution.
//!
//! Core registry of supported package managers with installation source
//! detection.

mod core;
mod gleam;
mod go;
mod haskell;
mod js;
mod ocaml;
mod php;
mod python;
mod ruby;
mod rust;
mod system;
mod versioner;
mod wrapper;
mod zig;

pub(crate) use core::{
    search_paths::{get_search_locations, scan_common_directories},
    version::normalize_version,
};
pub use core::{
    types::{
        Categorizable, Category, Detector, GroupedPmInfo, InstallMethod, PmInfo, Tool, ToolLister,
    },
    version::{UNKNOWN_VERSION, is_broken_version},
};
use std::{collections::HashSet, hash::Hash, path::Path, process::Command};

use dashmap::DashMap;
pub use gleam::Gleam;
pub use go::Go;
pub use haskell::{Cabal, Stack};
pub use js::{Bun, Deno, Npm, Pnpm, Yarn};
pub use ocaml::Opam;
pub use php::{Composer, Pecl};
pub use python::{Conda, Pdm, Pip, Pipenv, Pipx, Poetry, Uv};
pub use ruby::{Bundle, Bundler, Gem};
pub use rust::Cargo;
pub use system::{Homebrew, Macports, Nix};
pub use versioner::{Asdf, Mise, Nvm, Phpbrew, Pyenv, Rbenv, Rvm, Volta};
use which::which_all;
pub use wrapper::{Corepack, Ni};
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
    core::install_method::detect(path)
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

    let version = normalize_version(output.stdout);
    let install_method = determine_install_method(path);

    Some(PmInfo {
        name: name.to_string(),
        version,
        install_method,
        path: path.display().to_string(),
        latest_version: None,
    })
}

/// Scan all tools managed by all package managers.
/// Returns a map of package manager name to list of tools it manages.
pub fn scan_managed_tools() -> std::collections::HashMap<String, Vec<Tool>> {
    use std::collections::HashMap;

    let mut all_tools = HashMap::new();

    for lister in tool_listers() {
        if lister.is_available() {
            let tools = lister.list();
            if !tools.is_empty() {
                all_tools.insert(lister.name().to_string(), tools);
            }
        }
    }

    all_tools
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
        Box::new(Npm),
        Box::new(Pnpm),
        Box::new(Yarn),
        // OCaml
        Box::new(Opam),
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
        // Versioners
        Box::new(Asdf),
        Box::new(Mise),
        Box::new(Nvm),
        Box::new(Phpbrew),
        Box::new(Pyenv),
        Box::new(Rbenv),
        Box::new(Rvm),
        Box::new(Volta),
        // Wrappers
        Box::new(Corepack),
        Box::new(Ni),
    ]
}

/// All supported package managers that can list their installed tools.
pub fn tool_listers() -> Vec<Box<dyn ToolLister>> {
    vec![
        // JavaScript
        Box::new(Npm),
        Box::new(Yarn),
        Box::new(Pnpm),
        Box::new(Bun),
        // Python
        Box::new(Pip),
        Box::new(Pipx),
        // Rust
        Box::new(Cargo),
        // Go
        Box::new(Go),
    ]
}
