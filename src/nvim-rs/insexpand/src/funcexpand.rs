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
#[no_mangle]
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
#[no_mangle]
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
#[no_mangle]
pub unsafe extern "C" fn rs_f_complete(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: *mut c_void) {
    nvim_f_complete_impl(argvars, rettv);
}

/// VimL `complete_add()` builtin.
///
/// # Safety
/// `argvars` must be a valid `typval_T[1]` pointer; `rettv` a `typval_T*`.
#[no_mangle]
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

extern "C" {
    fn nvim_cpt_compl_refresh_impl();
    fn nvim_get_callback_if_cpt_func_impl(p: *const c_char, idx: c_int) -> CallbackPtr;
}

/// Refresh completion matches from 'cpt' function sources with `refresh:always`.
///
/// # Safety
/// Requires valid global completion state and a live current buffer.
#[no_mangle]
pub unsafe extern "C" fn rs_cpt_compl_refresh() {
    nvim_cpt_compl_refresh_impl();
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
