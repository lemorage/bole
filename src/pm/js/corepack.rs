use crate::pm::types::{AsOrigin, Origin};

/// Corepack - Node.js package manager wrapper
///
/// Corepack is a zero-runtime-dependency Node.js script that acts as a bridge
/// between Node.js projects and package managers (npm, pnpm, yarn).
/// It's typically installed via Node.js and manages other package managers.
pub struct Corepack;

impl Corepack {
    #[allow(unused)]
    const NAME: &'static str = "corepack";
}

impl AsOrigin for Corepack {
    fn as_origin() -> Origin {
        Origin::Wrapper("Corepack")
    }
}

// Note: Corepack is a wrapper, not a discoverable package manager
// It doesn't implement Find trait since we don't actively search for it
// It's detected through path patterns when other PMs are managed by it
