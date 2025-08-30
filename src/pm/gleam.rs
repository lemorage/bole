use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms},
};

pub struct Gleam;

impl Find for Gleam {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        "gleam"
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(self.name())
            .into_iter()
            .map(|mut pm_info| {
                // Clean gleam's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "gleam 1.11.1" -> "1.11.1"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}
