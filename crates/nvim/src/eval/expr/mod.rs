//! The expression grammar, one module per kind of operand or
//! operator.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

mod cursor;
pub(crate) use self::cursor::*;
mod level;
pub use self::level::*;
mod arith;
pub(crate) use self::arith::*;
mod compare;
pub(crate) use self::compare::*;
mod literal;
pub(crate) use self::literal::*;
mod container;
pub(crate) use self::container::*;
mod index;
pub(crate) use self::index::*;
mod call;
pub(crate) use self::call::*;
mod complete;
pub(crate) use self::complete::*;
