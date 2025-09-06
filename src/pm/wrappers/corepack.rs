use crate::{
    find::Find,
    pm::{
        PmInfo, find_all_pms,
        types::{AsOrigin, Origin},
    },
};

/// Corepack - Node.js package manager wrapper
///
/// Corepack is a zero-runtime-dependency Node.js script that acts as a bridge
/// between Node.js projects and package managers (npm, pnpm, yarn).
pub struct Corepack;

impl Corepack {
    const NAME: &'static str = "corepack";
}

impl AsOrigin for Corepack {
    fn as_origin() -> Origin {
        Origin::Wrapper("Corepack")
    }
}

impl Find for Corepack {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }
}
