use crate::{
    find::Find,
    pm::{
        Categorizable, Category, PmInfo,
        core::types::{ChainLink, Origin},
        find_all_pms,
    },
};

/// asdf - Universal version manager
///
/// asdf is a CLI tool that can manage multiple language runtime versions
/// on a per-project basis. It is used to install and switch between
/// different versions of languages.
pub struct Asdf;

impl Asdf {
    const NAME: &'static str = "asdf";
}

impl ChainLink for Asdf {
    fn as_origin() -> Origin {
        Origin::Wrapper("asdf")
    }
}

impl Find for Asdf {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &["~/.asdf/bin/asdf"]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
            .into_iter()
            .map(|mut pm_info| {
                // Clean asdf's verbose output
                if let Some(version) = pm_info.version.split_whitespace().nth(2) {
                    // "asdf version 0.18.0 (...)" -> "0.18.0"
                    pm_info.version = version.to_string();
                }
                pm_info
            })
            .collect()
    }
}

impl Categorizable for Asdf {
    fn category(&self) -> Category {
        Category::Tools
    }
}
