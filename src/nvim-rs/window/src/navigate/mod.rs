//! Window navigation functions.
//!
//! This module provides Rust implementations of window finding and navigation
//! functions from `src/nvim/window.c` and `src/nvim/eval/window.c`.
//!
//! # Submodules
//!
//! - [`find`]: Window finding and lookup functions
//! - [`movement`]: Window movement and cursor navigation
//! - [`frame`]: Frame-based navigation helpers

pub mod find;
pub mod frame;
pub mod movement;

// Re-export common items
pub use find::*;
pub use frame::*;
pub use movement::*;
