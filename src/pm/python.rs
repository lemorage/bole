pub mod conda;
pub mod pdm;
pub mod pip;
pub mod poetry;
pub mod uv;

pub use conda::Conda;
pub use pdm::Pdm;
pub use pip::Pip;
pub use poetry::Poetry;
pub use uv::Uv;

// All Python package managers are now accessible via the enum in src/pm.rs
// No need for dynamic allocations - zero-cost abstractions!
