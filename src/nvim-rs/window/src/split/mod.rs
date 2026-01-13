//! Window splitting functions.
//!
//! This module provides Rust implementations of window splitting functions
//! from `src/nvim/window.c`.
//!
//! # Submodules
//!
//! - [`validation`]: Split validation and size calculations
//! - [`execute`]: Split execution helper functions

pub mod execute;
pub mod validation;

// Re-export common items
pub use execute::*;
pub use validation::*;
