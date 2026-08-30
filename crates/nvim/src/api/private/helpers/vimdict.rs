//! Reading and writing a Vimscript dictionary through the API, which is
//! what `nvim_get_var` and its `b:`/`w:`/`t:`/`v:` siblings all come down
//! to. The checks are the interesting part: a key can be read-only, locked,
//! or fixed, the dictionary itself can be locked, and `v:` keys are typed
//! and may run a hook when assigned.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{DI_FLAGS_FIX, DI_FLAGS_LOCK, DI_FLAGS_RO, NIL};
use crate::api::private::converter::{object_to_vim, vim_to_object};
use crate::api_error;
use crate::eval::typval::{
    tv_clear, tv_copy, tv_dict_add, tv_dict_find, tv_dict_is_watched, tv_dict_item_alloc_len,
    tv_dict_item_remove, tv_dict_watcher_notify,
};
use crate::eval::vars::{before_set_vvar, get_vimvar_dict};
use crate::message_fmt::c_str;
use crate::types::{
    Arena, Error, Object, String_0, VAR_UNKNOWN, VarLock, dict_T, dictitem_T, kErrorTypeException,
    kErrorTypeNone, kErrorTypeValidation, ptrdiff_t, size_t, typval_T, typval_vval_union,
};
use core::ffi::c_int;
use core::ptr;

// -- Vimscript dictionaries ------------------------------------------------

/// The value `key` has in `dict`, as an API object. Nil — with `err` set —
/// when the key is absent.
pub(crate) unsafe fn dict_get_value(
    dict: *mut dict_T,
    key: String_0,
    arena: *mut Arena,
    err: &mut Error,
) -> Object {
    // SAFETY: `dict` is a live Vimscript dictionary and `key` borrows the
    // caller's text.
    let di = unsafe { tv_dict_find(dict, key.data(), key.len() as ptrdiff_t) };
    if di.is_null() {
        // SAFETY: `key` borrows the caller's NUL-terminated text.
        let key = unsafe { c_str(key.data()) };
        *err = api_error!(kErrorTypeValidation, "Key not found: {key}");
        return NIL;
    }
    // SAFETY: the lookup answered a live item of `dict`.
    unsafe { vim_to_object(&raw mut (*di).di_tv, arena, true) }
}

/// The item `key` names, having first reported through `err` any reason it
/// could not be assigned to (or, with `del`, removed).
///
/// A null return does not mean failure: an absent key is fine for an
/// assignment. Callers check `err`.
pub(crate) unsafe fn dict_check_writable(
    dict: *mut dict_T,
    key: String_0,
    del: bool,
    err: &mut Error,
) -> *mut dictitem_T {
    // SAFETY: as `dict_get_value`.
    let di = unsafe { tv_dict_find(dict, key.data(), key.len() as ptrdiff_t) };
    if !di.is_null() {
        // SAFETY: the lookup answered a live item.
        let flags = unsafe { (*di).di_flags } as c_int;
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
            // SAFETY: `key` borrows the caller's NUL-terminated text.
            let key = unsafe { c_str(key.data()) };
            *err = api_error!(kErrorTypeException, "Key is {why}: {key}");
        }
        return di;
    }
    // SAFETY: `dict` is a live Vimscript dictionary.
    let refused = if unsafe { (*dict).dv_lock.is_locked() } {
        Some((kErrorTypeException, c"Dict is locked"))
    } else if key.is_empty() {
        Some((kErrorTypeValidation, c"Key name is empty"))
    } else if key.len() > c_int::MAX as size_t {
        Some((kErrorTypeValidation, c"Key name is too long"))
    } else {
        None
    };
    if let Some((kind, msg)) = refused {
        *err = Error::from_message(kind, msg);
    }
    di
}

/// Set or remove `key` in `dict`. With `retval` the previous value comes
/// back, otherwise nil. Fires the dictionary's watchers either way.
pub(crate) unsafe fn dict_set_var(
    dict: *mut dict_T,
    key: String_0,
    value: Object,
    del: bool,
    retval: bool,
    arena: *mut Arena,
    err: &mut Error,
) -> Object {
    let mut rv = NIL;
    // SAFETY: as `dict_get_value`.
    let mut di = unsafe { dict_check_writable(dict, key, del, err) };
    if err.kind() != kErrorTypeNone {
        return rv;
    }
    // SAFETY: `dict` is live.
    let watched = unsafe { tv_dict_is_watched(dict) };

    if del {
        if di.is_null() {
            // SAFETY: `key` borrows the caller's NUL-terminated text.
            let key = unsafe { c_str(key.data()) };
            *err = api_error!(kErrorTypeValidation, "Key not found: {key}");
            return rv;
        }
        // SAFETY: `di` is the live item the lookup found. A raw pointer
        // rather than a borrow, because a watcher runs Lua.
        let old = unsafe { &raw mut (*di).di_tv };
        if watched {
            // SAFETY: as above; a removal has no new value to show.
            unsafe { tv_dict_watcher_notify(dict, key.data(), ptr::null_mut(), old) };
        }
        if retval {
            // SAFETY: as above.
            rv = unsafe { vim_to_object(old, arena, false) };
        }
        // SAFETY: `di` is an item of `dict`.
        unsafe { tv_dict_item_remove(dict, di) };
        return rv;
    }

    let mut tv = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VarLock::Unlocked,
        vval: typval_vval_union { v_number: 0 },
    };
    // SAFETY: `tv` is this frame's and `err` the caller's slot.
    unsafe { object_to_vim(value, &raw mut tv) };
    // Only filled in for a key that already existed; the watchers see an
    // unset value for a key that did not.
    let mut oldtv = typval_T {
        v_type: VAR_UNKNOWN,
        v_lock: VarLock::Unlocked,
        vval: typval_vval_union { v_number: 0 },
    };

    if di.is_null() {
        // SAFETY: `key` names its own bytes and `dict` is live.
        unsafe {
            di = tv_dict_item_alloc_len(key.data(), key.len());
            tv_dict_add(dict, di);
        }
    } else {
        if retval {
            // SAFETY: `di` is the live item the lookup found.
            rv = unsafe { vim_to_object(&raw mut (*di).di_tv, arena, false) };
        }
        // `v:` keys are typed, and some of them run a hook on assignment.
        let mut type_error = false;
        let accepted = dict != get_vimvar_dict() || {
            let (new, bad) = (&raw mut tv, &raw mut type_error);
            // SAFETY: `di` is live, and `tv`/`type_error` are this frame's.
            unsafe { before_set_vvar(key.data(), di, new, true, watched, bad) }
        };
        if !accepted {
            // SAFETY: `tv` is this frame's.
            unsafe { tv_clear(&raw mut tv) };
            if type_error {
                // SAFETY: `key` borrows the caller's NUL-terminated text.
                let key = unsafe { c_str(key.data()) };
                let e = api_error!(
                    kErrorTypeValidation,
                    "Setting v:{key} to value with wrong type"
                );
                *err = e;
            }
            return rv;
        }
        if watched {
            // SAFETY: `di` is live and `oldtv` this frame's.
            unsafe { tv_copy(&raw mut (*di).di_tv, &raw mut oldtv) };
        }
        // SAFETY: `di` is live.
        unsafe { tv_clear(&raw mut (*di).di_tv) };
    }

    // SAFETY: `di` is live and `tv` this frame's.
    unsafe { tv_copy(&raw mut tv, &raw mut (*di).di_tv) };
    if watched {
        // SAFETY: as above, and `oldtv` is this frame's.
        unsafe {
            tv_dict_watcher_notify(dict, key.data(), &raw mut tv, &raw mut oldtv);
            tv_clear(&raw mut oldtv);
        }
    }
    // SAFETY: `tv` is this frame's.
    unsafe { tv_clear(&raw mut tv) };
    rv
}
