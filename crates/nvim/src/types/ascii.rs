#![forbid(unsafe_code)]

// Canonical definitions from `ascii_defs.h`, hoisted out of the per-module
// copies c2rust emitted. One definition per logical name; every module
// re-exports here.

/// The string terminator, as the `int` a widened byte compares against.
/// `ascii_defs.h`'s `NUL`, one copy for the tree.
pub const NUL: ::core::ffi::c_int = 0;

/// The path separator as a string, for the code that appends one.
pub const PATHSEPSTR: &::core::ffi::CStr = c"/";
