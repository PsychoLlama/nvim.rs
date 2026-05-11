//! Free-all-functions teardown for VimL (EXITFREE path).
//!
//! Phase 6 (plan db85cc6b) from `src/nvim/eval/userfunc.c`:
//! - `free_all_functions` — only built when EXITFREE is defined.

#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]

use std::ffi::{c_char, c_int, c_void};

extern "C" {
    // Current funccal chain
    fn nvim_get_current_funccal() -> *mut c_void;
    fn nvim_fc_get_rettv(fc: *mut c_void) -> *mut c_void;
    fn tv_clear(tv: *mut c_void);
    fn rs_cleanup_function_call(fc: *mut c_void);
    fn restore_funccal();

    // func_hashtab iteration (from Phase 3 / userfunc.c)
    fn nvim_func_ht_used() -> usize;
    fn nvim_func_ht_changed() -> c_int;
    fn nvim_func_ht_array() -> *mut c_void;
    fn nvim_hashitem_is_empty(hi: *const c_void) -> c_int;
    fn nvim_hashitem_to_ufunc(hi: *const c_void) -> *mut c_void;
    fn nvim_hashitem_advance(hi: *mut c_void) -> *mut c_void;
    fn nvim_ufunc_get_name(fp: *mut c_void) -> *const c_char;

    // funccal_stack
    fn nvim_funccal_stack_head() -> *mut c_void;

    // Clear + free functions (refcount.rs and Rust-exported)
    fn rs_func_name_refcount(name: *const c_char) -> c_int;
    fn rs_func_clear(fp: *mut c_void, force: c_int);
    fn rs_func_free(fp: *mut c_void);

    // hash_clear the func_hashtab
    fn nvim_func_ht_hash_clear();
}

// =============================================================================
// rs_free_all_functions
// =============================================================================

/// Free all user-defined functions and clean up the funccal stack.
///
/// Only called on exit when EXITFREE is defined.
/// Mirrors `free_all_functions()` from `userfunc.c`.
///
/// # Safety
/// Must only be called once at process exit.
#[no_mangle]
pub unsafe extern "C" fn rs_free_all_functions() {
    // Clean up the current_funccal chain and the funccal stack.
    loop {
        let fc = nvim_get_current_funccal();
        if fc.is_null() {
            break;
        }
        let rettv = nvim_fc_get_rettv(fc);
        if !rettv.is_null() {
            tv_clear(rettv);
        }
        rs_cleanup_function_call(fc);
        // If current_funccal is now NULL but funccal_stack is non-empty, restore.
        if nvim_get_current_funccal().is_null() && !nvim_funccal_stack_head().is_null() {
            restore_funccal();
        }
    }

    // First pass: clear what the functions contain.
    // Restart from the beginning whenever ht_changed shifts.
    let mut skipped: u64 = 0;
    loop {
        let todo = nvim_func_ht_used();
        if todo == 0 {
            break;
        }
        let mut remaining = todo;
        let mut hi = nvim_func_ht_array();
        let mut restarted = false;
        while remaining > 0 {
            if nvim_hashitem_is_empty(hi) == 0 {
                let fp = nvim_hashitem_to_ufunc(hi);
                remaining -= 1;
                let name = nvim_ufunc_get_name(fp);
                if rs_func_name_refcount(name) != 0 {
                    skipped += 1;
                } else {
                    let changed = nvim_func_ht_changed();
                    rs_func_clear(fp, 1);
                    if nvim_func_ht_changed() != changed {
                        skipped = 0;
                        restarted = true;
                        break;
                    }
                }
            }
            hi = nvim_hashitem_advance(hi);
        }
        if !restarted {
            break;
        }
    }

    // Second pass: free the functions one at a time, restarting on each free.
    skipped = 0;
    loop {
        let used = nvim_func_ht_used();
        if used <= skipped as usize {
            break;
        }
        let mut todo = used;
        let mut hi = nvim_func_ht_array();
        let mut freed_one = false;
        while todo > 0 {
            if nvim_hashitem_is_empty(hi) == 0 {
                todo -= 1;
                let fp = nvim_hashitem_to_ufunc(hi);
                let name = nvim_ufunc_get_name(fp);
                if rs_func_name_refcount(name) != 0 {
                    skipped += 1;
                } else {
                    rs_func_free(fp);
                    skipped = 0;
                    freed_one = true;
                    break;
                }
            }
            hi = nvim_hashitem_advance(hi);
        }
        if !freed_one {
            break; // all remaining are refcounted; done
        }
    }

    if skipped == 0 {
        nvim_func_ht_hash_clear();
    }
}
