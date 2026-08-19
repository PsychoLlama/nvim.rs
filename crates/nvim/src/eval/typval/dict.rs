//! Allocating, freeing and filling a `dict_T`.
//!
//! [`tv_dict_alloc`] and [`tv_dict_unref`] are the reference-counted pair;
//! [`tv_dict_clear`] empties one without freeing it.  The `tv_dict_add_*`
//! family is the C header's overload set, each taking a key by pointer and
//! length and copying exactly that many bytes.  [`tv_dict_extend`] is
//! `extend()` with its three `action` modes, [`tv_dict_copy`] is
//! `copy()`/`deepcopy()` over a dictionary.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::semsg_c;
use crate::types::{CONV_NONE, FAIL, NUL, OK};

/// Allocate a `dictitem_T` sized for a `key_len`-byte key, and copy the key in.
///
/// The item is over-allocated so the NUL-terminated key fits in the `di_key`
/// flexible array member — but never below `size_of::<dictitem_T>()`, which is
/// what upstream's `MAX` guards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_item_alloc_len(
    key: *const ::core::ffi::c_char,
    key_len: size_t,
) -> *mut dictitem_T {
    unsafe {
        let key_offset = ::core::mem::offset_of!(dictitem_T, di_key);
        let size = ::core::mem::size_of::<dictitem_T>().max(key_offset + key_len + 1);
        let di = xmalloc(size) as *mut dictitem_T;
        let di_key = tv_dict_item_key(di);
        memcpy(di_key.cast(), key.cast(), key_len);
        *di_key.add(key_len) = NUL as ::core::ffi::c_char;
        (*di).di_flags = DI_FLAGS_ALLOC as uint8_t;
        (*di).di_tv.v_lock = VAR_UNLOCKED;
        (*di).di_tv.v_type = VAR_UNKNOWN;
        di
    }
}

/// [`tv_dict_item_alloc_len`] for a NUL-terminated key.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_item_alloc(key: *const ::core::ffi::c_char) -> *mut dictitem_T {
    unsafe { tv_dict_item_alloc_len(key, strlen(key)) }
}

/// Clear `item`'s value and free it, if it was allocated (rather than embedded
/// in a `funccall_S`'s fixed-variable array or a scope dictionary).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_item_free(item: *mut dictitem_T) {
    unsafe {
        tv_clear(&raw mut (*item).di_tv);
        if (*item).di_flags as ::core::ffi::c_uint & DI_FLAGS_ALLOC != 0 {
            xfree(item.cast());
        }
    }
}

/// A fresh item holding a copy of `di`'s key and value.
pub unsafe fn tv_dict_item_copy(di: *mut dictitem_T) -> *mut dictitem_T {
    unsafe {
        let new_di = tv_dict_item_alloc(tv_dict_item_key(di));
        tv_copy(&raw mut (*di).di_tv, &raw mut (*new_di).di_tv);
        new_di
    }
}

/// Remove `item` from `dict` and free it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_item_remove(dict: *mut dict_T, item: *mut dictitem_T) {
    unsafe {
        let hi = hash_find(&raw mut (*dict).dv_hashtab, tv_dict_item_key(item));
        if (*hi).is_kept() {
            hash_remove(&raw mut (*dict).dv_hashtab, hi);
        } else {
            semsg_c!(
                gettext(&raw const e_intern2 as *const ::core::ffi::c_char),
                c"tv_dict_item_remove()".as_ptr(),
            );
        }
        tv_dict_item_free(item);
    }
}

/// Allocate an empty dictionary.  The caller owns the reference count.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_alloc() -> *mut dict_T {
    unsafe {
        let d = xcalloc(1, ::core::mem::size_of::<dict_T>()) as *mut dict_T;

        // Prepend the dictionary to the list of dictionaries for garbage
        // collection.
        if let Some(first) = gc_first_dict.get().as_mut() {
            first.dv_used_prev = d;
        }
        (*d).dv_used_next = gc_first_dict.get();
        (*d).dv_used_prev = ::core::ptr::null_mut();
        gc_first_dict.set(d);

        hash_init(&raw mut (*d).dv_hashtab);
        (*d).dv_lock = VAR_UNLOCKED;
        (*d).dv_scope = VAR_NO_SCOPE;
        (*d).dv_refcount = 0;
        (*d).dv_copyID = 0;
        QUEUE_INIT(&raw mut (*d).watchers);
        (*d).lua_table_ref = LUA_NOREF as LuaRef;
        d
    }
}

/// Free every item and watcher of `d`, leaving the `dict_T` itself allocated
/// and empty.
pub unsafe fn tv_dict_free_contents(d: *mut dict_T) {
    unsafe {
        // Lock the hashtab so `hash_remove` below cannot rehash it under the
        // walk.
        hash_lock(&raw mut (*d).dv_hashtab);
        debug_assert!((*d).dv_hashtab.ht_locked > 0);
        for hi in tv_dict_iter(&*d) {
            // Remove the item before freeing it, so that a callback that
            // reaches this dictionary does not see a freed value.
            let di = tv_dict_hi2di(hi);
            hash_remove(&raw mut (*d).dv_hashtab, hi);
            tv_dict_item_free(di);
        }

        while !QUEUE_EMPTY(&raw mut (*d).watchers) {
            let w = (*d).watchers.next;
            QUEUE_REMOVE(w);
            tv_dict_watcher_free(tv_dict_watcher_node_data(w));
        }

        hash_clear(&raw mut (*d).dv_hashtab);
        (*d).dv_hashtab.ht_locked -= 1;
        hash_init(&raw mut (*d).dv_hashtab);
    }
}

/// Unlink `d` from the garbage collector's chain and free the `dict_T` itself.
pub unsafe fn tv_dict_free_dict(d: *mut dict_T) {
    unsafe {
        // Remove the dictionary from the list of dictionaries for garbage
        // collection.
        match (*d).dv_used_prev.as_mut() {
            Some(prev) => prev.dv_used_next = (*d).dv_used_next,
            None => gc_first_dict.set((*d).dv_used_next),
        }
        if let Some(next) = (*d).dv_used_next.as_mut() {
            next.dv_used_prev = (*d).dv_used_prev;
        }

        // NLUA_CLEAR_REF
        if (*d).lua_table_ref != LUA_NOREF {
            api_free_luaref((*d).lua_table_ref);
            (*d).lua_table_ref = LUA_NOREF as LuaRef;
        }
        xfree(d.cast());
    }
}

/// Free `d` and everything in it.  A no-op while `free_unref_items()` is
/// walking, which frees the whole graph itself.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_free(d: *mut dict_T) {
    unsafe {
        if tv_in_free_unref_items.get() {
            return;
        }
        tv_dict_free_contents(d);
        tv_dict_free_dict(d);
    }
}

/// Drop a reference to `d`, freeing it when the last one goes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_unref(d: *mut dict_T) {
    unsafe {
        if let Some(dict) = d.as_mut() {
            dict.dv_refcount -= 1;
            if dict.dv_refcount <= 0 {
                tv_dict_free(d);
            }
        }
    }
}

/// Add `item` to `d`.  `FAIL` when the key is already there, or when it would
/// shadow a builtin function in a scope dictionary.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_add(d: *mut dict_T, item: *mut dictitem_T) -> ::core::ffi::c_int {
    unsafe {
        let key = tv_dict_item_key(item);
        if tv_dict_wrong_func_name(d, &raw mut (*item).di_tv, key) != 0 {
            return FAIL;
        }
        hash_add(&raw mut (*d).dv_hashtab, key)
    }
}

/// Add `list` to `d` under `key`, taking a reference to it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_add_list(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    list: *mut list_T,
) -> ::core::ffi::c_int {
    unsafe {
        let item = tv_dict_item_alloc_len(key, key_len);
        (*item).di_tv.v_type = VAR_LIST;
        (*item).di_tv.vval.v_list = list;
        tv_list_ref(list);
        add_or_free(d, item)
    }
}

/// Add a copy of `tv` to `d` under `key`.
pub unsafe fn tv_dict_add_tv(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    tv: *mut typval_T,
) -> ::core::ffi::c_int {
    unsafe {
        let item = tv_dict_item_alloc_len(key, key_len);
        tv_copy(tv, &raw mut (*item).di_tv);
        add_or_free(d, item)
    }
}

/// Add `dict` to `d` under `key`, taking a reference to it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_add_dict(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    dict: *mut dict_T,
) -> ::core::ffi::c_int {
    unsafe {
        let item = tv_dict_item_alloc_len(key, key_len);
        (*item).di_tv.v_type = VAR_DICT;
        (*item).di_tv.vval.v_dict = dict;
        (*dict).dv_refcount += 1;
        add_or_free(d, item)
    }
}

/// Add the number `nr` to `d` under `key`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_add_nr(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    nr: varnumber_T,
) -> ::core::ffi::c_int {
    unsafe {
        let item = tv_dict_item_alloc_len(key, key_len);
        (*item).di_tv.v_type = VAR_NUMBER;
        (*item).di_tv.vval.v_number = nr;
        add_or_free(d, item)
    }
}

/// Add the float `nr` to `d` under `key`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_add_float(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    nr: float_T,
) -> ::core::ffi::c_int {
    unsafe {
        let item = tv_dict_item_alloc_len(key, key_len);
        (*item).di_tv.v_type = VAR_FLOAT;
        (*item).di_tv.vval.v_float = nr;
        add_or_free(d, item)
    }
}

/// Add the boolean `val` to `d` under `key`.
pub unsafe fn tv_dict_add_bool(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    val: BoolVarValue,
) -> ::core::ffi::c_int {
    unsafe {
        let item = tv_dict_item_alloc_len(key, key_len);
        (*item).di_tv.v_type = VAR_BOOL;
        (*item).di_tv.vval.v_bool = val;
        add_or_free(d, item)
    }
}

/// Add a copy of the NUL-terminated string `val` to `d` under `key`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_add_str(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    val: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe { tv_dict_add_str_len(d, key, key_len, val, -1) }
}

/// Add a copy of `val`'s first `len` bytes to `d` under `key`.  A negative
/// `len` means the whole NUL-terminated string; a NULL `val` stores NULL.
pub unsafe fn tv_dict_add_str_len(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    val: *const ::core::ffi::c_char,
    len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    unsafe {
        let s = if val.is_null() {
            ::core::ptr::null_mut()
        } else if len < 0 {
            xstrdup(val)
        } else {
            xstrndup(val, len as size_t)
        };
        tv_dict_add_allocated_str(d, key, key_len, s)
    }
}

/// Add `val` to `d` under `key`, taking ownership of the allocation.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_add_allocated_str(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    val: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe {
        let item = tv_dict_item_alloc_len(key, key_len);
        (*item).di_tv.v_type = VAR_STRING;
        (*item).di_tv.vval.v_string = val;
        add_or_free(d, item)
    }
}

/// Add a funcref to `fp` to `d` under `key`.
pub unsafe fn tv_dict_add_func(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    fp: *mut ufunc_T,
) -> ::core::ffi::c_int {
    unsafe {
        let item = tv_dict_item_alloc_len(key, key_len);
        (*item).di_tv.v_type = VAR_FUNC;
        (*item).di_tv.vval.v_string =
            xmemdupz((&raw mut (*fp).uf_name).cast(), (*fp).uf_namelen) as *mut ::core::ffi::c_char;
        if tv_dict_add(d, item) == FAIL {
            tv_dict_item_free(item);
            return FAIL;
        }
        func_ref((*item).di_tv.vval.v_string);
        OK
    }
}

/// The tail every `tv_dict_add_*` shares: hand `item` to `d`, or free it again
/// when the key is taken.
#[inline]
unsafe fn add_or_free(d: *mut dict_T, item: *mut dictitem_T) -> ::core::ffi::c_int {
    unsafe {
        if tv_dict_add(d, item) == FAIL {
            tv_dict_item_free(item);
            return FAIL;
        }
        OK
    }
}

/// Free every item of `d`, leaving it allocated and empty.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_clear(d: *mut dict_T) {
    unsafe {
        // Lock the hashtab so `hash_remove` below cannot rehash it under the
        // walk.
        hash_lock(&raw mut (*d).dv_hashtab);
        debug_assert!((*d).dv_hashtab.ht_locked > 0);
        for hi in tv_dict_iter(&*d) {
            tv_dict_item_free(tv_dict_hi2di(hi));
            hash_remove(&raw mut (*d).dv_hashtab, hi);
        }
        hash_unlock(&raw mut (*d).dv_hashtab);
    }
}

/// `extend(d1, d2, action)`: fold `d2`'s items into `d1`.
///
/// `action` is `"keep"`, `"force"` or `"error"`, tested by its first byte —
/// plus the internal `"move"`, which takes each item out of `d2` rather than
/// copying it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_extend(
    d1: *mut dict_T,
    d2: *mut dict_T,
    action: *const ::core::ffi::c_char,
) {
    unsafe {
        let watched = tv_dict_is_watched(d1);
        let arg_errmsg = gettext(c"extend() argument".as_ptr());
        let arg_errmsg_len = strlen(arg_errmsg);
        let action = *action as u8;

        if action == b'm' {
            hash_lock(&raw mut (*d2).dv_hashtab); // don't rehash on hash_remove()
        }

        for hi2 in tv_dict_iter(&*d2) {
            let di2 = tv_dict_hi2di(hi2);
            let di2_key = tv_dict_item_key(di2);
            let di1 = tv_dict_find(d1, di2_key, -1);
            // Check the key to be valid when adding to any scope.
            if (*d1).dv_scope != VAR_NO_SCOPE && !valid_varname(di2_key) {
                break;
            }
            if di1.is_null() {
                if action == b'm' {
                    // Cheap way to move a dict item from "d2" to "d1".
                    // If dict_add() fails then "d2" won't be empty.
                    if tv_dict_add(d1, di2) == OK {
                        hash_remove(&raw mut (*d2).dv_hashtab, hi2);
                        // Note upstream does not gate this on `watched`, unlike
                        // the copying branch below.
                        tv_dict_watcher_notify(
                            d1,
                            di2_key,
                            &raw mut (*di2).di_tv,
                            ::core::ptr::null_mut(),
                        );
                    }
                } else {
                    let new_di = tv_dict_item_copy(di2);
                    if tv_dict_add(d1, new_di) == FAIL {
                        tv_dict_item_free(new_di);
                    } else if watched {
                        tv_dict_watcher_notify(
                            d1,
                            tv_dict_item_key(new_di),
                            &raw mut (*new_di).di_tv,
                            ::core::ptr::null_mut(),
                        );
                    }
                }
            } else if action == b'e' {
                semsg_c!(gettext(c"E737: Key already exists: %s".as_ptr()), di2_key);
                break;
            } else if action == b'f' && di2 != di1 {
                if value_check_lock((*di1).di_tv.v_lock, arg_errmsg, arg_errmsg_len)
                    || var_check_ro(
                        (*di1).di_flags as ::core::ffi::c_int,
                        arg_errmsg,
                        arg_errmsg_len,
                    )
                {
                    break;
                }
                // Disallow replacing a builtin function.
                if tv_dict_wrong_func_name(d1, &raw mut (*di2).di_tv, di2_key) != 0 {
                    break;
                }

                let mut oldtv = TV_INITIAL_VALUE;
                if watched {
                    tv_copy(&raw mut (*di1).di_tv, &raw mut oldtv);
                }

                tv_clear(&raw mut (*di1).di_tv);
                tv_copy(&raw mut (*di2).di_tv, &raw mut (*di1).di_tv);

                if watched {
                    tv_dict_watcher_notify(
                        d1,
                        tv_dict_item_key(di1),
                        &raw mut (*di1).di_tv,
                        &raw mut oldtv,
                    );
                    tv_clear(&raw mut oldtv);
                }
            }
        }

        if action == b'm' {
            hash_unlock(&raw mut (*d2).dv_hashtab);
        }
    }
}

/// Whether `d1` and `d2` hold the same keys with equal values.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_equal(d1: *mut dict_T, d2: *mut dict_T, ic: bool) -> bool {
    unsafe {
        if d1 == d2 {
            return true;
        }
        if tv_dict_len(d1) != tv_dict_len(d2) {
            return false;
        }
        if tv_dict_len(d1) == 0 {
            return true;
        }
        if d1.is_null() || d2.is_null() {
            return false;
        }

        for hi in tv_dict_iter(&*d1) {
            let di1 = tv_dict_hi2di(hi);
            let di2 = tv_dict_find(d2, tv_dict_item_key(di1), -1);
            if di2.is_null() || !tv_equal(&raw mut (*di1).di_tv, &raw mut (*di2).di_tv, ic) {
                return false;
            }
        }
        true
    }
}

/// Copy `orig`, deeply when `deep`, converting keys through `conv`.
///
/// `copyID` is the garbage collector's mark: non-zero records the copy on the
/// original so a self-referencing dictionary resolves to the same copy.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_copy(
    conv: *const vimconv_T,
    orig: *mut dict_T,
    deep: bool,
    copyID: ::core::ffi::c_int,
) -> *mut dict_T {
    unsafe {
        if orig.is_null() {
            return ::core::ptr::null_mut();
        }

        let mut copy = tv_dict_alloc();
        if copyID != 0 {
            (*orig).dv_copyID = copyID;
            (*orig).dv_copydict = copy;
        }
        for hi in tv_dict_iter(&*orig) {
            let di = tv_dict_hi2di(hi);
            if got_int.get() {
                break;
            }
            let new_di = if conv.is_null() || (*conv).vc_type == CONV_NONE {
                tv_dict_item_alloc(tv_dict_item_key(di))
            } else {
                let di_key = tv_dict_item_key(di);
                let mut len = strlen(di_key);
                let key = string_convert(conv, di_key, &raw mut len);
                if key.is_null() {
                    // The conversion failed: keep the original key, but at the
                    // length `string_convert` left behind.
                    tv_dict_item_alloc_len(di_key, len)
                } else {
                    let new_di = tv_dict_item_alloc_len(key, len);
                    xfree(key.cast());
                    new_di
                }
            };
            if deep {
                if var_item_copy(
                    conv,
                    &raw mut (*di).di_tv,
                    &raw mut (*new_di).di_tv,
                    deep,
                    copyID,
                ) == FAIL
                {
                    xfree(new_di.cast());
                    break;
                }
            } else {
                tv_copy(&raw mut (*di).di_tv, &raw mut (*new_di).di_tv);
            }
            if tv_dict_add(copy, new_di) == FAIL {
                tv_dict_item_free(new_di);
                break;
            }
        }

        (*copy).dv_refcount += 1;
        if got_int.get() {
            tv_dict_unref(copy);
            copy = ::core::ptr::null_mut();
        }
        copy
    }
}

/// Mark every key of `dict` read-only and fixed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_set_keys_readonly(dict: *mut dict_T) {
    unsafe {
        for hi in tv_dict_iter(&*dict) {
            let di = tv_dict_hi2di(hi);
            (*di).di_flags |= (DI_FLAGS_RO | DI_FLAGS_FIX) as uint8_t;
        }
    }
}

/// Allocate an empty dictionary with the given lock status.
pub unsafe fn tv_dict_alloc_lock(lock: VarLockStatus) -> *mut dict_T {
    unsafe {
        let d = tv_dict_alloc();
        (*d).dv_lock = lock;
        d
    }
}

/// Allocate an empty dictionary and store it in `ret_tv` as the return value.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tv_dict_alloc_ret(ret_tv: *mut typval_T) {
    unsafe {
        let d = tv_dict_alloc_lock(VAR_UNLOCKED);
        tv_dict_set_ret(ret_tv, d);
    }
}

/// `remove()` over a dictionary: move `argvars[0][argvars[1]]` into `rettv`.
pub unsafe fn tv_dict_remove(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    arg_errmsg: *const ::core::ffi::c_char,
) {
    unsafe {
        if (*argvars.add(2)).v_type != VAR_UNKNOWN {
            semsg_c!(
                gettext(&raw const e_toomanyarg as *const ::core::ffi::c_char),
                c"remove()".as_ptr(),
            );
            return;
        }

        let d = (*argvars).vval.v_dict;
        if d.is_null() || value_check_lock((*d).dv_lock, arg_errmsg, TV_TRANSLATE as size_t) {
            return;
        }
        let key = tv_get_string_chk(argvars.add(1));
        if key.is_null() {
            return;
        }
        let di = tv_dict_find(d, key, -1);
        if di.is_null() {
            semsg_c!(
                gettext(&raw const e_dictkey as *const ::core::ffi::c_char),
                key,
            );
            return;
        }
        let flags = (*di).di_flags as ::core::ffi::c_int;
        if var_check_fixed(flags, arg_errmsg, TV_TRANSLATE as size_t)
            || var_check_ro(flags, arg_errmsg, TV_TRANSLATE as size_t)
        {
            return;
        }

        // Move the value out rather than copying it: `rettv` takes the
        // reference the item held.
        *rettv = (*di).di_tv;
        (*di).di_tv = TV_INITIAL_VALUE;
        tv_dict_item_remove(d, di);
        if tv_dict_is_watched(d) {
            tv_dict_watcher_notify(d, key, ::core::ptr::null_mut(), rettv);
        }
    }
}
