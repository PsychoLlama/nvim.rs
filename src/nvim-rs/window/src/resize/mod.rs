//! Window resize functions.
//!
//! This module provides Rust implementations of window resize calculations
//! and helper functions from `src/nvim/window.c`.
//!
//! # Submodules
//!
//! - [`calculate`]: Resize calculations and room checks
//! - [`execute`]: Resize execution helper functions
//! - [`frame`]: Frame tree helpers for resize operations

pub mod calculate;
pub mod execute;
pub mod frame;

// Re-export common items
pub use calculate::*;
pub use execute::*;
pub use frame::*;
