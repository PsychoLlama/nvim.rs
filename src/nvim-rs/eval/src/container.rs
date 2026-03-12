//! List/Dict/Blob container VimL built-in function implementations.
//!
//! Phase 3: f_remove, f_reverse
//! Phase 4: f_extend, f_extendnew, f_insert
//! Phase 5: f_filter, f_map, f_mapnew, f_foreach

#![allow(clippy::cast_sign_loss)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::cast_possible_wrap)]
#![allow(clippy::cast_lossless)]
#![allow(dead_code)]

use std::ffi::{c_char, c_int, c_void, CStr};

// =============================================================================
// Types
// =============================================================================

/// Opaque pointer to typval_T.
type TypvalPtr = *mut c_void;

/// Opaque handle for EvalFuncData union.
type EvalFuncData = *mut c_void;

/// Opaque pointer to list_T.
type ListPtr = *mut c_void;

/// Opaque pointer to blob_T.
type BlobPtr = *mut c_void;

/// Opaque pointer to listitem_T.
type ListItemPtr = *mut c_void;

/// varnumber_T (matches C int64_t).
type VarNumber = i64;

// =============================================================================
// Constants
// =============================================================================

/// VAR_UNKNOWN type constant (v_type == 0 means no value).
const VAR_UNKNOWN: c_int = 0;
const VAR_LIST: c_int = 4;
const VAR_BLOB: c_int = 10;

/// TV_TRANSLATE sentinel for value_check_lock.
const TV_TRANSLATE: usize = usize::MAX;

/// VarLockStatus: VAR_UNLOCKED = 0
const VAR_UNLOCKED: c_int = 0;

// =============================================================================
// C extern declarations
// =============================================================================

extern "C" {
    // typval field accessors (window_shim.c / eval_shim.c)
    fn nvim_eval_tv_idx(argvars: TypvalPtr, i: c_int) -> TypvalPtr;
    fn nvim_eval_tv_get_type(tv: *const c_void) -> c_int;
    fn nvim_eval_tv_get_list(tv: *const c_void) -> ListPtr;
    fn nvim_eval_tv_get_dict(tv: *const c_void) -> *mut c_void;
    fn nvim_tv_get_blob(tv: TypvalPtr) -> BlobPtr;
    fn nvim_tv_get_vstring(tv: TypvalPtr) -> *mut c_char;
    fn nvim_eval_tv_set_type(tv: TypvalPtr, t: c_int);
    fn nvim_eval_tv_set_string(tv: TypvalPtr, s: *mut c_char);

    // list accessors (eval/typval.c, eval_shim.c)
    fn nvim_list_get_lock(l: ListPtr) -> c_int;
    fn nvim_list_get_first(l: *const c_void) -> *const c_void;
    fn nvim_list_item_next(l: ListPtr, item: ListItemPtr) -> ListItemPtr;
    fn nvim_list_item_tv(item: ListItemPtr) -> TypvalPtr;
    fn tv_list_find(l: ListPtr, idx: c_int) -> ListItemPtr;
    fn tv_list_reverse(l: ListPtr);
    fn nvim_eval_tv_list_set_ret(rettv: TypvalPtr, l: ListPtr);

    // blob accessors (eval/typval.c, eval_shim.c)
    fn nvim_blob_len(b: BlobPtr) -> c_int;
    fn nvim_blob_get_byte(b: BlobPtr, idx: c_int) -> u8;
    fn nvim_blob_set_byte(b: BlobPtr, idx: c_int, c: u8);
    fn nvim_blob_set_ret(rettv: TypvalPtr, b: BlobPtr);
    fn nvim_blob_get_bv_lock(b: BlobPtr) -> c_int;

    // typval operations (eval/typval.c)
    fn tv_dict_remove(argvars: TypvalPtr, rettv: TypvalPtr, arg_errmsg: *const c_char);
    fn tv_blob_remove(argvars: TypvalPtr, rettv: TypvalPtr, arg_errmsg: *const c_char);
    fn tv_list_remove(argvars: TypvalPtr, rettv: TypvalPtr, arg_errmsg: *const c_char);
    fn tv_check_for_string_or_list_or_blob_arg(argvars: TypvalPtr, idx: c_int) -> c_int;
    fn value_check_lock(lock: c_int, name: *const c_char, name_len: usize) -> bool;
    fn tv_equal(tv1: TypvalPtr, tv2: TypvalPtr, ic: bool) -> bool;
    fn tv_get_number_chk(tv: TypvalPtr, error: *mut bool) -> VarNumber;
    fn tv_get_string_chk(tv: TypvalPtr) -> *const c_char;
    fn tv_copy(from: TypvalPtr, to: TypvalPtr);
    fn tv_list_append_tv(l: ListPtr, tv: TypvalPtr);

    // messaging
    fn semsg(fmt: *const c_char, ...) -> c_int;
    fn emsg(s: *const c_char) -> c_int;

    // string operations
    fn reverse_text(s: *mut c_char) -> *mut c_char;
    fn mb_strnicmp(s1: *const c_char, s2: *const c_char, n: usize) -> c_int;
    fn utfc_ptr2len(p: *const c_char) -> c_int;

    // extend/copy/insert operations (eval/typval.c)
    fn tv_dict_copy(
        conv: *const c_void,
        orig: *mut c_void,
        deep: bool,
        copyid: c_int,
    ) -> *mut c_void;
    fn tv_dict_extend(d1: *mut c_void, d2: *mut c_void, action: *const c_char);
    fn tv_dict_unref(d: *mut c_void);
    fn tv_list_copy(conv: *const c_void, orig: ListPtr, deep: bool, copyid: c_int) -> ListPtr;
    fn tv_list_extend(l1: ListPtr, l2: ListPtr, bef: ListItemPtr);
    fn tv_list_insert_tv(l: ListPtr, tv: TypvalPtr, item: ListItemPtr);
    fn nvim_list_get_len(l: *const c_void) -> c_int;
    fn nvim_dict_get_lock(d: *const c_void) -> c_int;
    fn nvim_tv_set_dict(tv: TypvalPtr, d: *mut c_void);

    // error strings (errors.h EXTERN const char arrays)
    static e_listdictblobarg: [c_char; 0];
    static e_listblobreq: [c_char; 0];
    static e_invarg2: [c_char; 0];
    static e_listdictarg: [c_char; 0];
    static e_list_index_out_of_range_nr: [c_char; 0];
}

// =============================================================================
// Helper
// =============================================================================

/// Get `&argvars[i]` as a raw typval pointer.
///
/// # Safety
/// `argvars` must be a valid typval_T array pointer.
unsafe fn argvar_at(argvars: TypvalPtr, i: c_int) -> TypvalPtr {
    unsafe { nvim_eval_tv_idx(argvars, i) }
}

// =============================================================================
// Phase 3 implementations
// =============================================================================

/// "remove({list}, {idx})" / "remove({dict}, {key})" / "remove({blob}, {idx})" function.
///
/// Dispatches to the appropriate C implementation based on the first argument type.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_remove"]
pub unsafe extern "C" fn rs_f_remove(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let tv0 = argvar_at(argvars, 0);
        let arg_errmsg = c"remove() argument".as_ptr();

        match nvim_eval_tv_get_type(tv0) {
            5 => {
                // VAR_DICT
                tv_dict_remove(argvars, rettv, arg_errmsg);
            }
            VAR_BLOB => {
                tv_blob_remove(argvars, rettv, arg_errmsg);
            }
            VAR_LIST => {
                tv_list_remove(argvars, rettv, arg_errmsg);
            }
            _ => {
                semsg(e_listdictblobarg.as_ptr(), c"remove()".as_ptr());
            }
        }
    }
}

/// "reverse({list})" / "reverse({blob})" / "reverse({string})" function.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_reverse"]
pub unsafe extern "C" fn rs_f_reverse(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        // FAIL = 0, OK = 1; return on FAIL
        if tv_check_for_string_or_list_or_blob_arg(argvars, 0) == 0 {
            return;
        }

        let tv0 = argvar_at(argvars, 0);

        match nvim_eval_tv_get_type(tv0) {
            VAR_BLOB => {
                let b = nvim_tv_get_blob(tv0);
                let len = nvim_blob_len(b);
                let mut i = 0;
                while i < len / 2 {
                    let tmp = nvim_blob_get_byte(b, i);
                    nvim_blob_set_byte(b, i, nvim_blob_get_byte(b, len - i - 1));
                    nvim_blob_set_byte(b, len - i - 1, tmp);
                    i += 1;
                }
                nvim_blob_set_ret(rettv, b);
            }
            2 => {
                // VAR_STRING
                let s = nvim_tv_get_vstring(tv0);
                // VAR_STRING = 2
                nvim_eval_tv_set_type(rettv, 2);
                if s.is_null() {
                    nvim_eval_tv_set_string(rettv, std::ptr::null_mut());
                } else {
                    nvim_eval_tv_set_string(rettv, reverse_text(s));
                }
            }
            VAR_LIST => {
                let l = nvim_eval_tv_get_list(tv0);
                let lock = nvim_list_get_lock(l);
                if !value_check_lock(lock, c"reverse() argument".as_ptr(), TV_TRANSLATE) {
                    tv_list_reverse(l);
                    nvim_eval_tv_list_set_ret(rettv, l);
                }
            }
            _ => {
                // tv_check_for_string_or_list_or_blob_arg should have caught this
            }
        }
    }
}

// =============================================================================
// Phase 4 implementations
// =============================================================================

const VAR_DICT: c_int = 5;

/// Extend a dict: append dict `argvars[1]` to `argvars[0]`.
/// For extendnew(), copies d1 first; otherwise operates in place.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
unsafe fn extend_dict_impl(
    argvars: TypvalPtr,
    arg_errmsg: *const c_char,
    is_new: bool,
    rettv: TypvalPtr,
) {
    unsafe {
        let tv0 = argvar_at(argvars, 0);
        let tv1 = argvar_at(argvars, 1);
        let mut d1 = nvim_eval_tv_get_dict(tv0);
        if d1.is_null() {
            // Locked (fixed) dict -- value_check_lock will emit the error
            value_check_lock(2 /*VAR_FIXED*/, arg_errmsg, TV_TRANSLATE); // always true
            return;
        }
        let d2 = nvim_eval_tv_get_dict(tv1);
        if d2.is_null() {
            // d2 is NULL: no-op, return copy of d1
            tv_copy(tv0, rettv);
            return;
        }

        if !is_new && value_check_lock(nvim_dict_get_lock(d1), arg_errmsg, TV_TRANSLATE) {
            return;
        }

        if is_new {
            d1 = tv_dict_copy(std::ptr::null(), d1, false, crate::rs_get_copyID());
            if d1.is_null() {
                return;
            }
        }

        // Determine action (default "force")
        let tv2 = argvar_at(argvars, 2);
        let action: *const c_char = if nvim_eval_tv_get_type(tv2) == VAR_UNKNOWN {
            c"force".as_ptr()
        } else {
            let s = tv_get_string_chk(tv2);
            if s.is_null() {
                // Type error; error message already given
                if is_new {
                    tv_dict_unref(d1);
                }
                return;
            }
            // Validate action is "keep", "force", or "error"
            let action_str = CStr::from_ptr(s).to_bytes();
            let valid = action_str == b"keep" || action_str == b"force" || action_str == b"error";
            if !valid {
                if is_new {
                    tv_dict_unref(d1);
                }
                semsg(e_invarg2.as_ptr(), s);
                return;
            }
            s
        };

        tv_dict_extend(d1, d2, action);

        if is_new {
            // Set rettv to the new dict
            nvim_eval_tv_set_type(rettv, VAR_DICT);
            nvim_tv_set_dict(rettv, d1);
        } else {
            tv_copy(tv0, rettv);
        }
    }
}

/// Extend a list: append list `argvars[1]` to `argvars[0]` before optional index.
/// For extendnew(), copies l1 first; otherwise operates in place.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
unsafe fn extend_list_impl(
    argvars: TypvalPtr,
    arg_errmsg: *const c_char,
    is_new: bool,
    rettv: TypvalPtr,
) {
    unsafe {
        let tv0 = argvar_at(argvars, 0);
        let tv1 = argvar_at(argvars, 1);
        let mut l1 = nvim_eval_tv_get_list(tv0);
        let l2 = nvim_eval_tv_get_list(tv1);

        if !is_new && value_check_lock(nvim_list_get_lock(l1), arg_errmsg, TV_TRANSLATE) {
            return;
        }

        if is_new {
            l1 = tv_list_copy(std::ptr::null(), l1, false, crate::rs_get_copyID());
            if l1.is_null() {
                return;
            }
        }

        let tv2 = argvar_at(argvars, 2);
        let item: ListItemPtr = if nvim_eval_tv_get_type(tv2) == VAR_UNKNOWN {
            std::ptr::null_mut()
        } else {
            let mut error = false;
            let before = tv_get_number_chk(tv2, &raw mut error) as c_int;
            if error {
                return; // Type error; errmsg already given
            }
            if before == nvim_list_get_len(l1) {
                std::ptr::null_mut()
            } else {
                let item = tv_list_find(l1, before);
                if item.is_null() {
                    semsg(e_list_index_out_of_range_nr.as_ptr(), before as i64);
                    return;
                }
                item
            }
        };
        tv_list_extend(l1, l2, item);

        if is_new {
            nvim_eval_tv_list_set_ret(rettv, l1);
        } else {
            tv_copy(tv0, rettv);
        }
    }
}

/// Common implementation of extend() and extendnew().
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
unsafe fn extend_impl(
    argvars: TypvalPtr,
    rettv: TypvalPtr,
    arg_errmsg: *const c_char,
    is_new: bool,
) {
    unsafe {
        let tv0 = argvar_at(argvars, 0);
        let tv1 = argvar_at(argvars, 1);
        let t0 = nvim_eval_tv_get_type(tv0);
        let t1 = nvim_eval_tv_get_type(tv1);
        if t0 == VAR_LIST && t1 == VAR_LIST {
            extend_list_impl(argvars, arg_errmsg, is_new, rettv);
        } else if t0 == VAR_DICT && t1 == VAR_DICT {
            extend_dict_impl(argvars, arg_errmsg, is_new, rettv);
        } else {
            semsg(
                e_listdictarg.as_ptr(),
                if is_new {
                    c"extendnew()".as_ptr()
                } else {
                    c"extend()".as_ptr()
                },
            );
        }
    }
}

/// "extend(list, list [, idx])" / "extend(dict, dict [, action])" function.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_extend"]
pub unsafe extern "C" fn rs_f_extend(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        extend_impl(argvars, rettv, c"extend() argument".as_ptr(), false);
    }
}

/// "extendnew(list, list [, idx])" / "extendnew(dict, dict [, action])" function.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_extendnew"]
pub unsafe extern "C" fn rs_f_extendnew(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        extend_impl(argvars, rettv, c"extendnew() argument".as_ptr(), true);
    }
}
