use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms},
};

pub struct Npm;

impl Npm {
    const NAME: &'static str = "npm";
}

impl Find for Npm {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }
}
