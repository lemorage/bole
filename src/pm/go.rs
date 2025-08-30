use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms_with_args},
};

pub struct Go;

impl Find for Go {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        "go"
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms_with_args(self.name(), &["version"])
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
