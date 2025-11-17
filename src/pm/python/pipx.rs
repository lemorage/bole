use std::process::Command;

use crate::pm::core::{
    types::{Categorizable, Category, Tool, ToolLister},
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
        let pipx_home = std::env::var("PIPX_HOME")
            .ok()
            .or_else(|| {
                Command::new("pipx")
                    .args(["environment", "--value", "PIPX_HOME"])
                    .output()
                    .ok()
                    .and_then(|o| String::from_utf8(o.stdout).ok())
                    .map(|s| s.trim().to_string())
            })
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .map(|h| format!("{}/.local/pipx", h.display()))
                    .unwrap_or_else(|| "/usr/local/pipx".to_string())
            });

        let output = Command::new("pipx").args(["list", "--short"]).output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout
                    .lines()
                    .filter_map(|line| {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 2 {
                            let name = parts[0];
                            let path = format!("{}/venvs/{}/bin/{}", pipx_home, name, name);

                            Some(Tool {
                                name: name.to_string(),
                                version: Some(parts[1]).version_or_unknown(),
                                path: Some(path),
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
