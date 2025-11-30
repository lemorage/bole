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

/// npm - Node.js package manager
pub struct Npm;

impl Npm {
    const NAME: &'static str = "npm";

    fn parse_npm_json(stdout: &[u8]) -> Option<Vec<Tool>> {
        let json = serde_json::from_slice::<serde_json::Value>(stdout).ok()?;
        let deps = json.get("dependencies")?.as_object()?;

        Some(
            deps.iter()
                .map(|(name, info)| Tool {
                    name: name.clone(),
                    version: info
                        .get("version")
                        .and_then(|v| v.as_str())
                        .version_or_unknown(),
                    path: None,
                    manager: Self::NAME.to_string(),
                })
                .collect(),
        )
    }
}

impl Find for Npm {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &["~/.volta/bin/npm", "~/.asdf/shims/npm"]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let http = ureq::agent();
        let latest = Upstream::Npm("npm").latest(&http).ok()?;
        let cmd = update_cmd(Self::NAME, &pm_info.install_method);
        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Npm {
    fn category(&self) -> Category {
        Category::JavaScript
    }
}

impl ToolLister for Npm {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn is_available(&self) -> bool {
        Command::new("npm").arg("--version").output().is_ok()
    }

    fn list(&self) -> Vec<Tool> {
        let output = Command::new("npm")
            .args(["list", "-g", "--depth=0", "--json"])
            .output()
            .ok()
            .filter(|o| o.status.success());

        output
            .and_then(|o| Self::parse_npm_json(&o.stdout))
            .unwrap_or_default()
    }

    fn owns(&self, tool_name: &str) -> Option<Tool> {
        self.list().into_iter().find(|t| t.name == tool_name)
    }
}
