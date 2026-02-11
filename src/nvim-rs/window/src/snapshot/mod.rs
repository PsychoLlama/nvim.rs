//! Window layout snapshot functions.
//!
//! This module provides Rust implementations of window layout snapshot
//! functions from `src/nvim/window.c`.
//!
//! Snapshots capture the window layout state for later restoration,
//! used primarily for help window handling.
//!
//! # Submodules
//!
//! - [`lifecycle`]: Snapshot allocation, deallocation, and traversal
//! - [`state`]: Snapshot state queries and validation

pub mod lifecycle;
pub mod state;

// Re-export common items
pub use lifecycle::*;
pub use state::*;
