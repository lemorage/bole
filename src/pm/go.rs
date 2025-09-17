use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms_with_args},
};

/// go - Go toolchain and module manager
pub struct Go;

impl Go {
    const NAME: &'static str = "go";
}

impl Find for Go {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &[
            "/usr/local/go/bin/go",
            "~/.local/bin/go",
            "/opt/homebrew/bin/go",
            "/usr/bin/go",
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
}

impl Categorizable for Go {
    fn category(&self) -> Category {
        Category::Go
    }
}
