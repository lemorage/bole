//! Discovery trait for package managers.
//!
//! Provides standardized discovery interface for all package managers.

/// Package manager discovery interface.
///
/// Implementations should return instances in priority order:
/// - First: active PATH-resolved instance
/// - Rest: additional locations, deduplicated by canonical path
pub trait Find {
    /// Discovery result type.
    type Output;
    /// Package manager name.
    fn name(&self) -> &'static str;
    /// Search path templates for this package manager.
    fn search_paths(&self) -> &'static [&'static str];
    /// Find all instances (PATH-first, deduplicated).
    fn find(&self) -> Vec<Self::Output>;
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
            &["/usr/bin/test", "/usr/local/bin/test"]
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
        assert_eq!(paths, &["/usr/bin/test", "/usr/local/bin/test"]);
    }

    #[test]
    fn find_blanket_impl_box_search_paths() {
        // Arrange
        let finder = MockFinder::new("cargo", vec![]);
        let finder_box: Box<MockFinder> = Box::new(finder);

        // Act
        let paths = <Box<MockFinder> as Find>::search_paths(&finder_box);

        // Assert
        assert_eq!(paths, &["/usr/bin/test", "/usr/local/bin/test"]);
    }
}
