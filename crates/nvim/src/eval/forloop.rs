//! `:for`: the list of things to iterate, and the step from one to the next.
//!
//! `forinfo_T` holds exactly one of three iterations and which field is
//! set is what says which: `fi_blob` a Blob by byte, `fi_string` a String
//! by character, `fi_list` (through `fi_lw`) a List by item. They are
//! tested in that order, so `free_for_info` and `next_for_item` agree
//! without anything recording a kind.
//!
//! Only the List form is *live*: the loop holds a watcher on it and sees
//! items added or removed while it runs. The Blob and the String are
//! copied up front, so changing the original mid-loop has no effect.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};
use core::mem::size_of;
use core::ptr::null_mut;

use crate::ascii::ascii_iswhite;
use crate::charset::skipwhite;
use crate::eval::typval::{
    tv_blob_copy, tv_blob_get, tv_blob_len, tv_blob_unref, tv_clear, tv_list_first, tv_list_unref,
    tv_list_watch_add, tv_list_watch_remove,
};
use crate::eval::vars::{ex_let_vars, skip_var_list};
use crate::eval::{EVAL_EVALUATE, NUL, OK, e_string_list_or_blob_required, eval0, forinfo_T};
use crate::main::emsg_skip;
use crate::mbyte::utfc_ptr2len;
use crate::memory::{xcalloc, xfree, xmemdupz, xstrdup};
use crate::message::emsg;
use crate::os::libc::gettext;
use crate::types::{
    VAR_BLOB, VAR_FIXED, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VAR_UNLOCKED, evalarg_T,
    exarg_T, listitem_T, size_t, typval_T, typval_vval_union, varnumber_T,
};

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VAR_UNLOCKED,
    vval: typval_vval_union { v_number: 0 },
};

/// Read the `for x in expr` header and set up the iteration. The answer is
/// always a `forinfo_T` the caller owns, even on the error paths, because
/// `:endfor` frees it either way; `errp` is what says the loop must not
/// run.
///
/// # Safety
/// `arg` must be NUL-terminated; `errp` and `evalarg` valid; `eap` null or
/// valid.
pub unsafe fn eval_for_line(
    arg: *const c_char,
    errp: *mut bool,
    eap: *mut exarg_T,
    evalarg: *mut evalarg_T,
) -> *mut c_void {
    unsafe {
        let fi = xcalloc(1, size_of::<forinfo_T>()) as *mut forinfo_T;
        let skip = (*evalarg).eval_flags & EVAL_EVALUATE as c_int == 0;
        *errp = true;

        let expr = skip_var_list(
            arg,
            &raw mut (*fi).fi_varcount,
            &raw mut (*fi).fi_semicolon,
            false,
        );
        if expr.is_null() {
            return fi as *mut c_void;
        }
        let expr = skipwhite(expr);
        if *expr.add(0) != b'i' as c_char
            || *expr.add(1) != b'n' as c_char
            || !(*expr.add(2) as c_int == NUL || ascii_iswhite(*expr.add(2) as c_int))
        {
            emsg(gettext(c"E690: Missing \"in\" after :for".as_ptr()));
            return fi as *mut c_void;
        }

        if skip {
            *emsg_skip.ptr() += 1;
        }
        let expr = skipwhite(expr.add(2));
        let mut tv = UNSET_TV;
        if eval0(expr as *mut c_char, &raw mut tv, eap, evalarg) == OK {
            *errp = false;
            if !skip {
                match tv.v_type {
                    VAR_LIST => {
                        let l = tv.vval.v_list;
                        if l.is_null() {
                            tv_clear(&raw mut tv);
                        } else {
                            // The reference moves into `fi`, and the watcher
                            // is what keeps the cursor valid across changes
                            // to the List while the loop runs.
                            (*fi).fi_list = l;
                            tv_list_watch_add(l, &raw mut (*fi).fi_lw);
                            (*fi).fi_lw.lw_item = tv_list_first(l);
                        }
                    }
                    VAR_BLOB => {
                        (*fi).fi_bi = 0;
                        if !tv.vval.v_blob.is_null() {
                            // Copied, so the loop is not affected by later
                            // changes to the Blob it was handed.
                            let mut btv = UNSET_TV;
                            tv_blob_copy(tv.vval.v_blob, &raw mut btv);
                            (*fi).fi_blob = btv.vval.v_blob;
                        }
                        tv_clear(&raw mut tv);
                    }
                    VAR_STRING => {
                        (*fi).fi_byte_idx = 0;
                        // The String is taken over rather than copied; a
                        // null one becomes an owned empty string so that
                        // `free_for_info` has something to free either way.
                        (*fi).fi_string = tv.vval.v_string;
                        tv.vval.v_string = null_mut();
                        if (*fi).fi_string.is_null() {
                            (*fi).fi_string = xstrdup(c"".as_ptr());
                        }
                    }
                    _ => {
                        emsg(gettext(e_string_list_or_blob_required.as_ptr()));
                        tv_clear(&raw mut tv);
                    }
                }
            }
        }
        if skip {
            *emsg_skip.ptr() -= 1;
        }
        fi as *mut c_void
    }
}

/// Assign the next item to the loop variables. False when the iteration is
/// over, or when the assignment failed.
///
/// # Safety
/// `fi_void` must be a `forinfo_T` from `eval_for_line`; `arg` the loop's
/// variable list.
pub unsafe fn next_for_item(fi_void: *mut c_void, arg: *mut c_char) -> bool {
    unsafe {
        let fi = fi_void as *mut forinfo_T;

        if !(*fi).fi_blob.is_null() {
            if (*fi).fi_bi >= tv_blob_len((*fi).fi_blob) {
                return false;
            }
            let mut tv = UNSET_TV;
            tv.v_type = VAR_NUMBER;
            tv.v_lock = VAR_FIXED;
            tv.vval.v_number = tv_blob_get((*fi).fi_blob, (*fi).fi_bi) as varnumber_T;
            (*fi).fi_bi += 1;
            return assign(fi, arg, &raw mut tv);
        }

        if !(*fi).fi_string.is_null() {
            let len = utfc_ptr2len((*fi).fi_string.offset((*fi).fi_byte_idx as isize));
            if len == 0 {
                return false;
            }
            let mut tv = UNSET_TV;
            tv.v_type = VAR_STRING;
            tv.v_lock = VAR_FIXED;
            tv.vval.v_string = xmemdupz(
                (*fi).fi_string.offset((*fi).fi_byte_idx as isize) as *const c_void,
                len as size_t,
            ) as *mut c_char;
            (*fi).fi_byte_idx += len;
            let ok = assign(fi, arg, &raw mut tv);
            // The typval was never handed over, so its String is ours.
            xfree(tv.vval.v_string as *mut c_void);
            return ok;
        }

        let item: *mut listitem_T = (*fi).fi_lw.lw_item;
        if item.is_null() {
            return false;
        }
        (*fi).fi_lw.lw_item = (*item).li_next;
        assign(fi, arg, &raw mut (*item).li_tv)
    }
}

/// Hand one item to the loop's variable list, copying it.
///
/// # Safety
/// As `next_for_item`.
unsafe fn assign(fi: *mut forinfo_T, arg: *mut c_char, tv: *mut typval_T) -> bool {
    unsafe {
        ex_let_vars(
            arg,
            tv,
            true,
            (*fi).fi_semicolon,
            (*fi).fi_varcount,
            false,
            null_mut(),
        ) == OK
    }
}

/// Release the iteration.
///
/// # Safety
/// `fi_void` must be null or a `forinfo_T` from `eval_for_line`.
pub unsafe fn free_for_info(fi_void: *mut c_void) {
    unsafe {
        let fi = fi_void as *mut forinfo_T;
        if fi.is_null() {
            return;
        }
        if !(*fi).fi_list.is_null() {
            tv_list_watch_remove((*fi).fi_list, &raw mut (*fi).fi_lw);
            tv_list_unref((*fi).fi_list);
        } else if !(*fi).fi_blob.is_null() {
            tv_blob_unref((*fi).fi_blob);
        } else {
            xfree((*fi).fi_string as *mut c_void);
        }
        xfree(fi as *mut c_void);
    }
}
