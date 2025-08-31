use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms},
};

pub struct Pdm;

impl Pdm {
    const NAME: &'static str = "pdm";
}

impl Find for Pdm {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
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
