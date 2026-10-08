//! Reading and writing a Vimscript dictionary through the API, which is
//! what `nvim_get_var` and its `b:`/`w:`/`t:`/`v:` siblings all come down
//! to. The checks are the interesting part: a key can be read-only, locked,
//! or fixed, the dictionary itself can be locked, and `v:` keys are typed
//! and may run a hook when assigned.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]
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

/// The item `key` names, or why it could not be assigned to (or, with `del`,
/// removed).
///
/// `Ok(null)` does not mean failure: an absent key is fine for an assignment.
/// The item is answered by address: the caller writes through it and then
/// reaches `dict` again.
pub(crate) fn dict_check_writable(
    dict: &DictRef,
    key: &String_0,
    del: bool,
) -> Result<*mut DictItem, Error> {
    let di = dict.find_ptr(key.as_bytes());
    if !di.is_null() {
        // SAFETY: the lookup answered a live item.
        let flags = c_int::from(unsafe { (*di).di_flags });
        let refused = if flags & DI_FLAGS_RO != 0 {
            Some("read-only")
        } else if flags & DI_FLAGS_LOCK != 0 {
            Some("locked")
        } else if del && flags & DI_FLAGS_FIX != 0 {
            Some("fixed")
        } else {
            None
        };
        if let Some(why) = refused {
            let key = key.as_cstr().to_string_lossy();
            return Err(api_error!(kErrorTypeException, "Key is {why}: {key}"));
        }
        return Ok(di);
    }
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
        None => Ok(di),
    }
}

/// Set or remove `key` in `dict`. With `retval` the previous value comes
/// back, otherwise nil. Fires the dictionary's watchers either way.
pub(crate) fn dict_set_var(
    dict: &DictRef,
    key: &String_0,
    value: Object,
    del: bool,
    retval: bool,
) -> Result<Object, Error> {
    let mut rv = Object::Nil;
    let mut di = dict_check_writable(dict, key, del)?;
    let watched = dict_is_watched(Some(dict));

    if del {
        if di.is_null() {
            let key = key.as_cstr().to_string_lossy();
            return Err(api_error!(kErrorTypeValidation, "Key not found: {key}"));
        }
        // SAFETY: `di` is the live item the lookup found. A raw pointer
        // rather than a borrow, because a watcher runs Lua.
        let old = unsafe { &raw mut (*di).di_tv };
        if watched {
            // SAFETY: as above; a removal has no new value to show.
            unsafe { dict_watcher_notify(dict, key.as_cstr(), None, Some(&*old)) };
        }
        if retval {
            // SAFETY: as above.
            rv = Object::from(unsafe { &*old });
        }
        tv_dict_item_remove(dict, key.as_bytes());
        return Ok(rv);
    }

    let mut tv = TypVal::from(value);
    // Only filled in for a key that already existed; the watchers see an
    // unset value for a key that did not.
    let mut oldtv = TV_INITIAL_VALUE;

    if di.is_null() {
        let _ = dict.edit().add_item(DictItem::boxed(key.as_bytes()));
        di = dict.find_ptr(key.as_bytes());
    } else {
        if retval {
            // SAFETY: `di` is the live item the lookup found.
            rv = Object::from(unsafe { &(*di).di_tv });
        }
        // `v:` keys are typed, and some of them run a hook on assignment.
        let verdict = if dict.as_ptr() == vimvar_dict().as_ptr() {
            before_set_vvar(key.as_bytes(), &mut tv, true, watched)
        } else {
            VvarStore::Store
        };
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
        if watched {
            // SAFETY: `di` is live and `oldtv` this frame's.
            unsafe { tv_copy(&(*di).di_tv, &mut oldtv) };
        }
        // SAFETY: `di` is live.
        unsafe { tv_clear(&mut (*di).di_tv) };
    }

    // SAFETY: `di` is live and `tv` this frame's.
    unsafe { tv_copy(&tv, &mut (*di).di_tv) };
    if watched {
        // SAFETY: as above, and `oldtv` is this frame's.
        dict_watcher_notify(dict, key.as_cstr(), Some(&tv), Some(&oldtv));
        tv_clear(&mut oldtv);
    }
    tv_clear(&mut tv);
    Ok(rv)
}
