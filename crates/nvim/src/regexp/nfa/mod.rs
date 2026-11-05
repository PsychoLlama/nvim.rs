//! The NFA engine: a Pike VM, from upstream's regexp_nfa.c.
//!
//! A pattern is parsed into a postfix program ([`parse`], [`atom`] and the
//! families under them, over [`postfix`]), the program is built into a state
//! machine ([`build`]), and the machine is run by advancing a list of threads
//! over the input one character at a time ([`matcher`], [`step`]).
//!
//! The lists themselves, and the walk that puts a state on one, are
//! [`list`]; [`sub`] is the capture sets the threads on them carry.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

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
mod op;
mod parse;
mod postfix;
mod program;
mod run;
mod step;
mod sub;

pub(crate) use self::exec::*;
pub(crate) use self::op::*;
pub(crate) use self::postfix::Postfix;
pub(crate) use self::program::*;
pub(crate) use self::run::*;

/// What a parse function answers: the postfix program has been appended to,
/// or why the pattern is refused -- an invalid pattern, or, under
/// 'regexpengine' 0, a pattern this engine declines to compile badly (a wide
/// `\{n,m}` bound), which `vim_regcomp` hands to the backtracking engine.
pub(crate) type Parsed<T = ()> = super::Compiled<T>;
