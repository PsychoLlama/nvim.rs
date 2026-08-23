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

pub type CheckItem =
    Option<unsafe fn(*mut ::core::ffi::c_void, *const ::core::ffi::c_char) -> varnumber_T>;
#[derive(Copy, Clone)]
pub struct FileDescriptor {
    pub fd: ::core::ffi::c_int,
    pub buffer: *mut ::core::ffi::c_char,
    pub read_pos: *mut ::core::ffi::c_char,
    pub write_pos: *mut ::core::ffi::c_char,
    pub wr: bool,
    pub eof: bool,
    pub non_blocking: bool,
    pub bytes_read: uint64_t,
}
