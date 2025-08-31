use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms},
};

pub struct Conda;

impl Conda {
    const NAME: &'static str = "conda";
}

impl Find for Conda {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
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
