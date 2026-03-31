//! findfunc subsystem migrated from ex_docmd.c (Phase 2).
//!
//! Implements:
//! - `expand_findfunc` -- expand file names via 'findfunc' (called from cmdexpand crate)
//! - `nvim_docmd_findfunc_find_file` -- find the nth file via 'findfunc'
//! - `nvim_docmd_free_findfunc_option_impl` -- free global 'findfunc' callback
//! - `nvim_docmd_set_ref_in_findfunc_impl` -- GC-mark global 'findfunc' callback

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

// Opaque list handle
type ListHandle = *mut c_void;
// Opaque callback handle
type CallbackHandle = *mut c_void;

const OK: c_int = 1;
const FAIL: c_int = 0;

extern "C" {
    // --- call_findfunc C wrapper ---
    // Calls call_findfunc() in C (which manages typval_T, Callback, sctx_T internals).
    // Returns an owned list_T* (caller must free with tv_list_free).
    fn nvim_docmd_call_findfunc(pat: *mut c_char, cmdcomplete: bool) -> ListHandle;

    // --- list accessors ---
    fn nvim_docmd_tv_list_len(l: ListHandle) -> c_int;
    fn tv_list_free(l: ListHandle);
    // tv_list_find_str: Rust export from typval crate - get item as string
    fn tv_list_find_str(l: ListHandle, n: c_int) -> *const c_char;

    // --- memory ---
    fn xmalloc(size: usize) -> *mut c_void;
    fn xstrdup(s: *const c_char) -> *mut c_char;

    // --- error messages ---
    fn semsg(fmt: *const c_char, ...);

    // --- ffu_cb accessor (for free/set_ref) ---
    fn nvim_docmd_get_ffu_cb_ptr() -> CallbackHandle;

    // --- callback operations ---
    fn callback_free(cb: CallbackHandle);
    fn rs_set_ref_in_callback(
        cb: CallbackHandle,
        copy_id: c_int,
        ht_stack: *mut c_void,
        list_stack: *mut c_void,
    ) -> bool;

    // --- error message strings ---
    fn nvim_docmd_e_cant_find_file_str_in_path() -> *const c_char;
    fn nvim_docmd_e_no_more_file_str_found_in_path() -> *const c_char;
}

// =============================================================================
// expand_findfunc
// =============================================================================

/// Find file names matching "pat" using 'findfunc' and return them in "files".
/// Used for expanding the :find, :sfind and :tabfind command argument.
/// Returns OK on success, FAIL otherwise.
///
/// # Safety
/// All pointers must be valid.
#[no_mangle]
pub unsafe extern "C" fn expand_findfunc(
    pat: *mut c_char,
    files: *mut *mut *mut c_char,
    num_matches: *mut c_int,
) -> c_int {
    *num_matches = 0;
    *files = ptr::null_mut();

    let l = nvim_docmd_call_findfunc(pat, true);
    if l.is_null() {
        return FAIL;
    }

    let len = nvim_docmd_tv_list_len(l);
    if len == 0 {
        tv_list_free(l);
        return FAIL;
    }

    // Allocate array of string pointers
    let arr = xmalloc(std::mem::size_of::<*mut c_char>() * len as usize) as *mut *mut c_char;

    let mut idx = 0i32;
    for i in 0..len {
        let s = tv_list_find_str(l, i);
        if !s.is_null() {
            *arr.add(idx as usize) = xstrdup(s);
            idx += 1;
        }
    }

    *num_matches = idx;
    *files = arr;

    tv_list_free(l);

    OK
}

// =============================================================================
// nvim_docmd_findfunc_find_file
// =============================================================================

/// Use 'findfunc' to find the nth file matching 'findarg'.
/// Replaces the C static findfunc_find_file() and the accessor wrapper.
///
/// # Safety
/// findarg must be a valid pointer with at least findarg_len+1 bytes writable.
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_findfunc_find_file(
    findarg: *mut c_char,
    findarg_len: usize,
    count: c_int,
) -> *mut c_char {
    let mut ret_fname: *mut c_char = ptr::null_mut();

    // Temporarily NUL-terminate at findarg_len
    let cc = *findarg.add(findarg_len);
    *findarg.add(findarg_len) = 0;

    let fname_list = nvim_docmd_call_findfunc(findarg, false);
    let fname_count = nvim_docmd_tv_list_len(fname_list);

    if fname_count == 0 {
        semsg(nvim_docmd_e_cant_find_file_str_in_path(), findarg);
    } else if count > fname_count {
        semsg(nvim_docmd_e_no_more_file_str_found_in_path(), findarg);
    } else {
        // count is 1-based
        let s = tv_list_find_str(fname_list, count - 1);
        if !s.is_null() {
            ret_fname = xstrdup(s);
        }
    }

    if !fname_list.is_null() {
        tv_list_free(fname_list);
    }

    // Restore original character
    *findarg.add(findarg_len) = cc;

    ret_fname
}

// =============================================================================
// nvim_docmd_free_findfunc_option_impl
// =============================================================================

/// Free the global 'findfunc' callback.
///
/// # Safety
/// Accesses global ffu_cb via C accessor.
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_free_findfunc_option_impl() {
    let cb = nvim_docmd_get_ffu_cb_ptr();
    callback_free(cb);
}

// =============================================================================
// nvim_docmd_set_ref_in_findfunc_impl
// =============================================================================

/// Mark the global 'findfunc' callback with "copyID" so it is not GC'd.
///
/// # Safety
/// Accesses global ffu_cb via C accessor.
#[no_mangle]
pub unsafe extern "C" fn nvim_docmd_set_ref_in_findfunc_impl(copy_id: c_int) -> bool {
    let cb = nvim_docmd_get_ffu_cb_ptr();
    rs_set_ref_in_callback(cb, copy_id, ptr::null_mut(), ptr::null_mut())
}
