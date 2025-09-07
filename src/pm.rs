mod attribution;
mod gleam;
mod go;
mod haskell;
mod js;
mod python;
mod rust;
mod system;
mod types;
mod wrappers;

use std::{path::Path, process::Command};

pub use attribution::query::{Querier, Resolver, Tool};
pub use gleam::Gleam;
pub use go::Go;
pub use haskell::{Cabal, Stack};
pub use js::{Bun, Deno, Npm, Pnpm, Yarn};
pub use python::{Conda, Pdm, Pip, Pipx, Poetry, Uv};
pub use rust::Cargo;
pub use system::{Homebrew, Macports, Nix};
pub use types::{Categorizable, Category, Detector, GroupedPmInfo, InstallMethod, PmInfo};
use which::which;
pub use wrappers::{Asdf, Corepack, Mise, Volta};

/// Main entry point for determining how a package manager was installed
pub fn determine_install_method(path: &Path) -> InstallMethod {
    attribution::determine(path)
}

/// Enhanced detector that finds ALL instances of a package manager
#[must_use]
pub(crate) fn find_all_pms(name: &str) -> Vec<PmInfo> {
    find_all_pms_with_args(name, &["--version"])
}

/// Enhanced detector helper that finds ALL instances with custom version args
#[must_use]
pub(crate) fn find_all_pms_with_args(name: &str, version_args: &[&str]) -> Vec<PmInfo> {
    let mut instances = Vec::new();
    let mut seen_canonical_paths = std::collections::HashSet::new();

    // PATH discovery
    if let Ok(path) = which(name)
        && let Some(pm_info) = try_detect_at_path(&path, name, version_args)
    {
        // Get canonical path for deduplication
        if let Ok(canonical) = std::fs::canonicalize(&path) {
            seen_canonical_paths.insert(canonical);
        }
        instances.push(pm_info);
    }

    // Known installation locations
    let search_locations = get_search_locations_for(name);
    for location in search_locations {
        if let Ok(canonical_location) = std::fs::canonicalize(&location) {
            // Skip if we've already seen this canonical path
            if seen_canonical_paths.contains(&canonical_location) {
                continue;
            }

            if location.exists()
                && let Some(pm_info) = try_detect_at_path(&location, name, version_args)
            {
                seen_canonical_paths.insert(canonical_location);
                instances.push(pm_info);
            }
        }
    }

    instances
}

/// Try to detect a package manager at a specific path
fn try_detect_at_path(path: &std::path::Path, name: &str, version_args: &[&str]) -> Option<PmInfo> {
    let version = match Command::new(path).args(version_args).output() {
        Ok(output) => String::from_utf8(output.stdout)
            .unwrap_or_default()
            .trim()
            .to_string(),
        Err(_) => return None, // If we can't run it, skip it
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

/// Get common installation locations for a specific package manager
fn get_search_locations_for(name: &str) -> Vec<std::path::PathBuf> {
    let mut locations = Vec::new();
    let home = std::env::var("HOME").unwrap_or_default();

    match name {
        "pnpm" => {
            // npm global installation
            locations.push(format!("{}/.npm/bin/pnpm", home).into());
            // pnpm's own directory
            locations.push(format!("{}/.local/share/pnpm/pnpm", home).into());
            // Homebrew locations
            locations.push("/opt/homebrew/bin/pnpm".into());
            locations.push("/usr/local/bin/pnpm".into());
            // System package
            locations.push("/usr/bin/pnpm".into());
        },
        "npm" => {
            // Node.js installations
            locations.push(format!("{}/.nvm/current/bin/npm", home).into());
            locations.push("/opt/homebrew/bin/npm".into());
            locations.push("/usr/local/bin/npm".into());
            locations.push("/usr/bin/npm".into());
        },
        "yarn" => {
            // npm global installation
            locations.push(format!("{}/.npm/bin/yarn", home).into());
            // Yarn's own installation
            locations.push(format!("{}/.yarn/bin/yarn", home).into());
            // Homebrew locations
            locations.push("/opt/homebrew/bin/yarn".into());
            locations.push("/usr/local/bin/yarn".into());
            // System package
            locations.push("/usr/bin/yarn".into());
        },
        "bun" => {
            // Bun's installation directory
            locations.push(format!("{}/.bun/bin/bun", home).into());
            // Homebrew locations
            locations.push("/opt/homebrew/bin/bun".into());
            locations.push("/usr/local/bin/bun".into());
        },
        "deno" => {
            // Deno's installation directory
            locations.push(format!("{}/.deno/bin/deno", home).into());
            // Homebrew locations
            locations.push("/opt/homebrew/bin/deno".into());
            locations.push("/usr/local/bin/deno".into());
        },
        "pip" => {
            // Python installations
            locations.push("/usr/bin/pip".into());
            locations.push("/usr/bin/pip3".into());
            locations.push(format!("{}/.local/bin/pip", home).into());
            locations.push("/opt/homebrew/bin/pip".into());
            locations.push("/usr/local/bin/pip".into());
        },
        "poetry" => {
            // Poetry's installation
            locations.push(format!("{}/.local/bin/poetry", home).into());
            locations.push(format!("{}/.poetry/bin/poetry", home).into());
            locations.push("/opt/homebrew/bin/poetry".into());
            locations.push("/usr/local/bin/poetry".into());
        },
        "uv" => {
            // UV's installation
            locations.push(format!("{}/.local/bin/uv", home).into());
            locations.push(format!("{}/.cargo/bin/uv", home).into());
            locations.push("/opt/homebrew/bin/uv".into());
            locations.push("/usr/local/bin/uv".into());
        },
        "conda" => {
            // Conda installations
            locations.push(format!("{}/miniconda3/bin/conda", home).into());
            locations.push(format!("{}/anaconda3/bin/conda", home).into());
            locations.push("/opt/miniconda3/bin/conda".into());
            locations.push("/opt/anaconda3/bin/conda".into());
        },
        "go" => {
            // Go installations
            locations.push("/usr/local/go/bin/go".into());
            locations.push(format!("{}/.local/bin/go", home).into());
            locations.push("/opt/homebrew/bin/go".into());
            locations.push("/usr/bin/go".into());
        },
        "brew" => {
            // Homebrew locations
            locations.push("/opt/homebrew/bin/brew".into());
            locations.push("/usr/local/bin/brew".into());
        },
        "cargo" => {
            // Rust toolchain
            locations.push(format!("{}/.cargo/bin/cargo", home).into());
            locations.push("/opt/homebrew/bin/cargo".into());
            locations.push("/usr/local/bin/cargo".into());
            locations.push("/usr/bin/cargo".into());
        },
        "pip3" => {
            // Python 3 pip variants
            locations.push("/usr/bin/pip3".into());
            locations.push(format!("{}/.local/bin/pip3", home).into());
            locations.push("/opt/homebrew/bin/pip3".into());
        },
        "asdf" => {
            // asdf installations
            locations.push(format!("{}/.asdf/bin/asdf", home).into());
            locations.push("/opt/homebrew/bin/asdf".into());
            locations.push("/usr/local/bin/asdf".into());
        },
        "volta" => {
            // volta installations
            locations.push(format!("{}/.volta/bin/volta", home).into());
            locations.push("/opt/homebrew/bin/volta".into());
            locations.push("/usr/local/bin/volta".into());
        },
        "mise" => {
            // mise installations
            locations.push(format!("{}/.local/bin/mise", home).into());
            locations.push(format!("{}/.cargo/bin/mise", home).into());
            locations.push("/opt/homebrew/bin/mise".into());
            locations.push("/usr/local/bin/mise".into());
        },
        "corepack" => {
            // corepack installations
            locations.push("/opt/homebrew/bin/corepack".into());
            locations.push("/usr/local/bin/corepack".into());
            // Usually comes with Node.js
            locations.push(format!("{}/.nvm/current/bin/corepack", home).into());
        },
        _ => {
            // Generic fallback locations
            locations.push(format!("{}/bin/{}", home, name).into());
            locations.push(format!("{}/.local/bin/{}", home, name).into());
            locations.push(format!("/opt/homebrew/bin/{}", name).into());
            locations.push(format!("/usr/local/bin/{}", name).into());
            locations.push(format!("/usr/bin/{}", name).into());
        },
    }

    locations
}

// Simple registry, just provides iteration over all package managers
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
        // Python
        Box::new(Conda),
        Box::new(Pdm),
        Box::new(Pip),
        Box::new(Poetry),
        Box::new(Uv),
        // Rust
        Box::new(Cargo),
        // Universal Wrappers
        Box::new(Asdf),
        Box::new(Volta),
        Box::new(Mise),
        Box::new(Corepack),
    ]
}
