use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms},
};

pub struct Conda;

impl Find for Conda {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        "conda"
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(self.name())
            .into_iter()
            .map(|mut pm_info| {
                // Clean conda's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "conda 25.7.0" -> "25.7.0"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}
