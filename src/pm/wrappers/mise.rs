use crate::{
    find::Find,
    pm::{
        PmInfo, find_all_pms,
        types::{AsOrigin, Origin},
    },
};

/// mise - Modern asdf alternative
///
/// mise (formerly rtx) is a fast, polyglot tool version manager.
/// It's compatible with asdf but faster and more feature-rich.
pub struct Mise;

impl Mise {
    const NAME: &'static str = "mise";
}

impl AsOrigin for Mise {
    fn as_origin() -> Origin {
        Origin::Wrapper("mise")
    }
}

impl Find for Mise {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }
}
