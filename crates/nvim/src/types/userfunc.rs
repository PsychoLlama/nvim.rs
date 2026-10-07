#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;

/// Fills in a call's arguments once its user function is known: the
/// argument slice, how many leading slots to leave, and the function.
/// Answers how many arguments the call now has.
pub type ArgvFunc = fn(&[TypVal], usize, &UserFunc) -> usize;
/// A funccall's place in the funccall table.
pub(crate) type FcId = crate::id_table::TableId<FuncCall>;
