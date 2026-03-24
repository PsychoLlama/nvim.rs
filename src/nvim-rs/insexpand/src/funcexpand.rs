//! VimL completion function expansion (Phase 9 of insexpand migration).
//!
//! This module provides Rust wrappers for:
//! - `expand_by_function`: call user completefunc/omnifunc/thesaurusfunc
//! - `ins_compl_add_tv`: parse a typval_T into a completion match
//! - `ins_compl_add_list`: iterate a VimL list and add completions
//! - `ins_compl_add_dict`: extract "words"/"refresh" from a dict
//! - `set_completion`: orchestrate complete() builtin
//! - `f_complete`, `f_complete_add`, `f_complete_check`, `f_preinserted`
//! - `cpt_compl_refresh`, `remove_old_matches`, `get_callback_if_cpt_func`
//!
//! All functions that touch opaque C types (typval_T, dict_T, list_T,
//! Callback, compl_T) delegate to compound C accessors following the
//! established pattern in the insexpand crate.

#![allow(clippy::not_unsafe_ptr_arg_deref)]

use std::ffi::c_void;
use std::os::raw::{c_char, c_int};

// =============================================================================
// Opaque pointer types
// =============================================================================

/// Opaque pointer to typval_T
type TypvalPtr = *mut c_void;

/// Opaque pointer to list_T
type ListPtr = *mut c_void;

/// Opaque pointer to dict_T
type DictPtr = *mut c_void;

/// Opaque pointer to Callback
type CallbackPtr = *mut c_void;

// =============================================================================
// Phase 1: VimL Completion Function Expansion
// =============================================================================

extern "C" {
    // Compound C accessors (contain the actual logic moved from static C fns)
    fn nvim_expand_by_function_full_impl(type_: c_int, base: *mut c_char, cb: CallbackPtr);
    fn nvim_ins_compl_add_tv_impl(tv: TypvalPtr, dir: c_int, fast: c_int) -> c_int;
    fn nvim_ins_compl_add_list_impl(list: ListPtr);
    fn nvim_ins_compl_add_dict_impl(dict: DictPtr);
}

/// Execute user-defined completion function and collect matches.
///
/// Calls completefunc/omnifunc/thesaurusfunc (findstart=0) and dispatches
/// the list/dict result to `rs_ins_compl_add_list` / `rs_ins_compl_add_dict`.
///
/// # Safety
/// Requires valid global completion state and a live C `Callback *` if non-null.
#[no_mangle]
pub unsafe extern "C" fn rs_expand_by_function(type_: c_int, base: *mut c_char, cb: CallbackPtr) {
    nvim_expand_by_function_full_impl(type_, base, cb);
}

/// Add a completion match from a VimL typval_T value.
///
/// Parses a string or dict typval and calls `ins_compl_add`.
///
/// Returns `OK`, `NOTDONE`, or `FAIL` (matching C return conventions).
///
/// # Safety
/// `tv` must be a valid `typval_T *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_add_tv(tv: TypvalPtr, dir: c_int, fast: c_int) -> c_int {
    nvim_ins_compl_add_tv_impl(tv, dir, fast)
}

/// Iterate a VimL list and add each item as a completion match.
///
/// # Safety
/// `list` must be a valid `list_T *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_add_list(list: ListPtr) {
    nvim_ins_compl_add_list_impl(list);
}

/// Extract `refresh` and `words` from a VimL dict and add completion matches.
///
/// # Safety
/// `dict` must be a valid `dict_T *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_ins_compl_add_dict(dict: DictPtr) {
    nvim_ins_compl_add_dict_impl(dict);
}

// =============================================================================
// Phase 2: VimL Builtin Functions
// =============================================================================

extern "C" {
    // Compound C accessors for all Phase 2 functions
    fn nvim_f_complete_impl(argvars: TypvalPtr, rettv: TypvalPtr);
    fn nvim_f_complete_add_impl(argvars: TypvalPtr, rettv: TypvalPtr);
    fn nvim_f_complete_check_impl(rettv: TypvalPtr);
    fn nvim_f_preinserted_impl(rettv: TypvalPtr);
}

/// VimL `complete_check()` builtin.
///
/// Saves/restores `RedrawingDisabled`, calls `rs_ins_compl_check_keys`,
/// and sets the return value to the interrupted flag.
///
/// # Safety
/// Requires valid global completion state.
#[export_name = "f_complete_check"]
pub unsafe extern "C" fn rs_f_complete_check(
    _argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: *mut c_void,
) {
    nvim_f_complete_check_impl(rettv);
}

/// VimL `preinserted()` builtin.
///
/// Returns 1 if the pre-insert effect is currently active.
///
/// # Safety
/// Requires valid global completion state.
#[export_name = "f_preinserted"]
pub unsafe extern "C" fn rs_f_preinserted(
    _argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: *mut c_void,
) {
    nvim_f_preinserted_impl(rettv);
}

/// VimL `complete()` builtin.
///
/// # Safety
/// `argvars` must be a valid `typval_T[2]` pointer; `rettv` a `typval_T*`.
#[export_name = "f_complete"]
pub unsafe extern "C" fn rs_f_complete(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: *mut c_void) {
    nvim_f_complete_impl(argvars, rettv);
}

/// VimL `complete_add()` builtin.
///
/// # Safety
/// `argvars` must be a valid `typval_T[1]` pointer; `rettv` a `typval_T*`.
#[export_name = "f_complete_add"]
pub unsafe extern "C" fn rs_f_complete_add(
    argvars: TypvalPtr,
    rettv: TypvalPtr,
    _fptr: *mut c_void,
) {
    nvim_f_complete_add_impl(argvars, rettv);
}

// =============================================================================
// Phase 3: set_completion Orchestration
// =============================================================================

extern "C" {
    fn nvim_set_completion_impl(startcol: c_int, list: ListPtr);
}

/// Orchestrate the `complete()` VimL function: clear state, add original
/// text and list matches, start completion, show popup menu.
///
/// # Safety
/// `list` must be a valid `list_T *` pointer.
#[no_mangle]
pub unsafe extern "C" fn rs_set_completion(startcol: c_int, list: ListPtr) {
    nvim_set_completion_impl(startcol, list);
}

// =============================================================================
// Phase 4: Refresh Orchestration
// =============================================================================

// nvim_cpt_compl_refresh_impl: deleted (Phase 2), inlined below as rs_cpt_compl_refresh
extern "C" {
    fn nvim_get_callback_if_cpt_func_impl(p: *const c_char, idx: c_int) -> CallbackPtr;

    // helpers for inlined rs_cpt_compl_refresh
    fn nvim_curbuf_get_b_p_cpt() -> *const c_char;
    #[link_name = "xstrdup"]
    fn xstrdup_funcexpand(s: *const c_char) -> *mut c_char;
    #[link_name = "xfree"]
    fn xfree_funcexpand(p: *mut u8);
    fn copy_option_part(
        pp: *mut *mut c_char,
        buf: *mut c_char,
        maxlen: usize,
        sep_chars: *const c_char,
    ) -> usize;
    fn rs_strip_caret_numbers_in_place(s: *mut c_char);
    fn rs_ins_compl_make_linear();
    fn rs_ins_compl_make_cyclic() -> c_int;
    fn rs_remove_old_matches();
    fn rs_get_userdefined_compl_info(
        col: c_int,
        cb: CallbackPtr,
        startcol_out: *mut c_int,
    ) -> c_int;
    fn rs_compl_source_start_timer(idx: c_int);
    fn rs_get_cpt_func_completion_matches(cb: CallbackPtr);
    fn rs_advance_cpt_sources_index_safe() -> c_int;
    fn rs_may_advance_cpt_index(cpt: *mut c_char) -> c_int;
    fn nvim_get_cursor_col() -> c_int;
    #[link_name = "IObuff"]
    static mut IObuff_funcexpand: [std::ffi::c_char; 1025];
}

const IOSIZE_FUNCEXPAND: usize = 1025;
const FAIL_FUNCEXPAND: c_int = 0;
const OK_FUNCEXPAND: c_int = 1;

/// Refresh completion matches from 'cpt' function sources with `refresh:always`.
///
/// Rust translation of nvim_cpt_compl_refresh_impl (Phase 2).
///
/// # Safety
/// Requires valid global completion state and a live current buffer.
#[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
#[no_mangle]
pub unsafe extern "C" fn rs_cpt_compl_refresh() {
    rs_ins_compl_make_linear();
    // Make a copy of 'cpt' in case the buffer gets wiped out
    let cpt = xstrdup_funcexpand(nvim_curbuf_get_b_p_cpt());
    rs_strip_caret_numbers_in_place(cpt);

    crate::vars::nvim_set_cpt_sources_index(0);
    let mut p = cpt;
    loop {
        // Skip delimiters
        while *p == b',' as std::ffi::c_char || *p == b' ' as std::ffi::c_char {
            p = p.add(1);
        }
        if *p == 0 {
            break;
        }

        let idx = crate::vars::nvim_get_cpt_sources_index();
        if crate::vars::nvim_cpt_sources_get_refresh_always(idx) != 0 {
            let cb = nvim_get_callback_if_cpt_func_impl(p, idx);
            if !cb.is_null() {
                rs_remove_old_matches();
                let mut startcol: c_int = 0;
                let ret =
                    rs_get_userdefined_compl_info(nvim_get_cursor_col(), cb, &raw mut startcol);
                if ret == FAIL_FUNCEXPAND {
                    if startcol == -3 {
                        crate::vars::nvim_cpt_sources_set_refresh_always(idx, 0);
                    } else {
                        startcol = -2;
                    }
                } else if startcol < 0 || startcol > nvim_get_cursor_col() {
                    startcol = nvim_get_cursor_col();
                }
                crate::vars::nvim_cpt_sources_set_startcol(idx, startcol);
                if ret == OK_FUNCEXPAND {
                    rs_compl_source_start_timer(idx);
                    rs_get_cpt_func_completion_matches(cb);
                }
            }
        }

        copy_option_part(
            &raw mut p,
            core::ptr::addr_of_mut!(IObuff_funcexpand).cast(),
            IOSIZE_FUNCEXPAND,
            c",".as_ptr(),
        );
        if rs_may_advance_cpt_index(p) != 0 {
            rs_advance_cpt_sources_index_safe();
        }
    }
    crate::vars::nvim_set_cpt_sources_index(-1);

    xfree_funcexpand(cpt.cast());
    // Make the list cyclic
    crate::vars::nvim_set_compl_matches(rs_ins_compl_make_cyclic());
}

/// Return the `Callback *` for a 'cpt' option entry if it is a function source.
///
/// Returns `NULL` if the entry at `p` is not an `o` or `F` function entry.
///
/// # Safety
/// `p` must be a valid C string pointer to the current position in the 'cpt' option.
#[no_mangle]
pub unsafe extern "C" fn rs_get_callback_if_cpt_func(p: *const c_char, idx: c_int) -> CallbackPtr {
    nvim_get_callback_if_cpt_func_impl(p, idx)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_phase_constants() {
        // Ensure module compiles
        assert_eq!(1, 1);
    }
}
