//! Subscript/index evaluation for VimL.
//!
//! This module implements `eval_index`, `check_can_index`, `eval_index_inner`,
//! and `f_slice`, migrated from `src/nvim/eval_shim.c`.
//!
//! ## Subscript Syntax
//!
//! - `dict.name` -- dictionary key lookup by identifier
//! - `list[idx]` -- list index
//! - `list[start:end]` -- list/string/blob slice
//! - `string[idx]` -- string byte index
//!
//! ## FFI Pattern
//!
//! Each function has a Rust implementation and a `#[no_mangle]` FFI export
//! (`rs_eval_index`, `rs_check_can_index`, `rs_eval_index_inner`, `rs_f_slice`).

#![allow(clippy::too_many_lines)]
#![allow(clippy::too_many_arguments)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::assign_op_pattern)]
#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use crate::eval::{EvalargHandle, TypevalHandle};

// =============================================================================
// Constants
// =============================================================================

/// OK return value from C functions
const OK: c_int = 1;
/// FAIL return value from C functions
const FAIL: c_int = 0;

/// EVAL_EVALUATE flag
const EVAL_EVALUATE: c_int = 1;

// VarType constants (must match C enum var_type_T)
const VAR_UNKNOWN: c_int = 0;
const VAR_NUMBER: c_int = 1;
const VAR_STRING: c_int = 2;
const VAR_FUNC: c_int = 3;
const VAR_LIST: c_int = 4;
const VAR_DICT: c_int = 5;
const VAR_FLOAT: c_int = 6;
const VAR_BOOL: c_int = 7;
const VAR_SPECIAL: c_int = 8;
const VAR_PARTIAL: c_int = 9;
const VAR_BLOB: c_int = 10;

/// VARNUMBER_MAX = INT64_MAX
const VARNUMBER_MAX: i64 = i64::MAX;

// =============================================================================
// C Extern Functions
// =============================================================================

extern "C" {
    // String utilities
    fn skipwhite(p: *const c_char) -> *mut c_char;

    // Typval operations
    fn tv_clear(tv: TypevalHandle);
    fn tv_check_str(tv: TypevalHandle) -> bool;
    fn tv_copy(from: TypevalHandle, to: TypevalHandle);

    // Typval field accessors
    fn nvim_tv_get_type(tv: TypevalHandle) -> c_int;
    fn nvim_tv_get_list(tv: TypevalHandle) -> *mut c_void;
    fn nvim_tv_get_blob(tv: TypevalHandle) -> *mut c_void;
    fn nvim_tv_get_dict(tv: TypevalHandle) -> *mut c_void;
    fn nvim_tv_set_type(tv: TypevalHandle, vtype: c_int);
    fn nvim_tv_set_string(tv: TypevalHandle, s: *mut c_char);

    // evalarg operations
    fn evalarg_get_flags(ea: EvalargHandle) -> c_int;

    // eval1 (recursive subscript parsing)
    fn eval1(arg: *mut *mut c_char, rettv: TypevalHandle, evalarg: EvalargHandle) -> c_int;

    // eval_isdictc (check if char is valid dict key char)
    fn rs_eval_isdictc(c: c_int) -> bool;

    // tv_get_number: get number value from typval
    fn tv_get_number(tv: TypevalHandle) -> i64;

    // tv_get_string: get string value (static buffer)
    fn tv_get_string(tv: TypevalHandle) -> *const c_char;

    // tv_get_string_chk: get string value with error check
    fn tv_get_string_chk(tv: TypevalHandle) -> *const c_char;

    // tv_list_slice_or_index: apply index/slice to list
    fn tv_list_slice_or_index(
        list: *mut c_void,
        is_range: bool,
        n1: i64,
        n2: i64,
        exclusive: bool,
        rettv: TypevalHandle,
        verbose: bool,
    ) -> c_int;

    // tv_blob_slice_or_index: apply index/slice to blob
    fn tv_blob_slice_or_index(
        blob: *const c_void,
        is_range: bool,
        n1: i64,
        n2: i64,
        exclusive: bool,
        rettv: TypevalHandle,
    ) -> c_int;

    // tv_dict_find: look up a key in a dict
    fn tv_dict_find(d: *const c_void, key: *const c_char, len: isize) -> *mut c_void; // returns dictitem_T*

    // nvim_di_get_tv: get typval pointer from dictitem
    fn nvim_di_get_tv(di: *mut c_void) -> TypevalHandle;

    // rs_is_luafunc: check if a partial is a lua func (from eval crate)
    fn rs_is_luafunc(pt: *const c_void) -> bool;

    // nvim_eval_tv_get_partial: get partial pointer from typval
    fn nvim_eval_tv_get_partial(tv: TypevalHandle) -> *mut c_void;

    // String operations
    fn xmemdupz(src: *const c_void, len: usize) -> *mut c_char;
    fn strlen(s: *const c_char) -> usize;

    // rs_string_slice / rs_char_from_string (from eval crate)
    fn rs_string_slice(s: *const c_char, first: i64, last: i64, exclusive: bool) -> *mut c_char;
    fn rs_char_from_string(s: *const c_char, index: i64) -> *mut c_char;

    // Error message accessor functions
    fn nvim_emsg_missbrac();
    fn nvim_emsg_cannot_index_funcref();
    fn nvim_emsg_using_float_as_string();
    fn nvim_emsg_cannot_index_special();
    fn nvim_emsg_cannot_slice_dict();
    fn nvim_semsg_dictkey(key: *const c_char);
    fn nvim_semsg_dictkey_len(keylen: isize, key: *const c_char);

    // Typval alloc/free helpers
    fn xmalloc(size: usize) -> *mut c_void;
    fn xfree(ptr: *mut c_void);
}

// =============================================================================
// Typval allocation helpers
// =============================================================================

/// Allocate a zero-initialized typval_T on the heap
unsafe fn alloc_typval() -> TypevalHandle {
    let ptr = xmalloc(64);
    ptr::write_bytes(ptr as *mut u8, 0, 64);
    TypevalHandle::from_ptr(ptr)
}

/// Free a heap-allocated typval_T
unsafe fn free_typval(tv: TypevalHandle) {
    if !tv.is_null() {
        xfree(tv.as_ptr());
    }
}

// =============================================================================
// check_can_index
// =============================================================================

/// Check if `rettv` can have an `[index]` or `[sli:ce]`.
///
/// Returns OK if indexing is allowed, FAIL with error message otherwise.
///
/// # Safety
/// - `rettv` must be a valid typval handle
pub unsafe fn check_can_index_impl(rettv: TypevalHandle, evaluate: bool, verbose: bool) -> c_int {
    let vtype = nvim_tv_get_type(rettv);

    match vtype {
        VAR_FUNC | VAR_PARTIAL => {
            if verbose {
                nvim_emsg_cannot_index_funcref();
            }
            FAIL
        }
        VAR_FLOAT => {
            if verbose {
                nvim_emsg_using_float_as_string();
            }
            FAIL
        }
        VAR_BOOL | VAR_SPECIAL => {
            if verbose {
                nvim_emsg_cannot_index_special();
            }
            FAIL
        }
        VAR_UNKNOWN => {
            if evaluate {
                nvim_emsg_cannot_index_special();
                return FAIL;
            }
            // FALLTHROUGH: same as valid types
            OK
        }
        VAR_STRING | VAR_NUMBER | VAR_LIST | VAR_DICT | VAR_BLOB => OK,
        _ => OK,
    }
}

/// FFI export for check_can_index.
///
/// # Safety
/// See `check_can_index_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_check_can_index(
    rettv: TypevalHandle,
    evaluate: bool,
    verbose: bool,
) -> c_int {
    check_can_index_impl(rettv, evaluate, verbose)
}

// =============================================================================
// eval_index
// =============================================================================

/// Evaluate an `[expr]` or `[expr:expr]` index. Also `dict.key`.
///
/// `*arg` points to the `[` or `.`.
///
/// Returns OK or FAIL. On return, `*arg` is advanced past the closing `]`.
///
/// # Safety
/// - `arg` must be a valid pointer to a mutable C string pointer
/// - `rettv` must be a valid typval handle
/// - `evalarg` can be null
pub unsafe fn eval_index_impl(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
    verbose: bool,
) -> c_int {
    let evaluate = !evalarg.is_null() && (evalarg_get_flags(evalarg) & EVAL_EVALUATE) != 0;

    if check_can_index_impl(rettv, evaluate, verbose) == FAIL {
        return FAIL;
    }

    let mut empty1 = false;
    let mut empty2 = false;
    let mut range = false;
    let mut key: *const c_char = ptr::null();
    let mut keylen: isize = -1;

    let var1 = alloc_typval();
    let var2 = alloc_typval();

    if **arg == b'.' as c_char {
        // dict.name
        key = (*arg).add(1);
        keylen = 0;
        while rs_eval_isdictc(*key.add(keylen as usize) as c_int) {
            keylen += 1;
        }
        if keylen == 0 {
            free_typval(var1);
            free_typval(var2);
            return FAIL;
        }
        *arg = skipwhite(key.add(keylen as usize));
    } else {
        // something[idx]
        *arg = skipwhite((*arg).add(1));
        if **arg == b':' as c_char {
            empty1 = true;
        } else if eval1(arg, var1, evalarg) == FAIL {
            free_typval(var1);
            free_typval(var2);
            return FAIL;
        } else if evaluate && !tv_check_str(var1) {
            tv_clear(var1);
            free_typval(var1);
            free_typval(var2);
            return FAIL;
        }

        // Get the second variable from inside the [:]
        if **arg == b':' as c_char {
            range = true;
            *arg = skipwhite((*arg).add(1));
            if **arg == b']' as c_char {
                empty2 = true;
            } else if eval1(arg, var2, evalarg) == FAIL {
                if !empty1 {
                    tv_clear(var1);
                }
                free_typval(var1);
                free_typval(var2);
                return FAIL;
            } else if evaluate && !tv_check_str(var2) {
                if !empty1 {
                    tv_clear(var1);
                }
                tv_clear(var2);
                free_typval(var1);
                free_typval(var2);
                return FAIL;
            }
        }

        // Check for the ']'
        if **arg != b']' as c_char {
            if verbose {
                nvim_emsg_missbrac();
            }
            if !empty1 {
                tv_clear(var1);
            }
            if range {
                tv_clear(var2);
            }
            free_typval(var1);
            free_typval(var2);
            return FAIL;
        }
        *arg = skipwhite((*arg).add(1)); // skip the ']'
    }

    let ret = if evaluate {
        let v1 = if empty1 { TypevalHandle::null() } else { var1 };
        let v2 = if empty2 { TypevalHandle::null() } else { var2 };
        let res = eval_index_inner_impl(rettv, range, v1, v2, false, key, keylen, verbose);
        if !empty1 {
            tv_clear(var1);
        }
        if range && !empty2 {
            tv_clear(var2);
        }
        res
    } else {
        OK
    };

    free_typval(var1);
    free_typval(var2);
    ret
}

/// FFI export for eval_index.
///
/// # Safety
/// See `eval_index_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_index(
    arg: *mut *mut c_char,
    rettv: TypevalHandle,
    evalarg: EvalargHandle,
    verbose: bool,
) -> c_int {
    eval_index_impl(arg, rettv, evalarg, verbose)
}

// =============================================================================
// eval_index_inner
// =============================================================================

/// Apply index or range to `rettv`.
///
/// - `var1`: first index, null for `[:expr]`
/// - `var2`: second index, null for `[expr]` and `[expr:]`
/// - `exclusive`: true for `slice()`: second index is exclusive
/// - `key`: dict key (non-null for dict.name access)
/// - `keylen`: length of key, -1 for NUL-terminated
///
/// # Safety
/// - `rettv` must be a valid typval handle
/// - `var1`/`var2` can be null
pub unsafe fn eval_index_inner_impl(
    rettv: TypevalHandle,
    is_range: bool,
    var1: TypevalHandle,
    var2: TypevalHandle,
    exclusive: bool,
    key: *const c_char,
    keylen: isize,
    verbose: bool,
) -> c_int {
    let mut n1: i64 = 0;
    let mut n2: i64 = 0;

    if !var1.is_null() && nvim_tv_get_type(rettv) != VAR_DICT {
        n1 = tv_get_number(var1);
    }

    if is_range {
        if nvim_tv_get_type(rettv) == VAR_DICT {
            if verbose {
                nvim_emsg_cannot_slice_dict();
            }
            return FAIL;
        }
        if !var2.is_null() {
            n2 = tv_get_number(var2);
        } else {
            n2 = VARNUMBER_MAX;
        }
    }

    let vtype = nvim_tv_get_type(rettv);
    match vtype {
        VAR_BOOL | VAR_SPECIAL | VAR_FUNC | VAR_FLOAT | VAR_PARTIAL | VAR_UNKNOWN => {
            // Not evaluating, skipping over subscript
        }

        VAR_NUMBER | VAR_STRING => {
            let s = tv_get_string(rettv);
            let v: *mut c_char;
            let len = strlen(s) as i64;
            if exclusive {
                if is_range {
                    v = rs_string_slice(s, n1, n2, exclusive);
                } else {
                    v = rs_char_from_string(s, n1);
                }
            } else if is_range {
                // The resulting variable is a substring.
                // If the indexes are out of range the result is empty.
                let mut n1 = n1;
                let mut n2 = n2;
                if n1 < 0 {
                    n1 = len + n1;
                    if n1 < 0 {
                        n1 = 0;
                    }
                }
                if n2 < 0 {
                    n2 = len + n2;
                } else if n2 >= len {
                    n2 = len;
                }
                if n1 >= len || n2 < 0 || n1 > n2 {
                    v = ptr::null_mut();
                } else {
                    v = xmemdupz(s.add(n1 as usize) as *const c_void, (n2 - n1 + 1) as usize);
                }
            } else {
                // Single character result
                if n1 >= len || n1 < 0 {
                    v = ptr::null_mut();
                } else {
                    v = xmemdupz(s.add(n1 as usize) as *const c_void, 1);
                }
            }
            tv_clear(rettv);
            nvim_tv_set_type(rettv, VAR_STRING);
            nvim_tv_set_string(rettv, v);
        }

        VAR_BLOB => {
            let blob = nvim_tv_get_blob(rettv);
            tv_blob_slice_or_index(blob, is_range, n1, n2, exclusive, rettv);
        }

        VAR_LIST => {
            let n1 = if var1.is_null() { 0 } else { n1 };
            let n2 = if var2.is_null() { VARNUMBER_MAX } else { n2 };
            let list = nvim_tv_get_list(rettv);
            if tv_list_slice_or_index(list, is_range, n1, n2, exclusive, rettv, verbose) == FAIL {
                return FAIL;
            }
        }

        VAR_DICT => {
            let resolved_key = if key.is_null() {
                let k = tv_get_string_chk(var1);
                if k.is_null() {
                    return FAIL;
                }
                k
            } else {
                key
            };

            let dict = nvim_tv_get_dict(rettv);
            let item = tv_dict_find(dict, resolved_key, keylen);

            if item.is_null() && verbose {
                if keylen > 0 {
                    nvim_semsg_dictkey_len(keylen, resolved_key);
                } else {
                    nvim_semsg_dictkey(resolved_key);
                }
            }

            if item.is_null() {
                return FAIL;
            }

            let item_tv = nvim_di_get_tv(item);
            // Inline tv_is_luafunc: v_type == VAR_PARTIAL && rs_is_luafunc(partial)
            if nvim_tv_get_type(item_tv) == VAR_PARTIAL
                && rs_is_luafunc(nvim_eval_tv_get_partial(item_tv))
            {
                return FAIL;
            }

            let tmp = alloc_typval();
            tv_copy(item_tv, tmp);
            tv_clear(rettv);
            tv_copy(tmp, rettv);
            tv_clear(tmp);
            free_typval(tmp);
        }

        _ => {}
    }

    OK
}

/// FFI export for eval_index_inner.
///
/// # Safety
/// See `eval_index_inner_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn rs_eval_index_inner(
    rettv: TypevalHandle,
    is_range: bool,
    var1: TypevalHandle,
    var2: TypevalHandle,
    exclusive: bool,
    key: *const c_char,
    keylen: isize,
    verbose: bool,
) -> c_int {
    eval_index_inner_impl(rettv, is_range, var1, var2, exclusive, key, keylen, verbose)
}

// =============================================================================
// f_slice
// =============================================================================

/// VimL `slice()` function implementation.
///
/// `argvars[0]` is the value to slice.
/// `argvars[1]` is the start index.
/// `argvars[2]` is the end index (optional, VAR_UNKNOWN if absent).
///
/// # Safety
/// - `argvars` must be a valid pointer to at least 3 typval_T values
/// - `rettv` must be a valid typval handle
pub unsafe fn f_slice_impl(argvars: TypevalHandle, rettv: TypevalHandle) {
    if check_can_index_impl(argvars, true, false) != OK {
        return;
    }

    tv_copy(argvars, rettv);

    // argvars[1] = argvars + sizeof(typval_T)
    // argvars[2] = argvars + 2*sizeof(typval_T)
    // We use nvim_tv_argvars_get to get individual args
    let var1 = nvim_f_slice_get_arg1(argvars);
    let var2_raw = nvim_f_slice_get_arg2(argvars);
    let var2 = if nvim_tv_get_type(var2_raw) == VAR_UNKNOWN {
        TypevalHandle::null()
    } else {
        var2_raw
    };

    eval_index_inner_impl(rettv, true, var1, var2, true, ptr::null(), 0, false);
}

extern "C" {
    // Helper to access argvars[1] and argvars[2] by index
    fn nvim_f_slice_get_arg1(argvars: TypevalHandle) -> TypevalHandle;
    fn nvim_f_slice_get_arg2(argvars: TypevalHandle) -> TypevalHandle;
}

/// FFI export for f_slice.
///
/// # Safety
/// See `f_slice_impl` for safety requirements.
#[no_mangle]
pub unsafe extern "C" fn f_slice(argvars: TypevalHandle, rettv: TypevalHandle, _fptr: *mut c_void) {
    f_slice_impl(argvars, rettv);
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(OK, 1);
        assert_eq!(FAIL, 0);
        assert_eq!(VARNUMBER_MAX, i64::MAX);
    }
}
