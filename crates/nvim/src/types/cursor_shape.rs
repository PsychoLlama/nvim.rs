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

pub type CursorShape = ::core::ffi::c_uint;
#[derive(Copy, Clone)]
pub struct cursorentry_T {
    pub full_name: *mut ::core::ffi::c_char,
    pub shape: CursorShape,
    pub mshape: ::core::ffi::c_int,
    pub percentage: ::core::ffi::c_int,
    pub blinkwait: ::core::ffi::c_int,
    pub blinkon: ::core::ffi::c_int,
    pub blinkoff: ::core::ffi::c_int,
    pub id: ::core::ffi::c_int,
    pub id_lm: ::core::ffi::c_int,
    pub name: *mut ::core::ffi::c_char,
    pub used_for: ::core::ffi::c_char,
}
