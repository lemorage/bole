use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms},
};

pub struct Macports;

impl Find for Macports {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        "port"
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(self.name())
            .into_iter()
            .map(|mut pm_info| {
                // Clean port's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "MacPorts 2.11.5" -> "2.11.5"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}
