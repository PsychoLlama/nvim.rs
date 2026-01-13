//! Window resize functions.
//!
//! This module provides Rust implementations of window resize calculations
//! and helper functions from `src/nvim/window.c`.
//!
//! # Submodules
//!
//! - [`calculate`]: Resize calculations and room checks
//! - [`execute`]: Resize execution helper functions

pub mod calculate;
pub mod execute;

// Re-export common items
pub use calculate::*;
pub use execute::*;
