//! `:for`: the list of things to iterate, and the step from one to the next.
//!
//! `ForInfo` holds exactly one of three iterations and which field is
//! set is what says which: `fi_blob` a Blob by byte, `fi_string` a String
//! by character, `fi_list` (through `fi_lw`) a List by item. They are
//! tested in that order, so `free_for_info` and `next_for_item` agree
//! without anything recording a kind.
//!
//! Only the List form is *live*: the loop holds a watcher on it and sees
//! items added or removed while it runs. The Blob and the String are
//! copied up front, so changing the original mid-loop has no effect.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use crate::eval::typval::TV_INITIAL_VALUE;
use core::ffi::{c_char, c_int, c_void};
use core::mem::{offset_of, size_of};

use crate::eval::typval::{
    blob_copy, blob_len, blob_unref, index_of, list_items_mut, list_unref, tv_copy,
};
use crate::eval::vars::{VarList, ex_let_vars, skip_var_list};
use crate::eval::vars::{clear_local, emsg_static};
use crate::eval::{Fi, ForInfo, e_string_list_or_blob_required, eval0_in_cmd};
use crate::guard::Suppress;
use crate::mbyte::utfc_ptr2len;
use crate::memory::{xcalloc, xfree, xmemdupz, xstrdup};
use crate::types::{ExArg, ListWatch, TypVal, VAR_BLOB, VAR_LIST, VAR_STRING, VarNumber, size_t};

/// A freshly declared typval.
const UNSET_TV: TypVal = TV_INITIAL_VALUE;

/// Read the `for x in expr` header and set up the iteration. The answer is
/// always a `ForInfo` the caller owns, even on the error paths, because
/// `:endfor` frees it either way; `errp` is what says the loop must not
/// run.
///
/// `skip` parses the header without evaluating it.
///
/// # Safety
/// `errp` must be valid.
pub unsafe fn eval_for_line(excmd: &mut ExArg, errp: *mut bool, skip: bool) -> *mut c_void {
    // SAFETY: `xcalloc` never answers NULL and hands back one zeroed
    // `ForInfo`, which the caller owns until `:endfor` frees it.
    let mut fi = unsafe { Fi::new(xcalloc(1, size_of::<ForInfo>()) as *mut ForInfo) };
    // SAFETY: the caller's promise about `errp`.
    unsafe { *errp = true };

    let arg = excmd.line.arg;
    let Some(targets) = skip_var_list(excmd.line.rest_of(arg), false) else {
        return fi.raw() as *mut c_void;
    };
    fi.fi_varcount = targets.count;
    fi.fi_semicolon = c_int::from(targets.semicolon);
    let line = &excmd.line;
    let at = line.skip_white(arg + targets.end);
    if !line.starts_with(at, b"in") || !matches!(line.byte_at(at + 2), 0 | b' ' | b'\t') {
        // SAFETY: the message is a NUL-terminated literal.
        emsg_static(c"E690: Missing \"in\" after :for");
        return fi.raw() as *mut c_void;
    }

    let _skipping = skip.then(Suppress::emsg_skip);
    let at = line.skip_white(at + 2);
    let mut tv = UNSET_TV;
    if eval0_in_cmd(excmd, at, &mut tv, !skip).is_ok() {
        // SAFETY: the caller's promise about `errp`.
        unsafe { *errp = false };
        if !skip {
            match tv.v_type() {
                VAR_LIST => {
                    let l = tv.list_or_null();
                    if l.is_null() {
                        // SAFETY: `tv` is this frame's.
                        clear_local(&mut tv);
                    } else {
                        // The reference moves into `fi`, and the watcher
                        // is what keeps the cursor valid across changes
                        // to the List while the loop runs.
                        fi.fi_list = l;
                        let lw = fi.field_ptr::<ListWatch>(offset_of!(ForInfo, fi_lw));
                        // The List holds `lw` from here on, so this write
                        // goes through the pointer rather than borrowing
                        // the whole record — `winlayer::live`'s note.
                        // SAFETY: `lw` is the `ForInfo`'s own watcher, which
                        // the loop's first step reads.
                        unsafe { (*lw).lw_index = 0 };
                        // SAFETY: `l` is the live List the typval held.
                        unsafe { (*l).watch_add(lw) };
                        // The reference is `fi`'s now.
                        tv.disown();
                    }
                }
                VAR_BLOB => {
                    fi.fi_bi = 0;
                    if !tv.blob_or_null().is_null() {
                        // Copied, so the loop is not affected by later
                        // changes to the Blob it was handed.
                        let mut btv = UNSET_TV;
                        // SAFETY: the value's own blob, borrowed for the copy.
                        blob_copy(unsafe { tv.blob_or_null().as_ref() }, &mut btv);
                        // SAFETY: the copy left a Blob in `btv`.
                        fi.fi_blob = btv.blob_or_null();
                        // The reference the copy took is `fi`'s now.
                        btv.disown();
                    }
                    // SAFETY: `tv` is this frame's.
                    clear_local(&mut tv);
                }
                VAR_STRING => {
                    fi.fi_byte_idx = 0;
                    // The String is taken over rather than copied; a
                    // null one becomes an owned empty string so that
                    // `free_for_info` has something to free either way.
                    // The string's ownership moves into `fi` with it, and
                    // nothing clears `tv` on this path, so it is not
                    // nulled out here.
                    fi.fi_string = tv.string_or_null();
                    tv.disown();
                    if fi.fi_string.is_null() {
                        // SAFETY: the literal is NUL-terminated.
                        fi.fi_string = unsafe { xstrdup(c"".as_ptr()) };
                    }
                }
                _ => {
                    // SAFETY: the message is a NUL-terminated literal, and
                    // `tv` is this frame's.
                    emsg_static(e_string_list_or_blob_required);
                    // SAFETY: `tv` is this frame's.
                    clear_local(&mut tv);
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
/// `fi_void` must be a `ForInfo` from `eval_for_line`; `arg` is the loop's
/// variable list.
pub unsafe fn next_for_item(fi_void: *mut c_void, arg: &[u8]) -> bool {
    // `eval_for_line` handed the List the address of `fi_lw`, so the List
    // is holding a pointer into this record for as long as the loop runs.
    // Every write below therefore goes through `rec` rather than through
    // `DerefMut`, which would borrow the whole `ForInfo` and pop it —
    // see `winlayer::live`'s note.
    // SAFETY: the caller's promise -- the loop's own `ForInfo`, which
    // `:endfor` keeps alive for as long as the loop runs.
    let fi = unsafe { Fi::new(fi_void as *mut ForInfo) };
    let rec = fi.raw();

    if !fi.fi_blob.is_null() {
        // SAFETY: `fi_blob` is the copy `eval_for_line` took.
        let blob = unsafe { &*fi.fi_blob };
        if fi.fi_bi >= blob_len(Some(blob)) {
            return false;
        }
        let mut tv = UNSET_TV;
        tv.write_number(VarNumber::from(blob.byte(fi.fi_bi)));
        // SAFETY: `rec` is the caller's record.
        unsafe { (*rec).fi_bi += 1 };
        return assign(&fi, arg, &mut tv);
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
        // SAFETY: `len` bytes from `at` are the character just measured.
        tv.write_string(unsafe { xmemdupz(at as *const c_void, len as size_t) as *mut c_char });
        // SAFETY: `rec` is the caller's record.
        unsafe { (*rec).fi_byte_idx += len };
        return assign(&fi, arg, &mut tv);
    }

    // The cursor is an index into the list, which
    // `watch_shift` moves at every insert and removal so that it
    // keeps naming the same item.  `ENDED` is upstream's NULL `lw_item`,
    // and is sticky: a loop whose body appends to the list it is walking
    // still ends.
    let Ok(at) = usize::try_from(fi.fi_lw.lw_index) else {
        return false;
    };
    // SAFETY: `fi_list` is the List the loop took a reference to.
    let items = list_items_mut(unsafe { fi.fi_list.as_mut() });
    let Some(item) = items.get_mut(at) else {
        return false;
    };
    // The item is copied out: assigning it runs the targets' index
    // expressions, which may edit the List being walked.
    let mut value = UNSET_TV;
    tv_copy(&item.li_tv, &mut value);
    let next = if at + 1 >= items.len() {
        ListWatch::ENDED
    } else {
        index_of(at + 1)
    };
    // SAFETY: `rec` is the caller's record.
    unsafe { (*rec).fi_lw.lw_index = next };
    assign(&fi, arg, &mut value)
}

/// Hand one item to the loop's variable list, which takes it over.
fn assign(fi: &Fi, arg: &[u8], tv: &mut TypVal) -> bool {
    let targets = VarList {
        end: 0,
        count: fi.fi_varcount,
        semicolon: fi.fi_semicolon != 0,
    };
    let ok = ex_let_vars(arg, tv, false, targets, false, None).is_ok();
    // Whatever the targets did not take is this frame's to release.
    clear_local(tv);
    ok
}

/// Release the iteration.
///
/// # Safety
/// `fi_void` must be null or a `ForInfo` from `eval_for_line`.
pub unsafe fn free_for_info(fi_void: *mut c_void) {
    if fi_void.is_null() {
        return;
    }
    // SAFETY: the caller's promise -- the loop's own `ForInfo`.
    let fi = unsafe { Fi::new(fi_void as *mut ForInfo) };
    if !fi.fi_list.is_null() {
        let lw = fi.field_ptr(offset_of!(ForInfo, fi_lw));
        // Read out first: `List::watch_remove` writes through `lw`, which
        // points into this record, so no borrow of the record may still be
        // alive while it runs.
        let list = fi.fi_list;
        // SAFETY: the watcher was added to this List by `eval_for_line`,
        // and the reference it took is released here.
        unsafe { (*list).watch_remove(lw) };
        // SAFETY: as above -- this releases the reference `fi` held.
        unsafe { list_unref(list) };
    } else if !fi.fi_blob.is_null() {
        // SAFETY: the Blob is the copy `eval_for_line` took.
        unsafe { blob_unref(fi.fi_blob) };
    } else {
        // SAFETY: the String is owned, and null is fine for `xfree`.
        unsafe { xfree(fi.fi_string as *mut c_void) };
    }
    // SAFETY: nothing reaches the `ForInfo` after `:endfor`.
    unsafe { xfree(fi.raw() as *mut c_void) };
}
