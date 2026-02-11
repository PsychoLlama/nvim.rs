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

/// Opaque handle for typval_T struct
type TvHandle = *const c_void;
/// Mutable opaque handle for typval_T struct
type TvHandleMut = *mut c_void;
/// Opaque handle for Callback struct
type CallbackHandle = *mut c_void;
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
    fn nvim_eval_tv_get_vstring(tv: TvHandle) -> *mut c_char;
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

    // Callback struct setters
    fn nvim_eval_cb_set_partial(cb: CallbackHandle, pt: PartialHandle);
    fn nvim_eval_cb_set_funcref(cb: CallbackHandle, name: *mut c_char);
    fn nvim_eval_cb_set_none(cb: CallbackHandle);

    // String/function operations
    fn get_scriptlocal_funcname(name: *const c_char) -> *mut c_char;
    fn func_ref(name: *const c_char);
    fn xstrdup(s: *const c_char) -> *mut c_char;

    // Lua operations
    fn nlua_is_table_from_lua(arg: TvHandle) -> c_int;
    fn nlua_register_table_as_callable(arg: TvHandle) -> *mut c_char;

    // Error reporting
    fn nvim_eval_emsg_e921();
}

/// Compare two function/partial typvals for equality.
///
/// # Safety
///
/// `tv1` and `tv2` must be valid typval pointers with v_type of VAR_FUNC or VAR_PARTIAL.
#[no_mangle]
pub unsafe extern "C" fn rs_func_equal(tv1: TvHandle, tv2: TvHandle, ic: bool) -> bool {
    // empty and NULL function name considered the same
    let mut s1 = if nvim_eval_tv_get_type(tv1) == VAR_FUNC {
        nvim_eval_tv_get_vstring(tv1)
    } else {
        rs_partial_name(nvim_eval_tv_get_partial(tv1))
    };
    if !s1.is_null() && *s1 == 0 {
        s1 = std::ptr::null_mut();
    }

    let mut s2 = if nvim_eval_tv_get_type(tv2) == VAR_FUNC {
        nvim_eval_tv_get_vstring(tv2)
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
/// `callback` must be a valid pointer to a Callback struct.
/// `arg` must be a valid pointer to a typval_T.
#[no_mangle]
pub unsafe extern "C" fn rs_callback_from_typval(callback: CallbackHandle, arg: TvHandle) -> bool {
    let v_type = nvim_eval_tv_get_type(arg);

    if v_type == VAR_PARTIAL && !nvim_eval_tv_get_partial(arg).is_null() {
        let pt = nvim_eval_tv_get_partial(arg);
        nvim_eval_partial_incref(pt);
        nvim_eval_cb_set_partial(callback, pt);
        return true;
    }

    if v_type == VAR_STRING {
        let vstr = nvim_eval_tv_get_vstring(arg);
        if !vstr.is_null() && (*vstr as u8).is_ascii_digit() {
            nvim_eval_emsg_e921();
            return false;
        }
    }

    if v_type == VAR_FUNC || v_type == VAR_STRING {
        let name = nvim_eval_tv_get_vstring(arg);
        if name.is_null() {
            nvim_eval_emsg_e921();
            return false;
        }
        if *name == 0 {
            nvim_eval_cb_set_none(callback);
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
        nvim_eval_cb_set_funcref(callback, funcref);
        return true;
    }

    if nlua_is_table_from_lua(arg) != 0 {
        let name = nlua_register_table_as_callable(arg);
        if !name.is_null() {
            nvim_eval_cb_set_funcref(callback, xstrdup(name));
            return true;
        }
        nvim_eval_emsg_e921();
        return false;
    }

    if v_type == VAR_SPECIAL || (v_type == VAR_NUMBER && nvim_eval_tv_get_vnumber(arg) == 0) {
        nvim_eval_cb_set_none(callback);
        return true;
    }

    nvim_eval_emsg_e921();
    false
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
