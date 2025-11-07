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
    Zig,

    // Version managers / Wrappers
    Tools,
}

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
            Category::Zig,
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
            Category::Zig => "zig",
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
            Category::Zig => "Zig package managers",
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

/// Information about a tool managed by a package manager.
#[derive(Debug, Clone, Serialize)]
pub struct Tool {
    pub name: String,
    pub version: String,
    pub path: Option<String>,
    pub manager: String,
}

/// Package managers that can categorize themselves.
pub trait Categorizable {
    fn category(&self) -> Category;
}

/// Package managers that can list their installed tools.
pub trait ToolLister {
    /// Package manager name (e.g., "pipx", "cargo").
    fn name(&self) -> &'static str;

    /// Check if available on the system.
    fn is_available(&self) -> bool;

    /// List all installed tools.
    fn list(&self) -> Vec<Tool>;

    /// Check if owns a specific tool.
    fn owns(&self, tool_name: &str) -> Option<Tool>;
}

/// Combined discovery and categorization trait for package managers.
pub trait Detector: crate::find::Find<Output = PmInfo> + Categorizable + Send + Sync {}

// Blanket implementation for any type that implements Find, Categorizable,
// Send, and Sync
impl<T> Detector for T where T: crate::find::Find<Output = PmInfo> + Categorizable + Send + Sync {}

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
    /// Latest available version (only when checking for updates)
    #[tabled(skip)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_version: Option<String>,
}

/// Grouped package manager information for clean display.
#[derive(Debug, Serialize, Tabled)]
pub struct GroupedPmInfo {
    #[tabled(rename = "Name")]
    pub name: String,
    #[tabled(rename = "Version")]
    pub version: String,
    #[tabled(rename = "Active Path")]
    pub active_path: String,
    #[tabled(rename = "Via")]
    pub install_method: InstallMethod,
    #[tabled(rename = "Others")]
    #[serde(skip)]
    pub alternatives: String,
    #[tabled(skip)]
    pub alternative_paths: Vec<String>,
}

impl GroupedPmInfo {
    pub fn from_instances(name: String, instances: Vec<PmInfo>) -> Self {
        if instances.is_empty() {
            panic!("Cannot create GroupedPmInfo from empty instances");
        }

        // Find the "primary" instance - prioritize PATH order
        let primary = &instances[0];
        let alternative_paths: Vec<String> =
            instances[1..].iter().map(|pm| pm.path.clone()).collect();
        let alternatives_count = alternative_paths.len();

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
            active_path: primary.path.clone(),
            install_method: primary.install_method.clone(),
            alternatives,
            alternative_paths,
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
            latest_version: None,
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
        assert_eq!(grouped.active_path, "/usr/bin/npm");
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
        assert_eq!(grouped.active_path, "/usr/bin/npm"); // Active path
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
        assert_eq!(grouped.active_path, "/usr/bin/pip"); // Active path
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

    #[test]
    fn category_all_includes_major_ecosystems() {
        // Arrange & Act
        let categories = Category::all();

        // Assert
        assert!(categories.contains(&Category::System));
        assert!(categories.contains(&Category::JavaScript));
        assert!(categories.contains(&Category::Python));
        assert!(categories.contains(&Category::Rust));
        assert!(categories.contains(&Category::Tools));
        assert!(categories.len() >= 10, "Should support major ecosystems");
    }

    #[test]
    fn category_name_matches_cli_expectations() {
        // Arrange
        let test_cases = [
            (Category::System, "system"),
            (Category::JavaScript, "javascript"),
            (Category::Python, "python"),
            (Category::PHP, "php"),
            (Category::Ruby, "ruby"),
            (Category::Rust, "rust"),
            (Category::Go, "go"),
            (Category::Haskell, "haskell"),
            (Category::Gleam, "gleam"),
            (Category::Zig, "zig"),
            (Category::Tools, "tools"),
        ];

        for (category, expected_name) in test_cases {
            // Act
            let name = category.name();

            // Assert
            assert_eq!(name, expected_name, "Category {:?} name mismatch", category);
        }
    }

    #[test]
    fn category_aliases_support_user_shortcuts() {
        // Arrange & Act
        let system_aliases = Category::System.aliases();
        let js_aliases = Category::JavaScript.aliases();
        let python_aliases = Category::Python.aliases();
        let ruby_aliases = Category::Ruby.aliases();
        let rust_aliases = Category::Rust.aliases();

        // Assert
        assert!(system_aliases.contains(&"sys"));
        assert!(js_aliases.contains(&"js"));
        assert!(js_aliases.contains(&"node"));
        assert!(js_aliases.contains(&"typescript"));
        assert!(python_aliases.contains(&"py"));
        assert!(ruby_aliases.contains(&"rb"));
        assert!(rust_aliases.contains(&"rs"));

        // Test categories without aliases
        let php_aliases = Category::PHP.aliases();
        let go_aliases = Category::Go.aliases();
        let gleam_aliases = Category::Gleam.aliases();
        assert!(php_aliases.is_empty(), "PHP should have no aliases");
        assert!(go_aliases.is_empty(), "Go should have no aliases");
        assert!(gleam_aliases.is_empty(), "Gleam should have no aliases");
    }

    #[test]
    fn category_description_provides_user_context() {
        // Arrange
        let test_cases = [
            (Category::System, "System package managers"),
            (Category::JavaScript, "JavaScript package managers"),
            (Category::Python, "Python package managers"),
            (Category::PHP, "PHP package managers"),
            (Category::Ruby, "Ruby package managers"),
            (Category::Rust, "Rust package managers"),
            (Category::Go, "Go package managers"),
            (Category::Haskell, "Haskell package managers"),
            (Category::Gleam, "Gleam package managers"),
            (Category::Zig, "Zig package managers"),
            (Category::Tools, "Version managers"),
        ];

        for (category, expected_desc) in test_cases {
            // Act
            let description = category.description();

            // Assert
            assert_eq!(
                description, expected_desc,
                "Category {:?} description mismatch",
                category
            );
            assert!(!description.is_empty(), "Description should not be empty");
        }
    }
}
