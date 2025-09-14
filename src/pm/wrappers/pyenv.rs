use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

/// pyenv - Python version manager
pub struct Pyenv;

impl Pyenv {
    const NAME: &'static str = "pyenv";
}

impl Find for Pyenv {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean pyenv's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(1) {
                    // "pyenv 2.3.24" -> "2.3.24"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}

impl Categorizable for Pyenv {
    fn category(&self) -> Category {
        Category::Tools
    }
}
