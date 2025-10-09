use crate::{
    find::Find,
    pm::{Categorizable, Category, PmInfo, find_all_pms_with_args},
};

/// zig - Zig toolchain and built-in package manager
pub struct Zig;

impl Zig {
    const NAME: &'static str = "zig";
}

impl Find for Zig {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms_with_args(Self::NAME, &["version"])
    }
}

impl Categorizable for Zig {
    fn category(&self) -> Category {
        Category::Zig
    }
}
