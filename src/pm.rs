mod gleam;
mod go;
mod haskell;
mod js;
mod python;
mod rust;
mod system;

use std::{fs, path::Path, process::Command};

pub use gleam::Gleam;
pub use go::Go;
pub use haskell::{Cabal, Stack};
pub use js::{Bun, Deno, Npm, Pnpm, Yarn};
pub use python::{Conda, Pdm, Pip, Poetry, Uv};
pub use rust::Cargo;
use serde::Serialize;
pub use system::{Homebrew, Macports, Nix};
use tabled::Tabled;
use which::which;

use crate::find::Find;

#[derive(Debug, Serialize, Tabled, Clone)]
pub enum InstallMethod {
    OfficialInstaller(&'static str),
    SystemPackageManager(&'static str),
    LanguageToolchain(&'static str),
    SystemProvided,
    Unknown,
}

impl std::fmt::Display for InstallMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallMethod::OfficialInstaller(via) => write!(f, "Official Installer ({})", via),
            InstallMethod::SystemPackageManager(pm) => write!(f, "{} Package", pm),
            InstallMethod::LanguageToolchain(tool) => write!(f, "{}", tool),
            InstallMethod::SystemProvided => write!(f, "System Provided"),
            InstallMethod::Unknown => write!(f, "Unknown"),
        }
    }
}

#[derive(Debug, Serialize, Tabled)]
pub struct PmInfo {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Version")]
    pub version: String,

    #[tabled(rename = "Installed Via")]
    pub install_method: InstallMethod,

    #[tabled(rename = "Path")]
    pub path: String,
}

// The main inference function
pub(crate) fn determine_install_method(path: &Path) -> InstallMethod {
    let canonical_path = match fs::canonicalize(path) {
        Ok(p) => p,
        Err(_) => return InstallMethod::Unknown,
    };
    let path_str = canonical_path.to_string_lossy();

    // 1. Check environment variables first
    if let Ok(homebrew_prefix) = std::env::var("HOMEBREW_PREFIX") {
        if path_str.starts_with(&homebrew_prefix) {
            // Check for more specific patterns first
            if path_str.contains("/corepack/dist/") {
                return InstallMethod::LanguageToolchain("homebrew → corepack");
            }
            return InstallMethod::SystemPackageManager("homebrew");
        }
    }

    // 2. Check well-known toolchain directories
    let home = std::env::var("HOME").unwrap_or_default();

    // Rust toolchain
    if path_str.contains(&format!("{}/.cargo/", home))
        || path_str.contains(&format!("{}/.rustup/", home))
    {
        return InstallMethod::LanguageToolchain("rustup");
    }

    // Node toolchains
    if path_str.contains(&format!("{}/.nvm/", home)) {
        return InstallMethod::LanguageToolchain("nvm");
    }

    // JavaScript runtimes (official installers)
    if path_str.contains(&format!("{}/.bun/", home)) {
        return InstallMethod::OfficialInstaller("Bun");
    }
    if path_str.contains(&format!("{}/.deno/", home)) {
        return InstallMethod::OfficialInstaller("Deno");
    }

    // Python package managers
    if path_str.contains(&format!("{}/.local/bin/", home)) {
        if path_str.contains("poetry") {
            return InstallMethod::OfficialInstaller("Poetry");
        }
        // Other common .local/bin tools
        return InstallMethod::OfficialInstaller("pipx/pip");
    }

    // pipx installations (the real location after symlink resolution)
    if path_str.contains(&format!("{}/.local/pipx/venvs/", home)) {
        return InstallMethod::SystemPackageManager("pipx");
    }

    // Poetry's alternative location
    if path_str.contains(&format!("{}/.poetry/", home)) {
        return InstallMethod::OfficialInstaller("Poetry");
    }

    // Nix
    if path_str.contains("/nix/store/") {
        return InstallMethod::SystemPackageManager("nix");
    }

    // System provided
    if path_str.starts_with("/System/") || path_str.starts_with("/usr/bin/") {
        return InstallMethod::SystemProvided;
    }

    // Official installers
    if path_str.starts_with("/usr/local/") {
        // Check for corepack first
        if path_str.contains("/corepack/dist/") {
            return InstallMethod::LanguageToolchain("nodejs → corepack");
        }
        return InstallMethod::OfficialInstaller("Direct Install");
    }

    InstallMethod::Unknown
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
    if let Ok(path) = which(name) {
        if let Some(pm_info) = try_detect_at_path(&path, name, version_args) {
            // Get canonical path for deduplication
            if let Ok(canonical) = std::fs::canonicalize(&path) {
                seen_canonical_paths.insert(canonical);
            }
            instances.push(pm_info);
        }
    }

    // Known installation locations
    let search_locations = get_search_locations_for(name);
    for location in search_locations {
        if let Ok(canonical_location) = std::fs::canonicalize(&location) {
            // Skip if we've already seen this canonical path
            if seen_canonical_paths.contains(&canonical_location) {
                continue;
            }

            if location.exists() {
                if let Some(pm_info) = try_detect_at_path(&location, name, version_args) {
                    seen_canonical_paths.insert(canonical_location);
                    instances.push(pm_info);
                }
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
pub fn all_package_managers() -> Vec<Box<dyn Find<Output = PmInfo>>> {
    vec![
        // System
        Box::new(Homebrew),
        Box::new(Macports),
        Box::new(Nix),
        // Python
        Box::new(Conda),
        Box::new(Pdm),
        Box::new(Pip),
        Box::new(Poetry),
        Box::new(Uv),
        // JavaScript/TypeScript
        Box::new(Bun),
        Box::new(Deno),
        Box::new(Npm),
        Box::new(Pnpm),
        Box::new(Yarn),
        // Haskell
        Box::new(Cabal),
        Box::new(Stack),
        // Rust
        Box::new(Cargo),
        // Go
        Box::new(Go),
        // Gleam
        Box::new(Gleam),
    ]
}
