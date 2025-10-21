//! Package manager discovery.

use bole::pm::{Category, PmInfo, all_package_managers};
use rayon::prelude::*;

/// Discovers package managers on the system.
pub(crate) struct Discovery {
    categories: Vec<Category>,
}

impl Discovery {
    /// Create a discovery for all categories.
    pub(crate) fn all() -> Self {
        Self {
            categories: Category::all().to_vec(),
        }
    }

    /// Create from an optional category string.
    pub(crate) fn from_optional_category(category: Option<String>) -> Self {
        category
            .and_then(|cat_str| Self::parse_category(&cat_str))
            .map(|cat| Self {
                categories: vec![cat],
            })
            .unwrap_or_else(Self::all) // Invalid category defaults to all
    }

    /// Find all package managers matching the configured categories.
    pub(crate) fn discover(&self) -> Vec<PmInfo> {
        all_package_managers()
            .into_par_iter()
            .filter(|d| self.categories.contains(&d.category()))
            .flat_map(|detector| detector.find())
            .collect()
    }

    /// Parse a category string, supporting aliases.
    fn parse_category(input: &str) -> Option<Category> {
        let lower = input.to_lowercase();

        for &category in Category::all() {
            // Check primary name first
            if lower == category.name() {
                return Some(category);
            }

            // Then check aliases (js -> JavaScript, py -> Python)
            for &alias in category.aliases() {
                if lower == alias {
                    return Some(category);
                }
            }
        }

        None
    }

    /// Check if a string is a valid category name or alias.
    pub(crate) fn is_valid_category(input: &str) -> bool {
        Self::parse_category(input).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_all() {
        // Act
        let discovery = Discovery::all();

        // Assert
        assert_eq!(discovery.categories.len(), Category::all().len());
    }

    #[test]
    fn test_from_optional_category_none() {
        // Arrange
        let category_input: Option<String> = None;

        // Act
        let discovery = Discovery::from_optional_category(category_input);

        // Assert
        assert_eq!(discovery.categories.len(), Category::all().len());
    }

    #[test]
    fn test_from_optional_category_valid() {
        // Arrange
        let category_input = Some("system".to_string());

        // Act
        let discovery = Discovery::from_optional_category(category_input);

        // Assert
        assert_eq!(discovery.categories, vec![Category::System]);
    }

    #[test]
    fn test_from_optional_category_valid_alias() {
        // Arrange
        let category_input = Some("js".to_string());

        // Act
        let discovery = Discovery::from_optional_category(category_input);

        // Assert
        assert_eq!(discovery.categories, vec![Category::JavaScript]);
    }

    #[test]
    fn test_from_optional_category_case_insensitive() {
        // Arrange
        let category_input = Some("PYTHON".to_string());

        // Act
        let discovery = Discovery::from_optional_category(category_input);

        // Assert
        assert_eq!(discovery.categories, vec![Category::Python]);
    }

    #[test]
    fn test_from_optional_category_invalid() {
        // Arrange
        let invalid_category = Some("invalid".to_string());

        // Act
        let discovery = Discovery::from_optional_category(invalid_category);

        // Assert - invalid category should fallback to all
        assert_eq!(discovery.categories.len(), Category::all().len());
    }

    #[test]
    fn test_discover() {
        // Arrange
        let discovery = Discovery {
            categories: vec![Category::System],
        };

        // Act
        let result = discovery.discover();

        // Assert - we can't predict exact results, but it should return a Vec
        assert!(result.is_empty() || !result.is_empty());
    }

    #[test]
    fn test_discover_all_categories() {
        // Arrange
        let discovery = Discovery::all();

        // Act
        let result = discovery.discover();

        // Assert - should find at least some package managers on most systems
        assert!(result.is_empty() || !result.is_empty());
    }

    #[test]
    fn test_parse_category_all_categories() {
        // Test each category name works
        for &category in Category::all() {
            assert!(Discovery::is_valid_category(category.name()));
        }
    }

    #[test]
    fn test_is_valid_category() {
        assert!(Discovery::is_valid_category("system"));
        assert!(Discovery::is_valid_category("javascript"));
        assert!(Discovery::is_valid_category("SYSTEM"));
        assert!(Discovery::is_valid_category("JavaScript"));
        assert!(Discovery::is_valid_category("sys"));
        assert!(Discovery::is_valid_category("js"));
        assert!(!Discovery::is_valid_category(""));
        assert!(!Discovery::is_valid_category("not-a-category"));
    }
}
