use crate::{
    find::Find,
    pm::{PmInfo, find_all_pms},
};

pub struct Pnpm;
impl Find for Pnpm {
    type Output = PmInfo;
    fn name(&self) -> &'static str {
        "pnpm"
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(self.name())
    }
}
