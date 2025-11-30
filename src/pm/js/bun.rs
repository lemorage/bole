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

/// bun - All-in-one JavaScript runtime and toolkit
pub struct Bun;

impl Bun {
    const NAME: &'static str = "bun";

    fn parse_bun_tree(stdout: &str) -> Vec<Tool> {
        stdout
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with("node_modules"))
            .filter_map(|line| {
                let clean = line.trim_start_matches(|c: char| !c.is_alphanumeric());
                let (name, version) = clean.split_once('@')?;

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

impl Find for Bun {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &["~/.bun/bin/bun"]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let http = ureq::agent();
        let latest = Upstream::GitHub {
            owner: "oven-sh",
            repo: "bun",
        }
        .latest(&http)
        .ok()?;
        let cmd = update_cmd(Self::NAME, &pm_info.install_method);
        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Bun {
    fn category(&self) -> Category {
        Category::JavaScript
    }
}

impl ToolLister for Bun {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn is_available(&self) -> bool {
        Command::new("bun").arg("--version").output().is_ok()
    }

    fn list(&self) -> Vec<Tool> {
        let output = Command::new("bun")
            .args(["pm", "ls", "-g"])
            .output()
            .ok()
            .filter(|o| o.status.success());

        output
            .map(|o| {
                let stdout = String::from_utf8_lossy(&o.stdout);
                Self::parse_bun_tree(&stdout)
            })
            .unwrap_or_default()
    }

    fn owns(&self, tool_name: &str) -> Option<Tool> {
        self.list().into_iter().find(|t| t.name == tool_name)
    }
}
