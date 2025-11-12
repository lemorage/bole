//! CLI behavior tests.

use assert_cmd::{Command, cargo};

/// Helper to create a bole command with color forced.
fn cmd_with_color() -> Command {
    let mut cmd = Command::new(cargo::cargo_bin!("bole"));
    cmd.env("CLICOLOR_FORCE", "1");
    cmd
}

#[test]
fn bare_command_shows_banner() {
    // Arrange
    let mut cmd = Command::new(cargo::cargo_bin!("bole"));

    // Act & Assert
    cmd.assert()
        .failure()
        .stdout(predicates::str::contains("BOLE"));
}

#[test]
fn help_flags_show_banner() {
    // Arrange
    let help_args = vec!["-h", "--help", "help"];

    // Act & Assert
    for arg in help_args {
        Command::new(cargo::cargo_bin!("bole"))
            .arg(arg)
            .assert()
            .stdout(predicates::str::contains("BOLE"));
    }
}

#[test]
fn show_command_has_tree_structure() {
    // Arrange
    let mut cmd = Command::new(cargo::cargo_bin!("bole"));

    // Act
    let assert = cmd.arg("show").assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Assert
    if stdout.contains("/") {
        // Has package managers
        assert!(stdout.contains("├──") || stdout.contains("└──"));
    }
}

#[test]
fn show_all_changes_output() {
    // Arrange
    let mut cmd = Command::new(cargo::cargo_bin!("bole"));

    // Act
    let assert = cmd.args(["show", "-a"]).assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Assert
    if stdout.contains("├──") || stdout.contains("└──") {
        // Should show status indicators for all paths
        assert!(stdout.contains("✓") || stdout.contains("-"));
    }
}

#[test]
fn show_tree_has_proper_indentation() {
    // Arrange
    let mut cmd = Command::new(cargo::cargo_bin!("bole"));

    // Act
    let assert = cmd.args(["show", "-t"]).assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Assert
    if stdout.contains("other location") {
        // Duplicates should be indented
        for line in stdout.lines() {
            if line.contains("other location") {
                assert!(line.starts_with("    ") || line.contains("│   └──"));
            }
        }
    }
}

#[test]
fn active_paths_are_green() {
    // Arrange
    let mut cmd = cmd_with_color();

    // Act
    let assert = cmd.args(["show", "-ta"]).assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Assert
    if let Some(line) = stdout.lines().find(|l| l.contains("✓") && l.contains("/")) {
        assert!(line.contains("\x1b[32m"));
    }
}

#[test]
fn inactive_paths_are_dim() {
    // Arrange
    let mut cmd = cmd_with_color();

    // Act
    let assert = cmd.args(["show", "-ta"]).assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Assert
    if let Some(line) = stdout.lines().find(|l| l.contains("- /")) {
        assert!(line.contains("\x1b[2m"));
    }
}

#[test]
fn version_format_preserved() {
    // Arrange
    let mut cmd = Command::new(cargo::cargo_bin!("bole"));

    // Act
    let assert = cmd.args(["show", "-t"]).assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Assert
    if stdout.contains("├──") || stdout.contains("└──") {
        // Version format: v1.2.3
        assert!(stdout.contains(" v") || stdout.contains(" unknown"));
        // Install method in brackets
        assert!(stdout.contains("[") && stdout.contains("]"));
    }
}

#[test]
fn json_output_is_valid() {
    // Arrange
    let mut cmd = Command::new(cargo::cargo_bin!("bole"));

    // Act
    let assert = cmd.args(["show", "--json"]).assert().success();
    let output = assert.get_output();

    // Assert
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(json.is_array());
}

#[test]
fn csv_has_proper_header() {
    // Arrange
    let mut cmd = Command::new(cargo::cargo_bin!("bole"));

    // Act
    let assert = cmd.args(["show", "--csv"]).assert().success();
    let output = assert.get_output();
    let csv = String::from_utf8_lossy(&output.stdout);

    // Assert
    let first_line = csv.lines().next().unwrap_or("");
    assert!(first_line.contains("Name"));
    assert!(first_line.contains("Version"));
    assert!(first_line.contains("Path"));
    assert!(first_line.contains(','));
}

#[test]
fn conflicting_formats_rejected() {
    // Arrange
    let mut cmd = Command::new(cargo::cargo_bin!("bole"));

    // Act & Assert
    cmd.args(["show", "--json", "--csv"]).assert().failure();
}

#[test]
#[ignore = "`check` makes slow network requests"]
fn check_shows_summary() {
    // Arrange
    let mut cmd = Command::new(cargo::cargo_bin!("bole"));

    // Act
    let assert = cmd
        .arg("check")
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Assert
    assert!(stdout.contains("Summary:") || stdout.contains("No package managers found"));
    if stdout.contains("Summary:") {
        assert!(stdout.contains("healthy"));
        assert!(stdout.contains("total"));
    }
}

#[test]
#[ignore = "`check -v` makes slow network requests"]
fn check_verbose_shows_progress() {
    // Arrange
    let mut cmd = Command::new(cargo::cargo_bin!("bole"));

    // Act
    let assert = cmd
        .args(["check", "-v"])
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Assert
    assert!(
        stdout.contains("Checking package manager health")
            || stdout.contains("No package managers found")
    );
    assert!(stdout.contains("Summary:") || stdout.contains("No package managers found"));
}

#[test]
#[ignore = "`check -vv` makes slow network requests"]
fn check_very_verbose_shows_categories() {
    // Arrange
    let mut cmd = Command::new(cargo::cargo_bin!("bole"));

    // Act
    let assert = cmd
        .args(["check", "-vv"])
        .timeout(std::time::Duration::from_secs(30))
        .assert()
        .success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);

    // Assert
    if !stdout.contains("No package managers found") {
        // Should show category names
        assert!(
            stdout.contains("system:")
                || stdout.contains("javascript:")
                || stdout.contains("python:")
                || stdout.contains("ruby:")
                || stdout.contains("rust:")
                || stdout.contains("tools:")
        );
        // Should show status indicators
        if stdout.contains("/") {
            assert!(stdout.contains("✓") || stdout.contains("✗") || stdout.contains("⚠"));
        }
    }
}
