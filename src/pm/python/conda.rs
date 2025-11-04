use crate::{
    find::{Bump, Find},
    pm::{
        core::{
            types::{Categorizable, Category, ChainLink, Origin, PmInfo},
            updater::update_cmd,
            upstream::Upstream,
        },
        find_all_pms,
    },
};

/// conda - Package and environment manager
pub struct Conda;

impl Conda {
    const NAME: &'static str = "conda";
}

impl ChainLink for Conda {
    fn as_origin() -> Origin {
        Origin::PackageManager("Conda")
    }
}

impl Find for Conda {
    type Output = PmInfo;

    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn search_paths(&self) -> &'static [&'static str] {
        &[
            // Home installs
            "~/miniconda3/bin/conda",
            "~/anaconda3/bin/conda",
            "~/miniconda/bin/conda",
            "~/anaconda/bin/conda",
            "~/miniforge3/bin/conda",
            "~/mambaforge/bin/conda",
            // /opt installs
            "/opt/miniconda3/bin/conda",
            "/opt/anaconda3/bin/conda",
            "/opt/miniforge3/bin/conda",
            "/opt/mambaforge/bin/conda",
            // /usr/local installs
            "/usr/local/miniconda3/bin/conda",
            "/usr/local/anaconda3/bin/conda",
            "/usr/local/miniforge3/bin/conda",
            "/usr/local/mambaforge/bin/conda",
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

    fn check_bump(&self, pm_info: &PmInfo) -> Option<Bump> {
        let upstream = Upstream::GitHub {
            owner: "conda",
            repo: "conda",
        };
        let http = ureq::agent();
        let latest = upstream.latest(&http).ok()?;
        let cmd = update_cmd(Self::NAME, &pm_info.install_method);
        Some(Bump { latest, cmd })
    }
}

impl Categorizable for Conda {
    fn category(&self) -> Category {
        Category::Python
    }
}
