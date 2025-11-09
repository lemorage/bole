use crate::{
    find::{Bump, Find},
    pm::{
        core::{
            types::{Categorizable, Category, PmInfo},
            updater::update_cmd,
            upstream::Upstream,
        },
        find_all_pms,
    },
};

/// opam - OCaml package manager
pub struct Opam;

impl Opam {
    const NAME: &'static str = "opam";
}

impl Find for Opam {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &["~/.opam/default/bin/opam"]
    }

    fn find(&self) -> Vec<PmInfo> {
        find_all_pms(Self::NAME)
    }

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let http = ureq::agent();
        let latest = Upstream::GitHub {
            owner: "ocaml",
            repo: "opam",
        }
        .latest(&http)
        .ok()?;
        let cmd = update_cmd(Self::NAME, &pm_info.install_method);
        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Opam {
    fn category(&self) -> Category {
        Category::OCaml
    }
}
