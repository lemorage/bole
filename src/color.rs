//! Terminal color support.

use std::{
    env,
    io::{self, IsTerminal},
};

/// Check if colors should be enabled.
fn should_color() -> bool {
    // CLICOLOR_FORCE=1 forces colors even in non-TTY
    if env::var("CLICOLOR_FORCE").is_ok_and(|v| v != "0") {
        return true;
    }

    // Check all disable conditions
    io::stdout().is_terminal()                             // Must be a TTY
        && env::var("NO_COLOR").is_err()                   // NO_COLOR disables (https://no-color.org/)
        && env::var("CLICOLOR").map_or(true, |v| v != "0") // CLICOLOR=0 disables
        && env::var("TERM").map_or(true, |v| v != "dumb") // TERM=dumb disables
}

/// Green text for success states.
pub(crate) fn success(s: &str) -> String {
    if should_color() {
        format!("\x1b[32m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}

/// Red text for error states.
pub(crate) fn error(s: &str) -> String {
    if should_color() {
        format!("\x1b[31m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}

/// Yellow text for warning states.
pub(crate) fn warning(s: &str) -> String {
    if should_color() {
        format!("\x1b[33m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}

/// Dimmed text for secondary information.
pub(crate) fn dim(s: &str) -> String {
    if should_color() {
        format!("\x1b[2m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}

/// Bold text for emphasis.
pub(crate) fn bold(s: &str) -> String {
    if should_color() {
        format!("\x1b[1m{}\x1b[0m", s)
    } else {
        s.to_string()
    }
}

/// Success status symbol.
pub(crate) fn check_mark() -> String {
    success("✓")
}

/// Error status symbol.
pub(crate) fn cross_mark() -> String {
    error("✗")
}

/// Warning status symbol.
pub(crate) fn warning_mark() -> String {
    warning("⚠")
}
