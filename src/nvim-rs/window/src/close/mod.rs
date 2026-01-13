//! Window close functions.
//!
//! This module provides Rust implementations of window close validation
//! and helper functions from `src/nvim/window.c`.
//!
//! # Submodules
//!
//! - [`validation`]: Close validation and safety checks
//! - [`execute`]: Close execution helper functions

pub mod execute;
pub mod validation;

// Re-export common items
pub use execute::*;
pub use validation::*;
