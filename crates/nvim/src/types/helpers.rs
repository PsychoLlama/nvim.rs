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

#[derive(Copy, Clone)]
pub struct TryState {
    pub(crate) current_exception: Option<ExcId>,
    pub got_int: ::core::ffi::c_int,
    pub did_throw: bool,
    pub need_rethrow: ::core::ffi::c_int,
    pub did_emsg: ::core::ffi::c_int,
}

impl TryState {
    /// The zeroed state a caller declares before handing it to `try_enter`,
    /// which overwrites every field. Nothing reads one of these before that.
    pub(crate) const INIT: TryState = TryState {
        current_exception: None,
        got_int: 0,
        did_throw: false,
        need_rethrow: 0,
        did_emsg: 0,
    };
}

impl Default for TryState {
    fn default() -> Self {
        Self::INIT
    }
}
