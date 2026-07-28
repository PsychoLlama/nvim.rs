//! The backtracking engine, transpiled from upstream's regexp_bt.c.
//!
//! Henry Spencer's matcher as Vim reshaped it: a pattern compiles to a
//! program of nodes and the matcher walks it, backtracking on failure.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

#[allow(unused_imports)]
use super::*;

mod atom;
mod collection;
mod compile;
mod equi_class;
mod escape;
mod exec;
mod literal;
mod matcher;
mod piece;
mod repeat;
mod resume;
mod single;
mod state;

pub(crate) use self::compile::*;
pub(crate) use self::exec::*;
pub(crate) use self::matcher::*;
pub use self::piece::*;
pub(crate) use self::state::*;
