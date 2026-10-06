//! Performing the assignment [`get_lval`](super::get_lval) resolved.
//!
//! The write half of the pair: [`set_var_lval`] stores the value through the
//! [`LValue`] the resolver answered, splitting by what kind of target it
//! is -- a whole variable by name, a Blob byte or range, a List slice, a key
//! to add, or the value in a slot. The slot is found again here, by the
//! handle and the index or key the resolver kept; nothing ran in between,
//! so it is where the resolver left it.
//!
//! The ownership rule that matters, and the one a tidier rewrite gets
//! wrong: `oldtv` here is a *separate* typval from the value being written.
//! It is the value a dictionary's watchers are told the key used to have, it
//! is only filled for a key that already existed, and its being left unset
//! is exactly how the notification tells a new key from an overwritten one.
//! Merging it with anything would notify with the wrong value and then clear
//! it twice.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use crate::cstr;
use crate::message_fmt::msg_bytes;
use crate::semsg;

use super::{LValue, Slot, Span, Target, UNSET_TV};
use crate::eval::executor::eexe_mod_op;
use crate::eval::typval::{
    BlobRef, DictRef, assign_range, blob_len, dict_is_watched, dict_watcher_notify, set_range,
    tv_check_lock_named, tv_copy, tv_get_number_chk, value_check_lock_named,
};
use crate::eval::vars::{
    clear_local, emsg_static, get_vimvar_dict, set_var_const_named, set_vvar_key,
    var_check_ro_named, with_var,
};
use crate::message::{e_cannot_mod, e_listreq};
use crate::types::{TypVal, VAR_BLOB, VAR_LIST, VAR_UNKNOWN, VarLock, VarNumber};

/// Whether `op` is one of the compound operators rather than `=` or none.
fn is_compound(op: Option<u8>) -> bool {
    op.is_some_and(|op| op != b'=')
}

/// The value to store: a copy of `value`, or `value` itself, which is left
/// empty.
fn stored(value: &mut TypVal, copy: bool) -> TypVal {
    if copy {
        let mut tv = UNSET_TV;
        tv_copy(value, &mut tv);
        tv
    } else {
        value.take()
    }
}

/// Perform the assignment `lval` resolved, of `value` with the compound
/// operator `op` (`+`, `-`, `*`, `/`, `%` or `.`), or plain when that is
/// `=` or absent. `copy` stores a copy and leaves `value` alone; otherwise
/// the value is moved where it can be. `is_const` is `:const`.
pub(crate) fn set_var_lval(
    lval: &mut LValue<'_>,
    value: &mut TypVal,
    copy: bool,
    is_const: bool,
    op: Option<u8>,
) {
    match &lval.target {
        Target::Variable => return set_whole_var(lval, value, copy, is_const, op),
        Target::Blob { blob, span } => {
            set_blob_var(lval, blob, *span, value, op);
            return;
        }
        Target::Slot { .. } | Target::NewKey { .. } => {}
    }

    // A locked container refuses the write; the lock to test is the
    // Dict's own when a key is being added to it.
    let lock = match &lval.target {
        Target::NewKey { dict, .. } => Some(dict.dv_lock),
        _ => lval.with_slot(|_, lock| *lock),
    };
    // The slot was found a moment ago and nothing has run since.
    let Some(lock) = lock else { return };
    if value_check_lock_named(lock, lval.name_and_rest()) {
        return;
    }

    if let Target::Slot {
        slot: Slot::Item { list, .. },
        span: span @ Span { range: true, .. },
    } = &lval.target
    {
        if is_const {
            emsg_static(c"E996: Cannot lock a range");
            return;
        }
        // Crash fix, upstream reads the union the wrong way here: the
        // lval resolver accepts a Blob value for a `[:]` because the
        // *target* may be a Blob, but a Blob target never reaches this
        // branch. So a Blob reaching it means a List target, and upstream
        // hands its `v_blob` to `list_assign_range` through `vval.v_list`
        // -- walking a `Blob` as a `List`. `let l = [1,2] | let l[0:] =
        // 0z11` is enough. Report what the assignment actually needs.
        if value.v_type() != VAR_LIST {
            emsg_static(e_listreq);
            return;
        }
        let src = value.list_handle();
        let name = lval.name_and_rest();
        let _ = assign_range(list, src.as_ref(), span.n1, span.n2, span.empty2, op, name);
        return;
    }

    if is_const {
        emsg_static(c"E996: Cannot lock a list or dict");
        return;
    }

    let key_slot = match &lval.target {
        Target::NewKey { dict, key } => {
            add_key(dict, key, value, copy, op);
            return;
        }
        Target::Slot {
            slot: Slot::Key { dict, key },
            ..
        } => {
            // Writing an *existing* key of the `v:` scope dictionary is a
            // write to a `v:` variable, and has to pass the same type
            // enforcement the unsubscripted spelling does. Upstream stores
            // straight into the item, which permanently re-types the
            // variable and, for `v:oldfiles`, crashes the next reader
            // (docket O-B14-10).
            if dict.as_ptr() == get_vimvar_dict() {
                set_vvar_key(key, value, copy, op);
                return;
            }
            // Only a watched dictionary needs them after the write.
            dict_is_watched(Some(dict)).then(|| (dict.clone(), key.clone()))
        }
        _ => None,
    };
    let Some((dict, key)) = key_slot else {
        write_slot(lval, value, copy, op);
        return;
    };
    let mut oldtv = UNSET_TV;
    lval.with_slot(|tv, _| tv_copy(tv, &mut oldtv));
    write_slot(lval, value, copy, op);
    notify_key(&dict, &key, lval, &oldtv);
    clear_local(&mut oldtv);
}

/// Store `value` in the slot `lval` names: in place for a compound
/// operator, replacing what is there otherwise.
fn write_slot(lval: &mut LValue<'_>, value: &mut TypVal, copy: bool, op: Option<u8>) {
    let Some(op) = op.filter(|&op| op != b'=') else {
        let new = stored(value, copy);
        lval.with_slot(move |tv, lock| {
            *tv = new;
            // Upstream leaves the assigned value unlocked, by hand on one
            // branch and through `tv_copy` on the other; the lock is the
            // slot's own.
            *lock = VarLock::Unlocked;
        });
        return;
    };
    // The operator works on a copy of the current value -- which shares
    // its List, Dict or Blob, so `+=` still extends the one in the slot --
    // and the result goes back. The slot itself stays put while the
    // operator reads the value: `:let l[0] += l` reads the List that holds
    // the slot.
    let mut current = UNSET_TV;
    if lval.with_slot(|tv, _| tv_copy(tv, &mut current)).is_none() {
        return;
    }
    if eexe_mod_op(&mut current, value, op).is_ok() {
        lval.with_slot(move |tv, _| *tv = current);
    } else {
        clear_local(&mut current);
    }
}

/// Add `key` to `dict`, holding `value`.
fn add_key(dict: &DictRef, key: &[u8], value: &mut TypVal, copy: bool, op: Option<u8>) {
    if is_compound(op) {
        let key = msg_bytes(key);
        semsg!("E716: Key not present in Dictionary: \"{key}\"");
        return;
    }
    let mut target = dict.clone();
    // The builtin-name check upstream runs here is the add's own; a
    // refusal leaves the dictionary alone.
    if target.add_value(key, stored(value, copy)).is_err() {
        return;
    }
    if dict_is_watched(Some(dict)) {
        // Nothing was saved, so this is the new-key case.
        let mut newtv = UNSET_TV;
        if let Some(item) = target.find(key) {
            tv_copy(&item.di_tv, &mut newtv);
        }
        cstr::with_terminated(key, |key| {
            dict_watcher_notify(dict, key, Some(&newtv), None)
        });
        clear_local(&mut newtv);
    }
}

/// Tell `dict`'s watchers that `key` changed from `oldtv` to what the slot
/// holds now.
fn notify_key(dict: &DictRef, key: &[u8], lval: &mut LValue<'_>, oldtv: &TypVal) {
    let mut newtv = UNSET_TV;
    lval.with_slot(|tv, _| tv_copy(tv, &mut newtv));
    // An old value of `VAR_UNKNOWN` is how the C told a new key; a key that
    // existed always has one.
    let old = (oldtv.v_type() != VAR_UNKNOWN).then_some(oldtv);
    cstr::with_terminated(key, |key| dict_watcher_notify(dict, key, Some(&newtv), old));
    clear_local(&mut newtv);
}

/// The whole-variable half of `set_var_lval`: the target is a variable,
/// by name.
fn set_whole_var(
    lval: &LValue<'_>,
    value: &mut TypVal,
    copy: bool,
    is_const: bool,
    op: Option<u8>,
) {
    let name = lval.name();
    let Some(op) = op.filter(|&op| op != b'=') else {
        set_var_const_named(name, value, copy, is_const);
        return;
    };
    // `+=`, `-=`, `*=`, `/=`, `%=` and `..=`.
    if is_const {
        emsg_static(e_cannot_mod);
        return;
    }
    // A lookup that may source an autoload script, as `eval_variable` is.
    let found = with_var(name, false, |item| {
        let mut tv = UNSET_TV;
        tv_copy(&item.di_tv, &mut tv);
        (tv, item.di_flags, item.di_lock)
    });
    let Some((mut tv, flags, lock)) = found else {
        let name = msg_bytes(name);
        semsg!("E121: Undefined variable: {name}");
        return;
    };
    let writable = !var_check_ro_named(flags.into(), name) && !tv_check_lock_named(lock, &tv, name);
    if writable && eexe_mod_op(&mut tv, value, op).is_ok() {
        // The folded value goes back by name.
        set_var_const_named(name, &mut tv, false, false);
    }
    clear_local(&mut tv);
}

/// Write a byte or a byte range into the Blob `lval` resolved.
fn set_blob_var(lval: &LValue<'_>, blob: &BlobRef, span: Span, value: &TypVal, op: Option<u8>) {
    if let Some(op) = op.filter(|&op| op != b'=') {
        let op = char::from(op);
        semsg!("E734: Wrong variable type for {op}=");
        return;
    }
    if value_check_lock_named(blob.bv_lock, lval.name()) {
        return;
    }

    if span.range && value.v_type() == VAR_BLOB {
        let n2 = if span.empty2 {
            blob_len(Some(blob)) - 1
        } else {
            span.n2
        };
        // `value` may hold that very blob, which the callee allows.
        let _ = set_range(blob, VarNumber::from(span.n1), VarNumber::from(n2), value);
        return;
    }

    if let Ok(val) = tv_get_number_chk(value) {
        if !(0..=255).contains(&val) {
            // Upstream's text is `"E1239: Invalid value for blob: 0x" PRIX64`,
            // which is missing the `%`: `val` has never reached the message.
            semsg!("E1239: Invalid value for blob: 0xlX");
        } else {
            let byte = u8::try_from(val).expect("a byte, just checked");
            blob.clone().set_or_append(span.n1, byte);
        }
    }
}
