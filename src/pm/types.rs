use std::{collections::HashMap, sync::LazyLock};

use serde::Serialize;
use tabled::Tabled;

/// Package manager categories for filtering and organization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    // System-wide package managers
    System,

    // Language-specific package managers
    JavaScript,
    Python,
    PHP,
    Ruby,
    Rust,
    Go,
    Haskell,
    Gleam,

    // Version managers / Wrappers
    Tools,
}

/// Lazy-initialized cache of category-to-managers mapping.
/// Built once on first access for optimal performance.
static CATEGORY_MANAGERS_CACHE: LazyLock<HashMap<Category, Vec<&'static str>>> =
    LazyLock::new(|| {
        let mut cache = HashMap::new();
        for &category in Category::all() {
            let managers = crate::pm::get_package_managers_in_category(category);
            cache.insert(category, managers);
        }
        cache
    });

impl Category {
    /// Returns all available categories.
    #[inline]
    pub const fn all() -> &'static [Category] {
        &[
            Category::System,
            Category::JavaScript,
            Category::Python,
            Category::PHP,
            Category::Ruby,
            Category::Rust,
            Category::Go,
            Category::Haskell,
            Category::Gleam,
            Category::Tools,
        ]
    }

    /// Returns the primary name for this category.
    #[inline]
    pub const fn name(self) -> &'static str {
        match self {
            Category::System => "system",
            Category::JavaScript => "javascript",
            Category::Python => "python",
            Category::PHP => "php",
            Category::Ruby => "ruby",
            Category::Rust => "rust",
            Category::Go => "go",
            Category::Haskell => "haskell",
            Category::Gleam => "gleam",
            Category::Tools => "tools",
        }
    }

    /// Returns all aliases for this category.
    #[inline]
    pub const fn aliases(self) -> &'static [&'static str] {
        match self {
            Category::System => &["sys"],
            Category::JavaScript => &["js", "typescript", "ts", "node.js", "node"],
            Category::Python => &["py"],
            Category::Ruby => &["rb"],
            Category::Rust => &["rs"],
            _ => &[],
        }
    }

    /// Returns the package manager names in this category.
    /// Uses lazy-initialized cache for optimal performance.
    #[inline]
    pub fn managers(self) -> &'static [&'static str] {
        &CATEGORY_MANAGERS_CACHE[&self]
    }

    /// Returns description of what this category contains.
    pub const fn description(self) -> &'static str {
        match self {
            Category::System => "System package managers",
            Category::JavaScript => "JavaScript package managers",
            Category::Python => "Python package managers",
            Category::PHP => "PHP package managers",
            Category::Ruby => "Ruby package managers",
            Category::Rust => "Rust package managers",
            Category::Go => "Go package managers",
            Category::Haskell => "Haskell package managers",
            Category::Gleam => "Gleam package managers",
            Category::Tools => "Version managers",
        }
    }
}

/// Installation method of a package manager.
#[derive(Debug, Serialize, Tabled, Clone, PartialEq)]
pub enum InstallMethod {
    Chain(Vec<Origin>),
    System,
    Unknown,
}

/// Single step in an installation chain.
#[derive(Debug, Serialize, Clone, PartialEq)]
pub enum Origin {
    PackageManager(&'static str),
    Toolchain(&'static str),
    Direct(Option<&'static str>),
    Wrapper(&'static str),
}

/*
TODO: Comprehensive support for all package managers and toolchains
When adding new PM support, implement AsOrigin trait with appropriate category:

Package Managers to support:
- Homebrew, MacPorts, Nix (macOS/Linux system)
- Apt, Dnf, Pacman (Linux system)
- Scoop, Winget, Chocolatey (Windows)
- Snap, Flatpak (Universal Linux)
- Pipx, Conda (Python ecosystem)

Toolchains to support:
- Rustup (Rust), Nvm (Node.js), Ghcup (Haskell)
- Asdf, Sdkman (Multi-language)
- Pyenv (Python), Go (Go toolchain)

Implementation pattern:
```rust
impl AsOrigin for NewPM {
    fn as_origin() -> Origin {
        Origin::PackageManager("NewPM")  // or Toolchain/Wrapper
    }
}
```
*/

/// Types that can represent themselves in an installation chain.
pub trait AsOrigin {
    fn as_origin() -> Origin;
}

/// Package managers that can categorize themselves.
pub trait Categorizable {
    fn category(&self) -> Category;
}

/// Combined discovery and categorization trait for package managers.
pub trait Detector: crate::find::Find<Output = PmInfo> + Categorizable {}

// Blanket implementation for any type that implements both Find and
// Categorizable
impl<T> Detector for T where T: crate::find::Find<Output = PmInfo> + Categorizable {}

impl std::fmt::Display for Origin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Origin::PackageManager(name) => write!(f, "{}", name),
            Origin::Toolchain(name) => write!(f, "{}", name),
            Origin::Direct(Some(name)) => write!(f, "Official Installer ({})", name),
            Origin::Direct(None) => write!(f, "Official Installer"),
            Origin::Wrapper(name) => write!(f, "{}", name),
        }
    }
}

impl std::fmt::Display for InstallMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallMethod::Chain(layers) => {
                let parts: Vec<String> = layers.iter().map(|l| l.to_string()).collect();
                write!(f, "{}", parts.join(" → "))
            },
            InstallMethod::System => write!(f, "System Provided"),
            InstallMethod::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Information about a discovered package manager instance.
#[derive(Debug, Serialize, Tabled, Clone)]
pub struct PmInfo {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Version")]
    pub version: String,
    #[tabled(rename = "Path")]
    pub path: String,
    #[tabled(rename = "Via")]
    pub install_method: InstallMethod,
}

/// Grouped package manager information for clean display.
#[derive(Debug, Serialize, Tabled)]
pub struct GroupedPmInfo {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Version")]
    pub version: String,
    #[tabled(rename = "Path")]
    pub primary_path: String,
    #[tabled(rename = "Via")]
    pub install_method: InstallMethod,
    #[tabled(rename = "Others")]
    pub alternatives: String,
}

impl GroupedPmInfo {
    pub fn from_instances(name: String, instances: Vec<PmInfo>) -> Self {
        if instances.is_empty() {
            panic!("Cannot create GroupedPmInfo from empty instances");
        }

        // Find the "primary" instance - prioritize PATH order
        let primary = &instances[0];
        let alternatives_count = instances.len().saturating_sub(1);

        let alternatives = if alternatives_count == 0 {
            "-".to_string()
        } else if alternatives_count == 1 {
            "1 other location".to_string()
        } else {
            format!("{} other locations", alternatives_count)
        };

        Self {
            name,
            version: primary.version.clone(),
            primary_path: primary.path.clone(),
            install_method: primary.install_method.clone(),
            alternatives,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to create test PmInfo instances
    fn create_pm_info(
        name: &str,
        version: &str,
        path: &str,
        install_method: InstallMethod,
    ) -> PmInfo {
        PmInfo {
            name: name.to_string(),
            version: version.to_string(),
            path: path.to_string(),
            install_method,
        }
    }

    #[test]
    fn test_origin_display_package_manager() {
        // Arrange
        let origin = Origin::PackageManager("Homebrew");

        // Act
        let display = format!("{}", origin);

        // Assert
        assert_eq!(display, "Homebrew");
    }

    #[test]
    fn test_origin_display_toolchain() {
        // Arrange
        let origin = Origin::Toolchain("Rustup");

        // Act
        let display = format!("{}", origin);

        // Assert
        assert_eq!(display, "Rustup");
    }

    #[test]
    fn test_origin_display_direct_with_name() {
        // Arrange
        let origin = Origin::Direct(Some("Poetry"));

        // Act
        let display = format!("{}", origin);

        // Assert
        assert_eq!(display, "Official Installer (Poetry)");
    }

    #[test]
    fn test_origin_display_direct_without_name() {
        // Arrange
        let origin = Origin::Direct(None);

        // Act
        let display = format!("{}", origin);

        // Assert
        assert_eq!(display, "Official Installer");
    }

    #[test]
    fn test_origin_display_wrapper() {
        // Arrange
        let origin = Origin::Wrapper("Corepack");

        // Act
        let display = format!("{}", origin);

        // Assert
        assert_eq!(display, "Corepack");
    }

    #[test]
    fn test_install_method_display_system() {
        // Arrange
        let method = InstallMethod::System;

        // Act
        let display = format!("{}", method);

        // Assert
        assert_eq!(display, "System Provided");
    }

    #[test]
    fn test_install_method_display_unknown() {
        // Arrange
        let method = InstallMethod::Unknown;

        // Act
        let display = format!("{}", method);

        // Assert
        assert_eq!(display, "Unknown");
    }

    #[test]
    fn test_install_method_display_single_chain() {
        // Arrange
        let method = InstallMethod::Chain(vec![Origin::PackageManager("Homebrew")]);

        // Act
        let display = format!("{}", method);

        // Assert
        assert_eq!(display, "Homebrew");
    }

    #[test]
    fn test_install_method_display_double_chain() {
        // Arrange
        let method = InstallMethod::Chain(vec![
            Origin::Toolchain("Node.js"),
            Origin::Wrapper("Corepack"),
        ]);

        // Act
        let display = format!("{}", method);

        // Assert
        assert_eq!(display, "Node.js → Corepack");
    }

    #[test]
    fn test_install_method_display_complex_chain() {
        // Arrange
        let method = InstallMethod::Chain(vec![
            Origin::PackageManager("Homebrew"),
            Origin::Toolchain("Node.js"),
            Origin::Wrapper("Corepack"),
        ]);

        // Act
        let display = format!("{}", method);

        // Assert
        assert_eq!(display, "Homebrew → Node.js → Corepack");
    }

    #[test]
    fn test_install_method_display_mixed_origin_types() {
        // Arrange
        let method = InstallMethod::Chain(vec![
            Origin::Direct(Some("Poetry")),
            Origin::PackageManager("Pipx"),
            Origin::Toolchain("Pyenv"),
        ]);

        // Act
        let display = format!("{}", method);

        // Assert
        assert_eq!(display, "Official Installer (Poetry) → Pipx → Pyenv");
    }

    #[test]
    fn test_install_method_display_empty_chain() {
        // Arrange - Edge case that shouldn't happen in practice
        let method = InstallMethod::Chain(vec![]);

        // Act
        let display = format!("{}", method);

        // Assert
        assert_eq!(display, "");
    }

    #[test]
    fn test_grouped_pm_info_single_instance() {
        // Arrange
        let pm_info = create_pm_info(
            "npm",
            "8.19.2",
            "/usr/bin/npm",
            InstallMethod::Chain(vec![Origin::PackageManager("Homebrew")]),
        );
        let instances = vec![pm_info];

        // Act
        let grouped = GroupedPmInfo::from_instances("npm".to_string(), instances);

        // Assert
        assert_eq!(grouped.name, "npm");
        assert_eq!(grouped.version, "8.19.2");
        assert_eq!(grouped.primary_path, "/usr/bin/npm");
        assert_eq!(format!("{}", grouped.install_method), "Homebrew");
        assert_eq!(grouped.alternatives, "-");
    }

    #[test]
    fn test_grouped_pm_info_two_instances() {
        // Arrange
        let instances = vec![
            create_pm_info("npm", "8.19.2", "/usr/bin/npm", InstallMethod::System),
            create_pm_info(
                "npm",
                "9.0.0",
                "/usr/local/bin/npm",
                InstallMethod::Chain(vec![Origin::Direct(Some("Node.js"))]),
            ),
        ];

        // Act
        let grouped = GroupedPmInfo::from_instances("npm".to_string(), instances);

        // Assert
        assert_eq!(grouped.name, "npm");
        assert_eq!(grouped.version, "8.19.2"); // Primary version
        assert_eq!(grouped.primary_path, "/usr/bin/npm"); // Primary path
        assert_eq!(format!("{}", grouped.install_method), "System Provided");
        assert_eq!(grouped.alternatives, "1 other location");
    }

    #[test]
    fn test_grouped_pm_info_multiple_instances() {
        // Arrange
        let instances = vec![
            create_pm_info("pip", "22.3.1", "/usr/bin/pip", InstallMethod::System),
            create_pm_info(
                "pip",
                "23.0.0",
                "/usr/local/bin/pip",
                InstallMethod::Chain(vec![Origin::PackageManager("Homebrew")]),
            ),
            create_pm_info(
                "pip",
                "22.0.0",
                "/home/user/.local/bin/pip",
                InstallMethod::Chain(vec![Origin::Direct(Some("pipx/pip"))]),
            ),
        ];

        // Act
        let grouped = GroupedPmInfo::from_instances("pip".to_string(), instances);

        // Assert
        assert_eq!(grouped.name, "pip");
        assert_eq!(grouped.version, "22.3.1"); // Primary version (first in list)
        assert_eq!(grouped.primary_path, "/usr/bin/pip"); // Primary path
        assert_eq!(format!("{}", grouped.install_method), "System Provided");
        assert_eq!(grouped.alternatives, "2 other locations");
    }

    #[test]
    #[should_panic(expected = "Cannot create GroupedPmInfo from empty instances")]
    fn test_grouped_pm_info_empty_instances_panics() {
        // Arrange
        let instances = vec![];

        // Act
        GroupedPmInfo::from_instances("npm".to_string(), instances);
    }

    #[test]
    fn test_grouped_pm_info_preserves_primary_install_method() {
        // Arrange
        let complex_method = InstallMethod::Chain(vec![
            Origin::PackageManager("Homebrew"),
            Origin::Toolchain("Node.js"),
            Origin::Wrapper("Corepack"),
        ]);
        let instances = vec![
            create_pm_info(
                "pnpm",
                "7.14.0",
                "/opt/homebrew/bin/pnpm",
                complex_method.clone(),
            ),
            create_pm_info(
                "pnpm",
                "6.32.0",
                "/usr/local/bin/pnpm",
                InstallMethod::Unknown,
            ),
        ];

        // Act
        let grouped = GroupedPmInfo::from_instances("pnpm".to_string(), instances);

        // Assert
        assert_eq!(
            format!("{}", grouped.install_method),
            "Homebrew → Node.js → Corepack"
        );
    }

    #[test]
    fn test_pm_info_debug_serialization() {
        // Arrange
        let pm_info = create_pm_info(
            "cargo",
            "1.70.0",
            "/home/user/.cargo/bin/cargo",
            InstallMethod::Chain(vec![Origin::Toolchain("Rustup")]),
        );

        // Assert
        assert_eq!(pm_info.name, "cargo");
        assert_eq!(pm_info.version, "1.70.0");
        assert_eq!(pm_info.path, "/home/user/.cargo/bin/cargo");
        assert_eq!(
            pm_info.install_method,
            InstallMethod::Chain(vec![Origin::Toolchain("Rustup")])
        );
    }

    #[test]
    fn test_install_method_clone() {
        // Arrange
        let original = InstallMethod::Chain(vec![
            Origin::PackageManager("Homebrew"),
            Origin::Direct(Some("Node.js")),
        ]);

        // Act
        let cloned = original.clone();

        // Assert
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_origin_clone() {
        // Arrange
        let original = Origin::Direct(Some("Poetry"));

        // Act
        let cloned = original.clone();

        // Assert
        assert_eq!(original, cloned);
    }

    #[test]
    fn test_alternatives_count_edge_cases() {
        // Arrange - Test saturating_sub behavior
        let single_instance = vec![create_pm_info(
            "test",
            "1.0",
            "/usr/bin/test",
            InstallMethod::Unknown,
        )];

        // Act
        let grouped = GroupedPmInfo::from_instances("test".to_string(), single_instance);

        // Assert - Should handle zero alternatives correctly
        assert_eq!(grouped.alternatives, "-");
    }
}
