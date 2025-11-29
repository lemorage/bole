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
            .output();

        match output {
            Ok(output) if output.status.success() => {
                // Parse JSON output
                if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                    if let Some(deps) = json.get("dependencies").and_then(|d| d.as_object()) {
                        return deps
                            .iter()
                            .map(|(name, info)| Tool {
                                name: name.clone(),
                                version: info
                                    .get("version")
                                    .and_then(|v| v.as_str())
                                    .version_or_unknown(),
                                path: None,
                                manager: Self::NAME.to_string(),
                            })
                            .collect();
                    }
                }
                Vec::new()
            },
            _ => Vec::new(),
        }
    }

    fn owns(&self, tool_name: &str) -> Option<Tool> {
        self.list().into_iter().find(|t| t.name == tool_name)
    }
}
