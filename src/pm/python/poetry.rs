use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms},
};

pub struct Poetry;

impl Find for Poetry {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        "poetry"
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(self.name())
            .into_iter()
            .map(|mut pm_info| {
                // Clean poetry's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(2) {
                    // "Poetry (version 2.1.2)" -> "2.1.2"
                    let v = version.trim_end_matches(|c: char| c == ')' || c.is_whitespace());
                    pm_info.version = v.to_string();
                }
                pm_info
            })
            .collect()
    }
}
