//! Normal mode operator commands.
//!
//! This module provides operator command helpers organized by operation type.

pub mod change;
pub mod delete;
pub mod format;
pub mod yank;

pub use change::*;
pub use delete::*;
pub use format::*;
pub use yank::*;
