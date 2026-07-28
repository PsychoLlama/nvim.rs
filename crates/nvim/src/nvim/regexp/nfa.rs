//! The NFA engine: a Pike VM, from upstream's regexp_nfa.c.
//!
//! A pattern is parsed into a postfix program ([`parse`], [`atom`] and the
//! families under them, over [`postfix`]), the program is built into a state
//! machine ([`build`]), and the machine is run by advancing a list of threads
//! over the input one character at a time ([`matcher`], [`step`]).
//!
//! The lists themselves, and the walk that puts a state on one, are
//! [`list`]; [`sub`] is the capture sets the threads on them carry.

#![deny(unsafe_op_in_unsafe_fn)]

mod assertions;
mod atom;
mod build;
mod classes;
mod collection;
mod compile;
mod composing;
mod cursor;
mod equi_class;
mod escape;
mod exec;
mod list;
mod literal;
mod matcher;
mod parse;
mod postfix;
mod run;
mod step;
mod sub;

pub(crate) use self::exec::*;
pub(crate) use self::run::*;
