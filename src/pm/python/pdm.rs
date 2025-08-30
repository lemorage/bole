use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms},
};

pub struct Pdm;

impl Find for Pdm {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        "pdm"
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(self.name())
            .into_iter()
            .map(|mut pm_info| {
                // Clean pdm's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(2) {
                    // "PDM, version 2.25.9" -> "2.25.9"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}
