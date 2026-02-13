//! FFI declarations for C accessor functions
//!
//! All opaque handle types and extern "C" declarations for accessing
//! C-side data structures used by the cmdhist module.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::{c_char, c_int, c_void};

// =============================================================================
// Opaque Handle Types
// =============================================================================

/// Opaque pointer to histentry_T
pub type HistEntryPtr = *mut c_void;

/// Opaque pointer to typval_T
pub type TypvalPtr = *mut c_void;

/// Opaque pointer to exarg_T
pub type ExargPtr = *mut c_void;

/// Opaque pointer to expand_T
pub type ExpandPtr = *mut c_void;

/// Opaque handle for EvalFuncData union (8-byte union passed by value)
pub type EvalFuncData = *mut c_void;

// =============================================================================
// Phase 1: Core Accessors
// =============================================================================

extern "C" {
    // -- History state --
    pub fn nvim_get_hislen() -> c_int;
    pub fn get_histentry(hist_type: c_int) -> HistEntryPtr;
    pub fn get_hisidx(hist_type: c_int) -> *mut c_int;
    pub fn get_hisnum(hist_type: c_int) -> *mut c_int;

    // -- histentry_T field accessors --
    pub fn nvim_cmdhist_he_get_hisnum(he: HistEntryPtr) -> c_int;
    pub fn nvim_cmdhist_he_set_hisnum(he: HistEntryPtr, val: c_int);
    pub fn nvim_cmdhist_he_get_hisstr(he: HistEntryPtr) -> *mut c_char;
    pub fn nvim_cmdhist_he_set_hisstr(he: HistEntryPtr, val: *mut c_char);
    pub fn nvim_cmdhist_he_get_hisstrlen(he: HistEntryPtr) -> usize;
    pub fn nvim_cmdhist_he_set_hisstrlen(he: HistEntryPtr, val: usize);
    pub fn nvim_cmdhist_he_get_timestamp(he: HistEntryPtr) -> u64;
    pub fn nvim_cmdhist_he_set_timestamp(he: HistEntryPtr, val: u64);
    pub fn nvim_cmdhist_he_get_additional_data(he: HistEntryPtr) -> *mut c_void;
    pub fn nvim_cmdhist_he_set_additional_data(he: HistEntryPtr, val: *mut c_void);
    pub fn nvim_cmdhist_he_clear(he: HistEntryPtr);
    pub fn nvim_cmdhist_he_copy(dst: HistEntryPtr, src: HistEntryPtr);
    pub fn nvim_cmdhist_he_at(base: HistEntryPtr, idx: c_int) -> HistEntryPtr;

    // -- Memory --
    pub fn nvim_cmdhist_xfree(ptr: *mut c_void);
    pub fn nvim_cmdhist_xmalloc(size: usize) -> *mut c_void;
    pub fn nvim_cmdhist_xstrnsave(s: *const c_char, len: usize) -> *mut c_char;

    // -- String --
    pub fn nvim_cmdhist_strnicmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    pub fn nvim_cmdhist_vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;

    // -- Global --
    pub fn nvim_cmdhist_get_cmdline_firstc() -> c_int;

    // -- Array ops --
    pub fn nvim_cmdhist_memset_entries(dst: HistEntryPtr, count: c_int);
    pub fn nvim_cmdhist_memcpy_entries(dst: HistEntryPtr, src: HistEntryPtr, count: c_int);

    // -- Sizeof --
    pub fn nvim_cmdhist_sizeof_histentry() -> usize;
}

// =============================================================================
// Phase 2: History Modification Accessors
// =============================================================================

extern "C" {
    pub fn nvim_cmdhist_get_p_hi() -> i64;
    pub fn nvim_cmdhist_get_maptick() -> c_int;
    pub fn nvim_cmdhist_os_time() -> u64;
    pub fn nvim_cmdhist_get_cmdmod_cmod_flags() -> c_int;
    pub fn nvim_cmdhist_set_hislen(val: c_int);
    pub fn set_histentry(hist_type: c_int, entry: HistEntryPtr);
    pub fn nvim_cmdhist_strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
}
