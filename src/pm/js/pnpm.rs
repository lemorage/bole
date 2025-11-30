use std::process::Command;

use crate::{
    find::{Bump, Find},
    pm::{
        core::{
            types::{Categorizable, Category, PmInfo, Tool, ToolLister},
            updater::update_cmd,
            upstream::Upstream,
            version::VersionExt,
        },
        find_all_pms,
    },
};

/// pnpm - Fast disk space efficient package manager
pub struct Pnpm;

impl Pnpm {
    const NAME: &'static str = "pnpm";

    fn parse_pnpm_list(stdout: &str) -> Vec<Tool> {
        stdout
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with("Legend:"))
            .filter_map(|line| {
                let mut parts = line.split_whitespace();
                let name = parts.next()?;
                let version = parts.next()?;

                Some(Tool {
                    name: name.to_string(),
                    version: Some(version).version_or_unknown(),
                    path: None,
                    manager: Self::NAME.to_string(),
                })
            })
            .collect()
    }
}

impl Find for Pnpm {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &[
            "~/.local/share/pnpm/pnpm",
            "~/Library/pnpm/pnpm",
            "~/.volta/bin/pnpm",
            "~/.asdf/shims/pnpm",
        ]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let http = ureq::agent();
        let latest = Upstream::Npm("pnpm").latest(&http).ok()?;
        let cmd = update_cmd(Self::NAME, &pm_info.install_method);
        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Pnpm {
    fn category(&self) -> Category {
        Category::JavaScript
    }
}

impl ToolLister for Pnpm {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn is_available(&self) -> bool {
        Command::new("pnpm").arg("--version").output().is_ok()
    }

    fn list(&self) -> Vec<Tool> {
        let output = Command::new("pnpm")
            .args(["list", "-g", "--depth=0"])
            .output()
            .ok()
            .filter(|o| o.status.success());

        output
            .map(|o| {
                let stdout = String::from_utf8_lossy(&o.stdout);
                Self::parse_pnpm_list(&stdout)
            })
            .unwrap_or_default()
    }

    fn owns(&self, tool_name: &str) -> Option<Tool> {
        self.list().into_iter().find(|t| t.name == tool_name)
    }
}
