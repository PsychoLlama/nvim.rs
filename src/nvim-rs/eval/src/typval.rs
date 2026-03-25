//! `#[repr(C)]` struct mirrors for `typval_T`, `partial_T`, `dict_T`, `list_T`,
//! and `CallbackReader`.
//!
//! Layouts are verified by `_Static_assert` blocks in `eval_shim.c`.
//!
//! Each struct only defines the fields that Rust actually accesses; trailing
//! fields that are never touched are elided.

#![allow(dead_code)]
#![allow(clippy::struct_field_names)]
#![allow(clippy::pub_underscore_fields)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::cast_ptr_alignment)]
#![allow(clippy::must_use_candidate)]
#![allow(non_snake_case)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// typval_T (16 bytes, verified by _Static_assert)
// =============================================================================

/// `typval_T.vval` union – 8 bytes, at offset 8.
///
/// Only the variants used by Rust are named; all share the same 8 bytes.
#[repr(C)]
pub union TypvalVval {
    /// `v_number` (VAR_NUMBER): i64
    pub v_number: i64,
    /// `v_float` (VAR_FLOAT): f64
    pub v_float: f64,
    /// `v_string` (VAR_STRING / VAR_FUNC): *mut c_char
    pub v_string: *mut c_char,
    /// `v_list` (VAR_LIST): *mut c_void (list_T*)
    pub v_list: *mut c_void,
    /// `v_dict` (VAR_DICT): *mut c_void (dict_T*)
    pub v_dict: *mut c_void,
    /// `v_partial` (VAR_PARTIAL): *mut c_void (partial_T*)
    pub v_partial: *mut c_void,
    /// `v_blob` (VAR_BLOB): *mut c_void (blob_T*)
    pub v_blob: *mut c_void,
}

/// Rust mirror of C `typval_T` (16 bytes).
///
/// Layout:
/// ```text
/// offset 0:  v_type  (i32, VarType enum)
/// offset 4:  v_lock  (i32, VarLockStatus enum)
/// offset 8:  vval    (8-byte union)
/// ```
#[repr(C)]
pub struct TypvalT {
    /// Variable type (VarType enum, stored as i32).
    pub v_type: c_int,
    /// Variable lock status (VarLockStatus enum, stored as i32).
    pub v_lock: c_int,
    /// Actual value union.
    pub vval: TypvalVval,
}

// =============================================================================
// partial_T (48 bytes, verified by _Static_assert)
// =============================================================================

/// Rust mirror of C `partial_T` (48 bytes).
///
/// Layout:
/// ```text
/// offset 0:  pt_refcount (i32)
/// offset 4:  pt_copyID   (i32)
/// offset 8:  pt_name     (*mut c_char)
/// offset 16: pt_func     (*mut c_void, ufunc_T*)
/// offset 24: pt_auto     (bool, 1 byte)
/// [3 bytes padding]
/// offset 28: pt_argc     (i32)
/// offset 32: pt_argv     (*mut c_void, typval_T*)
/// offset 40: pt_dict     (*mut c_void, dict_T*)
/// ```
#[repr(C)]
pub struct PartialT {
    /// Reference count.
    pub pt_refcount: c_int,
    /// Copy ID (for GC).
    pub pt_copyID: c_int,
    /// Function name (NULL if `pt_func` is set).
    pub pt_name: *mut c_char,
    /// Function pointer (NULL if `pt_name` is set).
    pub pt_func: *mut c_void, // ufunc_T*
    /// When true, partial was auto-created from `dict.member`.
    pub pt_auto: bool,
    pub _pad: [u8; 3],
    /// Number of arguments.
    pub pt_argc: c_int,
    /// Allocated argument array (`typval_T *`).
    pub pt_argv: *mut TypvalT,
    /// Dict for "self".
    pub pt_dict: *mut c_void, // dict_T*
}

// =============================================================================
// Partial dict_T stub (fields up to dv_copyID, used for GC)
// =============================================================================

/// Partial Rust mirror of C `dict_T`.
///
/// Only the first four fields are defined — sufficient for GC copy-ID access.
/// `dv_hashtab` is at offset 16 and is accessed via raw pointer arithmetic.
///
/// Layout (first 16 bytes):
/// ```text
/// offset 0:  dv_lock     (i32, VarLockStatus)
/// offset 4:  dv_scope    (i32, ScopeType)
/// offset 8:  dv_refcount (i32)
/// offset 12: dv_copyID   (i32)
/// ```
#[repr(C)]
pub struct DictTHead {
    pub dv_lock: c_int,
    pub dv_scope: c_int,
    pub dv_refcount: c_int,
    pub dv_copyID: c_int,
    // dv_hashtab starts at offset 16 — accessed via raw pointer arithmetic
}

/// Return a raw pointer to the `dv_hashtab` field of a `dict_T`.
///
/// `dv_hashtab` is at offset 16 from the start of `dict_T`.
/// It is typed as `*mut c_void` since the Rust code treats hashtab_T opaquely.
///
/// # Safety
///
/// `dd` must be a valid non-null pointer to a `dict_T`.
#[inline]
pub unsafe fn dict_get_ht(dd: *mut c_void) -> *mut c_void {
    // SAFETY: dv_hashtab is at offset 16 from dict_T start.
    unsafe { (dd as *mut u8).add(16) as *mut c_void }
}

// =============================================================================
// Partial list_T stub (field lv_copyID at offset 68)
// =============================================================================

/// Read `lv_copyID` from a `list_T` pointer.
///
/// `lv_copyID` is at offset 68 from the start of `list_T`.
///
/// # Safety
///
/// `ll` must be a valid non-null pointer to a `list_T`.
#[inline]
pub unsafe fn list_get_copyid(ll: *const c_void) -> c_int {
    unsafe { *((ll as *const u8).add(68) as *const c_int) }
}

/// Set `lv_copyID` in a `list_T`.
///
/// # Safety
///
/// `ll` must be a valid non-null mutable pointer to a `list_T`.
#[inline]
pub unsafe fn list_set_copyid(ll: *mut c_void, copyid: c_int) {
    unsafe {
        *((ll as *mut u8).add(68) as *mut c_int) = copyid;
    }
}

// =============================================================================
// CallbackReader (64 bytes)
// =============================================================================

/// GArray stub (24 bytes, matching C garray_T layout).
#[repr(C)]
pub struct GArrayT {
    pub ga_len: c_int,
    pub ga_maxlen: c_int,
    pub ga_itemsize: c_int,
    pub ga_growsize: c_int,
    pub ga_data: *mut c_void,
}

/// Callback union data (8 bytes, at offset 0 of Callback).
#[repr(C)]
pub union CallbackData {
    pub funcref: *mut c_char,
    pub partial: *mut c_void,
    pub luaref: c_int,
}

/// Rust mirror of C `Callback` struct (16 bytes).
///
/// Layout:
/// ```text
/// offset 0:  data    (8 bytes, union: funcref / partial / luaref)
/// offset 8:  cb_type (i32)
/// [4 bytes padding]
/// ```
#[repr(C)]
pub struct CallbackT {
    pub data: CallbackData,
    pub cb_type: c_int,
    pub _pad: [u8; 4],
}

/// Rust mirror of C `CallbackReader` (64 bytes).
///
/// Layout:
/// ```text
/// offset 0:  cb       (Callback, 16 bytes)
/// offset 16: self_    (dict_T*, 8 bytes)
/// offset 24: buffer   (garray_T, 24 bytes)
/// offset 48: eof      (bool, 1 byte)
/// offset 49: buffered (bool, 1 byte)
/// offset 50: fwd_err  (bool, 1 byte)
/// [5 bytes padding]
/// offset 56: type_    (*const c_char, 8 bytes)
/// ```
#[repr(C)]
pub struct CallbackReaderT {
    pub cb: CallbackT,
    pub self_: *mut c_void, // dict_T*
    pub buffer: GArrayT,
    pub eof: bool,
    pub buffered: bool,
    pub fwd_err: bool,
    pub _pad2: [u8; 5],
    pub type_: *const c_char,
}
