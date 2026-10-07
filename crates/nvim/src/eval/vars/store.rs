//! Assigning to a name, and the four checks that can refuse.
//!
//! [`set_var_const`] is the single entry point every assignment reaches; the
//! `var_check_*` trio reads `di_flags` and produces E46 / E795 / E1122, and
//! [`var_wrong_func_name_named`] and [`valid_varname_named`] reject the name
//! itself.

#![forbid(unsafe_code)]

use crate::cstr;
use crate::eval::typval::DictRef;
use crate::semsg;
use crate::tr_plural;
use core::ffi::c_int;

use super::*;
use crate::message::emsg;
use crate::message_fmt::{emsg_text, msg_bytes};
use crate::types::NUL;

// ---------------------------------------------------------------------
// Reporting, and the one place a value this frame owns is freed.
//
// These live beside the `var_check_*` trio rather than in `mod.rs` because
// this is the file whose job is refusing an assignment and saying why; every
// other file of the family reaches them through the `pub use self::store::*`
// re-export.

/// The translation of one of the editor's message strings, which are held as
/// NUL-terminated `static` byte arrays.
///
/// Safe by construction: a `CStr` carries its terminator in the type, and
/// what `gettext` answers is either that `static` or one of its own -- both
/// of which outlive the report it is passed to.
pub(crate) fn translate(msg: &'static CStr) -> &'static CStr {
    gettext(msg)
}

/// Report one of the editor's `static` messages, translated.
pub(crate) fn emsg_static(msg: &'static CStr) {
    emsg(translate(msg));
}

/// Clear a value this frame owns, freeing whatever it holds.
///
/// Safe: `tv_clear`'s only precondition is a live, writable value, which an
/// exclusive borrow of the caller's own local is. Nothing it frees runs user
/// code, so the borrow cannot be re-entered through.
pub(crate) fn clear_local(tv: &mut TypVal) {
    tv_clear(&mut *tv);
}

/// Store `tv` in the variable `name`.
pub(crate) fn set_var(name: &[u8], tv: &mut TypVal, copy: bool) {
    set_var_const(name, tv, copy, false);
}

/// Store `tv` in the variable `name`, creating it if it does not exist.
///
/// `copy` asks for a copy of the value; without it `tv` is moved out of and
/// left `VAR_UNKNOWN`, except for the two scalar types, which are copied
/// either way.  `is_const` is `:const`: the variable is created locked, and
/// an *existing* one is refused outright.
pub(crate) fn set_var_const(name: &[u8], tv: &mut TypVal, copy: bool, is_const: bool) {
    let Some(home) = find_var_home(name).filter(|home| home.name_at < name.len()) else {
        let name = msg_bytes(name);
        semsg!("E461: Illegal variable name: {name}");
        return;
    };
    let varname = &name[home.name_at..];
    // A compat name has no dictionary of its own to watch.
    let watched = home.kind != ScopeKind::Compat && dict_is_watched(Some(&home.dict));

    let mut found = located_item(locate_in(&home, name, true));
    if tv.is_func() {
        // Checking the name can source an autoload script: look again after.
        if var_wrong_func_name_named(name, found.is_none()) {
            return;
        }
        found = located_item(locate_in(&home, name, true));
    }

    // The old value and a copy of the new one, for the watchers: the old
    // one unset for a new variable.
    let (old, new_copy);
    if let Some((dict, slot)) = found {
        if is_const {
            emsg_static(e_cannot_mod);
            return;
        }

        // The order is upstream's and is kept for backwards
        // compatibility: read-only first, then the value's lock, then
        // the variable's.
        let Some((flags, lock)) = dict
            .item_at(slot)
            .map(|item| (c_int::from(item.di_flags), item.di_lock))
        else {
            return;
        };
        if var_check_ro_named(flags, name)
            || value_check_lock(lock, LockName::Bytes(name))
            || var_check_lock_named(flags, name)
        {
            return;
        }

        // A `v:` variable keeps its declared type, and two of them have
        // a side effect on assignment; `before_set_vvar` is both, and it
        // answers when it has already done the store itself.
        if home.kind == ScopeKind::Vim {
            match before_set_vvar(varname, tv, copy, watched) {
                VvarStore::Store => {}
                VvarStore::Done => return,
                VvarStore::TypeError => {
                    let varname = msg_bytes(varname);
                    semsg!("E963: Setting v:{varname} to value with wrong type");
                    return;
                }
            }
        }

        // One borrow of the item: the old value out, the new one in, and
        // the slot unlocked -- upstream leaves the stored value
        // `VAR_UNLOCKED`, and the lock is the item's.
        let new = store_value(tv, copy);
        let Some(replaced) = dict.edit().item_at_mut(slot).map(|item| {
            item.di_lock = VarLock::Unlocked;
            let old = ::core::mem::replace(&mut item.di_tv, new);
            (old, watched.then(|| item.di_tv.clone()))
        }) else {
            return;
        };
        (old, new_copy) = replaced;
    } else {
        // A new variable. `v:` and `a:` do not take one.
        if matches!(home.kind, ScopeKind::Vim | ScopeKind::Args) {
            let name = msg_bytes(name);
            semsg!("E461: Illegal variable name: {name}");
            return;
        }
        if !valid_varname_named(varname) {
            return;
        }
        let mut item = DictItem::boxed(varname);
        if is_const {
            item.di_flags |= DI_FLAGS_LOCK;
        }
        let moved = !(copy || tv.v_type() == VAR_NUMBER || tv.v_type() == VAR_FLOAT);
        item.di_tv = store_value(tv, copy);
        new_copy = watched.then(|| item.di_tv.clone());
        if let Err(mut item) = home.dict.edit().insert(item) {
            // Only a key already there is refused, which the lookup above
            // ruled out; the value goes back to the caller as it came.
            if moved {
                *tv = item.di_tv.take();
            }
            return;
        }
        old = TypVal::Unknown;
    }

    if let Some(new) = new_copy {
        cstr::with_terminated(varname, |key| {
            dict_watcher_notify(&home.dict, key, Some(&new), Some(&old));
        });
    }
    drop(old);

    if is_const {
        // Like `:lockvar! name`: lock the value and what it contains,
        // but only where the reference count is one, so that only
        // literal values are locked. Found again: a watcher may have
        // deleted the variable, and the value is locked out of the item
        // so that no borrow of it is held across the descent.
        let Some((dict, slot)) = located_item(locate_in(&home, name, true)) else {
            return;
        };
        let Some((mut lock, value)) = dict
            .edit()
            .item_at_mut(slot)
            .map(|item| (item.di_lock, item.di_tv.take()))
        else {
            return;
        };
        tv_item_lock(&mut lock, &value, DICT_MAXNEST, true, true);
        if let Some(item) = dict.edit().item_at_mut(slot) {
            item.di_lock = lock;
            item.di_tv = value;
        }
    }
}

/// The dictionary and slot of a found variable; `None` for a miss or a
/// bare scope name.
fn located_item(found: Option<Located>) -> Option<(DictRef, usize)> {
    match found? {
        Located::Item { dict, slot } => Some((dict, slot)),
        Located::Entry(_) => None,
    }
}

/// The value a store puts in the variable: a copy when asked for or a
/// scalar, otherwise moved out of `tv`.
fn store_value(tv: &mut TypVal, copy: bool) -> TypVal {
    if copy || tv.v_type() == VAR_NUMBER || tv.v_type() == VAR_FLOAT {
        tv.clone()
    } else {
        tv.take()
    }
}

/// Whether `flags` says the variable may not be written, reporting E46 or
/// E794 naming `name` if so.
pub(crate) fn var_check_ro_named(flags: c_int, name: &[u8]) -> bool {
    let error_message = if flags & DI_FLAGS_RO as c_int != 0 {
        e_cannot_change_readonly_variable_str
    } else if flags & DI_FLAGS_RO_SBX as c_int != 0 && sandbox.get() != 0 {
        e_cannot_set_variable_in_sandbox_str
    } else {
        return false;
    };
    let len = crate::narrow::len_as_int(name.len());
    emsg_text(tr_plural!(gettext(error_message), len, msg_bytes(name)));
    true
}

/// Whether `flags` says the variable may not be deleted, reporting E795
/// naming `name` if so.
pub(crate) fn var_check_fixed_named(flags: c_int, name: &[u8]) -> bool {
    if flags & DI_FLAGS_FIX as c_int == 0 {
        return false;
    }
    let name = msg_bytes(name);
    semsg!("E795: Cannot delete variable {name}");
    true
}

/// Whether `flags` says the variable is locked, reporting E1122 naming
/// `name` if so.
pub(crate) fn var_check_lock_named(flags: c_int, name: &[u8]) -> bool {
    if flags & DI_FLAGS_LOCK as c_int == 0 {
        return false;
    }
    let name = msg_bytes(name);
    semsg!("E1122: Variable is locked: {name}");
    true
}

/// Whether `name` may not hold a Funcref, reporting E704 or E705 if so.
///
/// A Funcref has to look like a function name -- capitalised, or scoped to
/// `w:`/`b:`/`s:`/`t:`, or autoloaded -- and `new_var` additionally forbids
/// shadowing a function that already exists.
pub(crate) fn var_wrong_func_name_named(name: &[u8], new_var: bool) -> bool {
    let lead = name.first().copied().unwrap_or(NUL as u8);
    let has_scope = lead != NUL as u8 && name.get(1) == Some(&b':');
    // The character the capital is wanted at: past a scope prefix, if
    // there is one.
    let first = match has_scope {
        true => name.get(2).copied().unwrap_or(NUL as u8),
        false => lead,
    };
    let func_scope = has_scope && b"wbst".contains(&lead);

    if !func_scope && !first.is_ascii_uppercase() && !name.contains(&b'#') {
        let name = msg_bytes(name);
        semsg!("E704: Funcref variable name must start with a capital: {name}");
        return true;
    }
    // Don't allow hiding a function. With an existing variable this may
    // be assigning another function to the same one, whose type the
    // caller checks.
    if new_var && function_exists(name, false) {
        let name = msg_bytes(name);
        semsg!("E705: Variable name conflicts with existing function: {name}");
        return true;
    }
    false
}

/// Whether `varname` is spellable as a variable name, reporting E461 if not.
pub(crate) fn valid_varname_named(varname: &[u8]) -> bool {
    for (i, &c) in varname.iter().enumerate() {
        if !eval_isnamec1(c_int::from(c))
            && (i == 0 || !c.is_ascii_digit())
            && c != AUTOLOAD_CHAR as u8
        {
            let varname = msg_bytes(varname);
            semsg!("E461: Illegal variable name: {varname}");
            return false;
        }
    }
    true
}
