//! The NFA engine, transpiled from upstream's regexp_nfa.c.
//!
//! A pattern compiles to postfix form, the postfix form to a state
//! machine, and the matcher advances a list of threads over the input.
//!
//! Moved out of the parent module as it stood after transpilation;
//! the bodies are unchanged.

#[allow(unused_imports)]
use super::*;

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
mod literal;
mod matcher;
mod parse;
mod postfix;
mod run;
mod step;
mod sub;

pub(crate) use self::build::*;
pub(crate) use self::compile::*;
pub(crate) use self::exec::*;
pub(crate) use self::matcher::*;
pub(crate) use self::parse::*;
pub(crate) use self::run::*;
pub(crate) use self::sub::*;
