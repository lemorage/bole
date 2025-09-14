//! # Bole
//!
//! Package manager discovery system for the `bole` CLI tool.
//!
//! Detects package managers across all ecosystems: JavaScript, Python, Rust, system tools,
//! and version managers. Determines installation sources and resolves complex dependency chains.
//!
//! ## Architecture
//!
//! - [`find::Find`] - Core discovery trait implemented by all detectors
//! - [`pm::Detector`] - Combines discovery with categorization
//! - [`pm::all_package_managers()`] - Registry of all supported tools
//! - [`pm::determine_install_method()`] - Installation source attribution

pub mod find;
pub mod pm;
