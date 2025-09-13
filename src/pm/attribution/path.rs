use std::{fs, path::Path};

use crate::pm::{
    system::Homebrew,
    types::{AsOrigin, InstallMethod, Origin},
    wrappers::{Asdf, Corepack, Mise, Volta},
};

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
    if let Ok(homebrew_prefix) = std::env::var("HOMEBREW_PREFIX")
        && path_str.starts_with(&homebrew_prefix)
    {
        // Check for more specific patterns
        if path_str.contains("/corepack/dist/") {
            return Some(InstallMethod::Chain(vec![
                Homebrew::as_origin(),
                Corepack::as_origin(),
            ]));
        }
        return Some(InstallMethod::Chain(vec![Homebrew::as_origin()]));
    }
    None
}

/// Check language toolchain patterns in user home directory
fn check_toolchain_patterns(path_str: &str, home: &str) -> Option<InstallMethod> {
    // Rust toolchain
    if path_str.contains(&format!("{}/.cargo/", home))
        || path_str.contains(&format!("{}/.rustup/", home))
    {
        return Some(InstallMethod::Chain(vec![Origin::Toolchain("Rustup")]));
    }

    // Node toolchains
    if path_str.contains(&format!("{}/.nvm/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Toolchain("NVM")]));
    }

    // Haskell toolchain
    if path_str.contains(&format!("{}/.ghcup/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Toolchain("GHCup")]));
    }

    // Ruby toolchains
    if path_str.contains(&format!("{}/.rvm/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Toolchain("RVM")]));
    }
    if path_str.contains(&format!("{}/.rbenv/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Toolchain("rbenv")]));
    }

    // Python toolchain
    if path_str.contains(&format!("{}/.pyenv/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Toolchain("pyenv")]));
    }

    // PHP toolchain
    if path_str.contains(&format!("{}/.phpbrew/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Toolchain("phpbrew")]));
    }

    // Universal version managers
    // Note: Corepack uses different detection (filesystem symlinks),
    // while these use runtime PATH modification
    if path_str.contains(&format!("{}/.asdf/", home)) {
        return Some(InstallMethod::Chain(vec![Asdf::as_origin()]));
    }
    if path_str.contains(&format!("{}/.volta/", home)) {
        return Some(InstallMethod::Chain(vec![Volta::as_origin()]));
    }
    if path_str.contains(&format!("{}/.local/share/mise/", home))
        || path_str.contains(&format!("{}/.config/mise/", home))
    {
        return Some(InstallMethod::Chain(vec![Mise::as_origin()]));
    }

    // JavaScript runtimes (official installers)
    if path_str.contains(&format!("{}/.bun/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Direct(Some("Bun"))]));
    }
    if path_str.contains(&format!("{}/.deno/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Direct(Some("Deno"))]));
    }

    // Python package managers
    if path_str.contains(&format!("{}/.local/bin/", home)) {
        if path_str.contains("poetry") {
            return Some(InstallMethod::Chain(vec![Origin::Direct(Some("Poetry"))]));
        }
        // Other common .local/bin tools
        return Some(InstallMethod::Chain(vec![Origin::Direct(Some("pipx/pip"))]));
    }

    // pipx installations (the real location after symlink resolution)
    if path_str.contains(&format!("{}/.local/pipx/venvs/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::PackageManager("Pipx")]));
    }

    // Poetry's alternative location
    if path_str.contains(&format!("{}/.poetry/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Direct(Some("Poetry"))]));
    }

    None
}

/// Check system-wide and official installer patterns
fn check_system_patterns(path_str: &str) -> InstallMethod {
    // Nix
    if path_str.contains("/nix/store/") {
        return InstallMethod::Chain(vec![Origin::PackageManager("Nix")]);
    }

    // System provided
    if path_str.starts_with("/System/") || path_str.starts_with("/usr/bin/") {
        return InstallMethod::System;
    }

    // Official installers
    if path_str.starts_with("/usr/local/") {
        // Check for corepack first
        if path_str.contains("/corepack/dist/") {
            return InstallMethod::Chain(vec![Origin::Toolchain("Node.js"), Corepack::as_origin()]);
        }
        return InstallMethod::Chain(vec![Origin::Direct(Some("Direct Install"))]);
    }

    InstallMethod::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_with_nonexistent_path() {
        // Arrange
        let nonexistent_path = std::path::PathBuf::from("/this/path/does/not/exist");

        // Act
        let result = detect(&nonexistent_path);

        // Assert
        assert_eq!(result, InstallMethod::Unknown);
    }
}
