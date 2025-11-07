use std::process::Command;

use crate::pm::core::{
    tool_lister::{Tool, ToolLister},
    types::{Categorizable, Category},
    version::VersionExt,
};

/// Pipx - Python application installer and manager
///
/// Note: This is an attribution-only tool. We use pipx to determine what
/// Python tools it has installed, but we don't actively hunt for pipx itself
/// as a "package manager to discover"
pub struct Pipx;

impl Pipx {
    const NAME: &'static str = "pipx";
}

impl ToolLister for Pipx {
    fn name(&self) -> &'static str {
        Self::NAME
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
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            Some(Tool {
                                name: parts[0].to_string(),
                                version: Some(parts[1]).version_or_unknown(),
                                path: None, // pipx doesn't provide paths in short format
                                manager: Self::NAME.to_string(),
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

impl Categorizable for Pipx {
    fn category(&self) -> Category {
        Category::Python
    }
}
