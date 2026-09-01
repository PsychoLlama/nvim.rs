//! The backtracking engine, transpiled from upstream's regexp_bt.c.
//!
//! Henry Spencer's matcher as Vim reshaped it: a pattern compiles to a
//! program of nodes and the matcher walks it, backtracking on failure.
//!
//! The program emitter and the parser above it are [`compile`], [`piece`]
//! and [`atom`]; the walk that runs a program is [`matcher`] over
//! [`single`], with [`resume`] unwinding the decisions it saved on
//! [`state`]'s stack.

#![deny(unsafe_op_in_unsafe_fn)]

mod atom;
mod collection;
mod compile;
mod equi_class;
mod escape;
mod exec;
mod literal;
mod matcher;
mod op;
mod piece;
mod repeat;
mod resume;
mod single;
mod state;

pub(crate) use self::compile::*;
pub(crate) use self::exec::*;
pub use self::piece::*;
pub(crate) use self::state::*;
