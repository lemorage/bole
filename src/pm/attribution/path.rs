use std::{fs, path::Path};

use crate::pm::types::InstallMethod;

/// Path-based installation method detection
pub fn detect(path: &Path) -> InstallMethod {
    let canonical_path = match fs::canonicalize(path) {
        Ok(p) => p,
        Err(_) => return InstallMethod::Unknown,
    };
    let path_str = canonical_path.to_string_lossy();

    // Check environment variables first
    if let Some(method) = check_environment_patterns(&path_str) {
        return method;
    }

    // Check well-known toolchain directories
    let home = std::env::var("HOME").unwrap_or_default();
    if let Some(method) = check_toolchain_patterns(&path_str, &home) {
        return method;
    }

    // Check system and official installer patterns
    check_system_patterns(&path_str)
}

/// Check environment-based patterns (e.g., HOMEBREW_PREFIX)
fn check_environment_patterns(path_str: &str) -> Option<InstallMethod> {
    if let Ok(homebrew_prefix) = std::env::var("HOMEBREW_PREFIX") {
        if path_str.starts_with(&homebrew_prefix) {
            // Check for more specific patterns
            if path_str.contains("/corepack/dist/") {
                return Some(InstallMethod::LanguageToolchain("homebrew → corepack"));
            }
            return Some(InstallMethod::SystemPackageManager("homebrew"));
        }
    }
    None
}

/// Check language toolchain patterns in user home directory
fn check_toolchain_patterns(path_str: &str, home: &str) -> Option<InstallMethod> {
    // Rust toolchain
    if path_str.contains(&format!("{}/.cargo/", home))
        || path_str.contains(&format!("{}/.rustup/", home))
    {
        return Some(InstallMethod::LanguageToolchain("rustup"));
    }

    // Node toolchains
    if path_str.contains(&format!("{}/.nvm/", home)) {
        return Some(InstallMethod::LanguageToolchain("nvm"));
    }

    // Haskell toolchain
    if path_str.contains(&format!("{}/.ghcup/", home)) {
        return Some(InstallMethod::LanguageToolchain("ghcup"));
    }

    // JavaScript runtimes (official installers)
    if path_str.contains(&format!("{}/.bun/", home)) {
        return Some(InstallMethod::OfficialInstaller("Bun"));
    }
    if path_str.contains(&format!("{}/.deno/", home)) {
        return Some(InstallMethod::OfficialInstaller("Deno"));
    }

    // Python package managers
    if path_str.contains(&format!("{}/.local/bin/", home)) {
        if path_str.contains("poetry") {
            return Some(InstallMethod::OfficialInstaller("Poetry"));
        }
        // Other common .local/bin tools
        return Some(InstallMethod::OfficialInstaller("pipx/pip"));
    }

    // pipx installations (the real location after symlink resolution)
    if path_str.contains(&format!("{}/.local/pipx/venvs/", home)) {
        return Some(InstallMethod::SystemPackageManager("pipx"));
    }

    // Poetry's alternative location
    if path_str.contains(&format!("{}/.poetry/", home)) {
        return Some(InstallMethod::OfficialInstaller("Poetry"));
    }

    None
}

/// Check system-wide and official installer patterns
fn check_system_patterns(path_str: &str) -> InstallMethod {
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
