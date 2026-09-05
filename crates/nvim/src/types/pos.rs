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

pub type ColNr = ::core::ffi::c_int;
pub type LineNr = int32_t;
#[derive(Copy, Clone, Default)]
pub struct lpos_T {
    pub lnum: LineNr,
    pub col: ColNr,
}
#[derive(Copy, Clone, Default, PartialEq, Eq)]
#[repr(C)]
pub struct pos_T {
    pub lnum: LineNr,
    pub col: ColNr,
    pub coladd: ColNr,
}

impl pos_T {
    /// The same position moved to `col`, for the read-modify-write of a
    /// position held in a [`crate::global_cell::GlobalCell`].
    pub fn with_col(self, col: ColNr) -> pos_T {
        pos_T { col, ..self }
    }

    /// The same position moved to `lnum`.
    pub fn with_lnum(self, lnum: LineNr) -> pos_T {
        pos_T { lnum, ..self }
    }
}
