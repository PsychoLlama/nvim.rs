//! Reading and writing a Vimscript dictionary through the API, which is
//! what `nvim_get_var` and its `b:`/`w:`/`t:`/`v:` siblings all come down
//! to. The checks are the interesting part: a key can be read-only, locked,
//! or fixed, the dictionary itself can be locked, and `v:` keys are typed
//! and may run a hook when assigned.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::{DI_FLAGS_FIX, DI_FLAGS_LOCK, DI_FLAGS_RO};
use crate::api_error;
use crate::eval::typval::DictRef;
use crate::eval::typval::TV_INITIAL_VALUE;
use crate::eval::typval::{
    dict_find, dict_is_watched, dict_watcher_notify, tv_clear, tv_copy, tv_dict_item_remove,
};
use crate::eval::vars::{VvarStore, before_set_vvar, vimvar_dict};
use crate::types::{
    DictItem, Error, Object, String_0, kErrorTypeException, kErrorTypeValidation, size_t,
};
use crate::types::{ScopeDictItem, TypVal};
use core::ffi::c_int;

// -- Vimscript dictionaries ------------------------------------------------

/// The dictionary a buffer's, window's or tab page's scope entry holds.
pub(crate) fn scope_vars(entry: &ScopeDictItem) -> DictRef {
    entry
        .di_tv
        .dict_handle()
        .expect("a live scope holds its dictionary")
}

/// The value `key` has in `dict`, as an API object; an error when the key is
/// absent.
pub(crate) fn dict_get_value(dict: &DictRef, key: &String_0) -> Result<Object, Error> {
    let Some(di) = dict_find(Some(dict), key.as_bytes()) else {
        let key = key.as_cstr().to_string_lossy();
        return Err(api_error!(kErrorTypeValidation, "Key not found: {key}"));
    };
    Ok(Object::from(&di.di_tv))
}

/// Whether `key` is in `dict`, or why it could not be assigned to (or,
/// with `del`, removed). An absent key is fine for an assignment.
pub(crate) fn dict_check_writable(
    dict: &DictRef,
    key: &String_0,
    del: bool,
) -> Result<bool, Error> {
    match dict.find(key.as_bytes()) {
        Some(item) => item_writable(item, key, del).map(|()| true),
        None => absent_writable(dict, key).map(|()| false),
    }
}

/// Why the existing `item` under `key` could not be assigned to (or, with
/// `del`, removed), if anything stops it.
fn item_writable(item: &DictItem, key: &String_0, del: bool) -> Result<(), Error> {
    let flags = c_int::from(item.di_flags);
    let refused = if flags & DI_FLAGS_RO != 0 {
        Some("read-only")
    } else if flags & DI_FLAGS_LOCK != 0 {
        Some("locked")
    } else if del && flags & DI_FLAGS_FIX != 0 {
        Some("fixed")
    } else {
        None
    };
    match refused {
        Some(why) => {
            let key = key.as_cstr().to_string_lossy();
            Err(api_error!(kErrorTypeException, "Key is {why}: {key}"))
        }
        None => Ok(()),
    }
}

/// Why `key`, absent from `dict`, could not be added to it, if anything
/// stops it.
fn absent_writable(dict: &DictRef, key: &String_0) -> Result<(), Error> {
    let refused = if dict.dv_lock.is_locked() {
        Some((kErrorTypeException, c"Dict is locked"))
    } else if key.is_empty() {
        Some((kErrorTypeValidation, c"Key name is empty"))
    } else if key.len() > c_int::MAX as size_t {
        Some((kErrorTypeValidation, c"Key name is too long"))
    } else {
        None
    };
    match refused {
        Some((kind, msg)) => Err(Error::from_message(kind, msg)),
        None => Ok(()),
    }
}

/// Set or remove `key` in `dict`. With `retval` the previous value comes
/// back, otherwise nil. Fires the dictionary's watchers either way.
///
/// The item is found again by key after every step that can run user code
/// (a watcher is Vimscript or Lua): nothing holds it across one.
///
/// An assignment to anything but `v:` is [`dict_assign`]; what is left here
/// is removal and `v:`, whose typed keys and hooks need the item re-found.
pub(crate) fn dict_set_var(
    dict: &DictRef,
    key: &String_0,
    value: Object,
    del: bool,
    retval: bool,
) -> Result<Object, Error> {
    if !del && !dict.ptr_eq(&vimvar_dict()) {
        return dict_assign(dict, key, value, retval);
    }
    let mut rv = Object::Nil;
    let exists = dict_check_writable(dict, key, del)?;
    let watched = dict_is_watched(Some(dict));
    let current = |dict: &DictRef| {
        dict.find(key.as_bytes())
            .map(|item| Object::from(&item.di_tv))
    };

    if del {
        if !exists {
            let key = key.as_cstr().to_string_lossy();
            return Err(api_error!(kErrorTypeValidation, "Key not found: {key}"));
        }
        if watched {
            // The watchers are shown a copy: they may do anything to the
            // item, a removal has no new value to show.
            let mut old = TV_INITIAL_VALUE;
            if let Some(item) = dict.find(key.as_bytes()) {
                tv_copy(&item.di_tv, &mut old);
            }
            dict_watcher_notify(dict, key.as_cstr(), None, Some(&old));
            tv_clear(&mut old);
        }
        if retval {
            // What the item holds after its watchers ran, as upstream read it.
            rv = current(dict).unwrap_or(Object::Nil);
        }
        tv_dict_item_remove(dict, key.as_bytes());
        return Ok(rv);
    }

    let mut tv = TypVal::from(value);
    // Only filled in for a key that already existed; the watchers see an
    // unset value for a key that did not.
    let mut oldtv = TV_INITIAL_VALUE;

    if exists {
        if retval {
            rv = current(dict).unwrap_or(Object::Nil);
        }
        // Past here the dictionary is `v:`: its keys are typed, and some of
        // them run a hook on assignment.
        let verdict = before_set_vvar(key.as_bytes(), &mut tv, true, watched);
        if verdict != VvarStore::Store {
            tv_clear(&mut tv);
            if verdict == VvarStore::TypeError {
                let key = key.as_cstr().to_string_lossy();
                return Err(api_error!(
                    kErrorTypeValidation,
                    "Setting v:{key} to value with wrong type"
                ));
            }
            return Ok(rv);
        }
    }
    match dict.edit().find_mut(key.as_bytes()) {
        Some(item) => {
            if watched {
                tv_copy(&item.di_tv, &mut oldtv);
            }
            tv_clear(&mut item.di_tv);
            tv_copy(&tv, &mut item.di_tv);
        }
        None => {
            // Added empty and then filled, as upstream does: adding a Funcref
            // under a lower-case name would be refused (E704).
            let _ = dict.edit().add_item(DictItem::boxed(key.as_bytes()));
            if let Some(item) = dict.edit().find_mut(key.as_bytes()) {
                tv_copy(&tv, &mut item.di_tv);
            }
        }
    }
    if watched {
        dict_watcher_notify(dict, key.as_cstr(), Some(&tv), Some(&oldtv));
        tv_clear(&mut oldtv);
    }
    tv_clear(&mut tv);
    Ok(rv)
}

/// [`dict_set_var`]'s assignment to anything but `v:`, which no hook
/// stands between: the item is found once and written in place. Nothing
/// between the lookup and the write runs user code; the watchers run after,
/// with no item held. With `retval` the previous value comes back.
fn dict_assign(
    dict: &DictRef,
    key: &String_0,
    value: Object,
    retval: bool,
) -> Result<Object, Error> {
    let watched = dict_is_watched(Some(dict));
    let mut tv = TypVal::from(value);
    // Only filled in for a key that already existed; the watchers see an
    // unset value for a key that did not.
    let mut oldtv = TV_INITIAL_VALUE;
    let mut rv = Object::Nil;
    if let Some(item) = dict.edit().find_mut(key.as_bytes()) {
        if let Err(err) = item_writable(item, key, false) {
            tv_clear(&mut tv);
            return Err(err);
        }
        if retval {
            rv = Object::from(&item.di_tv);
        }
        if watched {
            tv_copy(&item.di_tv, &mut oldtv);
        }
        tv_clear(&mut item.di_tv);
        tv_copy(&tv, &mut item.di_tv);
    } else {
        if let Err(err) = absent_writable(dict, key) {
            tv_clear(&mut tv);
            return Err(err);
        }
        // Added empty and then filled, as upstream does: adding a Funcref
        // under a lower-case name would be refused (E704).
        let _ = dict.edit().add_item(DictItem::boxed(key.as_bytes()));
        if let Some(item) = dict.edit().find_mut(key.as_bytes()) {
            tv_copy(&tv, &mut item.di_tv);
        }
    }
    if watched {
        dict_watcher_notify(dict, key.as_cstr(), Some(&tv), Some(&oldtv));
        tv_clear(&mut oldtv);
    }
    tv_clear(&mut tv);
    Ok(rv)
}
