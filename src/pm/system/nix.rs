use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

pub struct Nix;

impl Nix {
    const NAME: &'static str = "nix";
}

impl Find for Nix {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        "nix"
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean nix's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(2) {
                    // "nix (Nix) 2.30.2" -> "2.30.2"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}

impl Categorizable for Nix {
    fn category(&self) -> Category {
        Category::System
    }
}
