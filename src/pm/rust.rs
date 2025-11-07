use std::process::Command;

use crate::{
    find::{Bump, Find},
    pm::{
        core::{
            tool_lister::{Tool, ToolLister},
            types::{Categorizable, Category, PmInfo},
            updater::update_cmd,
            upstream::Upstream,
            version::VersionExt,
        },
        find_all_pms_with_args,
    },
};

/// cargo - Rust package manager and build tool
pub struct Cargo;

impl Cargo {
    const NAME: &'static str = "cargo";
}

impl Find for Cargo {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &["~/.cargo/bin/cargo"]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms_with_args(Self::NAME, &["-V"])
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

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let http = ureq::agent();
        let latest = Upstream::GitHub {
            owner: "rust-lang",
            repo: "rust",
        }
        .latest(&http)
        .ok()?;
        let cmd = update_cmd(Self::NAME, &pm_info.install_method);
        Some(Bump { latest, cmd })
    }
}

impl ToolLister for Cargo {
    fn name(&self) -> &'static str {
        Self::NAME
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
                    if !line.starts_with(' ')
                        && line.contains(' ')
                        && let Some((name, version_part)) = line.split_once(' ')
                    {
                        let version = version_part
                            .trim_start_matches('v')
                            .split(':')
                            .next()
                            .version_or_unknown();

                        tools.push(Tool {
                            name: name.to_string(),
                            version,
                            path: None,
                            manager: Self::NAME.to_string(),
                        });
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

impl Categorizable for Cargo {
    fn category(&self) -> Category {
        Category::Rust
    }
}
