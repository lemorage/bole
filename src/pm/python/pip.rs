use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms},
};

pub struct Pip;

impl Find for Pip {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        "pip"
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(self.name())
            .into_iter()
            .map(|mut pm_info| {
                // Clean pip's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "pip 25.1.1 from /long/path..." -> "25.1.1"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}
