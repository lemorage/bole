use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms},
};

pub struct Cabal;

impl Find for Cabal {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        "cabal"
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(self.name())
            .into_iter()
            .map(|mut pm_info| {
                // Clean cabal's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(2) {
                    // "cabal-install version 3.12.1.0" -> "3.12.1.0"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}
