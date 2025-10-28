//! Filtering and grouping operations for package managers.

use std::collections::HashMap;

use bole::pm::{GroupedPmInfo, PmInfo, all_package_managers};

/// Filters for package manager lists.
pub(crate) struct Filter;

impl Filter {
    /// Keep only package managers with no version (broken).
    pub(crate) fn broken(pms: Vec<PmInfo>) -> Vec<PmInfo> {
        pms.into_iter()
            .filter(|pm| pm.version.trim().is_empty())
            .collect()
    }

    /// Keep only package managers with valid versions.
    #[allow(dead_code)]
    pub(crate) fn healthy(pms: Vec<PmInfo>) -> Vec<PmInfo> {
        pms.into_iter()
            .filter(|pm| !pm.version.trim().is_empty())
            .collect()
    }

    /// Keep only package managers with available updates.
    pub(crate) fn outdated(pms: Vec<PmInfo>) -> Vec<PmInfo> {
        // Get detectors once for all checks
        let detectors = all_package_managers();

        pms.into_iter()
            .filter(|pm| Self::is_outdated(pm, &detectors))
            .collect()
    }

    /// Check if a package manager has an available update.
    fn is_outdated(pm: &PmInfo, detectors: &[Box<dyn bole::pm::Detector>]) -> bool {
        // Broken PMs can't be outdated
        if pm.version.trim().is_empty() {
            return false;
        }

        detectors
            .iter()
            .find(|d| d.name() == pm.name)
            .and_then(|d| d.check_bump(pm))
            .map(|bump| bump.latest != pm.version)
            .unwrap_or(false) // No detector or no bump info means not outdated
    }
}

/// Groups package managers by name.
pub(crate) struct Grouper;

impl Grouper {
    /// Group by name, preserving PATH order.
    pub(crate) fn by_name(pms: Vec<PmInfo>) -> Vec<GroupedPmInfo> {
        let mut grouped: HashMap<String, Vec<PmInfo>> = HashMap::new();

        // Collect all instances by name
        for pm in pms {
            grouped.entry(pm.name.clone()).or_default().push(pm);
        }

        let mut result: Vec<_> = grouped
            .into_iter()
            .map(|(name, instances)| GroupedPmInfo::from_instances(name, instances))
            .collect();

        // Sort groups alphabetically for consistent output
        result.sort_by(|a, b| a.name.cmp(&b.name));
        result
    }

    /// Extract only the primary instance from each group.
    pub(crate) fn primary_only(pms: Vec<PmInfo>) -> Vec<PmInfo> {
        let grouped = Self::by_name(pms);

        grouped
            .into_iter()
            .map(|g| PmInfo {
                name: g.name,
                version: g.version,
                path: g.active_path,
                install_method: g.install_method,
                latest_version: None, // Not preserved during grouping
            })
            .collect()
    }
}

/// Sorts package manager lists.
pub(crate) struct Sorter;

impl Sorter {
    /// Sort alphabetically by name.
    pub(crate) fn by_name(mut pms: Vec<PmInfo>) -> Vec<PmInfo> {
        pms.sort_by(|a, b| a.name.cmp(&b.name));
        pms
    }

    /// Sort by category, then by name within each category.
    #[allow(dead_code)]
    pub(crate) fn by_category(mut pms: Vec<PmInfo>) -> Vec<PmInfo> {
        // Build lookup table once for O(1) lookups during sort
        let detectors = all_package_managers();
        let name_to_category: HashMap<String, bole::pm::Category> = detectors
            .iter()
            .map(|d| (d.name().to_string(), d.category()))
            .collect();

        pms.sort_by(|a, b| {
            let cat_a = name_to_category.get(&a.name);
            let cat_b = name_to_category.get(&b.name);

            match (cat_a, cat_b) {
                (Some(ca), Some(cb)) if ca != cb => {
                    // Respect the order defined in Category::all()
                    let order_a = bole::pm::Category::all()
                        .iter()
                        .position(|&c| c == *ca)
                        .unwrap_or(usize::MAX); // Unknown categories go last
                    let order_b = bole::pm::Category::all()
                        .iter()
                        .position(|&c| c == *cb)
                        .unwrap_or(usize::MAX);
                    order_a.cmp(&order_b)
                },
                // Same category or missing info: sort by name
                _ => a.name.cmp(&b.name),
            }
        });

        pms
    }
}

#[cfg(test)]
mod tests {
    use bole::pm::InstallMethod;

    use super::*;

    fn test_pm(name: &str, version: &str) -> PmInfo {
        PmInfo {
            name: name.to_string(),
            version: version.to_string(),
            path: format!("/usr/bin/{}", name),
            install_method: InstallMethod::Unknown,
            latest_version: None,
        }
    }

    fn test_pm_with_path(name: &str, version: &str, path: &str) -> PmInfo {
        PmInfo {
            name: name.to_string(),
            version: version.to_string(),
            path: path.to_string(),
            install_method: InstallMethod::Unknown,
            latest_version: None,
        }
    }

    #[test]
    fn test_filter_broken() {
        // Arrange
        let pms = vec![
            test_pm("cargo", "1.70.0"),
            test_pm("npm", "8.0.0"),
            test_pm("pip", "   "),
            test_pm("go", "\t\n"),
        ];

        // Act
        let broken = Filter::broken(pms);

        // Assert
        assert_eq!(broken.len(), 2);
        assert_eq!(broken[0].name, "pip");
        assert_eq!(broken[1].name, "go");
    }

    #[test]
    fn test_filter_healthy() {
        // Arrange
        let pms = vec![
            test_pm("npm", "8.0.0"),
            test_pm("pip", ""),
            test_pm("cargo", "1.70.0"),
        ];

        // Act
        let healthy = Filter::healthy(pms);

        // Assert
        assert_eq!(healthy.len(), 2);
        assert_eq!(healthy[0].name, "npm");
        assert_eq!(healthy[1].name, "cargo");
    }

    #[test]
    fn test_filter_outdated_empty_version() {
        // Arrange: broken or unknown PMs should not be considered outdated
        let pms = vec![
            test_pm("pip", ""),
            test_pm("npm", "   "),
            test_pm("unknown-pm", "1.0.0"),
        ];

        // Act
        let outdated = Filter::outdated(pms);

        // Assert
        assert_eq!(outdated.len(), 0);
    }

    #[test]
    fn test_grouper_by_name() {
        // Arrange
        let pms = vec![
            test_pm_with_path("npm", "8.0.0", "/usr/bin/npm"),
            test_pm_with_path("npm", "9.0.0", "/usr/local/bin/npm"),
            test_pm("pip", "22.0"),
        ];

        // Act
        let grouped = Grouper::by_name(pms);

        // Assert
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].name, "npm");
        assert_eq!(grouped[1].name, "pip");
        assert_eq!(grouped[0].alternative_paths.len(), 1);
        assert_eq!(grouped[1].alternative_paths.len(), 0);
    }

    #[test]
    fn test_grouper_primary_only() {
        // Arrange
        let pms = vec![
            test_pm_with_path("npm", "8.0.0", "/usr/bin/npm"),
            test_pm_with_path("npm", "9.0.0", "/usr/local/bin/npm"),
            test_pm("pip", "22.0"),
            test_pm_with_path("cargo", "1.70", "/usr/bin/cargo"),
            test_pm_with_path("cargo", "1.71", "/home/.cargo/bin/cargo"),
        ];

        let primary = Grouper::primary_only(pms);
        assert_eq!(primary.len(), 3);

        // Should keep only first instance of each
        let npm = primary.iter().find(|pm| pm.name == "npm").unwrap();
        assert_eq!(npm.version, "8.0.0");
        assert_eq!(npm.path, "/usr/bin/npm");
    }

    #[test]
    fn test_grouper_single_instances() {
        // Arrange
        let pms = vec![
            test_pm("npm", "8.0.0"),
            test_pm("pip", "22.0"),
            test_pm("cargo", "1.70.0"),
        ];

        // Act
        let grouped = Grouper::by_name(pms.clone());

        // Assert
        assert_eq!(grouped.len(), 3);
        for group in &grouped {
            assert_eq!(group.alternatives, "-");
            assert_eq!(group.alternative_paths.len(), 0);
        }

        // Act
        let primary = Grouper::primary_only(pms);

        // Assert
        assert_eq!(primary.len(), 3);
    }

    #[test]
    fn test_sort_by_name() {
        // Arrange
        let pms = vec![
            test_pm("pip", "22.0"),
            test_pm("cargo", "1.70.0"),
            test_pm("npm", "8.0.0"),
            test_pm("zig", "0.11.0"),
            test_pm("asdf", "0.10.0"),
        ];

        // Act
        let sorted = Sorter::by_name(pms);

        // Assert
        assert_eq!(sorted[0].name, "asdf");
        assert_eq!(sorted[1].name, "cargo");
        assert_eq!(sorted[2].name, "npm");
        assert_eq!(sorted[3].name, "pip");
        assert_eq!(sorted[4].name, "zig");
    }

    #[test]
    fn test_sort_by_name_empty() {
        let pms = vec![];
        let sorted = Sorter::by_name(pms);
        assert_eq!(sorted.len(), 0);
    }

    #[test]
    fn test_sort_by_category() {
        // Arrange
        let pms = vec![
            test_pm("npm", "8.0.0"),        // JavaScript
            test_pm("pip", "22.0"),         // Python
            test_pm("brew", "4.0.0"),       // System
            test_pm("cargo", "1.70.0"),     // Rust
            test_pm("unknown-pm", "1.0.0"), // Unknown
        ];

        let sorted = Sorter::by_category(pms);

        // System should come first (based on Category::all() order)
        assert_eq!(sorted[0].name, "brew");

        // Within same category, should be sorted by name
        let names: Vec<String> = sorted.iter().map(|pm| pm.name.clone()).collect();
        assert!(
            names.iter().position(|n| n == "brew").unwrap()
                < names.iter().position(|n| n == "npm").unwrap()
        );
    }

    #[test]
    fn test_sort_by_category_same_category() {
        // Arrange
        let pms = vec![
            test_pm("yarn", "1.0.0"),
            test_pm("npm", "8.0.0"),
            test_pm("pnpm", "7.0.0"),
            test_pm("bun", "1.0.0"),
        ];

        // Act
        let sorted = Sorter::by_category(pms);

        // Assert: all are JavaScript, so should be sorted by name
        assert_eq!(sorted[0].name, "bun");
        assert_eq!(sorted[1].name, "npm");
        assert_eq!(sorted[2].name, "pnpm");
        assert_eq!(sorted[3].name, "yarn");
    }

    #[test]
    fn test_grouper_by_name_with_install_methods() {
        // Arrange
        let pms = vec![
            PmInfo {
                name: "npm".to_string(),
                version: "8.0.0".to_string(),
                path: "/usr/bin/npm".to_string(),
                install_method: InstallMethod::Unknown,
                latest_version: None,
            },
            PmInfo {
                name: "npm".to_string(),
                version: "9.0.0".to_string(),
                path: "/usr/local/bin/npm".to_string(),
                install_method: InstallMethod::System,
                latest_version: None,
            },
        ];

        // Act
        let grouped = Grouper::by_name(pms);

        // Assert
        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped[0].name, "npm");
        assert_eq!(grouped[0].version, "8.0.0");
        assert_eq!(grouped[0].active_path, "/usr/bin/npm");
    }

    #[test]
    fn test_is_outdated_edge_cases() {
        // Arrange
        let detectors = all_package_managers();

        // Empty version
        let pm_empty = test_pm("npm", "");
        assert!(!Filter::is_outdated(&pm_empty, &detectors));

        // Whitespace version
        let pm_whitespace = test_pm("npm", "   ");
        assert!(!Filter::is_outdated(&pm_whitespace, &detectors));

        // Unknown PM (not in detector list)
        let pm_unknown = test_pm("unknown-tool", "1.0.0");
        assert!(!Filter::is_outdated(&pm_unknown, &detectors));
    }

    #[test]
    fn test_grouper_empty_list() {
        let grouped = Grouper::by_name(vec![]);
        assert_eq!(grouped.len(), 0);

        let primary = Grouper::primary_only(vec![]);
        assert_eq!(primary.len(), 0);
    }
}
