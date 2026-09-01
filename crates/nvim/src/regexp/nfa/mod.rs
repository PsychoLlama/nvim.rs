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
mod op;
mod parse;
mod postfix;
mod run;
mod step;
mod sub;

pub(crate) use self::exec::*;
pub(crate) use self::op::*;
pub(crate) use self::run::*;

/// The parser refused the pattern, so there is no postfix program.
///
/// It stands for both of upstream's reasons to answer `FAIL`, which it never
/// told apart either:
///
/// * the pattern is invalid, and the arm that found it has already reported
///   which `E` code applies and set `rc_did_emsg`;
/// * 'regexpengine' is 0 and this engine has *declined* a pattern it could
///   only compile badly — a wide `\{n,m}` bound. Nothing is reported and
///   `vim_regcomp` hands the pattern to the backtracking engine instead.
///
/// The two are told apart by `rc_did_emsg`, which outlives the call, not by
/// this value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Rejected;

/// What a parse function answers: the postfix program has been appended to,
/// or the pattern was refused.
pub(crate) type Parsed<T = ()> = Result<T, Rejected>;
