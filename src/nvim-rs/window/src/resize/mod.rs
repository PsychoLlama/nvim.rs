//! Window resize functions.
//!
//! This module provides Rust implementations of window resize calculations
//! and helper functions from `src/nvim/window.c`.
//!
//! # Submodules
//!
//! - [`calculate`]: Resize calculations and room checks
//! - [`execute`]: Resize execution helper functions
//! - [`fraction`]: Fraction and scroll default calculations
//! - [`frame`]: Frame tree helpers for resize operations
//! - [`minsize`]: Minimum size calculations for tabpages

pub mod calculate;
pub mod execute;
pub mod fraction;
pub mod frame;
pub mod minsize;
pub mod validate;

// Re-export common items
pub use calculate::*;
pub use execute::*;
pub use fraction::*;
pub use frame::*;
pub use minsize::*;
pub use validate::*;
