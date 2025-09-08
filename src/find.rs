/// Generic discovery trait used by lightweight "detectors" to enumerate
/// instances of a given tool or resource.
///
/// In this codebase it is implemented by package‑manager detectors (npm, pip,
/// brew, etc.). Implementations should return all discoverable instances with
/// the following conventions:
/// - The first element is the active instance resolved from PATH.
/// - Additional elements are instances found in well‑known install locations.
/// - Results should be de‑duplicated by canonical path.
///
/// The `Output` type is the per‑instance info struct (e.g. `PmInfo`).
pub trait Find {
    /// Per‑instance information yielded by this finder.
    type Output;
    /// Stable identifier for the thing being discovered.
    fn name(&self) -> &'static str;
    /// Run discovery and return all found instances in the order described
    /// above (active first, then other locations), with duplicates removed.
    fn find(&self) -> Vec<Self::Output>;
}

/// Handy blanket impls so `&T` and `Box<T>` can be used anywhere a `Find`
/// implementor is expected, without forcing clones or moves.
impl<T: Find + ?Sized> Find for &T {
    type Output = T::Output;
    fn name(&self) -> &'static str {
        (**self).name()
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
}
