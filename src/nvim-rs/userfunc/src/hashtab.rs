//! Small hashtab foundation functions for userfunc.
//!
//! Phase 1 migration (plan db85cc6b) from `src/nvim/eval/userfunc.c`:
//! - `find_func` (Rust: rs_find_func)
//! - `add_nr_var` (Rust: rs_add_nr_var)
//! - `register_closure` (Rust: rs_register_closure)
//! - `nvim_func_remove_impl` (inlined into rs_func_remove_ht)
//! - `nvim_set_ref_in_functions_impl` (inlined into rs_set_ref_in_functions via nvim_func_ht_foreach)
//!
//! Wave 2 Phase 1: `find_func` body moved to Rust (exported as `find_func`).
//! `apply_autocmds_for_funcundefined` body moved to Rust.

#![allow(clippy::missing_safety_doc)]

use std::ffi::{c_char, c_int, c_void};

// EVENT_FUNCUNDEFINED value (autocmd_defs.h kEventFuncundefined = 68)
const EVENT_FUNCUNDEFINED: c_int = 68;

extern "C" {
    // Global func hashtab lookup/remove (Phase 1 new accessors)
    fn nvim_func_ht_remove_fp(fp: *mut c_void) -> c_int;

    // func_tbl_get: returns &func_hashtab (hashtab_T *)
    fn func_tbl_get() -> *mut c_void;

    // hash_find: find key in hashtab, returns hashitem_T * (Rust-implemented, export_name="hash_find")
    fn hash_find(ht: *mut c_void, key: *const c_char) -> *mut c_void;

    // hashitem empty check (HASHITEM_EMPTY macro equivalent)
    fn nvim_hashitem_is_empty(hi: *const c_void) -> c_int;

    // HI2UF macro equivalent
    fn nvim_hashitem_to_ufunc(hi: *const c_void) -> *mut c_void;

    // apply_autocmds for funcundefined
    fn apply_autocmds(
        event: c_int,
        fname: *const c_char,
        fname_io: *const c_char,
        force: bool,
        buf: *mut c_void,
    ) -> bool;

    // funccall_T accessors for register_closure
    fn get_current_funccal() -> *mut c_void;
    fn nvim_ufunc_get_scoped(fp: *mut c_void) -> *mut c_void;
    fn nvim_ufunc_set_scoped(fp: *mut c_void, fc: *mut c_void);
    fn nvim_fc_increment_refcount(fc: *mut c_void);
    fn nvim_fc_ufuncs_push(fc: *mut c_void, fp: *mut c_void);

    // funccal_unref from funccal.rs (for register_closure: unref old scoped)
    fn rs_funccal_unref(fc: *mut c_void, fp: *mut c_void, force: c_int);

    // add_nr_var: C helper that does the full init sequence
    fn nvim_add_nr_var(dp: *mut c_void, v: *mut c_void, name: *const c_char, nr: i64);

    // ufunc name accessor (for set_ref_in_functions_cb)
    fn nvim_ufunc_get_name(fp: *mut c_void) -> *const c_char;

    // set_ref_in_func for nvim_set_ref_in_functions_impl inline
    fn set_ref_in_func(name: *mut c_char, fp: *mut c_void, copy_id: c_int) -> c_int;
    fn rs_func_name_refcount(name: *const c_char) -> c_int;
    fn nvim_func_ht_foreach(
        cb: unsafe extern "C" fn(fp: *mut c_void, ctx: *mut c_void),
        ctx: *mut c_void,
    );
}

// =============================================================================
// find_func  (Wave 2 Phase 1: C body deleted, Rust owns the symbol)
// =============================================================================

/// Find a function by name in the global function hashtable.
/// Returns NULL if not found.
///
/// Replaces C `find_func` — body is now here; C file has `extern` decl only.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string.
#[unsafe(export_name = "find_func")]
pub unsafe extern "C" fn rs_find_func(name: *const c_char) -> *mut c_void {
    let ht = unsafe { func_tbl_get() };
    let hi = unsafe { hash_find(ht, name) };
    if unsafe { nvim_hashitem_is_empty(hi) } != 0 {
        return std::ptr::null_mut();
    }
    unsafe { nvim_hashitem_to_ufunc(hi) }
}

// =============================================================================
// apply_autocmds_for_funcundefined  (Wave 2 Phase 1: C body deleted)
// =============================================================================

/// Apply FuncUndefined autocmd and return result.
///
/// Replaces C `apply_autocmds_for_funcundefined` — body is now here.
///
/// # Safety
/// `name` must be a valid NUL-terminated C string.
#[unsafe(export_name = "apply_autocmds_for_funcundefined")]
pub unsafe extern "C" fn rs_apply_autocmds_for_funcundefined(name: *const c_char) -> c_int {
    c_int::from(unsafe {
        apply_autocmds(EVENT_FUNCUNDEFINED, name, name, true, std::ptr::null_mut())
    })
}

// =============================================================================
// rs_add_nr_var
// =============================================================================

/// Add a number variable "name" with value nr to dict dp using dictitem v.
///
/// # Safety
/// All pointers must be valid. `name` must be a valid NUL-terminated string
/// fitting in di_key (≤ DICTITEM_KEY_LEN bytes).
#[no_mangle]
pub unsafe extern "C" fn rs_add_nr_var(
    dp: *mut c_void,
    v: *mut c_void,
    name: *const c_char,
    nr: i64,
) {
    unsafe { nvim_add_nr_var(dp, v, name, nr) };
}

// =============================================================================
// rs_register_closure
// =============================================================================

/// Register function `fp` as using `current_funccal` as its scope.
///
/// Mirrors C `register_closure(fp)`:
/// - If fp->uf_scoped already equals current_funccal, does nothing.
/// - Otherwise unrefs the old scoped fc, sets the new one, increments
///   the new fc's refcount, and pushes fp onto fc_ufuncs.
///
/// # Safety
/// `fp` must be a valid `ufunc_T *`. Current funccal state must be consistent.
#[no_mangle]
pub unsafe extern "C" fn rs_register_closure(fp: *mut c_void) {
    let current_fc = unsafe { get_current_funccal() };
    let old_scoped = unsafe { nvim_ufunc_get_scoped(fp) };
    if old_scoped == current_fc {
        // no change
        return;
    }
    unsafe { rs_funccal_unref(old_scoped, fp, 0) };
    unsafe { nvim_ufunc_set_scoped(fp, current_fc) };
    unsafe { nvim_fc_increment_refcount(current_fc) };
    unsafe { nvim_fc_ufuncs_push(current_fc, fp) };
}

// =============================================================================
// rs_func_remove_ht  (inline of nvim_func_remove_impl)
// =============================================================================

/// Remove fp from the global function hashtable.
/// Returns 1 if the entry was deleted, 0 if not found.
///
/// This is the direct Rust implementation; C `nvim_func_remove_impl` is deleted.
/// `rs_func_remove` in refcount.rs calls this.
#[no_mangle]
pub unsafe extern "C" fn rs_func_remove_ht(fp: *mut c_void) -> c_int {
    unsafe { nvim_func_ht_remove_fp(fp) }
}

// =============================================================================
// set_ref_in_functions_cb / rs_set_ref_in_functions_inner
// =============================================================================

/// Per-ufunc callback used by the inline implementation of
/// `nvim_set_ref_in_functions_impl`.
struct SetRefCtx {
    copy_id: c_int,
    abort: bool,
}

unsafe extern "C" fn set_ref_in_functions_cb(fp: *mut c_void, ctx_ptr: *mut c_void) {
    let ctx = unsafe { &mut *ctx_ptr.cast::<SetRefCtx>() };
    if ctx.abort {
        return;
    }
    let name = unsafe { nvim_ufunc_get_name(fp) };
    if unsafe { rs_func_name_refcount(name) } == 0
        && unsafe { set_ref_in_func(std::ptr::null_mut(), fp, ctx.copy_id) } != 0
    {
        ctx.abort = true;
    }
}

/// Inline implementation of C `nvim_set_ref_in_functions_impl`.
/// Called by `rs_set_ref_in_functions` in gc.rs.
///
/// # Safety
/// Accesses global C state.
#[no_mangle]
pub unsafe extern "C" fn rs_set_ref_in_functions_inner(copy_id: c_int) -> c_int {
    let mut ctx = SetRefCtx {
        copy_id,
        abort: false,
    };
    unsafe {
        nvim_func_ht_foreach(
            set_ref_in_functions_cb,
            std::ptr::addr_of_mut!(ctx).cast::<c_void>(),
        );
    };
    c_int::from(ctx.abort)
}
