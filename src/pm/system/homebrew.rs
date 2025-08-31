use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms},
};

pub struct Homebrew;

impl Homebrew {
    const NAME: &'static str = "brew";
}

impl Find for Homebrew {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean homebrew's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "Homebrew 4.6.3" -> "4.6.3"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}
