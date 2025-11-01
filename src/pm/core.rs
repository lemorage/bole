//! Core infrastructure for package manager operations.
//!
//! This module provides the foundational utilities that all package managers
//! rely on, including:
//!
//! - **Attribution**: Determines how package managers were installed and what
//!   tools they manage
//! - **Version checking**: Unified interface to query upstream registries for
//!   latest versions
//! - **Update commands**: Generates appropriate update commands based on
//!   installation context
//! - **Path discovery**: Utilities for finding package managers in non-standard
//!   locations
//! - **Types**: Core data structures (`PmInfo`, `Category`) used throughout

pub(super) mod attribution;
pub(super) mod search_paths;
pub mod types;
pub(super) mod updater;
pub(super) mod upstream;
