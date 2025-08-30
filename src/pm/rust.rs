use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms_with_args},
};

pub struct Cargo;

impl Find for Cargo {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        "cargo"
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms_with_args(self.name(), &["-V"])
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
}
