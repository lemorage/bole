use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms},
};

pub struct Npm;

impl Find for Npm {
    type Output = PmInfo;
    fn name(&self) -> &'static str {
        "npm"
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(self.name())
    }
}

// No extra ops; discovery-only per current CLI usage
