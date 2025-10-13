use std::process::Command;

use serde_json::Value;

use crate::{
    find::{Bump, Find},
    pm::{
        Categorizable, Category, PmInfo,
        attribution::query::{Querier, Tool},
        find_all_pms,
        types::{InstallMethod, Origin},
    },
};

/// pip - Python package installer
pub struct Pip;

impl Pip {
    const NAME: &'static str = "pip";
}

impl Find for Pip {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean pip's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "pip 25.1.1 from /long/path..." -> "25.1.1"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        // Use PEP 691 JSON Simple API
        let output = Command::new("curl")
            .args([
                "-s",
                "-H",
                "Accept: application/vnd.pypi.simple.v1+json",
                "https://pypi.org/simple/pip/",
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }
        let parsed: Value = serde_json::from_slice(&output.stdout).ok()?;
        let versions = parsed["versions"].as_array()?;
        let latest = versions.last()?.as_str()?.to_string();

        // Determine update command based on installation method
        let cmd = match &pm_info.install_method {
            InstallMethod::Chain(origins) => {
                // Check the first origin in the chain
                if let Some(first) = origins.first() {
                    match first {
                        Origin::PackageManager("Homebrew") => "brew upgrade python",
                        Origin::PackageManager("MacPorts") => "sudo port upgrade python",
                        Origin::Wrapper("Pyenv") => "pyenv install --skip-existing",
                        Origin::Wrapper("Asdf") => "asdf install python latest",
                        _ => "python -m pip install --upgrade pip",
                    }
                } else {
                    "python -m pip install --upgrade pip"
                }
            },
            _ => "python -m pip install --upgrade pip",
        };

        Some(Bump { latest, cmd })
    }
}

impl Querier for Pip {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn is_available(&self) -> bool {
        Command::new("pip").args(["--version"]).output().is_ok()
    }

    fn list(&self) -> Vec<Tool> {
        let output = Command::new("pip")
            .args(["list", "--format=freeze"])
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                stdout
                    .lines()
                    .filter_map(|line| {
                        if let Some((name, version)) = line.split_once("==") {
                            Some(Tool {
                                name: name.to_string(),
                                version: version.to_string(),
                                path: None,
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
        // For pip, we need to check if the tool has a console script
        let output = Command::new("pip").args(["show", tool_name]).output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);

                // Parse version from pip show output
                let version = stdout
                    .lines()
                    .find(|line| line.starts_with("Version:"))
                    .and_then(|line| line.split_once(": "))
                    .map(|(_, v)| v.to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                Some(Tool {
                    name: tool_name.to_string(),
                    version,
                    path: None,
                    manager: Self::NAME.to_string(),
                })
            },
            _ => None,
        }
    }
}

impl Categorizable for Pip {
    fn category(&self) -> Category {
        Category::Python
    }
}
