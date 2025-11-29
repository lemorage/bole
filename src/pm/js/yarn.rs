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
            .output();

        match output {
            Ok(output) if output.status.success() => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                // Yarn outputs multiple JSON objects, one per line
                let mut tools = Vec::new();
                for line in stdout.lines() {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                        if json.get("type").and_then(|t| t.as_str()) == Some("tree") {
                            if let Some(data) = json.get("data") {
                                if let Some(trees) = data.get("trees").and_then(|t| t.as_array()) {
                                    for tree in trees {
                                        if let Some(name) =
                                            tree.get("name").and_then(|n| n.as_str())
                                        {
                                            // Parse "package@version" format
                                            let parts: Vec<&str> = name.rsplitn(2, '@').collect();
                                            if parts.len() == 2 {
                                                tools.push(Tool {
                                                    name: parts[1].to_string(),
                                                    version: Some(parts[0]).version_or_unknown(),
                                                    path: None,
                                                    manager: Self::NAME.to_string(),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                tools
            },
            _ => Vec::new(),
        }
    }

    fn owns(&self, tool_name: &str) -> Option<Tool> {
        self.list().into_iter().find(|t| t.name == tool_name)
    }
}
