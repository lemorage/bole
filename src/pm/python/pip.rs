use std::process::Command;

use crate::{
    find::Find,
    pm::{
        PmInfo,
        attribution::query::{Querier, Tool},
        find_all_pms,
    },
};

pub struct Pip;

impl Find for Pip {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        "pip"
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(<Self as Find>::name(self))
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
}

impl Querier for Pip {
    fn name(&self) -> &'static str {
        "pip"
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
                                manager: <Self as Querier>::name(self).to_string(),
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
                    manager: <Self as Querier>::name(self).to_string(),
                })
            },
            _ => None,
        }
    }
}
