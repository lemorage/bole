use std::process::Command;

use crate::pm::attribution::query::{Querier, Tool};

/// Pipx - Python application installer and manager
///
/// Note: This is an attribution-only tool. We use pipx to determine what
/// Python tools it has installed, but we don't actively hunt for pipx itself
/// as a "package manager to discover"
pub struct Pipx;

impl Querier for Pipx {
    fn name(&self) -> &'static str {
        "pipx"
    }

    fn is_available(&self) -> bool {
        Command::new("pipx").args(["--version"]).output().is_ok()
    }

    fn list(&self) -> Vec<Tool> {
        let output = Command::new("pipx").args(["list", "--short"]).output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout
                    .lines()
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.trim().split_whitespace().collect();
                        if parts.len() >= 2 {
                            Some(Tool {
                                name: parts[0].to_string(),
                                version: parts[1].to_string(),
                                path: None, // pipx doesn't provide paths in short format
                                manager: self.name().to_string(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect()
            },
            _ => Vec::new(),
        }
    }

    fn owns(&self, tool_name: &str) -> Option<Tool> {
        self.list().into_iter().find(|tool| tool.name == tool_name)
    }
}
