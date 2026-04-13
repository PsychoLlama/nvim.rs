//! Compile-time constants inlined from C headers.
//!
//! These replace the constant-returning C accessor functions that were in
//! runtime_ffi.c (e.g. `nvim_rt_maxpathl`, `nvim_rt_iosize`, etc.).
//!
//! Values are validated by `_Static_assert` in runtime_ffi.c.

use std::ffi::{c_char, c_int};

/// Maximum path length (MAXPATHL). On Linux PATH_MAX=4096 > DEFAULT_MAXPATHL,
/// so MAXPATHL == PATH_MAX == 4096.
pub const MAXPATHL: usize = 4096;

/// I/O buffer size (IOSIZE = 1024 + 1).
pub const IOSIZE: usize = 1025;

/// NUL character.
pub const NUL: c_char = 0;

/// PROF_YES: profiling is busy.
pub const PROF_YES: c_int = 1;

/// DOSO_VIMRC: do_source() flag for loading vimrc.
pub const DOSO_VIMRC: c_int = 1;

/// SID_STR: script context for sourcing a string with no script item.
pub const SID_STR: c_int = -10;

/// DOCMD_VERBOSE: include command in error message.
pub const DOCMD_VERBOSE: c_int = 0x01;

/// DOCMD_NOWAIT: don't call wait_return() and friends.
pub const DOCMD_NOWAIT: c_int = 0x02;

/// DOCMD_REPEAT: repeat execution until getline() returns NULL.
pub const DOCMD_REPEAT: c_int = 0x04;

/// CSTP_FINISH: ":finish" is pending.
pub const CSTP_FINISH: c_int = 32;

/// EW_DIR: expand directory names.
pub const EW_DIR: c_int = 0x01;

/// EW_FILE: expand file names.
pub const EW_FILE: c_int = 0x02;

/// EW_NOBREAK: do not invoke breakcheck.
pub const EW_NOBREAK: c_int = 0x40000;

/// CPO_CONCAT: 'C' flag in 'cpoptions' - don't concatenate sourced lines.
pub const CPO_CONCAT: c_int = b'C' as c_int;

/// CONV_NONE: no encoding conversion.
pub const CONV_NONE: c_int = 0;
