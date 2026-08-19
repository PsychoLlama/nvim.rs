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

pub type off_T = off_t;

/// `os_defs.h`'s floor for [`MAXPATHL`]. Upstream raises `MAXPATHL` to
/// `PATH_MAX` where that is larger; this port does not, so the two are equal
/// and both names are kept because the transpiled code uses both.
pub const DEFAULT_MAXPATHL: ::core::ffi::c_int = 4096;

/// The longest path the editor will build, and the size of every scratch
/// buffer it builds one in (`NameBuff` among them). One copy for the tree;
/// the places that index with it say `MAXPATHL as usize`.
pub const MAXPATHL: ::core::ffi::c_int = DEFAULT_MAXPATHL;

/// `macros_defs.h`'s `os_fopen` mode: read, binary.
pub const READBIN: &::core::ffi::CStr = c"rb";
