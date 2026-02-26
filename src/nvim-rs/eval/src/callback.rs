//! Function comparison and callback conversion utilities migrated from eval.c.
//!
//! - `func_equal`: Compare two function/partial typvals
//! - `callback_from_typval`: Convert typval to Callback struct

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr,
    clippy::borrow_as_ptr
)]

use std::ffi::{c_char, c_int, c_void};

// Re-export CallbackT from eval_exec via FFI -- we reconstruct it here since
// eval crate doesn't depend on eval_exec.
//
// Layout verified by _Static_assert in eval_shim.c:
//   sizeof(Callback) == 16
//   offsetof(Callback, data) == 0
//   offsetof(Callback, type) == 8

/// Union data field of Callback (8 bytes).
#[repr(C)]
pub union CallbackData {
    /// kCallbackFuncref: heap-allocated function name string.
    pub funcref: *mut c_char,
    /// kCallbackPartial: pointer to partial_T.
    pub partial: *mut c_void,
    /// kCallbackLua: Lua registry reference.
    pub luaref: c_int,
}

/// Rust mirror of C `Callback` struct (16 bytes).
#[repr(C)]
pub struct CallbackT {
    /// Union data (funcref / partial / luaref).
    pub data: CallbackData,
    /// Which variant is active.
    pub cb_type: c_int,
    // 4 bytes trailing padding
}

const K_CALLBACK_NONE: c_int = 0;
const K_CALLBACK_PARTIAL: c_int = 2;

/// Opaque handle for typval_T struct
type TvHandle = *const c_void;
/// Mutable opaque handle for typval_T struct
type TvHandleMut = *mut c_void;
/// Opaque handle for partial_T struct
type PartialHandle = *mut c_void;
/// Opaque handle for dict_T struct
type DictHandle = *mut c_void;

// C VarType enum values (verified by _Static_assert in eval.c)
const VAR_FUNC: c_int = 3;
const VAR_SPECIAL: c_int = 8;
const VAR_PARTIAL: c_int = 9;
const VAR_STRING: c_int = 2;
const VAR_NUMBER: c_int = 1;

extern "C" {
    // typval field accessors
    fn nvim_eval_tv_get_type(tv: TvHandle) -> c_int;
    fn nvim_tv_get_vstring(tv: TvHandleMut) -> *mut c_char;
    fn nvim_eval_tv_get_partial(tv: TvHandle) -> PartialHandle;
    fn nvim_eval_tv_get_vnumber(tv: TvHandle) -> i64;

    // partial field accessors
    fn nvim_eval_partial_get_dict(pt: PartialHandle) -> DictHandle;
    fn nvim_eval_partial_get_argc(pt: PartialHandle) -> c_int;
    fn nvim_eval_partial_get_argv(pt: PartialHandle, idx: c_int) -> TvHandleMut;
    fn nvim_eval_partial_incref(pt: PartialHandle);

    // Already in Rust (cross-crate FFI)
    fn rs_partial_name(pt: PartialHandle) -> *mut c_char;

    // Comparison functions (remain in C)
    fn tv_dict_equal(d1: DictHandle, d2: DictHandle, ic: bool) -> bool;
    fn tv_equal(tv1: TvHandleMut, tv2: TvHandleMut, ic: bool) -> bool;

    // String/function operations
    fn get_scriptlocal_funcname(name: *const c_char) -> *mut c_char;
    fn func_ref(name: *const c_char);
    fn xstrdup(s: *const c_char) -> *mut c_char;

    // Lua operations
    fn nlua_is_table_from_lua(arg: TvHandle) -> c_int;
    fn nlua_register_table_as_callable(arg: TvHandle) -> *mut c_char;

    // Error reporting
    fn emsg(s: *const c_char) -> c_int;
}

static E921_MSG: &[u8] = b"E921: Invalid callback argument\0";

/// Compare two function/partial typvals for equality.
///
/// # Safety
///
/// `tv1` and `tv2` must be valid typval pointers with v_type of VAR_FUNC or VAR_PARTIAL.
#[no_mangle]
pub unsafe extern "C" fn rs_func_equal(tv1: TvHandle, tv2: TvHandle, ic: bool) -> bool {
    // empty and NULL function name considered the same
    let mut s1 = if nvim_eval_tv_get_type(tv1) == VAR_FUNC {
        nvim_tv_get_vstring(tv1.cast_mut())
    } else {
        rs_partial_name(nvim_eval_tv_get_partial(tv1))
    };
    if !s1.is_null() && *s1 == 0 {
        s1 = std::ptr::null_mut();
    }

    let mut s2 = if nvim_eval_tv_get_type(tv2) == VAR_FUNC {
        nvim_tv_get_vstring(tv2.cast_mut())
    } else {
        rs_partial_name(nvim_eval_tv_get_partial(tv2))
    };
    if !s2.is_null() && *s2 == 0 {
        s2 = std::ptr::null_mut();
    }

    if s1.is_null() || s2.is_null() {
        if s1 != s2 {
            return false;
        }
    } else if libc::strcmp(s1, s2) != 0 {
        return false;
    }

    // empty dict and NULL dict is different
    let d1 = if nvim_eval_tv_get_type(tv1) == VAR_FUNC {
        std::ptr::null_mut()
    } else {
        nvim_eval_partial_get_dict(nvim_eval_tv_get_partial(tv1))
    };
    let d2 = if nvim_eval_tv_get_type(tv2) == VAR_FUNC {
        std::ptr::null_mut()
    } else {
        nvim_eval_partial_get_dict(nvim_eval_tv_get_partial(tv2))
    };
    if d1.is_null() || d2.is_null() {
        if d1 != d2 {
            return false;
        }
    } else if !tv_dict_equal(d1, d2, ic) {
        return false;
    }

    // empty list and no list considered the same
    let a1 = if nvim_eval_tv_get_type(tv1) == VAR_FUNC {
        0
    } else {
        nvim_eval_partial_get_argc(nvim_eval_tv_get_partial(tv1))
    };
    let a2 = if nvim_eval_tv_get_type(tv2) == VAR_FUNC {
        0
    } else {
        nvim_eval_partial_get_argc(nvim_eval_tv_get_partial(tv2))
    };
    if a1 != a2 {
        return false;
    }
    for i in 0..a1 {
        if !tv_equal(
            nvim_eval_partial_get_argv(nvim_eval_tv_get_partial(tv1), i),
            nvim_eval_partial_get_argv(nvim_eval_tv_get_partial(tv2), i),
            ic,
        ) {
            return false;
        }
    }
    true
}

/// Get a callback from `arg`. It can be a Funcref or a function name.
///
/// Returns true on success, false on failure.
///
/// # Safety
///
/// `callback` must be a valid pointer to a Callback (CallbackT) struct.
/// `arg` must be a valid pointer to a typval_T.
#[no_mangle]
pub unsafe extern "C" fn rs_callback_from_typval(callback: *mut CallbackT, arg: TvHandle) -> bool {
    let v_type = nvim_eval_tv_get_type(arg);

    if v_type == VAR_PARTIAL && !nvim_eval_tv_get_partial(arg).is_null() {
        let pt = nvim_eval_tv_get_partial(arg);
        nvim_eval_partial_incref(pt);
        // cb->data.partial = pt; cb->type = kCallbackPartial;
        (*callback).data.partial = pt;
        (*callback).cb_type = K_CALLBACK_PARTIAL;
        return true;
    }

    if v_type == VAR_STRING {
        let vstr = nvim_tv_get_vstring(arg.cast_mut());
        if !vstr.is_null() && (*vstr as u8).is_ascii_digit() {
            emsg(E921_MSG.as_ptr() as *const c_char);
            return false;
        }
    }

    if v_type == VAR_FUNC || v_type == VAR_STRING {
        let name = nvim_tv_get_vstring(arg.cast_mut());
        if name.is_null() {
            emsg(E921_MSG.as_ptr() as *const c_char);
            return false;
        }
        if *name == 0 {
            // cb->data.funcref = NULL; cb->type = kCallbackNone;
            (*callback).data.funcref = std::ptr::null_mut();
            (*callback).cb_type = K_CALLBACK_NONE;
            return true;
        }

        let mut funcref: *mut c_char = if v_type == VAR_STRING {
            get_scriptlocal_funcname(name)
        } else {
            std::ptr::null_mut()
        };
        if funcref.is_null() {
            funcref = xstrdup(name);
        }
        func_ref(funcref);
        // cb->data.funcref = funcref; cb->type = kCallbackFuncref;
        (*callback).data.funcref = funcref;
        (*callback).cb_type = 1; // K_CALLBACK_FUNCREF
        return true;
    }

    if nlua_is_table_from_lua(arg) != 0 {
        let name = nlua_register_table_as_callable(arg);
        if !name.is_null() {
            (*callback).data.funcref = xstrdup(name);
            (*callback).cb_type = 1; // K_CALLBACK_FUNCREF
            return true;
        }
        emsg(E921_MSG.as_ptr() as *const c_char);
        return false;
    }

    if v_type == VAR_SPECIAL || (v_type == VAR_NUMBER && nvim_eval_tv_get_vnumber(arg) == 0) {
        // cb->data.funcref = NULL; cb->type = kCallbackNone;
        (*callback).data.funcref = std::ptr::null_mut();
        (*callback).cb_type = K_CALLBACK_NONE;
        return true;
    }

    emsg(E921_MSG.as_ptr() as *const c_char);
    false
}

// =============================================================================
// Phase 5 (eval_shim pass 4): partial_free + partial_unref
// =============================================================================

extern "C" {
    fn tv_clear(tv: TvHandleMut);
    fn xfree(ptr: *mut c_void);
    fn nvim_dict_unref(dict: DictHandle);
    fn nvim_func_unref(name: *mut c_char);
    fn nvim_func_ptr_unref(func: *mut c_void);
    fn nvim_eval_partial_get_name(pt: PartialHandle) -> *mut c_char;
    fn nvim_eval_partial_get_func(pt: PartialHandle) -> *mut c_void;
    /// Decrements pt_refcount and returns true if it drops to <= 0.
    fn nvim_partial_decref_and_check(pt: PartialHandle) -> bool;
}

/// Free all resources held by a partial_T and the partial itself.
///
/// Equivalent to C `partial_free` (static).
///
/// # Safety
/// - `pt` must be a valid non-null pointer to a partial_T.
unsafe fn partial_free_impl(pt: PartialHandle) {
    let argc = nvim_eval_partial_get_argc(pt);
    for i in 0..argc {
        tv_clear(nvim_eval_partial_get_argv(pt, i));
    }
    // Free the argv array itself (pt->pt_argv).
    // nvim_eval_partial_get_argv(pt, 0) == pt->pt_argv + 0 == pt->pt_argv.
    xfree(nvim_eval_partial_get_argv(pt, 0));
    nvim_dict_unref(nvim_eval_partial_get_dict(pt));
    let name = nvim_eval_partial_get_name(pt);
    if name.is_null() {
        let func = nvim_eval_partial_get_func(pt);
        nvim_func_ptr_unref(func);
    } else {
        nvim_func_unref(name);
        xfree(name.cast());
    }
    xfree(pt);
}

/// Unreference a closure: decrement the reference count and free it when zero.
///
/// Equivalent to C `partial_unref`.
///
/// # Safety
/// - `pt` may be null (no-op).
#[export_name = "partial_unref"]
pub unsafe extern "C" fn rs_partial_unref(pt: PartialHandle) {
    if pt.is_null() {
        return;
    }
    if nvim_partial_decref_and_check(pt) {
        partial_free_impl(pt);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_var_type_constants() {
        assert_eq!(VAR_NUMBER, 1);
        assert_eq!(VAR_STRING, 2);
        assert_eq!(VAR_FUNC, 3);
        assert_eq!(VAR_SPECIAL, 8);
        assert_eq!(VAR_PARTIAL, 9);
    }
}
