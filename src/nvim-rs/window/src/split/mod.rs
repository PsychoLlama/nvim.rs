//! Window splitting functions.
//!
//! This module provides Rust implementations of window splitting functions
//! from `src/nvim/window.c`.
//!
//! # Submodules
//!
//! - [`validation`]: Split validation and size calculations
//! - (future) [`execute`]: Split execution functions

pub mod validation;

// Re-export common items
pub use validation::*;
