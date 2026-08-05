//! Reading and writing a Vimscript dictionary through the API, which is
//! what `nvim_get_var` and its `b:`/`w:`/`t:`/`v:` siblings all come down
//! to. The checks are the interesting part: a key can be read-only, locked,
//! or fixed, the dictionary itself can be locked, and `v:` keys are typed
//! and may run a hook when assigned.

#![deny(unsafe_op_in_unsafe_fn)]

use super::{DI_FLAGS_FIX, DI_FLAGS_LOCK, DI_FLAGS_RO, NIL, api_set_error};
use crate::src::nvim::api::private::converter::{object_to_vim, vim_to_object};
use crate::src::nvim::eval::typval::{
    tv_clear, tv_copy, tv_dict_add, tv_dict_find, tv_dict_is_watched, tv_dict_item_alloc_len,
    tv_dict_item_remove, tv_dict_watcher_notify,
};
use crate::src::nvim::eval::vars::{before_set_vvar, get_vimvar_dict};
use crate::src::nvim::types::api::{kErrorTypeException, kErrorTypeNone, kErrorTypeValidation};
use crate::src::nvim::types::{
    Arena, Error, Object, String_0, VAR_UNKNOWN, VAR_UNLOCKED, dict_T, dictitem_T, ptrdiff_t,
    size_t, typval_T, typval_vval_union,
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
    err: *mut Error,
) -> Object {
    // SAFETY: `dict` is a live Vimscript dictionary and `key` borrows the
    // caller's text.
    unsafe {
        let di = tv_dict_find(dict, key.data, key.size as ptrdiff_t);
        if di.is_null() {
            api_set_error(
                err,
                kErrorTypeValidation,
                c"Key not found: %s".as_ptr(),
                key.data,
            );
            return NIL;
        }
        vim_to_object(&raw mut (*di).di_tv, arena, true)
    }
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
    err: *mut Error,
) -> *mut dictitem_T {
    // SAFETY: as `dict_get_value`.
    unsafe {
        let di = tv_dict_find(dict, key.data, key.size as ptrdiff_t);
        if !di.is_null() {
            let flags = (*di).di_flags as c_int;
            if flags & DI_FLAGS_RO != 0 {
                api_set_error(
                    err,
                    kErrorTypeException,
                    c"Key is read-only: %s".as_ptr(),
                    key.data,
                );
            } else if flags & DI_FLAGS_LOCK != 0 {
                api_set_error(
                    err,
                    kErrorTypeException,
                    c"Key is locked: %s".as_ptr(),
                    key.data,
                );
            } else if del && flags & DI_FLAGS_FIX != 0 {
                api_set_error(
                    err,
                    kErrorTypeException,
                    c"Key is fixed: %s".as_ptr(),
                    key.data,
                );
            }
        } else if (*dict).dv_lock as u64 != 0 {
            api_set_error(err, kErrorTypeException, c"Dict is locked".as_ptr());
        } else if key.size == 0 {
            api_set_error(err, kErrorTypeValidation, c"Key name is empty".as_ptr());
        } else if key.size > c_int::MAX as size_t {
            api_set_error(err, kErrorTypeValidation, c"Key name is too long".as_ptr());
        }
        di
    }
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
    err: *mut Error,
) -> Object {
    // SAFETY: as `dict_get_value`.
    unsafe {
        let mut rv = NIL;
        let mut di = dict_check_writable(dict, key, del, err);
        if (*err).type_0 != kErrorTypeNone {
            return rv;
        }
        let watched = tv_dict_is_watched(dict);

        if del {
            if di.is_null() {
                api_set_error(
                    err,
                    kErrorTypeValidation,
                    c"Key not found: %s".as_ptr(),
                    key.data,
                );
                return rv;
            }
            if watched {
                tv_dict_watcher_notify(dict, key.data, ptr::null_mut(), &raw mut (*di).di_tv);
            }
            if retval {
                rv = vim_to_object(&raw mut (*di).di_tv, arena, false);
            }
            tv_dict_item_remove(dict, di);
            return rv;
        }

        let mut tv = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };
        object_to_vim(value, &raw mut tv, err);
        // Only filled in for a key that already existed; the watchers see an
        // unset value for a key that did not.
        let mut oldtv = typval_T {
            v_type: VAR_UNKNOWN,
            v_lock: VAR_UNLOCKED,
            vval: typval_vval_union { v_number: 0 },
        };

        if di.is_null() {
            di = tv_dict_item_alloc_len(key.data, key.size);
            tv_dict_add(dict, di);
        } else {
            if retval {
                rv = vim_to_object(&raw mut (*di).di_tv, arena, false);
            }
            // `v:` keys are typed, and some of them run a hook on assignment.
            let mut type_error = false;
            if dict == get_vimvar_dict()
                && !before_set_vvar(
                    key.data,
                    di,
                    &raw mut tv,
                    true,
                    watched,
                    &raw mut type_error,
                )
            {
                tv_clear(&raw mut tv);
                if type_error {
                    let fmt = c"Setting v:%s to value with wrong type".as_ptr();
                    api_set_error(err, kErrorTypeValidation, fmt, key.data);
                }
                return rv;
            }
            if watched {
                tv_copy(&raw mut (*di).di_tv, &raw mut oldtv);
            }
            tv_clear(&raw mut (*di).di_tv);
        }

        tv_copy(&raw mut tv, &raw mut (*di).di_tv);
        if watched {
            tv_dict_watcher_notify(dict, key.data, &raw mut tv, &raw mut oldtv);
            tv_clear(&raw mut oldtv);
        }
        tv_clear(&raw mut tv);
        rv
    }
}
