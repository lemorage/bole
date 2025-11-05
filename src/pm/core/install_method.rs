use std::{fs, path::Path};

use crate::pm::core::types::{InstallMethod, Origin};

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
                Origin::PackageManager("Homebrew"),
                Origin::Wrapper("Corepack"),
            ]));
        }
        return Some(InstallMethod::Chain(vec![Origin::PackageManager(
            "Homebrew",
        )]));
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
    if path_str.contains(&format!("{}/.fnm/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Toolchain("fnm")]));
    }
    if path_str.contains(&format!("{}/.n/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Toolchain("n")]));
    }
    if path_str.contains(&format!("{}/.nodenv/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Toolchain("nodenv")]));
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

    // Go toolchains
    if path_str.contains(&format!("{}/.g/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Toolchain("g")]));
    }
    if path_str.contains(&format!("{}/.gvm/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Toolchain("gvm")]));
    }

    // PHP toolchain
    if path_str.contains(&format!("{}/.phpbrew/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Toolchain("phpbrew")]));
    }

    // OCaml toolchain
    if path_str.contains(&format!("{}/.opam/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Toolchain("opam")]));
    }

    // Perl toolchain
    if path_str.contains(&format!("{}/.perlbrew/", home))
        || path_str.contains(&format!("{}/perl5/perlbrew/", home))
    {
        return Some(InstallMethod::Chain(vec![Origin::Toolchain("perlbrew")]));
    }

    // Elixir toolchains
    if path_str.contains(&format!("{}/.kiex/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Toolchain("kiex")]));
    }
    if path_str.contains(&format!("{}/.exenv/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Toolchain("exenv")]));
    }

    // Lua toolchain
    if path_str.contains(&format!("{}/.luaver/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Toolchain("luaver")]));
    }

    // Universal version managers
    // Note: Corepack uses different detection (filesystem symlinks),
    // while these use runtime PATH modification
    if path_str.contains(&format!("{}/.asdf/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Wrapper("asdf")]));
    }
    if path_str.contains(&format!("{}/.volta/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Wrapper("Volta")]));
    }
    if path_str.contains(&format!("{}/.local/share/mise/", home))
        || path_str.contains(&format!("{}/.config/mise/", home))
    {
        return Some(InstallMethod::Chain(vec![Origin::Wrapper("mise")]));
    }

    // JavaScript runtimes (official installers)
    if path_str.contains(&format!("{}/.bun/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Direct(Some("Bun"))]));
    }
    if path_str.contains(&format!("{}/.deno/", home)) {
        return Some(InstallMethod::Chain(vec![Origin::Direct(Some("Deno"))]));
    }

    // Conda environments
    if path_str.contains(&format!("{}/.conda/", home))
        || path_str.contains(&format!("{}/anaconda3/", home))
        || path_str.contains(&format!("{}/miniconda3/", home))
        || path_str.contains(&format!("{}/mambaforge/", home))
        || path_str.contains(&format!("{}/miniforge/", home))
        || path_str.contains("/opt/conda/")
        || path_str.contains("/opt/anaconda/")
        || path_str.contains("/opt/miniconda/")
    {
        return Some(InstallMethod::Chain(vec![Origin::PackageManager("Conda")]));
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

    // MacPorts
    if path_str.starts_with("/opt/local/") {
        // Verify it's actually MacPorts by checking for MacPorts-specific paths
        if std::path::Path::new("/opt/local/etc/macports").exists()
            || std::path::Path::new("/opt/local/var/macports").exists()
            || std::path::Path::new("/opt/local/libexec/macports").exists()
        {
            return InstallMethod::Chain(vec![Origin::PackageManager("MacPorts")]);
        }
        return InstallMethod::Chain(vec![Origin::Direct(Some("/opt/local"))]);
    }

    // Snap
    if path_str.starts_with("/snap/") || path_str.contains("/var/lib/snapd/") {
        return InstallMethod::Chain(vec![Origin::PackageManager("Snap")]);
    }

    // Flatpak
    if path_str.contains("/var/lib/flatpak/") || path_str.contains("/.var/app/") {
        return InstallMethod::Chain(vec![Origin::PackageManager("Flatpak")]);
    }

    // Scoop (Windows)
    if path_str.contains("\\scoop\\apps\\") || path_str.contains("/scoop/apps/") {
        return InstallMethod::Chain(vec![Origin::PackageManager("Scoop")]);
    }

    // Chocolatey (Windows)
    if path_str.contains("\\ProgramData\\chocolatey\\")
        || path_str.contains("/ProgramData/chocolatey/")
    {
        return InstallMethod::Chain(vec![Origin::PackageManager("Chocolatey")]);
    }

    // System provided
    if path_str.starts_with("/System/")
        || path_str.starts_with("/usr/bin/")
        || path_str.starts_with("/bin/")
        || path_str.starts_with("/sbin/")
        || path_str.starts_with("/usr/sbin/")
    {
        return InstallMethod::System;
    }

    // Official installers
    if path_str.starts_with("/usr/local/") {
        // Check for corepack first
        if path_str.contains("/corepack/dist/") {
            return InstallMethod::Chain(vec![
                Origin::Toolchain("Node.js"),
                Origin::Wrapper("Corepack"),
            ]);
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
