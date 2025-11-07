//! Core infrastructure for package manager operations.
//!
//! This module provides the foundational utilities that all package managers
//! rely on, including:
//!
//! - **Install detection**: Determines how package managers were installed
//! - **Path discovery**: Utilities for finding package managers in non-standard
//!   locations
//! - **Types**: Core data structures (`PmInfo`, `Category`) used throughout
//! - **Update commands**: Generates appropriate update commands based on
//!   installation context
//! - **Upstream checking**: Unified interface to query upstream registries for
//!   latest versions
//! - **Version handling**: Consistent version extraction and normalization with
//!   `??` as the unknown marker

pub(super) mod install_method;
pub(super) mod search_paths;
pub mod types;
pub(super) mod updater;
pub(super) mod upstream;
pub(super) mod version;
