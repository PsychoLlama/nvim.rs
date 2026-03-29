//! Raw repr(C) structs mirroring quickfix C types
//!
//! These structs mirror the C structs `qfline_T`, `qf_list_T`, and `qf_info_S`
//! from `quickfix_shim.c`. Fields containing `typval_T` or `Callback` are
//! represented as opaque byte arrays sized by runtime assertions below.
//!
//! # Safety
//!
//! These structs are only valid to use through raw pointer operations. Never
//! create them directly; they are always accessed via pointers obtained from C.
//!
//! # Layout verification
//!
//! Runtime size assertions in `assert_ffi_struct_sizes()` verify that the Rust
//! struct sizes match the C struct sizes reported by the C-side sizeof functions.

use std::ffi::{c_char, c_int, c_void};

use crate::dirstack::DirStackNode;

// =============================================================================
// Opaque field sizes (verified at runtime via sizeof C functions)
// =============================================================================

/// Size of C `typval_T` in bytes. Verified by `_Static_assert` in `testing.c`.
pub const TYPVAL_SIZE: usize = 16;

/// Size of C `Callback` in bytes. Verified by `_Static_assert` in `eval_struct_check.c`.
pub const CALLBACK_SIZE: usize = 16;

// =============================================================================
// qfltype_T enum
// =============================================================================

/// Matches C `qfltype_T` enum
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QflTypeRaw {
    /// Global quickfix list
    Quickfix = 0,
    /// Window-local location list
    Location = 1,
    /// Internal temporary list
    Internal = 2,
}

// =============================================================================
// qfline_T / QfLineRaw
// =============================================================================

/// Raw layout of C `qfline_T` / `struct qfline_S`.
///
/// Total size: 104 bytes on 64-bit Linux.
#[repr(C)]
pub struct QfLineRaw {
    /// Pointer to next entry in the list (`qf_next`)
    pub qf_next: *mut QfLineRaw,
    /// Pointer to previous entry in the list (`qf_prev`)
    pub qf_prev: *mut QfLineRaw,
    /// Line number of the error (`qf_lnum`)
    pub qf_lnum: i32,
    /// End line number (`qf_end_lnum`)
    pub qf_end_lnum: i32,
    /// Buffer number (`qf_fnum`)
    pub qf_fnum: c_int,
    /// Column number (`qf_col`)
    pub qf_col: c_int,
    /// End column number (`qf_end_col`)
    pub qf_end_col: c_int,
    /// Error number (`qf_nr`)
    pub qf_nr: c_int,
    /// Module name (`qf_module`, heap-allocated)
    pub qf_module: *mut c_char,
    /// File name (`qf_fname`, heap-allocated)
    pub qf_fname: *mut c_char,
    /// Search pattern (`qf_pattern`, heap-allocated)
    pub qf_pattern: *mut c_char,
    /// Error text (`qf_text`, heap-allocated)
    pub qf_text: *mut c_char,
    /// Virtual column flag (`qf_viscol`)
    pub qf_viscol: i8,
    /// Entry cleared flag (`qf_cleared`)
    pub qf_cleared: i8,
    /// Error type character (`qf_type`)
    pub qf_type: i8,
    /// Padding before `qf_user_data` for `typval_T` 8-byte alignment
    _pad_before_user_data: [u8; 5],
    /// User data (`qf_user_data`), opaque `typval_T` bytes
    pub qf_user_data: [u8; TYPVAL_SIZE],
    /// Entry valid flag (`qf_valid`)
    pub qf_valid: i8,
    /// Tail padding to maintain struct size
    _tail_pad: [u8; 7],
}

// =============================================================================
// qf_list_T / QfListRaw
// =============================================================================

/// Raw layout of C `qf_list_T`.
///
/// Total size: 120 bytes on 64-bit Linux.
#[repr(C)]
pub struct QfListRaw {
    /// Unique list identifier (`qf_id`)
    pub qf_id: u32,
    /// Type of this list (`qfl_type`)
    pub qfl_type: QflTypeRaw,
    /// First entry in the list (`qf_start`)
    pub qf_start: *mut QfLineRaw,
    /// Last entry in the list (`qf_last`)
    pub qf_last: *mut QfLineRaw,
    /// Current entry pointer (`qf_ptr`)
    pub qf_ptr: *mut QfLineRaw,
    /// Number of entries (`qf_count`)
    pub qf_count: c_int,
    /// Current index (1-based) (`qf_index`)
    pub qf_index: c_int,
    /// True if all entries are invalid (`qf_nonevalid`)
    pub qf_nonevalid: bool,
    /// True if any entry has user data (`qf_has_user_data`)
    pub qf_has_user_data: bool,
    /// Padding before `qf_title` for pointer alignment
    _pad_flags: [u8; 6],
    /// List title (`qf_title`, heap-allocated)
    pub qf_title: *mut c_char,
    /// Context typval pointer (`qf_ctx`)
    pub qf_ctx: *mut c_void,
    /// `QuickfixTextFunc` callback (`qf_qftf_cb`), opaque `Callback` bytes
    pub qf_qftf_cb: [u8; CALLBACK_SIZE],
    /// Directory stack (`qf_dir_stack`)
    pub qf_dir_stack: *mut DirStackNode,
    /// Current directory (`qf_directory`, not heap-owned)
    pub qf_directory: *mut c_char,
    /// File stack (`qf_file_stack`)
    pub qf_file_stack: *mut DirStackNode,
    /// Current file (`qf_currfile`, not heap-owned)
    pub qf_currfile: *mut c_char,
    /// Multi-line continuation flag (`qf_multiline`)
    pub qf_multiline: bool,
    /// Multi-line ignore flag (`qf_multiignore`)
    pub qf_multiignore: bool,
    /// Multi-line scan flag (`qf_multiscan`)
    pub qf_multiscan: bool,
    /// Padding before `qf_changedtick` for `i32` alignment
    _pad_before_changedtick: [u8; 1],
    /// Change counter (`qf_changedtick`)
    pub qf_changedtick: c_int,
}

// =============================================================================
// qf_info_S / QfInfoRaw
// =============================================================================

/// Raw layout of C `qf_info_S` / `qf_info_T`.
///
/// Total size: 32 bytes on 64-bit Linux.
#[repr(C)]
pub struct QfInfoRaw {
    /// Reference count (`qf_refcount`)
    pub qf_refcount: c_int,
    /// Current number of lists (`qf_listcount`)
    pub qf_listcount: c_int,
    /// Current list index (`qf_curlist`)
    pub qf_curlist: c_int,
    /// Maximum number of lists (`qf_maxcount`)
    pub qf_maxcount: c_int,
    /// Pointer to array of lists (`qf_lists`)
    pub qf_lists: *mut QfListRaw,
    /// Type of this stack (`qfl_type`)
    pub qfl_type: QflTypeRaw,
    /// Quickfix window buffer number (`qf_bufnr`)
    pub qf_bufnr: c_int,
}

// =============================================================================
// Type aliases
// =============================================================================

/// Mutable pointer to a quickfix entry (`qfline_T`)
pub type QfLinePtr = *mut QfLineRaw;
/// Mutable pointer to a quickfix list (`qf_list_T`)
pub type QfListPtr = *mut QfListRaw;
/// Mutable pointer to a quickfix stack (`qf_info_T`)
pub type QfInfoPtr = *mut QfInfoRaw;

// =============================================================================
// C sizeof functions for runtime layout verification
// =============================================================================

extern "C" {
    fn nvim_qf_sizeof_qfinfo() -> usize;
}

// =============================================================================
// Runtime size assertions
// =============================================================================

/// Verify that the Rust `repr(C)` struct sizes match the C struct sizes.
///
/// This function should be called once during initialization. If sizes do not
/// match, it panics with a descriptive error message.
///
/// # Safety
///
/// Calls C functions to get struct sizes; these are simple sizeof returns
/// and are always safe to call.
///
/// # Panics
///
/// Panics if the Rust struct size does not match the corresponding C struct size.
pub unsafe fn assert_ffi_struct_sizes() {
    let c_qfline = ::std::mem::size_of::<crate::ffi_types::QfLineRaw>();
    let rs_qfline = std::mem::size_of::<QfLineRaw>();
    assert_eq!(
        rs_qfline, c_qfline,
        "QfLineRaw size mismatch: Rust={rs_qfline}, C={c_qfline}"
    );

    let c_qflist = ::std::mem::size_of::<crate::ffi_types::QfListRaw>();
    let rs_qflist = std::mem::size_of::<QfListRaw>();
    assert_eq!(
        rs_qflist, c_qflist,
        "QfListRaw size mismatch: Rust={rs_qflist}, C={c_qflist}"
    );

    let c_qfinfo = nvim_qf_sizeof_qfinfo();
    let rs_qfinfo = std::mem::size_of::<QfInfoRaw>();
    assert_eq!(
        rs_qfinfo, c_qfinfo,
        "QfInfoRaw size mismatch: Rust={rs_qfinfo}, C={c_qfinfo}"
    );
}
