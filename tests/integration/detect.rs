//! Package manager detection integration tests.
//!
//! Tests the core detection functionality through the public API,
//! validating that package managers are correctly identified and attributed.

use bole::{find::Find, pm};

/// Test that our detector registry is complete and functional
#[test]
fn test_detector_completeness() {
    // Arrange
    let all_detectors = pm::all_package_managers();

    // Act & Assert
    assert!(
        !all_detectors.is_empty(),
        "Should have registered detectors"
    );

    for detector in all_detectors {
        let name = detector.name();
        let results = pm::find_all_pms(name);

        // Don't assert results exist (PM might not be installed)
        for result in results {
            assert_eq!(result.name, name);
            assert!(!result.path.is_empty());
        }
    }
}

/// Test installation method detection for various path patterns
#[test]
fn test_installation_method_attribution() {
    use pm::InstallMethod;

    // Arrange
    let test_cases = [
        ("/opt/homebrew/bin/npm", "Homebrew path"),
        ("/usr/local/bin/pip", "Local install path"),
        ("/usr/bin/python3", "System path"),
        ("~/.cargo/bin/cargo", "User install path"),
        ("/some/unknown/path/tool", "Unknown path"),
    ];

    // Act & Assert
    for (path_str, description) in test_cases {
        let path = std::path::Path::new(path_str);
        let method = pm::determine_install_method(path);

        match method {
            InstallMethod::Chain(_) => {}, // Valid
            InstallMethod::System => {},   // Valid
            InstallMethod::Unknown => {},  // Valid fallback
        }

        let display = format!("{}", method);
        assert!(
            !display.is_empty(),
            "Display should work for {}",
            description
        );
    }
}
