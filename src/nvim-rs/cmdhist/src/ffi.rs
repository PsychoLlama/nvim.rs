//! FFI declarations for C accessor functions
//!
//! All opaque handle types and extern "C" declarations for accessing
//! C-side data structures used by the cmdhist module.

#![allow(clippy::must_use_candidate)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::missing_const_for_fn)]
#![allow(clippy::not_unsafe_ptr_arg_deref)]
#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};

use crate::state::HistoryEntry;

// =============================================================================
// Opaque Handle Types
// =============================================================================

/// Pointer to histentry_T (now a repr(C) Rust type)
pub type HistEntryPtr = *mut HistoryEntry;

/// Opaque pointer to typval_T
pub type TypvalPtr = *mut c_void;

/// Opaque pointer to exarg_T
pub type ExargPtr = *mut c_void;

/// Opaque pointer to expand_T
pub type ExpandPtr = *mut c_void;

/// Opaque handle for EvalFuncData union (8-byte union passed by value)
pub type EvalFuncData = *mut c_void;

// =============================================================================
// Phase 1: Core Accessors (state functions now live in state.rs as Rust statics)
// =============================================================================

// NOTE: nvim_get_hislen, get_histentry, set_histentry, get_hisidx, get_hisnum
// are now #[no_mangle] Rust functions in state.rs -- no extern "C" needed.

/// Access `hislen` from within Rust.
pub unsafe fn nvim_get_hislen() -> c_int {
    crate::state::hislen
}

/// Access `get_histentry` from within Rust.
pub unsafe fn get_histentry(hist_type: c_int) -> HistEntryPtr {
    crate::state::get_histentry(hist_type)
}

/// Access `set_histentry` from within Rust.
pub unsafe fn set_histentry(hist_type: c_int, entry: HistEntryPtr) {
    crate::state::set_histentry(hist_type, entry);
}

/// Access `get_hisidx` from within Rust.
pub unsafe fn get_hisidx(hist_type: c_int) -> *mut c_int {
    crate::state::get_hisidx(hist_type)
}

/// Access `get_hisnum` from within Rust.
pub unsafe fn get_hisnum(hist_type: c_int) -> *mut c_int {
    crate::state::get_hisnum(hist_type)
}

// =============================================================================
// Phase 2: HistoryEntry field accessors -- now direct struct field access
// =============================================================================

/// Get `hisnum` field of a history entry.
pub unsafe fn nvim_cmdhist_he_get_hisnum(he: HistEntryPtr) -> c_int {
    (*he).hisnum
}

/// Set `hisnum` field of a history entry.
pub unsafe fn nvim_cmdhist_he_set_hisnum(he: HistEntryPtr, val: c_int) {
    (*he).hisnum = val;
}

/// Get `hisstr` field of a history entry.
pub unsafe fn nvim_cmdhist_he_get_hisstr(he: HistEntryPtr) -> *mut c_char {
    (*he).hisstr
}

/// Set `hisstr` field of a history entry.
pub unsafe fn nvim_cmdhist_he_set_hisstr(he: HistEntryPtr, val: *mut c_char) {
    (*he).hisstr = val;
}

/// Get `hisstrlen` field of a history entry.
pub unsafe fn nvim_cmdhist_he_get_hisstrlen(he: HistEntryPtr) -> usize {
    (*he).hisstrlen
}

/// Set `hisstrlen` field of a history entry.
pub unsafe fn nvim_cmdhist_he_set_hisstrlen(he: HistEntryPtr, val: usize) {
    (*he).hisstrlen = val;
}

/// Get `timestamp` field of a history entry.
pub unsafe fn nvim_cmdhist_he_get_timestamp(he: HistEntryPtr) -> u64 {
    (*he).timestamp
}

/// Set `timestamp` field of a history entry.
pub unsafe fn nvim_cmdhist_he_set_timestamp(he: HistEntryPtr, val: u64) {
    (*he).timestamp = val;
}

/// Get `additional_data` field of a history entry.
pub unsafe fn nvim_cmdhist_he_get_additional_data(he: HistEntryPtr) -> *mut c_void {
    (*he).additional_data
}

/// Set `additional_data` field of a history entry.
pub unsafe fn nvim_cmdhist_he_set_additional_data(he: HistEntryPtr, val: *mut c_void) {
    (*he).additional_data = val;
}

/// Zero-fill a history entry (equivalent to CLEAR_POINTER).
pub unsafe fn nvim_cmdhist_he_clear(he: HistEntryPtr) {
    std::ptr::write_bytes(he, 0, 1);
}

/// Copy a history entry (equivalent to `*dst = *src`).
pub unsafe fn nvim_cmdhist_he_copy(dst: HistEntryPtr, src: HistEntryPtr) {
    std::ptr::copy_nonoverlapping(src, dst, 1);
}

/// Get pointer to the `idx`-th history entry in an array.
pub unsafe fn nvim_cmdhist_he_at(base: HistEntryPtr, idx: c_int) -> HistEntryPtr {
    base.add(idx as usize)
}

// =============================================================================
// Memory / String / Global -- Phase 3: direct C externs replace wrappers
// =============================================================================

extern "C" {
    fn xfree(ptr: *mut c_void);
    fn xmalloc(size: usize) -> *mut c_void;
    fn xstrnsave(s: *const c_char, len: usize) -> *mut c_char;
    fn strncasecmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    pub fn vim_strchr(s: *const c_char, c: c_int) -> *mut c_char;
    pub fn get_cmdline_firstc() -> c_int;
}

/// Free memory (wraps C `xfree`).
pub unsafe fn nvim_cmdhist_xfree(ptr: *mut c_void) {
    xfree(ptr);
}

/// Allocate memory (wraps C `xmalloc`).
pub unsafe fn nvim_cmdhist_xmalloc(size: usize) -> *mut c_void {
    xmalloc(size)
}

/// Save a string slice (wraps C `xstrnsave`).
pub unsafe fn nvim_cmdhist_xstrnsave(s: *const c_char, len: usize) -> *mut c_char {
    xstrnsave(s, len)
}

/// Case-insensitive string compare (wraps `strncasecmp`, equivalent to C STRNICMP).
pub unsafe fn nvim_cmdhist_strnicmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int {
    strncasecmp(s1, s2, n)
}

/// Find character in string (wraps C `vim_strchr`).
pub unsafe fn nvim_cmdhist_vim_strchr(s: *const c_char, c: c_int) -> *mut c_char {
    vim_strchr(s, c)
}

/// Get first character of current command line (wraps C `get_cmdline_firstc`).
pub unsafe fn nvim_cmdhist_get_cmdline_firstc() -> c_int {
    get_cmdline_firstc()
}

/// Zero-fill `count` history entries starting at `dst`.
pub unsafe fn nvim_cmdhist_memset_entries(dst: HistEntryPtr, count: c_int) {
    std::ptr::write_bytes(dst, 0, count as usize);
}

/// Copy `count` history entries from `src` to `dst`.
pub unsafe fn nvim_cmdhist_memcpy_entries(dst: HistEntryPtr, src: HistEntryPtr, count: c_int) {
    std::ptr::copy_nonoverlapping(src, dst, count as usize);
}

/// Return the size of a single HistoryEntry.
pub fn nvim_cmdhist_sizeof_histentry() -> usize {
    std::mem::size_of::<HistoryEntry>()
}

// =============================================================================
// Phase 2/3: History Modification Accessors -- global accessors
// =============================================================================

extern "C" {
    static p_hi: i64;
    static maptick: c_int;
    pub fn os_time() -> u64;
    pub fn nvim_cmdhist_get_cmdmod_cmod_flags() -> c_int;
    fn strcmp(s1: *const c_char, s2: *const c_char) -> c_int;
}

/// Get `'history'` option value.
pub unsafe fn nvim_cmdhist_get_p_hi() -> i64 {
    p_hi
}

/// Get current maptick value.
pub unsafe fn nvim_cmdhist_get_maptick() -> c_int {
    maptick
}

/// Get current OS time.
pub unsafe fn nvim_cmdhist_os_time() -> u64 {
    os_time()
}

/// Compare two C strings (wraps `strcmp`).
pub unsafe fn nvim_cmdhist_strcmp(s1: *const c_char, s2: *const c_char) -> c_int {
    strcmp(s1, s2)
}

/// Set history length - calls our own exported fn.
pub unsafe fn nvim_cmdhist_set_hislen(val: c_int) {
    crate::state::nvim_cmdhist_set_hislen(val);
}

// =============================================================================
// Phase 3: Deletion Accessors (regexp wrappers)
// =============================================================================

extern "C" {
    pub fn nvim_cmdhist_regcomp(str: *const c_char, flags: c_int) -> *mut c_void;
    pub fn nvim_cmdhist_regexec(rm: *mut c_void, str: *const c_char) -> c_int;
    pub fn nvim_cmdhist_regfree(rm: *mut c_void);
}

// =============================================================================
// Phase 4: VimL Function Accessors (typval wrappers)
// =============================================================================

extern "C" {
    pub fn nvim_cmdhist_tv_get_string_chk(tv: TypvalPtr) -> *const c_char;
    pub fn nvim_cmdhist_tv_get_string_buf(tv: TypvalPtr, buf: *mut c_char) -> *const c_char;
    pub fn nvim_cmdhist_tv_get_number(tv: TypvalPtr) -> i64;
    pub fn nvim_cmdhist_tv_get_number_chk(tv: TypvalPtr, error: *mut c_void) -> i64;
    pub fn nvim_cmdhist_tv_get_type(tv: TypvalPtr) -> c_int;
    pub fn nvim_cmdhist_tv_idx(tv: TypvalPtr, idx: c_int) -> TypvalPtr;
    pub fn nvim_cmdhist_rettv_set_number(rettv: TypvalPtr, val: i64);
    pub fn nvim_cmdhist_rettv_set_string(rettv: TypvalPtr, s: *mut c_char);
    pub fn nvim_cmdhist_rettv_set_type(rettv: TypvalPtr, typ: c_int);
    pub fn rs_check_secure() -> c_int;
    #[link_name = "strlen"]
    pub fn nvim_cmdhist_strlen(s: *const c_char) -> usize;
}
