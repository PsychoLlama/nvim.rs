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

/// What `:normal` sets aside. Not `Copy`: `tabuf` owns the typeahead it
/// saved. `Default` is the state its callers declare it in.
#[derive(Default)]
pub struct save_state_T {
    pub save_msg_scroll: ::core::ffi::c_int,
    pub save_restart_edit: ::core::ffi::c_int,
    pub save_msg_didout: bool,
    pub save_State: ::core::ffi::c_int,
    pub save_finish_op: bool,
    pub save_opcount: ::core::ffi::c_int,
    pub save_reg_executing: ::core::ffi::c_int,
    pub save_pending_end_reg_executing: bool,
    pub tabuf: tasave_T,
}
