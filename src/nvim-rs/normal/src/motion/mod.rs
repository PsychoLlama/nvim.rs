//! Normal mode motion commands.
//!
//! This module provides motion command helpers organized by direction type.

pub mod horizontal;
pub mod search;
pub mod vertical;
pub mod word;

pub use horizontal::*;
pub use search::*;
pub use vertical::*;
pub use word::*;
