//! `:for` loop implementation -- eval_for_line, next_for_item, free_for_info.
//!
//! Migrated from `eval_shim.c` Phase 3.
//!
//! The `ForInfo` struct is defined here in Rust with `#[repr(C)]` matching
//! the layout of the old `forinfo_T` C typedef (deleted in Phase 2 pass 10).

#![allow(clippy::too_many_lines)]
#![allow(unsafe_op_in_unsafe_fn)]

use std::ffi::{c_char, c_int, c_void};
use std::ptr;

use crate::eval::{eval0_impl, EvalargHandle, ExargHandle, TypevalHandle};

// =============================================================================
// Constants (matching C)
// =============================================================================

const OK: c_int = 1;
#[allow(dead_code)]
const FAIL: c_int = 0;
const EVAL_EVALUATE: c_int = 1;

const VAR_STRING: c_int = 2;
const VAR_LIST: c_int = 4;
const VAR_BLOB: c_int = 10;

// =============================================================================
// Struct definitions mirroring C layout
// =============================================================================

/// Mirrors C `listwatch_T` (struct listwatch_S) from eval/typval_defs.h.
/// Two pointers: lw_item (listitem_T*) and lw_next (listwatch_T*).
#[repr(C)]
struct ListWatch {
    lw_item: *mut c_void, // listitem_T*
    lw_next: *mut c_void, // listwatch_T* (next watcher in linked list)
}

/// Mirrors the old `forinfo_T` typedef from eval_shim.c (now deleted).
/// Rust now owns this struct -- no more opaque void* passing through accessors.
///
/// Layout (verified to match C forinfo_T):
///   offset  0: fi_semicolon (int)
///   offset  4: fi_varcount (int)
///   offset  8: fi_lw (ListWatch, 16 bytes: two pointers)
///   offset 24: fi_list (*mut c_void)
///   offset 32: fi_bi (int)
///   [4 bytes padding]
///   offset 40: fi_blob (*mut c_void)
///   offset 48: fi_string (*mut c_char)
///   offset 56: fi_byte_idx (int)
///   [4 bytes trailing padding]
///   sizeof = 64
#[repr(C)]
struct ForInfo {
    fi_semicolon: c_int,
    fi_varcount: c_int,
    fi_lw: ListWatch,
    fi_list: *mut c_void, // list_T*
    fi_bi: c_int,
    // 4 bytes of C alignment padding inserted here by repr(C)
    fi_blob: *mut c_void, // blob_T*
    fi_string: *mut c_char,
    fi_byte_idx: c_int,
    // 4 bytes of trailing padding to bring sizeof to 64
}

// =============================================================================
// C Extern Functions
// =============================================================================

extern "C" {
    // skip_var_list / ex_let_vars (direct C calls, not through nvim_ wrappers)
    fn skip_var_list(
        arg: *const c_char,
        var_count: *mut c_int,
        semicolon: *mut c_int,
        silent: bool,
    ) -> *const c_char;

    // ex_let_vars wrappers (kept as C thin wrappers -- construct typval_T in C)
    fn nvim_ex_let_vars_number(
        arg: *mut c_char,
        n: i64,
        copy: bool,
        semicolon: c_int,
        varcount: c_int,
    ) -> bool;
    fn nvim_ex_let_vars_string_owned(
        arg: *mut c_char,
        s: *mut c_char,
        semicolon: c_int,
        varcount: c_int,
    ) -> bool;
    fn nvim_ex_let_vars_list_item(
        arg: *mut c_char,
        item: *mut c_void,
        semicolon: c_int,
        varcount: c_int,
    ) -> bool;

    // tv_list_watch_add/remove (called directly with &fi.fi_lw)
    fn tv_list_watch_add(l: *mut c_void, lw: *mut ListWatch);
    fn tv_list_watch_remove(l: *mut c_void, lw: *mut ListWatch);

    // list/blob operations
    fn nvim_list_item_next(l: *mut c_void, item: *mut c_void) -> *mut c_void;
    fn nvim_tv_list_first(l: *mut c_void) -> *mut c_void;
    fn nvim_tv_list_unref(l: *mut c_void);
    fn nvim_tv_blob_unref(b: *mut c_void);
    fn nvim_tv_blob_copy(from: *mut c_void, to: TypevalHandle);

    // TV type / value accessors
    fn nvim_tv_get_type(tv: TypevalHandle) -> c_int;
    fn nvim_tv_get_list(tv: TypevalHandle) -> *mut c_void;
    fn nvim_tv_get_blob(tv: TypevalHandle) -> *mut c_void;
    fn nvim_tv_get_vstring(tv: TypevalHandle) -> *mut c_char;
    fn nvim_tv_set_vstring_owned(tv: TypevalHandle, s: *mut c_char);
    fn tv_clear(tv: TypevalHandle);
    fn nvim_blob_len(b: *const c_void) -> c_int;
    fn nvim_blob_get(b: *const c_void, idx: c_int) -> c_int;

    // evalarg_get_flags: deleted -- use EvalargHandle::flags() directly (Phase 14)
    fn skipwhite(p: *const c_char) -> *mut c_char;
    fn emsg(s: *const c_char) -> c_int;

    // xmemdupz / xstrdup / xfree / xmalloc / xcalloc
    fn xmemdupz(src: *const c_void, len: usize) -> *mut c_char;
    fn xstrdup(s: *const c_char) -> *mut c_char;
    fn xfree(ptr: *mut c_void);
    fn xmalloc(size: usize) -> *mut c_void;

    // utfc_ptr2len
    fn utfc_ptr2len(p: *const c_char) -> c_int;

    // Phase 12: emsg_skip accessed directly as a global
    static mut emsg_skip: c_int;

    fn rs_ascii_iswhite(c: c_int) -> c_int;
}

// Error messages
static E_MISSING_IN: &[u8] = b"E690: Missing \"in\" after :for\0";
static E_STRING_LIST_OR_BLOB: &[u8] = b"E714: List, String, Tuple or Blob required\0";

/// Allocate a temporary typval on heap (64 bytes, zeroed).
unsafe fn alloc_typval() -> TypevalHandle {
    let ptr = xmalloc(64);
    ptr::write_bytes(ptr as *mut u8, 0, 64);
    TypevalHandle::from_ptr(ptr)
}

/// Free a heap-allocated typval.
unsafe fn free_typval(tv: TypevalHandle) {
    if !tv.is_null() {
        xfree(tv.as_ptr());
    }
}

/// Get a byte at a pointer (0 if null).
#[inline]
unsafe fn get_byte(p: *const c_char) -> u8 {
    if p.is_null() {
        0
    } else {
        *p as u8
    }
}

/// Allocate a zeroed ForInfo struct on the heap and return as *mut ForInfo.
unsafe fn alloc_forinfo() -> *mut ForInfo {
    let ptr = xmalloc(std::mem::size_of::<ForInfo>()) as *mut ForInfo;
    ptr::write_bytes(ptr as *mut u8, 0, std::mem::size_of::<ForInfo>());
    ptr
}

// =============================================================================
// eval_for_line
// =============================================================================

/// Evaluate the expression in `:for var in expr`, set up iteration state.
///
/// Returns an opaque `forinfo_T *` (as `*mut c_void`) to be passed to
/// `rs_next_for_item` and `rs_free_for_info`. Never returns null.
///
/// # Safety
/// - `arg` must be a valid C string
/// - `errp` must be a valid pointer
/// - `evalarg` must be non-null (contains eval_flags)
pub unsafe fn eval_for_line_impl(
    arg: *const c_char,
    errp: *mut bool,
    eap: ExargHandle,
    evalarg: EvalargHandle,
) -> *mut c_void {
    let fi = alloc_forinfo();
    let skip = (evalarg.flags() & EVAL_EVALUATE) == 0;

    // Default: there is an error.
    *errp = true;

    let mut varcount: c_int = 0;
    let mut semicolon: c_int = 0;
    let expr = skip_var_list(arg, &mut varcount, &mut semicolon, false);
    (*fi).fi_varcount = varcount;
    (*fi).fi_semicolon = semicolon;

    if expr.is_null() {
        return fi as *mut c_void;
    }

    let expr = skipwhite(expr);

    // Check for "in" keyword: must be "in" followed by NUL or whitespace
    let b0 = get_byte(expr);
    let b1 = get_byte(expr.add(1));
    let b2 = get_byte(expr.add(2));
    if b0 != b'i' || b1 != b'n' || !(b2 == 0 || rs_ascii_iswhite(c_int::from(b2)) != 0) {
        emsg(E_MISSING_IN.as_ptr() as *const c_char);
        return fi as *mut c_void;
    }

    if skip {
        emsg_skip += 1;
    }

    let expr_after_in = skipwhite(expr.add(2));
    let tv = alloc_typval();
    if eval0_impl(expr_after_in as *mut c_char, tv, eap, evalarg) == OK {
        *errp = false;
        if !skip {
            let tv_type = nvim_tv_get_type(tv);
            if tv_type == VAR_LIST {
                let l = nvim_tv_get_list(tv);
                if l.is_null() {
                    // null list is like empty list: do nothing
                    tv_clear(tv);
                } else {
                    // No need to increment refcount, already set for list in tv
                    (*fi).fi_list = l;
                    tv_list_watch_add(l, &mut (*fi).fi_lw);
                    let first = nvim_tv_list_first(l);
                    (*fi).fi_lw.lw_item = first;
                    // Don't call tv_clear: the list is now owned by fi
                }
            } else if tv_type == VAR_BLOB {
                (*fi).fi_bi = 0;
                let b = nvim_tv_get_blob(tv);
                if !b.is_null() {
                    // Make a copy so iteration still works if blob is changed
                    let btv = alloc_typval();
                    nvim_tv_blob_copy(b, btv);
                    let blob_copy = nvim_tv_get_blob(btv);
                    (*fi).fi_blob = blob_copy;
                    free_typval(btv);
                }
                tv_clear(tv);
            } else if tv_type == VAR_STRING {
                (*fi).fi_byte_idx = 0;
                let s = nvim_tv_get_vstring(tv);
                if s.is_null() {
                    let empty = xstrdup(c"".as_ptr());
                    (*fi).fi_string = empty;
                } else {
                    // Take ownership of the string from tv
                    (*fi).fi_string = s;
                    nvim_tv_set_vstring_owned(tv, ptr::null_mut());
                }
                tv_clear(tv);
            } else {
                emsg(E_STRING_LIST_OR_BLOB.as_ptr() as *const c_char);
                tv_clear(tv);
            }
        } else {
            tv_clear(tv);
        }
    } else {
        tv_clear(tv);
    }

    free_typval(tv);

    if skip {
        emsg_skip -= 1;
    }

    fi as *mut c_void
}

/// FFI export for eval_for_line.
///
/// # Safety
/// See `eval_for_line_impl` for safety requirements.
#[export_name = "eval_for_line"]
pub unsafe extern "C" fn rs_eval_for_line(
    arg: *const c_char,
    errp: *mut bool,
    eap: ExargHandle,
    evalarg: EvalargHandle,
) -> *mut c_void {
    eval_for_line_impl(arg, errp, eap, evalarg)
}

// =============================================================================
// next_for_item
// =============================================================================

/// Advance to the next item in a `:for` loop.
///
/// Returns `true` when a valid item was found, `false` when at end or error.
///
/// # Safety
/// - `fi_void` must be a valid `ForInfo *` returned by `rs_eval_for_line`
/// - `arg` must be a valid mutable C string (variable name(s))
pub unsafe fn next_for_item_impl(fi_void: *mut c_void, arg: *mut c_char) -> bool {
    let fi = fi_void as *mut ForInfo;
    let semicolon = (*fi).fi_semicolon;
    let varcount = (*fi).fi_varcount;

    if !(*fi).fi_blob.is_null() {
        let blob = (*fi).fi_blob;
        let bi = (*fi).fi_bi;
        if bi >= nvim_blob_len(blob as *const c_void) {
            return false;
        }
        let byte_val = i64::from(nvim_blob_get(blob as *const c_void, bi));
        (*fi).fi_bi = bi + 1;
        return nvim_ex_let_vars_number(arg, byte_val, true, semicolon, varcount);
    }

    if !(*fi).fi_string.is_null() {
        let s = (*fi).fi_string;
        let byte_idx = (*fi).fi_byte_idx;
        let len = utfc_ptr2len(s.add(byte_idx as usize));
        if len == 0 {
            return false;
        }
        let dup = xmemdupz(s.add(byte_idx as usize) as *const c_void, len as usize);
        (*fi).fi_byte_idx = byte_idx + len;
        return nvim_ex_let_vars_string_owned(arg, dup, semicolon, varcount);
    }

    // List iteration
    let item = (*fi).fi_lw.lw_item;
    if item.is_null() {
        return false;
    }
    let list = (*fi).fi_list;
    let next = nvim_list_item_next(list, item);
    (*fi).fi_lw.lw_item = next;
    nvim_ex_let_vars_list_item(arg, item, semicolon, varcount)
}

/// FFI export for next_for_item.
///
/// # Safety
/// See `next_for_item_impl` for safety requirements.
#[export_name = "next_for_item"]
pub unsafe extern "C" fn rs_next_for_item(fi_void: *mut c_void, arg: *mut c_char) -> bool {
    next_for_item_impl(fi_void, arg)
}

// =============================================================================
// free_for_info
// =============================================================================

/// Free the structure used to store info used by `:for`.
///
/// # Safety
/// - `fi_void` must be a valid `ForInfo *` or null
pub unsafe fn free_for_info_impl(fi_void: *mut c_void) {
    if fi_void.is_null() {
        return;
    }
    let fi = fi_void as *mut ForInfo;
    if !(*fi).fi_list.is_null() {
        tv_list_watch_remove((*fi).fi_list, &mut (*fi).fi_lw);
        nvim_tv_list_unref((*fi).fi_list);
    } else if !(*fi).fi_blob.is_null() {
        nvim_tv_blob_unref((*fi).fi_blob);
    } else {
        let s = (*fi).fi_string;
        if !s.is_null() {
            xfree(s as *mut c_void);
        }
    }
    xfree(fi_void);
}

/// FFI export for free_for_info.
///
/// # Safety
/// See `free_for_info_impl` for safety requirements.
#[export_name = "free_for_info"]
pub unsafe extern "C" fn rs_free_for_info(fi_void: *mut c_void) {
    free_for_info_impl(fi_void);
}
