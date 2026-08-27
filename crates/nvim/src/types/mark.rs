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

pub type MarkAdjustMode = ::core::ffi::c_uint;
pub type MarkGet = ::core::ffi::c_uint;
pub type MarkMove = ::core::ffi::c_uint;
pub type MarkMoveRes = ::core::ffi::c_uint;
/// Not `Copy`: `additional_data` is the ShaDa extra data the mark owns,
/// and `free_fmark` takes a mark by value to release it.
#[derive(Clone)]
pub struct fmark_T {
    pub mark: pos_T,
    pub fnum: ::core::ffi::c_int,
    pub timestamp: Timestamp,
    pub view: fmarkv_T,
    pub additional_data: *mut AdditionalData,
}
#[derive(Copy, Clone)]
pub struct fmarkv_T {
    pub topline_offset: linenr_T,
    pub skipcol: colnr_T,
}

impl fmarkv_T {
    /// The view an unset mark carries: `topline_offset` at `MAXLNUM` means
    /// "remember nothing", so `mark_view_restore` computes a topline far
    /// below line 1 and gives up.
    pub const NONE: Self = Self {
        topline_offset: crate::pos::MAXLNUM.cast_signed(),
        skipcol: 0,
    };
}

impl fmark_T {
    /// A mark that is not set.
    ///
    /// This is the value a caller lending `mark_get` (or `pos_to_mark`) a
    /// slot starts that slot from: the lookups fill in `mark` and `fnum`, and
    /// leave the remaining fields as they found them.
    pub const UNSET: Self = Self {
        mark: pos_T {
            lnum: 0,
            col: 0,
            coladd: 0,
        },
        fnum: 0,
        timestamp: 0,
        view: fmarkv_T::NONE,
        additional_data: ::core::ptr::null_mut(),
    };
}
/// Not `Copy`: an owned `fname` on top of [`fmark_T`]'s own.
#[derive(Clone)]
pub struct xfmark_T {
    pub fmark: fmark_T,
    pub fname: *mut ::core::ffi::c_char,
}
