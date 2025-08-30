use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms},
};

pub struct Uv;

impl Find for Uv {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        "uv"
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(self.name())
            .into_iter()
            .map(|mut pm_info| {
                // Clean uv's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "uv 0.8.4 (...)" -> "0.8.4"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}
