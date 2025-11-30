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

/// yarn - JavaScript package manager
pub struct Yarn;

impl Yarn {
    const NAME: &'static str = "yarn";

    fn parse_yarn_ndjson(stdout: &str) -> Vec<Tool> {
        stdout
            .lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .filter(|json| json.get("type").and_then(|t| t.as_str()) == Some("tree"))
            .flat_map(|json| {
                json.get("data")
                    .and_then(|d| d.get("trees"))
                    .and_then(|t| t.as_array())
                    .cloned()
                    .unwrap_or_default()
            })
            .filter_map(|tree| {
                let name = tree.get("name")?.as_str()?;
                let (pkg, version) = name.rsplit_once('@')?;

                Some(Tool {
                    name: pkg.to_string(),
                    version: Some(version).version_or_unknown(),
                    path: None,
                    manager: Self::NAME.to_string(),
                })
            })
            .collect()
    }
}

impl Find for Yarn {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &[
            "~/.yarn/bin/yarn",
            "~/.volta/bin/yarn",
            "~/.asdf/shims/yarn",
        ]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let http = ureq::agent();
        let latest = Upstream::Npm("yarn").latest(&http).ok()?;
        let cmd = update_cmd(Self::NAME, &pm_info.install_method);
        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Yarn {
    fn category(&self) -> Category {
        Category::JavaScript
    }
}

impl ToolLister for Yarn {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn is_available(&self) -> bool {
        Command::new("yarn").arg("--version").output().is_ok()
    }

    fn list(&self) -> Vec<Tool> {
        let output = Command::new("yarn")
            .args(["global", "list", "--json"])
            .output()
            .ok()
            .filter(|o| o.status.success());

        output
            .map(|o| {
                let stdout = String::from_utf8_lossy(&o.stdout);
                Self::parse_yarn_ndjson(&stdout)
            })
            .unwrap_or_default()
    }

    fn owns(&self, tool_name: &str) -> Option<Tool> {
        self.list().into_iter().find(|t| t.name == tool_name)
    }
}
