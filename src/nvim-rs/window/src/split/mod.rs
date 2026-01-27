//! Window splitting functions.
//!
//! This module provides Rust implementations of window splitting functions
//! from `src/nvim/window.c`.
//!
//! # Submodules
//!
//! - [`validation`]: Split validation and size calculations
//! - [`execute`]: Split execution helper functions
//! - [`frame`]: Frame tree helpers for split operations

pub mod execute;
pub mod frame;
pub mod validation;

// Re-export common items
pub use execute::*;
pub use frame::*;
pub use validation::*;
