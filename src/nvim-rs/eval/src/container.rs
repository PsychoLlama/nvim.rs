//! List/Dict/Blob container VimL built-in function implementations.
//!
//! Phase 3: f_remove, f_reverse
//! Phase 4: f_extend, f_extendnew
//! Phase 5: f_add, f_insert, f_count (count_string, count_list, count_dict)

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
// Structs (matching C layout)
// =============================================================================

/// garray_T -- must match C layout exactly.
#[repr(C)]
#[allow(clippy::struct_field_names)]
struct GArray {
    ga_len: c_int,
    ga_maxlen: c_int,
    ga_itemsize: c_int,
    ga_growsize: c_int,
    ga_data: *mut c_void,
}

// =============================================================================
// C extern declarations
// =============================================================================

use super::typval::TypvalT as TypvalTRepr;

#[inline]
unsafe fn tv_get_list(tv: *const c_void) -> ListPtr {
    (*tv.cast::<TypvalTRepr>()).vval.v_list
}

#[inline]
unsafe fn tv_get_dict_ptr(tv: *const c_void) -> *mut c_void {
    (*tv.cast::<TypvalTRepr>()).vval.v_dict
}

#[inline]
unsafe fn tv_set_dict_ptr(tv: TypvalPtr, d: *mut c_void) {
    (*tv.cast::<TypvalTRepr>()).vval.v_dict = d;
}

extern "C" {
    // typval field accessors (window_shim.c / eval_shim.c)
    fn nvim_eval_tv_idx(argvars: TypvalPtr, i: c_int) -> TypvalPtr;
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

    // error strings (errors.h EXTERN const char arrays)
    static e_listdictblobarg: [c_char; 0];
    static e_listblobreq: [c_char; 0];
    static e_invarg2: [c_char; 0];
    static e_listdictarg: [c_char; 0];
    static e_list_index_out_of_range_nr: [c_char; 0];
    static e_invarg: [c_char; 0];
    static e_listblobarg: [c_char; 0];
    static e_invalblob: [c_char; 0];
    static e_string_required: [c_char; 0];
}

extern "C" {
    // Phase 5: f_add, f_insert, f_count helpers
    fn blob_get_ga(b: BlobPtr) -> *mut GArray;
    fn ga_grow(ga: *mut GArray, n: c_int);
    fn ga_append(ga: *mut GArray, c: u8);
    fn ga_init(ga: *mut GArray, itemsize: c_int, growsize: c_int);
    fn ga_concat(ga: *mut GArray, s: *const c_char);
    fn tv_get_string(tv: TypvalPtr) -> *const c_char;
    fn strstr(haystack: *const c_char, needle: *const c_char) -> *const c_char;
    fn memmove(dest: *mut c_void, src: *const c_void, n: usize) -> *mut c_void;
    fn strlen(s: *const c_char) -> usize;

    // Dict iteration (tag_shim.c): hashitem_T iterator over dict_T.
    fn nvim_tag_dict_iter_start(d: *const c_void) -> *mut c_void;
    fn nvim_tag_dict_iter_next(d: *const c_void, hi: *const c_void) -> *mut c_void;

    // dictitem_T helpers (eval_shim.c, eval/vars.c)
    fn nvim_hi2dictitem(hi: *mut c_void) -> *mut c_void;
    fn nvim_di_get_tv(di: *mut c_void) -> TypvalPtr;

    // typval number setter (eval/typval.c)
    fn nvim_tv_set_number(tv: TypvalPtr, n: VarNumber);
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

        match (*tv0.cast::<TypvalTRepr>()).v_type {
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

        match (*tv0.cast::<TypvalTRepr>()).v_type {
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
                let l = tv_get_list(tv0);
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
        let mut d1 = tv_get_dict_ptr(tv0);
        if d1.is_null() {
            // Locked (fixed) dict -- value_check_lock will emit the error
            value_check_lock(2 /*VAR_FIXED*/, arg_errmsg, TV_TRANSLATE); // always true
            return;
        }
        let d2 = tv_get_dict_ptr(tv1);
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
        let action: *const c_char = if (*tv2.cast::<TypvalTRepr>()).v_type == VAR_UNKNOWN {
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
            tv_set_dict_ptr(rettv, d1);
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
        let mut l1 = tv_get_list(tv0);
        let l2 = tv_get_list(tv1);

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
        let item: ListItemPtr = if (*tv2.cast::<TypvalTRepr>()).v_type == VAR_UNKNOWN {
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
        let t0 = (*tv0.cast::<TypvalTRepr>()).v_type;
        let t1 = (*tv1.cast::<TypvalTRepr>()).v_type;
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

// =============================================================================
// Phase 5 implementations
// =============================================================================

/// "add(list, item)" / "add(blob, nr)" function.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_add"]
pub unsafe extern "C" fn rs_f_add(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        // Default return value: 1 (failed)
        nvim_tv_set_number(rettv, 1);

        let tv0 = argvar_at(argvars, 0);
        let t0 = (*tv0.cast::<TypvalTRepr>()).v_type;

        if t0 == VAR_LIST {
            let l = tv_get_list(tv0);
            if !value_check_lock(
                nvim_list_get_lock(l),
                c"add() argument".as_ptr(),
                TV_TRANSLATE,
            ) {
                let tv1 = argvar_at(argvars, 1);
                tv_list_append_tv(l, tv1);
                tv_copy(tv0, rettv);
            }
        } else if t0 == VAR_BLOB {
            let b = nvim_tv_get_blob(tv0);
            if !b.is_null()
                && !value_check_lock(
                    nvim_blob_get_bv_lock(b),
                    c"add() argument".as_ptr(),
                    TV_TRANSLATE,
                )
            {
                let tv1 = argvar_at(argvars, 1);
                let mut error = false;
                let n = tv_get_number_chk(tv1, &raw mut error);
                if !error {
                    ga_append(blob_get_ga(b), n as u8);
                    tv_copy(tv0, rettv);
                }
            }
        } else {
            emsg(e_listblobreq.as_ptr());
        }
    }
}

/// "insert(list, item [, idx])" / "insert(blob, nr [, idx])" function.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_insert"]
pub unsafe extern "C" fn rs_f_insert(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let tv0 = argvar_at(argvars, 0);
        let t0 = (*tv0.cast::<TypvalTRepr>()).v_type;
        let tv1 = argvar_at(argvars, 1);
        let tv2 = argvar_at(argvars, 2);

        if t0 == VAR_BLOB {
            let b = nvim_tv_get_blob(tv0);
            if b.is_null()
                || value_check_lock(
                    nvim_blob_get_bv_lock(b),
                    c"insert() argument".as_ptr(),
                    TV_TRANSLATE,
                )
            {
                return;
            }

            let mut before: c_int = 0;
            let len = nvim_blob_len(b);

            if (*tv2.cast::<TypvalTRepr>()).v_type != VAR_UNKNOWN {
                let mut error = false;
                before = tv_get_number_chk(tv2, &raw mut error) as c_int;
                if error {
                    return;
                }
                if before < 0 || before > len {
                    semsg(e_invarg2.as_ptr(), tv_get_string(tv2));
                    return;
                }
            }

            let mut error = false;
            let val = tv_get_number_chk(tv1, &raw mut error) as c_int;
            if error {
                return;
            }
            if !(0..=255).contains(&val) {
                semsg(e_invarg2.as_ptr(), tv_get_string(tv1));
                return;
            }

            let ga = blob_get_ga(b);
            ga_grow(ga, 1);
            let p = (*ga).ga_data.cast::<u8>();
            memmove(
                p.add(before as usize + 1).cast::<c_void>(),
                p.add(before as usize).cast_const().cast::<c_void>(),
                (len - before) as usize,
            );
            *p.add(before as usize) = val as u8;
            (*ga).ga_len += 1;

            tv_copy(tv0, rettv);
        } else if t0 != VAR_LIST {
            semsg(e_listblobarg.as_ptr(), c"insert()".as_ptr());
        } else {
            let l = tv_get_list(tv0);
            if value_check_lock(
                nvim_list_get_lock(l),
                c"insert() argument".as_ptr(),
                TV_TRANSLATE,
            ) {
                return;
            }

            let mut before: VarNumber = 0;
            if (*tv2.cast::<TypvalTRepr>()).v_type != VAR_UNKNOWN {
                let mut error = false;
                before = tv_get_number_chk(tv2, &raw mut error);
                if error {
                    return;
                }
            }

            let item: ListItemPtr = if before == nvim_list_get_len(l) as VarNumber {
                std::ptr::null_mut()
            } else {
                let item = tv_list_find(l, before as c_int);
                if item.is_null() {
                    semsg(e_list_index_out_of_range_nr.as_ptr(), before);
                    return;
                }
                item
            };

            tv_list_insert_tv(l, tv1, item);
            tv_copy(tv0, rettv);
        }
    }
}

/// Count occurrences of `needle` in string `haystack`.
///
/// # Safety
/// Pointers must be valid null-terminated C strings or NULL.
unsafe fn count_string_impl(haystack: *const c_char, needle: *const c_char, ic: bool) -> VarNumber {
    unsafe {
        if haystack.is_null() || needle.is_null() || *needle == 0 {
            return 0;
        }
        let needlelen = strlen(needle);
        let mut n: VarNumber = 0;
        let mut p = haystack;
        if ic {
            while *p != 0 {
                if mb_strnicmp(p, needle, needlelen) == 0 {
                    n += 1;
                    p = p.add(needlelen);
                } else {
                    // MB_PTR_ADV: advance by one multibyte char
                    p = p.add(utfc_ptr2len(p) as usize);
                }
            }
        } else {
            loop {
                let next = strstr(p, needle);
                if next.is_null() {
                    break;
                }
                n += 1;
                p = next.add(needlelen);
            }
        }
        n
    }
}

/// Count occurrences of `needle` in list `l` starting at index `idx`.
///
/// # Safety
/// `l` and `needle` must be valid pointers.
unsafe fn count_list_impl(l: ListPtr, needle: TypvalPtr, idx: VarNumber, ic: bool) -> VarNumber {
    unsafe {
        if nvim_list_get_len(l) == 0 {
            return 0;
        }
        let li = tv_list_find(l, idx as c_int);
        if li.is_null() {
            semsg(e_list_index_out_of_range_nr.as_ptr(), idx);
            return 0;
        }
        let mut n: VarNumber = 0;
        let mut cur = li;
        while !cur.is_null() {
            let item_tv = nvim_list_item_tv(cur);
            if tv_equal(item_tv, needle, ic) {
                n += 1;
            }
            cur = nvim_list_item_next(l, cur);
        }
        n
    }
}

/// Count occurrences of `needle` in dict `d`.
///
/// # Safety
/// `d` and `needle` must be valid pointers (or `d` may be NULL).
unsafe fn count_dict_impl(d: *mut c_void, needle: TypvalPtr, ic: bool) -> VarNumber {
    unsafe {
        if d.is_null() {
            return 0;
        }
        let mut n: VarNumber = 0;
        let mut hi = nvim_tag_dict_iter_start(d);
        while !hi.is_null() {
            let di = nvim_hi2dictitem(hi);
            if !di.is_null() {
                let item_tv = nvim_di_get_tv(di);
                if tv_equal(item_tv, needle, ic) {
                    n += 1;
                }
            }
            hi = nvim_tag_dict_iter_next(d, hi);
        }
        n
    }
}

/// "count()" function -- count occurrences of item in list/string/dict.
///
/// # Safety
/// `argvars` and `rettv` must be valid typval pointers.
#[export_name = "f_count"]
pub unsafe extern "C" fn rs_f_count(argvars: TypvalPtr, rettv: TypvalPtr, _fptr: EvalFuncData) {
    unsafe {
        let mut n: VarNumber = 0;
        let mut ic: c_int = 0;
        let mut error = false;

        let tv0 = argvar_at(argvars, 0);
        let tv1 = argvar_at(argvars, 1);
        let tv2 = argvar_at(argvars, 2);
        let tv3 = argvar_at(argvars, 3);

        if (*tv2.cast::<TypvalTRepr>()).v_type != VAR_UNKNOWN {
            ic = tv_get_number_chk(tv2, &raw mut error) as c_int;
        }

        let t0 = (*tv0.cast::<TypvalTRepr>()).v_type;

        if !error && t0 == 2 {
            // VAR_STRING = 2
            let haystack = nvim_tv_get_vstring(tv0);
            let needle = tv_get_string_chk(tv1);
            n = count_string_impl(haystack.cast_const(), needle, ic != 0);
        } else if !error && t0 == VAR_LIST {
            let idx: VarNumber = if (*tv2.cast::<TypvalTRepr>()).v_type != VAR_UNKNOWN
                && (*tv3.cast::<TypvalTRepr>()).v_type != VAR_UNKNOWN
            {
                tv_get_number_chk(tv3, &raw mut error)
            } else {
                0
            };
            if !error {
                let l = tv_get_list(tv0);
                n = count_list_impl(l, tv1, idx, ic != 0);
            }
        } else if !error && t0 == VAR_DICT {
            let d = tv_get_dict_ptr(tv0);
            if !d.is_null() {
                if (*tv2.cast::<TypvalTRepr>()).v_type != VAR_UNKNOWN
                    && (*tv3.cast::<TypvalTRepr>()).v_type != VAR_UNKNOWN
                {
                    emsg(e_invarg.as_ptr());
                } else {
                    n = count_dict_impl(d, tv1, ic != 0);
                }
            }
        } else if !error {
            semsg(
                c"E706: Argument of %s must be a List, String or Dictionary".as_ptr(),
                c"count()".as_ptr(),
            );
        }

        // Set rettv->vval.v_number = n
        nvim_tv_set_number(rettv, n);
    }
}
