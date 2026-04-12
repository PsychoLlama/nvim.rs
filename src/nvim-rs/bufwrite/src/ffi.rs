//! FFI type aliases and extern declarations for bufwrite.
//!
//! Opaque handle types for C structs that Rust accesses only via accessor functions.

use std::ffi::{c_char, c_int, c_void};

/// Opaque handle to a C `buf_T` struct.
pub type BufHandle = *mut std::ffi::c_void;

/// Opaque handle to a C `exarg_T` struct.
pub type ExargHandle = *mut std::ffi::c_void;

/// Opaque handle to a C `FileInfo` struct.
pub type FileInfoHandle = *mut std::ffi::c_void;

/// Opaque handle to a C `vim_acl_T`.
pub type AclHandle = *mut std::ffi::c_void;

// Return value constants matching C definitions
pub const OK: c_int = 1;
pub const FAIL: c_int = 0;
pub const NOTDONE: c_int = 2;

// iconv_t is a pointer type on all supported platforms
type IconvHandle = *mut c_void;
const ICONV_INVALID: isize = -1;

/// Rust mirror of the C `struct bw_info`.
///
/// The layout matches the C struct exactly (verified by `_Static_assert(sizeof(struct bw_info) == 104)`
/// in bufwrite.c). Fields are accessed directly in Rust code instead of through individual
/// C accessor functions.
///
/// # Safety
///
/// This struct must be initialized via `BwInfo::new()` before use. The `bw_iconv_fd`
/// field is a C `iconv_t` value, which is a pointer on all supported platforms.
#[repr(C)]
pub struct BwInfo {
    /// File descriptor for output
    pub bw_fd: c_int,
    /// Padding to align bw_buf pointer
    _pad0: [u8; 4],
    /// Buffer with data to be written
    pub bw_buf: *mut c_char,
    /// Length of data in bw_buf
    pub bw_len: c_int,
    /// FIO_ encoding flags
    pub bw_flags: c_int,
    /// Unconverted bytes carried over between calls (max CONV_RESTLEN)
    pub bw_rest: [u8; crate::CONV_RESTLEN],
    /// Padding to align bw_restlen after bw_rest[30]
    _pad1: [u8; 2],
    /// Number of bytes in bw_rest
    pub bw_restlen: c_int,
    /// True on first write call (for iconv init flush)
    pub bw_first: c_int,
    /// Buffer for encoding-converted output
    pub bw_conv_buf: *mut c_char,
    /// Allocated size of bw_conv_buf
    pub bw_conv_buflen: usize,
    /// Non-zero if a conversion error occurred
    pub bw_conv_error: c_int,
    /// Line number where first conversion error occurred (or 0)
    pub bw_conv_error_lnum: i32,
    /// Line number at the start of this write batch
    pub bw_start_lnum: i32,
    /// Padding to align bw_iconv_fd pointer
    _pad2: [u8; 4],
    /// iconv descriptor, or `(iconv_t)-1` if not in use
    pub bw_iconv_fd: IconvHandle,
}

const _: () = assert!(
    std::mem::size_of::<BwInfo>() == 104,
    "BwInfo size mismatch: must be 104 bytes to match C struct bw_info"
);

impl BwInfo {
    /// Create a zero-initialized `BwInfo` with iconv disabled.
    ///
    /// Mirrors C `nvim_bw_info_init`: memset to 0, then set `bw_iconv_fd = (iconv_t)-1`.
    #[must_use]
    pub fn new() -> Self {
        let mut s = Self {
            bw_fd: 0,
            _pad0: [0; 4],
            bw_buf: std::ptr::null_mut(),
            bw_len: 0,
            bw_flags: 0,
            bw_rest: [0; crate::CONV_RESTLEN],
            _pad1: [0; 2],
            bw_restlen: 0,
            bw_first: 0,
            bw_conv_buf: std::ptr::null_mut(),
            bw_conv_buflen: 0,
            bw_conv_error: 0,
            bw_conv_error_lnum: 0,
            bw_start_lnum: 0,
            _pad2: [0; 4],
            bw_iconv_fd: std::ptr::null_mut(),
        };
        // iconv_t(-1) = all-ones pointer
        s.bw_iconv_fd = ICONV_INVALID as *mut c_void;
        s
    }

    /// Check if iconv is active (bw_iconv_fd != (iconv_t)-1).
    #[must_use]
    pub fn has_iconv(&self) -> bool {
        self.bw_iconv_fd as isize != ICONV_INVALID
    }
}

impl Default for BwInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// `BwInfo` handle type -- direct pointer to the struct.
pub type BwInfoHandle = *mut BwInfo;
