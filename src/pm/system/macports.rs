use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

/// port - MacPorts package manager for macOS
pub struct Macports;

impl Macports {
    const NAME: &'static str = "port";
}

impl Find for Macports {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &["/opt/local/bin/port"]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
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

impl Categorizable for Macports {
    fn category(&self) -> Category {
        Category::System
    }
}
