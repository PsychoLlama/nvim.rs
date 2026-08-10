//! `:substitute` and everything under it.
//!
//! Split three ways because `do_sub` alone is 1,220 lines: `parse` holds the
//! pieces that read the command line and the previous pattern, `exec` is
//! `do_sub` itself, and `report` is what the user sees afterwards -- the
//! "N substitutions on N lines" summary, and the `'inccommand'` preview.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

// The carve of the transpiled module; see each child's docs.
mod exec;
mod parse;
mod report;

pub(crate) use self::exec::*;
pub use self::parse::*;
pub use self::report::*;
