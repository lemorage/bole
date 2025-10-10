//! Discovery trait for package managers.
//!
//! Provides standardized discovery interface for all package managers.

use crate::pm::PmInfo;

/// Available version bump information.
#[derive(Debug, Clone)]
pub struct Bump {
    /// Latest available version
    pub latest: String,
    /// Command to bump to latest version
    pub cmd: &'static str,
}

/// Package manager discovery interface.
///
/// Implementations should return instances in priority order:
/// - First: active PATH-resolved instances
/// - Rest: additional tool-specific paths, deduplicated by canonical path
pub trait Find {
    /// Discovery result type.
    type Output;
    /// Package manager name.
    fn name(&self) -> &'static str;
    /// Additional search paths outside PATH.
    /// Only include tool-specific directories, not standard bins.
    fn search_paths(&self) -> &'static [&'static str] {
        &[] // Most PMs are found via PATH
    }
    /// Find all instances (PATH-first, deduplicated).
    fn find(&self) -> Vec<Self::Output>;
    /// Check if a version bump is available.
    /// Returns Some(Bump) if outdated, None if up-to-date or unknown.
    fn check_bump(&self, _pm_info: &PmInfo) -> Option<Bump> {
        None
    }
}

/// Blanket implementations for references and boxes (`&T` and `Box<T>`).
impl<T: Find + ?Sized> Find for &T {
    type Output = T::Output;
    fn name(&self) -> &'static str {
        (**self).name()
    }
    fn search_paths(&self) -> &'static [&'static str] {
        (**self).search_paths()
    }
    fn find(&self) -> Vec<Self::Output> {
        (**self).find()
    }
    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        (**self).check_bump(pm_info)
    }
}

impl<T: Find + ?Sized> Find for Box<T> {
    type Output = T::Output;
    fn name(&self) -> &'static str {
        (**self).name()
    }
    fn search_paths(&self) -> &'static [&'static str] {
        (**self).search_paths()
    }
    fn find(&self) -> Vec<Self::Output> {
        (**self).find()
    }
    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        (**self).check_bump(pm_info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pm::{InstallMethod, PmInfo};

    struct MockFinder {
        name: &'static str,
        results: Vec<PmInfo>,
    }

    impl MockFinder {
        fn new(name: &'static str, results: Vec<PmInfo>) -> Self {
            Self { name, results }
        }
    }

    impl Find for MockFinder {
        type Output = PmInfo;

        fn name(&self) -> &'static str {
            self.name
        }

        fn search_paths(&self) -> &'static [&'static str] {
            &["~/.volta/bin/test", "~/.asdf/shims/test"]
        }

        fn find(&self) -> Vec<Self::Output> {
            self.results.clone()
        }
    }

    fn test_pm_info() -> PmInfo {
        PmInfo {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            path: "/usr/bin/test".to_string(),
            install_method: InstallMethod::Unknown,
            latest_version: None,
        }
    }

    #[test]
    fn find_trait_implementation() {
        // Arrange
        let finder = MockFinder::new("npm", vec![test_pm_info()]);

        // Act
        let name = finder.name();
        let results = finder.find();

        // Assert
        assert_eq!(name, "npm");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn find_blanket_impl_reference_name() {
        // Arrange
        let finder = MockFinder::new("test", vec![]);
        let finder_ref: &MockFinder = &finder;

        // Act
        let name = <&MockFinder as Find>::name(&finder_ref);

        // Assert
        assert_eq!(name, "test");
    }

    #[test]
    fn find_blanket_impl_reference_find() {
        // Arrange
        let finder = MockFinder::new("test", vec![test_pm_info()]);
        let finder_ref: &MockFinder = &finder;

        // Act
        let results = <&MockFinder as Find>::find(&finder_ref);

        // Assert
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn find_blanket_impl_box_name() {
        // Arrange
        let finder = MockFinder::new("test", vec![]);
        let finder_box: Box<MockFinder> = Box::new(finder);

        // Act
        let name = <Box<MockFinder> as Find>::name(&finder_box);

        // Assert
        assert_eq!(name, "test");
    }

    #[test]
    fn find_blanket_impl_box_find() {
        // Arrange
        let finder = MockFinder::new("test", vec![test_pm_info()]);
        let finder_box: Box<MockFinder> = Box::new(finder);

        // Act
        let results = <Box<MockFinder> as Find>::find(&finder_box);

        // Assert
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn find_empty_results() {
        // Arrange
        let finder = MockFinder::new("empty", vec![]);

        // Act
        let results = finder.find();

        // Assert
        assert!(results.is_empty());
    }

    #[test]
    fn find_multiple_results() {
        // Arrange
        let pm1 = test_pm_info();
        let pm2 = test_pm_info();
        let finder = MockFinder::new("multi", vec![pm1, pm2]);

        // Act
        let results = finder.find();

        // Assert
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn find_blanket_impl_reference_search_paths() {
        // Arrange
        let finder = MockFinder::new("npm", vec![]);
        let finder_ref: &MockFinder = &finder;

        // Act
        let paths = <&MockFinder as Find>::search_paths(&finder_ref);

        // Assert
        assert_eq!(paths, &["~/.volta/bin/test", "~/.asdf/shims/test"]);
    }

    #[test]
    fn find_blanket_impl_box_search_paths() {
        // Arrange
        let finder = MockFinder::new("cargo", vec![]);
        let finder_box: Box<MockFinder> = Box::new(finder);

        // Act
        let paths = <Box<MockFinder> as Find>::search_paths(&finder_box);

        // Assert
        assert_eq!(paths, &["~/.volta/bin/test", "~/.asdf/shims/test"]);
    }

    #[test]
    fn default_check_bump_is_none() {
        // Arrange
        let finder = MockFinder::new("test", vec![]);
        let pm = test_pm_info();

        // Act
        let bump = <MockFinder as Find>::check_bump(&finder, &pm);

        // Assert
        assert!(bump.is_none());
    }

    #[test]
    fn blanket_impl_reference_check_bump_none() {
        // Arrange
        let finder = MockFinder::new("test", vec![]);
        let finder_ref: &MockFinder = &finder;
        let pm = test_pm_info();

        // Act
        let bump = <&MockFinder as Find>::check_bump(&finder_ref, &pm);

        // Assert
        assert!(bump.is_none());
    }

    #[test]
    fn blanket_impl_box_check_bump_none() {
        // Arrange
        let finder: Box<MockFinder> = Box::new(MockFinder::new("test", vec![]));
        let pm = test_pm_info();

        // Act
        let bump = <Box<MockFinder> as Find>::check_bump(&finder, &pm);

        // Assert
        assert!(bump.is_none());
    }
}
