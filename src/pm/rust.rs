use std::process::Command;

use crate::{
    find::Find,
    pm::{
        PmInfo,
        attribution::query::{Querier, Tool},
        find_all_pms_with_args,
    },
};

pub struct Cargo;

impl Find for Cargo {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        "cargo"
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms_with_args(<Self as Find>::name(self), &["-V"])
            .into_iter()
            .map(|mut pm_info| {
                // Clean cargo's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "cargo 1.89.0 (c24e10642 2025-06-23)" -> "1.89.0"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}

impl Querier for Cargo {
    fn name(&self) -> &'static str {
        "cargo"
    }

    fn is_available(&self) -> bool {
        Command::new("cargo").args(["--version"]).output().is_ok()
    }

    fn list(&self) -> Vec<Tool> {
        let output = Command::new("cargo").args(["install", "--list"]).output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let mut tools = Vec::new();

                for line in stdout.lines() {
                    if !line.starts_with(' ') && line.contains(' ') {
                        if let Some((name, version_part)) = line.split_once(' ') {
                            let version = version_part
                                .trim_start_matches('v')
                                .split(':')
                                .next()
                                .unwrap_or("unknown")
                                .to_string();

                            tools.push(Tool {
                                name: name.to_string(),
                                version,
                                path: None,
                                manager: <Self as Querier>::name(self).to_string(),
                            });
                        }
                    }
                }
                tools
            },
            _ => Vec::new(),
        }
    }

    fn owns(&self, tool_name: &str) -> Option<Tool> {
        self.list().into_iter().find(|tool| tool.name == tool_name)
    }
}
