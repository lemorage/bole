use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms},
};

/// conda - Package and environment manager
pub struct Conda;

impl Conda {
    const NAME: &'static str = "conda";
}

impl Find for Conda {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &[
            "~/miniconda3/bin/conda",
            "~/anaconda3/bin/conda",
            "/opt/miniconda3/bin/conda",
            "/opt/anaconda3/bin/conda",
        ]
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

impl Categorizable for Conda {
    fn category(&self) -> Category {
        Category::Python
    }
}
