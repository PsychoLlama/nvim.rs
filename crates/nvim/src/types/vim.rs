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

/// The `int` return convention almost every editor function still speaks:
/// `OK` on success, `FAIL` on failure. `vim_defs.h`'s, one copy for the tree.
pub const OK: ::core::ffi::c_int = 1;
/// The failing half of the [`OK`] convention.
pub const FAIL: ::core::ffi::c_int = 0;

pub type CdCause = ::core::ffi::c_int;
pub type CdScope = ::core::ffi::c_int;
/// Which scope a `:cd` applies to (`getcwd()`/`haslocaldir()` report it).
pub const kCdScopeInvalid: CdScope = -1;
pub const kCdScopeWindow: CdScope = 0;
pub const kCdScopeTabpage: CdScope = 1;
pub const kCdScopeGlobal: CdScope = 2;
pub type Direction = ::core::ffi::c_int;
