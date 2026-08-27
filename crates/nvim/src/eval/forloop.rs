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
use core::mem::{offset_of, size_of};
use core::ptr::null_mut;

use crate::ascii::ascii_iswhite;
use crate::charset::skipwhite;
use crate::eval::typval::{
    tv_blob_copy, tv_blob_get, tv_blob_len, tv_blob_unref, tv_clear, tv_list_first, tv_list_unref,
    tv_list_watch_add, tv_list_watch_remove,
};
use crate::eval::vars::{ex_let_vars, skip_var_list};
use crate::eval::{EVAL_EVALUATE, Fi, e_string_list_or_blob_required, eval0, forinfo_T};
use crate::guard::Suppress;
use crate::mbyte::utfc_ptr2len;
use crate::memory::{xcalloc, xfree, xmemdupz, xstrdup};
use crate::message::emsg;
use crate::os::cshim::gettext;
use crate::types::{
    NUL, OK, VAR_BLOB, VAR_LIST, VAR_NUMBER, VAR_STRING, VAR_UNKNOWN, VarLock, evalarg_T, exarg_T,
    listitem_T, size_t, typval_T, typval_vval_union, varnumber_T,
};

/// A freshly declared typval.
const UNSET_TV: typval_T = typval_T {
    v_type: VAR_UNKNOWN,
    v_lock: VarLock::Unlocked,
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
    // SAFETY: `xcalloc` never answers NULL and hands back one zeroed
    // `forinfo_T`, which the caller owns until `:endfor` frees it.
    let mut fi = unsafe { Fi::new(xcalloc(1, size_of::<forinfo_T>()) as *mut forinfo_T) };
    // SAFETY: the caller's promise about `evalarg` and `errp`.
    let skip = unsafe { (*evalarg).eval_flags } & EVAL_EVALUATE as c_int == 0;
    // SAFETY: as above.
    unsafe { *errp = true };

    let varcount = fi.field_ptr(offset_of!(forinfo_T, fi_varcount));
    let semicolon = fi.field_ptr(offset_of!(forinfo_T, fi_semicolon));
    // SAFETY: the caller's promise that `arg` is NUL-terminated; the two
    // out-parameters are the `forinfo_T`'s own fields.
    let expr = unsafe { skip_var_list(arg, varcount, semicolon, false) };
    if expr.is_null() {
        return fi.raw() as *mut c_void;
    }
    // SAFETY: `expr` points into `arg`, which is NUL-terminated, so the
    // three bytes tested below stop at the terminator.
    let expr = unsafe { skipwhite(expr) };
    if unsafe { *expr.add(0) } != b'i' as c_char
        || unsafe { *expr.add(1) } != b'n' as c_char
        || !(unsafe { *expr.add(2) } as c_int == NUL
            || ascii_iswhite(unsafe { *expr.add(2) } as c_int))
    {
        // SAFETY: the message is a NUL-terminated literal.
        unsafe { emsg(gettext(c"E690: Missing \"in\" after :for".as_ptr())) };
        return fi.raw() as *mut c_void;
    }

    let _skipping = skip.then(Suppress::emsg_skip);
    // SAFETY: as above -- two bytes into a NUL-terminated string.
    let expr = unsafe { skipwhite(expr.add(2)) };
    let mut tv = UNSET_TV;
    // SAFETY: `expr` is NUL-terminated, `tv` is this frame's, and `eap` and
    // `evalarg` are the caller's.
    if unsafe { eval0(expr as *mut c_char, &raw mut tv, eap, evalarg) } == OK {
        // SAFETY: the caller's promise about `errp`.
        unsafe { *errp = false };
        if !skip {
            match tv.v_type {
                VAR_LIST => {
                    // SAFETY: `VAR_LIST` says `v_list` is the live member.
                    let l = unsafe { tv.vval.v_list };
                    if l.is_null() {
                        // SAFETY: `tv` is this frame's.
                        unsafe { tv_clear(&raw mut tv) };
                    } else {
                        // The reference moves into `fi`, and the watcher
                        // is what keeps the cursor valid across changes
                        // to the List while the loop runs.
                        fi.fi_list = l;
                        let lw = fi.field_ptr(offset_of!(forinfo_T, fi_lw));
                        // SAFETY: `l` is the live List the typval held, and
                        // `lw` is the `forinfo_T`'s own watcher.
                        unsafe { tv_list_watch_add(l, lw) };
                        // SAFETY: as above.
                        fi.fi_lw.lw_item = unsafe { tv_list_first(l) };
                    }
                }
                VAR_BLOB => {
                    fi.fi_bi = 0;
                    // SAFETY: `VAR_BLOB` says `v_blob` is the live member.
                    if !unsafe { tv.vval.v_blob }.is_null() {
                        // Copied, so the loop is not affected by later
                        // changes to the Blob it was handed.
                        let mut btv = UNSET_TV;
                        // SAFETY: as above; `btv` is this frame's.
                        unsafe { tv_blob_copy(tv.vval.v_blob, &raw mut btv) };
                        // SAFETY: the copy left a Blob in `btv`.
                        fi.fi_blob = unsafe { btv.vval.v_blob };
                    }
                    // SAFETY: `tv` is this frame's.
                    unsafe { tv_clear(&raw mut tv) };
                }
                VAR_STRING => {
                    fi.fi_byte_idx = 0;
                    // The String is taken over rather than copied; a
                    // null one becomes an owned empty string so that
                    // `free_for_info` has something to free either way.
                    // SAFETY: `VAR_STRING` says `v_string` is the live
                    // member, and the ownership moves into `fi`.
                    fi.fi_string = unsafe { tv.vval.v_string };
                    tv.vval.v_string = null_mut();
                    if fi.fi_string.is_null() {
                        // SAFETY: the literal is NUL-terminated.
                        fi.fi_string = unsafe { xstrdup(c"".as_ptr()) };
                    }
                }
                _ => {
                    // SAFETY: the message is a NUL-terminated literal, and
                    // `tv` is this frame's.
                    unsafe {
                        emsg(gettext(e_string_list_or_blob_required.as_ptr()));
                        tv_clear(&raw mut tv);
                    };
                }
            }
        }
    }
    fi.raw() as *mut c_void
}

/// Assign the next item to the loop variables. False when the iteration is
/// over, or when the assignment failed.
///
/// # Safety
/// `fi_void` must be a `forinfo_T` from `eval_for_line`; `arg` the loop's
/// variable list.
pub unsafe fn next_for_item(fi_void: *mut c_void, arg: *mut c_char) -> bool {
    // SAFETY: the caller's promise -- the loop's own `forinfo_T`, which
    // `:endfor` keeps alive for as long as the loop runs.
    let mut fi = unsafe { Fi::new(fi_void as *mut forinfo_T) };

    if !fi.fi_blob.is_null() {
        // SAFETY: `fi_blob` is the copy `eval_for_line` took.
        if fi.fi_bi >= unsafe { tv_blob_len(fi.fi_blob) } {
            return false;
        }
        let mut tv = UNSET_TV;
        tv.v_type = VAR_NUMBER;
        tv.v_lock = VarLock::Fixed;
        // SAFETY: as above; `fi_bi` is inside the Blob.
        tv.vval.v_number = unsafe { tv_blob_get(fi.fi_blob, fi.fi_bi) } as varnumber_T;
        fi.fi_bi += 1;
        // SAFETY: `tv` is this frame's, and `arg` the caller's list.
        return unsafe { assign(fi, arg, &raw mut tv) };
    }

    if !fi.fi_string.is_null() {
        // SAFETY: `fi_string` is owned and NUL-terminated, and `fi_byte_idx`
        // is a character boundary inside it.
        let at = unsafe { fi.fi_string.offset(fi.fi_byte_idx as isize) };
        // SAFETY: as above.
        let len = unsafe { utfc_ptr2len(at) };
        if len == 0 {
            return false;
        }
        let mut tv = UNSET_TV;
        tv.v_type = VAR_STRING;
        tv.v_lock = VarLock::Fixed;
        // SAFETY: `len` bytes from `at` are the character just measured.
        tv.vval.v_string = unsafe { xmemdupz(at as *const c_void, len as size_t) as *mut c_char };
        fi.fi_byte_idx += len;
        // SAFETY: `tv` is this frame's, and `arg` the caller's list.
        let ok = unsafe { assign(fi, arg, &raw mut tv) };
        // The typval was never handed over, so its String is ours.
        // SAFETY: the string allocated just above.
        unsafe { xfree(tv.vval.v_string as *mut c_void) };
        return ok;
    }

    let item: *mut listitem_T = fi.fi_lw.lw_item;
    if item.is_null() {
        return false;
    }
    // SAFETY: the watcher keeps `lw_item` pointing at a live item.
    fi.fi_lw.lw_item = unsafe { (*item).li_next };
    // SAFETY: as above -- the item's typval is the List's own.
    unsafe { assign(fi, arg, &raw mut (*item).li_tv) }
}

/// Hand one item to the loop's variable list, copying it.
///
/// # Safety
/// As `next_for_item`.
unsafe fn assign(fi: Fi, arg: *mut c_char, tv: *mut typval_T) -> bool {
    let (semicolon, varcount) = (fi.fi_semicolon, fi.fi_varcount);
    // SAFETY: the caller's promise -- `arg` is the loop's variable list and
    // `tv` the item being assigned.
    unsafe { ex_let_vars(arg, tv, true, semicolon, varcount, false, null_mut()) == OK }
}

/// Release the iteration.
///
/// # Safety
/// `fi_void` must be null or a `forinfo_T` from `eval_for_line`.
pub unsafe fn free_for_info(fi_void: *mut c_void) {
    if fi_void.is_null() {
        return;
    }
    // SAFETY: the caller's promise -- the loop's own `forinfo_T`.
    let fi = unsafe { Fi::new(fi_void as *mut forinfo_T) };
    if !fi.fi_list.is_null() {
        let lw = fi.field_ptr(offset_of!(forinfo_T, fi_lw));
        // SAFETY: the watcher was added to this List by `eval_for_line`,
        // and the reference it took is released here.
        unsafe {
            tv_list_watch_remove(fi.fi_list, lw);
            tv_list_unref(fi.fi_list);
        };
    } else if !fi.fi_blob.is_null() {
        // SAFETY: the Blob is the copy `eval_for_line` took.
        unsafe { tv_blob_unref(fi.fi_blob) };
    } else {
        // SAFETY: the String is owned, and null is fine for `xfree`.
        unsafe { xfree(fi.fi_string as *mut c_void) };
    }
    // SAFETY: nothing reaches the `forinfo_T` after `:endfor`.
    unsafe { xfree(fi.raw() as *mut c_void) };
}
