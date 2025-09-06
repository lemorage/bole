use crate::{
    find::Find,
    pm::{
        PmInfo, find_all_pms,
        types::{AsOrigin, Origin},
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

impl AsOrigin for Asdf {
    fn as_origin() -> Origin {
        Origin::Wrapper("asdf")
    }
}

impl Find for Asdf {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }
}
