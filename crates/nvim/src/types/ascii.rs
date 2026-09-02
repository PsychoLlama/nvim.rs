#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical definitions from `ascii_defs.h`, hoisted out of the per-module
// copies c2rust emitted. One definition per logical name; every module
// re-exports here.

/// The string terminator, as the `int` a widened byte compares against.
/// `ascii_defs.h`'s `NUL`, one copy for the tree.
pub const NUL: ::core::ffi::c_int = 0;

/// The path separator as a string, for the code that appends one.
pub const PATHSEPSTR: &::core::ffi::CStr = c"/";

/// A horizontal tab, as the `int` a widened byte compares against.
pub const TAB: ::core::ffi::c_int = 9;

/// A line feed, as the `int` a widened byte compares against.
pub const NL: ::core::ffi::c_int = 10;

/// A carriage return, as the `int` a widened byte compares against.
pub const CAR: ::core::ffi::c_int = 13;

/// The escape byte, as the `int` a widened byte compares against.
pub const ESC: ::core::ffi::c_int = 27;

/// A backspace, as the `int` a widened byte compares against.
pub const BS: ::core::ffi::c_int = 8;

/// The delete byte, as the `int` a widened byte compares against.
pub const DEL: ::core::ffi::c_int = 0x7f;
