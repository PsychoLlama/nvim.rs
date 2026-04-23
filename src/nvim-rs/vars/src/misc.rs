//! Miscellaneous variable utility functions for VimL.
//!
//! Phase 9: Migrated from `src/nvim/eval/vars.c`.
//!
//! Functions:
//! - `rs_garbage_collect_scriptvars`: GC mark pass for script variables
//! - `rs_set_internal_string_var`: Set an internal string variable by name

#![allow(unsafe_op_in_unsafe_fn)]
#![allow(clippy::ptr_as_ptr)]
#![allow(clippy::cast_lossless)]
#![allow(clippy::manual_c_str_literals)]

use std::ffi::{c_char, c_int, c_void};

// VAR_STRING typval type constant (matches C VarType::VAR_STRING = 1)
const VAR_STRING: c_int = 1;

// Typval size in bytes (must match sizeof(typval_T) = 24)
const TYPVAL_SIZE: usize = 24;

// =============================================================================
// C extern declarations
// =============================================================================

extern "C" {
    // --- script vars iteration ---
    fn nvim_script_items_len() -> c_int;
    fn nvim_get_script_vars_dict(sid: c_int) -> *mut c_void;
    fn nvim_dict_get_hashtab(dict: *mut c_void) -> *mut c_void;

    // --- set_var ---
    fn set_var(name: *const c_char, name_len: usize, tv: *const c_void, copy: bool);

    // --- string ops ---
    fn strlen(s: *const c_char) -> usize;

    // --- GC ---
    fn rs_set_ref_in_ht(ht: *mut c_void, copy_id: c_int, list_stack: *mut *mut c_void) -> bool;
}

/// Mark all script variable hashtabs as referenced for garbage collection.
///
/// Matches C `garbage_collect_scriptvars`. Returns true if GC should abort.
///
/// # Safety
/// Called only during the GC mark phase with a valid copy_id.
#[no_mangle]
pub unsafe extern "C" fn rs_garbage_collect_scriptvars(copy_id: c_int) -> bool {
    let mut abort = false;
    let len = nvim_script_items_len();
    for i in 1..=len {
        let dict = nvim_get_script_vars_dict(i);
        if !dict.is_null() {
            let ht = nvim_dict_get_hashtab(dict);
            if !ht.is_null() {
                abort = abort || rs_set_ref_in_ht(ht, copy_id, std::ptr::null_mut());
            }
        }
    }
    abort
}

/// Set an internal variable to a string value.
///
/// Matches C `set_internal_string_var`. Creates the variable if it does not
/// already exist.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string.
/// `value` must be a valid NUL-terminated C string or NULL.
#[no_mangle]
pub unsafe extern "C" fn rs_set_internal_string_var(name: *const c_char, value: *mut c_char) {
    // Build a VAR_STRING typval on the heap.
    let mut tv_buf = [0u8; TYPVAL_SIZE];
    let tv = tv_buf.as_mut_ptr() as *mut c_void;

    // v_type at offset 0
    let vtype_ptr = tv as *mut c_int;
    *vtype_ptr = VAR_STRING;

    // vval.v_string at offset 8
    let vstring_ptr = tv.add(8) as *mut *mut c_char;
    *vstring_ptr = value;

    let name_len = strlen(name);
    set_var(name, name_len, tv, true);
}
