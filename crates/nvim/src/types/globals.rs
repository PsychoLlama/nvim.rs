#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical definitions from `globals.h`, hoisted out of the per-module copies
// c2rust emitted. One definition per logical name; every module re-exports
// here.

/// The size of `IObuff`, the shared scratch every message and every file line
/// is formatted through. One copy for the tree; the places that index with it
/// say `IOSIZE as usize`.
pub const IOSIZE: ::core::ffi::c_int = 1024 + 1;
