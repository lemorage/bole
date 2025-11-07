//! Version handling utilities for consistent version extraction and display.

/// Standard marker for unknown/unavailable versions.
pub const UNKNOWN_VERSION: &str = "??";

/// Extension trait for chainable version extraction.
pub trait VersionExt {
    /// Extract version or return the unknown marker (`??`).
    fn version_or_unknown(self) -> String;
}

impl VersionExt for &str {
    #[inline]
    fn version_or_unknown(self) -> String {
        let trimmed = self.trim();
        if trimmed.is_empty() {
            UNKNOWN_VERSION.to_string()
        } else {
            trimmed.to_string()
        }
    }
}

impl VersionExt for Option<&str> {
    #[inline]
    fn version_or_unknown(self) -> String {
        self.map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .unwrap_or(UNKNOWN_VERSION)
            .to_string()
    }
}

impl VersionExt for Option<String> {
    #[inline]
    fn version_or_unknown(self) -> String {
        self.as_deref().version_or_unknown()
    }
}

/// Normalize version string from command output.
#[inline]
pub fn normalize_version(output: Vec<u8>) -> String {
    String::from_utf8(output)
        .ok()
        .as_deref()
        .version_or_unknown()
}

/// Check if a version string indicates the tool is broken.
#[inline]
pub fn is_broken_version(version: &str) -> bool {
    let v = version.trim();
    v == UNKNOWN_VERSION || v.is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_ext_trait() {
        // Test with &str
        assert_eq!("1.2.3".version_or_unknown(), "1.2.3");
        assert_eq!("  1.2.3  ".version_or_unknown(), "1.2.3");
        assert_eq!("".version_or_unknown(), UNKNOWN_VERSION);
        assert_eq!("   ".version_or_unknown(), UNKNOWN_VERSION);

        // Test with Option<&str>
        assert_eq!(Some("1.2.3").version_or_unknown(), "1.2.3");
        assert_eq!(Some("  1.2.3  ").version_or_unknown(), "1.2.3");
        assert_eq!(Some("").version_or_unknown(), UNKNOWN_VERSION);
        assert_eq!(None::<&str>.version_or_unknown(), UNKNOWN_VERSION);

        // Test with Option<String>
        assert_eq!(Some("1.2.3".to_string()).version_or_unknown(), "1.2.3");
        assert_eq!(None::<String>.version_or_unknown(), UNKNOWN_VERSION);
    }

    #[test]
    fn test_normalize_version() {
        assert_eq!(normalize_version(b"1.2.3".to_vec()), "1.2.3");
        assert_eq!(normalize_version(b"  1.2.3  ".to_vec()), "1.2.3");
        assert_eq!(normalize_version(b"".to_vec()), UNKNOWN_VERSION);
        assert_eq!(normalize_version(vec![0xFF, 0xFE]), UNKNOWN_VERSION); // Invalid UTF-8
    }

    #[test]
    fn test_is_broken_version() {
        assert!(is_broken_version(UNKNOWN_VERSION));
        assert!(is_broken_version(""));
        assert!(is_broken_version("  "));
        assert!(!is_broken_version("1.0.0"));
    }
}
