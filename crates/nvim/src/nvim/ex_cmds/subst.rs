//! `:substitute` and everything under it.
//!
//! Split by stage, because upstream's `do_sub` is one function of a thousand
//! lines: `parse` holds the pieces that read the command line and the
//! previous pattern, `args` turns a whole `:s` argument into what the engine
//! needs, `exec` is the engine and the state its stages share, `confirm` is
//! the `c` flag's dialogue, `replace` builds one replacement and puts the
//! rebuilt line in the buffer, and `report` is what the user sees afterwards
//! -- the "N substitutions on N lines" summary, and the `'inccommand'`
//! preview.
//!
//! Original: `src/nvim/ex_cmds.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;

// The carve of the transpiled module; see each child's docs.
mod args;
mod confirm;
mod exec;
mod parse;
mod replace;
mod report;

pub(crate) use self::exec::*;
pub use self::parse::*;
pub use self::report::*;
