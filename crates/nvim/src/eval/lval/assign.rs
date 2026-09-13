//! Performing the assignment [`get_lval`](super::get_lval) resolved.
//!
//! The write half of the pair: `set_var_lval` takes the [`LVal`] the
//! resolver filled in and stores the value through it, splitting by what
//! kind of target the record describes -- a whole variable by name, a Blob
//! byte or range, a List slice, or one item of a List or Dictionary.
//!
//! The ownership rule that matters, and the one a tidier rewrite gets
//! wrong: `oldtv` here is a *separate* typval from the value being written.
//! It is the value a dictionary's watchers are told the key used to have, it
//! is only filled for a key that already existed, and its being left unset
//! is exactly how the notification tells a new key from an overwritten one.
//! Merging it with anything would notify with the wrong value and then clear
//! it twice.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::message_fmt::c_str;
use crate::semsg;
use core::ffi::{c_char, c_int};
use core::ptr::null_mut;

use crate::eval::executor::eexe_mod_op;
use crate::eval::typval::{
    di_lock, tv_blob_len, tv_blob_set_append, tv_blob_set_range, tv_check_lock, tv_clear, tv_copy,
    tv_dict_add, tv_dict_is_watched, tv_dict_item_alloc, tv_dict_item_free, tv_dict_watcher_notify,
    tv_dict_wrong_func_name, tv_get_number_chk, tv_list_assign_range, value_check_lock,
};
use crate::eval::vars::{clear_local, emsg_static};
use crate::eval::vars::{
    eval_variable, get_vimvar_dict, set_var, set_var_const, set_vvar_item, var_check_ro,
};
use crate::eval::{Lv, TV_CSTRING, Tv};
use crate::message::{e_cannot_mod, e_listreq};
use crate::types::{
    DictItem, LVal, NUL, TypVal, VAR_BLOB, VAR_LIST, VAR_UNKNOWN, VarLock, VarNumber, size_t,
    uint8_t,
};

use super::UNSET_TV;

/// Perform the assignment `get_lval` resolved. `endp` is the cursor after
/// the left-hand side, which is terminated in place while a message might
/// name the variable.
///
/// # Safety
/// `lval` must come from `get_lval`; `endp` must point into the same writable
/// string; `result` must be valid.
pub unsafe fn set_var_lval(
    lval: *mut LVal,
    endp: *mut c_char,
    result: &mut TypVal,
    copy: bool,
    is_const: bool,
    op: *const c_char,
) {
    // SAFETY, for every region in this body and in the two helpers below:
    // the caller's promise is that `lval` is the record `get_lval` filled in
    // and outlives the call, that `result` is the value being assigned, and
    // that `endp` points into the same writable NUL-terminated string. Each
    // union member read is the one the `v_type` just tested names; a
    // non-null `op` is NUL-terminated; `oldtv` and `tv` are frame locals;
    // and every message named is a literal or a shared `e_*` text. The
    // notes below add only what is local to a site.
    let (mut lval, value) = unsafe { (Lv::new(lval), Tv::new(result)) };
    if lval.ll_tv.is_null() {
        // SAFETY: as above; `endp` points into the same writable string.
        unsafe { set_whole_var(lval.raw(), endp, result, copy, is_const, op) };
        return;
    }

    // A locked container refuses the write; the lock to test is the
    // Dict's own when a key is being added to it.
    // SAFETY: a pending new key means `ll_tv` holds the Dict it goes into.
    let target = unsafe { Tv::new(lval.ll_tv) };
    let lock = if lval.ll_newkey.is_null() {
        // SAFETY: `ll_lock` is the lock of the slot `ll_tv` points into,
        // set beside it whenever it is.
        unsafe { *lval.ll_lock }
    } else {
        // SAFETY: as above -- the Dict the key is being added to.
        unsafe { (*target.dict_or_null()).dv_lock }
    };
    if unsafe { value_check_lock(lock, lval.ll_name, TV_CSTRING as size_t) } {
        return;
    }

    if lval.ll_range {
        if is_const {
            emsg_static(c"E996: Cannot lock a range");
            return;
        }
        // Crash fix, upstream reads the union the wrong way here: the
        // lval resolver accepts a Blob value for a `[:]` because the
        // *target* may be a Blob, but a Blob target leaves `ll_tv`
        // null and never reaches this branch. So a Blob reaching it
        // means a List target, and upstream hands its `v_blob` to
        // `tv_list_assign_range` through `vval.v_list` — walking a
        // `Blob` as a `List`. `let l = [1,2] | let l[0:] = 0z11`
        // is enough. Report what the assignment actually needs.
        if value.v_type() != VAR_LIST {
            emsg_static(e_listreq);
            return;
        }
        let src = value.list_or_null();
        let (list, n1, n2) = (lval.ll_list, lval.ll_n1, lval.ll_n2);
        let (empty2, name) = (lval.ll_empty2, lval.ll_name);
        // SAFETY: as above.
        let _ = unsafe { tv_list_assign_range(list, src, n1, n2, empty2, op, name) };
        return;
    }

    // The value the watchers are told the key used to have. It stays
    // unset for a key that did not exist, and that is how the
    // notification below tells the two cases apart — see the module
    // docs. It must never be the same typval as the new value.
    let mut oldtv = UNSET_TV;
    let dict = lval.ll_dict;
    let watched = unsafe { tv_dict_is_watched(dict) };

    if is_const {
        emsg_static(c"E996: Cannot lock a list or dict");
        return;
    }

    // Writing an *existing* key of the `v:` scope dictionary is a write
    // to a `v:` variable, and has to pass the same type enforcement the
    // unsubscripted spelling does. Upstream stores straight into the
    // item, which permanently re-types the variable and, for
    // `v:oldfiles`, crashes the next reader (docket O-B14-10). A new key
    // cannot happen here: `get_lval` refuses to add one to `v:`.
    if dict == get_vimvar_dict() && lval.ll_newkey.is_null() {
        // SAFETY: `ll_di` is the existing item, and `result` the caller's.
        unsafe { set_vvar_item(lval.ll_di, result, copy, op) };
        return;
    }

    // Whether the value still has to be stored: `+=` and friends modify the
    // target in place and leave nothing to assign.
    let assign;
    if !lval.ll_newkey.is_null() {
        // The key has to be added to the Dictionary first.
        if !op.is_null() && unsafe { *op } != b'=' as c_char {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let ll_newkey = unsafe { c_str(lval.ll_newkey) };
            semsg!("E716: Key not present in Dictionary: \"{ll_newkey}\"");
            return;
        }
        // SAFETY: `ll_tv` holds the Dict; `ll_newkey` is the owned key text.
        let target = unsafe { Tv::new(lval.ll_tv).dict_or_null() };
        if unsafe { tv_dict_wrong_func_name(target, result, lval.ll_newkey) } != 0 {
            return;
        }
        let di = unsafe { tv_dict_item_alloc(lval.ll_newkey) };
        if unsafe { tv_dict_add(target, di) }.is_err() {
            unsafe { tv_dict_item_free(di) };
            return;
        }
        // SAFETY: `di` belongs to the Dict; its typval is the target.
        (lval.ll_tv, lval.ll_lock) = unsafe { (&raw mut (*di).di_tv, di_lock(di)) };
        assign = true;
    } else {
        if watched {
            // SAFETY: this frame's separate record of the old value.
            unsafe { tv_copy(&*lval.ll_tv, &mut oldtv) };
        }
        assign = op.is_null() || unsafe { *op } == b'=' as c_char;
        if assign {
            unsafe { tv_clear(&mut *lval.ll_tv) };
        } else {
            // SAFETY: the live target and the caller's value.
            let _ = unsafe { eexe_mod_op(lval.ll_tv, result, op) };
        }
    }

    if assign {
        if copy {
            unsafe { tv_copy(result, &mut *lval.ll_tv) };
        } else {
            // SAFETY: the value moves out of `result`, which is reset after it.
            let mut target = unsafe { Tv::new(lval.ll_tv) };
            // SAFETY: as above -- the take resets `result`, so nothing
            // frees the value twice.
            *target = (*result).take();
        }
        // Upstream leaves the assigned value unlocked, by hand on one branch
        // and through `tv_copy` on the other; the lock is the slot's own.
        unsafe { *lval.ll_lock = VarLock::Unlocked };
    }

    if !watched {
        return;
    }
    if oldtv.v_type() == VAR_UNKNOWN {
        // Nothing was saved, so this is the new-key case.
        debug_assert!(!lval.ll_newkey.is_null());
        // SAFETY: the watched Dict, its new key, and the value just written.
        unsafe { tv_dict_watcher_notify(dict, lval.ll_newkey, Some(&*lval.ll_tv), None) };
    } else {
        let di = lval.ll_di;
        // SAFETY: an item of the dictionary being written to, which owns its
        // key for as long as it is in the table.
        let key = unsafe { (*di).di_key.as_ptr() }.cast_mut();
        // SAFETY: the watched Dict, its key, the new value and the old copy.
        let new = unsafe { &*lval.ll_tv };
        unsafe { tv_dict_watcher_notify(dict, key, Some(new), Some(&oldtv)) };
        clear_local(&mut oldtv);
    }
}

/// The `ll_tv == NULL` half of `set_var_lval`: the target is a whole
/// variable by name, or a Blob byte or byte range.
///
/// # Safety
/// As `set_var_lval`.
unsafe fn set_whole_var(
    lval: *mut LVal,
    endp: *mut c_char,
    result: &mut TypVal,
    copy: bool,
    is_const: bool,
    op: *const c_char,
) {
    let lval = unsafe { Lv::new(lval) };
    // Terminate the left-hand side in place: the messages below name the
    // variable and would otherwise print the rest of the command too.
    // SAFETY: the caller's promise -- `endp` points into the same writable NUL-terminated string.
    let cc = unsafe { *endp };
    // SAFETY: as above -- the byte is put back before returning.
    unsafe { *endp = NUL as c_char };

    if !lval.ll_blob.is_null() {
        // Upstream's three early returns here leave the left-hand side
        // terminated in place rather than putting `cc` back. Preserved:
        // anything that reads the command line after a rejected Blob
        // assignment sees the truncated form.
        // SAFETY: `lval` has `ll_blob` set, and `result` is the caller's.
        if !unsafe { set_blob_var(lval.raw(), result, op) } {
            return;
        }
    } else if !op.is_null() && unsafe { *op } != b'=' as c_char {
        // `+=`, `-=`, `*=`, `/=`, `%=` and `..=`.
        if is_const {
            emsg_static(e_cannot_mod);
            unsafe { *endp = cc };
            return;
        }
        let mut tv = UNSET_TV;
        let mut di: *mut DictItem = null_mut();
        let (name, name_len) = (lval.ll_name, lval.ll_name_len);
        // SAFETY: the name is the one `get_lval` resolved, and `tv` and `di` are this frame's.
        let dip = &raw mut di;
        let found =
            unsafe { eval_variable(name, name_len as c_int, Some(&mut tv), dip, true, false) };
        if found.is_ok() {
            // SAFETY: a non-null `di` is live; `tv` is this frame's copy.
            let (n, dtv, dlock) = if di.is_null() {
                (0, null_mut(), VarLock::Unlocked)
            } else {
                // SAFETY: `di` is live, so naming its typval reads nothing,
                // and its lock is the slot's.
                (
                    unsafe { (*di).di_flags } as c_int,
                    unsafe { &raw mut (*di).di_tv },
                    unsafe { *di_lock(di) },
                )
            };
            let writable = di.is_null()
                || (!unsafe { var_check_ro(n, name, TV_CSTRING as size_t) }
                    && !unsafe { tv_check_lock(dlock, &*dtv, name, TV_CSTRING as size_t) });
            if writable && unsafe { eexe_mod_op(&raw mut tv, result, op) }.is_ok() {
                // SAFETY: as above -- the folded value goes back by name.
                unsafe { set_var(name, name_len, &mut tv, false) };
            }
            clear_local(&mut tv);
        }
    } else {
        let (name, name_len) = (lval.ll_name, lval.ll_name_len);
        // SAFETY: the name is the one `get_lval` resolved, and `result` is the caller's value.
        unsafe { set_var_const(name, name_len, result, copy, is_const) };
    }

    unsafe { *endp = cc };
}

/// Write a byte or a byte range into the Blob `lval` resolved. Answers
/// whether the caller should put the terminated left-hand side back — the
/// three refusal paths say no, which is upstream's.
///
/// # Safety
/// As `set_var_lval`, with `lval->ll_blob` set.
unsafe fn set_blob_var(lval: *mut LVal, result: &mut TypVal, op: *const c_char) -> bool {
    // SAFETY: the caller's promise -- both outlive the call.
    let (mut lval, value) = unsafe { (Lv::new(lval), Tv::new(result)) };
    if !op.is_null() && unsafe { *op } != b'=' as c_char {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let op = unsafe { c_str(op) };
        semsg!("E734: Wrong variable type for {op}=");
        return false;
    }
    // SAFETY: the caller's promise: `ll_blob` is live, the name resolved.
    let lock = unsafe { (*lval.ll_blob).bv_lock };
    // SAFETY: as above.
    let locked = unsafe { value_check_lock(lock, lval.ll_name, TV_CSTRING as size_t) };
    if locked {
        return false;
    }

    if lval.ll_range && value.v_type() == VAR_BLOB {
        if lval.ll_empty2 {
            lval.ll_n2 = unsafe { tv_blob_len(lval.ll_blob) } - 1;
        }
        let (blob, n1, n2) = (
            lval.ll_blob,
            lval.ll_n1 as VarNumber,
            lval.ll_n2 as VarNumber,
        );
        // SAFETY: as above; `result` holds the Blob being assigned.
        if unsafe { tv_blob_set_range(blob, n1, n2, result) }.is_err() {
            return false;
        }
        return true;
    }

    if let Ok(val) = tv_get_number_chk(result) {
        if !(0..=255).contains(&val) {
            // Upstream's text is `"E1239: Invalid value for blob: 0x" PRIX64`,
            // which is missing the `%`: `val` has never reached the message.
            let _ = val;
            semsg!("E1239: Invalid value for blob: 0xlX");
        } else {
            // SAFETY: `ll_blob` is the live Blob and `ll_n1` a byte of it.
            unsafe { tv_blob_set_append(lval.ll_blob, lval.ll_n1, val as uint8_t) };
        }
    }
    true
}
