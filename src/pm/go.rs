use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

use crate::{
    find::{Bump, Find},
    pm::{
        core::{
            types::{Categorizable, Category, PmInfo, Tool, ToolLister},
            updater::update_cmd,
            upstream::Upstream,
            version::VersionExt,
        },
        find_all_pms_with_args,
    },
};

/// go - Go toolchain and module manager
pub struct Go;

impl Go {
    const NAME: &'static str = "go";

    /// Extract version from `go version -m` output.
    fn get_tool_version(binary_path: &Path) -> Option<String> {
        let output = Command::new("go")
            .args(["version", "-m", binary_path.to_str()?])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        // Parse output to find the module version
        // Format: "\tmod\tmodule_path\tversion\thash"
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .find(|line| line.trim_start().starts_with("mod\t"))
            .and_then(|line| {
                let parts: Vec<&str> = line.trim_start().split('\t').collect();
                parts.get(2).map(|v| v.to_string())
            })
    }
}

impl Find for Go {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &[
            "/usr/local/go/bin/go",
            "/opt/homebrew/opt/go/libexec/bin/go",
            "/usr/local/opt/go/libexec/bin/go",
            "/home/linuxbrew/.linuxbrew/opt/go/libexec/bin/go",
        ]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms_with_args(Self::NAME, &["version"])
            .into_iter()
            .map(|mut pm_info| {
                // Clean go's verbose output
                if let Some(version_info) = pm_info.version.split_whitespace().nth(2) {
                    // "go version go1.24.5 darwin/arm64" -> "1.24.5"
                    let version = match version_info.strip_prefix("go") {
                        Some(rest) if rest.starts_with(|c: char| c.is_ascii_digit()) => rest,
                        _ => version_info,
                    };
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let http = ureq::agent();
        let latest = Upstream::GitHub {
            owner: "golang",
            repo: "go",
        }
        .latest(&http)
        .ok()?;
        let cmd = update_cmd(Self::NAME, &pm_info.install_method);
        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Go {
    fn category(&self) -> Category {
        Category::Go
    }
}

impl ToolLister for Go {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn is_available(&self) -> bool {
        Command::new("go").arg("version").output().is_ok()
    }

    fn list(&self) -> Vec<Tool> {
        let gopath = env::var("GOPATH")
            .ok()
            .or_else(|| env::var("HOME").ok().map(|home| format!("{}/go", home)))
            .unwrap_or_else(|| "/usr/local/go".to_string());

        let bin_dir = PathBuf::from(gopath).join("bin");
        if !bin_dir.exists() {
            return Vec::new();
        }

        // Read all binaries in GOPATH/bin
        let entries = match std::fs::read_dir(&bin_dir) {
            Ok(entries) => entries,
            Err(_) => return Vec::new(),
        };

        entries
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_type()
                    .map(|ft| ft.is_file() || ft.is_symlink())
                    .unwrap_or(false)
            })
            .filter_map(|entry| {
                let path = entry.path();
                let name = path.file_name()?.to_str()?.to_string();

                // Use `go version -m` to get module info
                let version = Self::get_tool_version(&path)
                    .as_deref()
                    .version_or_unknown();

                Some(Tool {
                    name,
                    version,
                    path: Some(path.to_string_lossy().to_string()),
                    manager: Self::NAME.to_string(),
                })
            })
            .collect()
    }

    fn owns(&self, tool_name: &str) -> Option<Tool> {
        self.list().into_iter().find(|t| t.name == tool_name)
    }
}
