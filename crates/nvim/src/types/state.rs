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

pub type StateCheckCallback = Option<unsafe fn(*mut VimState) -> ::core::ffi::c_int>;
pub type StateExecuteCallback =
    Option<unsafe fn(*mut VimState, ::core::ffi::c_int) -> ::core::ffi::c_int>;
pub struct VimState {
    pub check: StateCheckCallback,
    pub execute: StateExecuteCallback,
}
