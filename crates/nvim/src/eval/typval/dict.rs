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
use crate::types::{CONV_NONE, FAIL, NUL, OK, Refcount};

/// The dictionary already has an entry under that key — or, in a scope
/// dictionary, the key would shadow a builtin function — so nothing was
/// stored and the value the caller passed is still the caller's to free.
///
/// The `tv_dict_add_*` family is `extern "C"` and pinned by the ABI ledger,
/// so it goes on answering `OK`/`FAIL`. This is the name that `FAIL` has, for
/// Rust callers that convert at the call with [`added`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct KeyTaken;

/// One of the `tv_dict_add_*` answers as a `Result`.
///
/// The conversion belongs at the FFI edge and nowhere else: a caller that
/// writes several keys in a row wants `?`, not a `status == OK` ladder.
pub fn added(status: ::core::ffi::c_int) -> Result<(), KeyTaken> {
    if status == FAIL {
        Err(KeyTaken)
    } else {
        Ok(())
    }
}

/// Allocate a `dictitem_T` sized for a `key_len`-byte key, and copy the key in.
///
/// The item is over-allocated so the NUL-terminated key fits in the `di_key`
/// flexible array member — but never below `size_of::<dictitem_T>()`, which is
/// what upstream's `MAX` guards.
///
/// # Safety
/// `key` must be readable for `key_len` bytes; it need not be
/// NUL-terminated, since the copy terminates itself.
///
/// The item comes back owned by the caller, holding `VAR_UNKNOWN`. It has
/// to reach either [`tv_dict_add`] (which takes it on `OK` and leaves it on
/// `FAIL`) or [`tv_dict_item_free`]; nothing else frees it.
pub unsafe fn tv_dict_item_alloc_len(
    key: *const ::core::ffi::c_char,
    key_len: size_t,
) -> *mut dictitem_T {
    let key_offset = ::core::mem::offset_of!(dictitem_T, di_key);
    let size = ::core::mem::size_of::<dictitem_T>().max(key_offset + key_len + 1);
    let di = unsafe { xmalloc(size) } as *mut dictitem_T;
    let di_key = tv_dict_item_key(di);
    unsafe { memcpy(di_key.cast(), key.cast(), key_len) };
    unsafe { *di_key.add(key_len) = NUL as ::core::ffi::c_char };
    // SAFETY: freshly allocated just above.
    let mut item = unsafe { Di::new(di) };
    item.di_flags = DI_FLAGS_ALLOC as uint8_t;
    item.di_tv.v_lock = VarLock::Unlocked;
    item.di_tv.v_type = VAR_UNKNOWN;
    di
}

/// [`tv_dict_item_alloc_len`] for a NUL-terminated key.
///
/// # Safety
/// `key` must be a NUL-terminated string. Otherwise as
/// [`tv_dict_item_alloc_len`], including the caller's ownership of the
/// result.
pub unsafe fn tv_dict_item_alloc(key: *const ::core::ffi::c_char) -> *mut dictitem_T {
    unsafe { tv_dict_item_alloc_len(key, strlen(key)) }
}

/// Clear `item`'s value and free it, if it was allocated (rather than embedded
/// in a `funccall_S`'s fixed-variable array or a scope dictionary).
///
/// # Safety
/// `item` must be a live item that is **not** in any hashtab — remove it
/// first, or the hashtab is left pointing at freed memory. An item with
/// `DI_FLAGS_ALLOC` is dangling afterwards; one embedded in a `funccall_S`
/// or a scope dictionary is merely emptied.
pub unsafe fn tv_dict_item_free(item: *mut dictitem_T) {
    unsafe { tv_clear(&raw mut (*item).di_tv) };
    if unsafe { (*item).di_flags } as ::core::ffi::c_uint & DI_FLAGS_ALLOC != 0 {
        unsafe { xfree(item.cast()) };
    }
}

/// A fresh item holding a copy of `di`'s key and value.
///
/// # Safety
/// `di` must be a live item. The copy is the caller's, with the same
/// obligation as [`tv_dict_item_alloc_len`]'s result.
pub unsafe fn tv_dict_item_copy(di: *mut dictitem_T) -> *mut dictitem_T {
    let new_di = unsafe { tv_dict_item_alloc(tv_dict_item_key(di)) };
    unsafe { tv_copy(&raw mut (*di).di_tv, &raw mut (*new_di).di_tv) };
    new_di
}

/// Remove `item` from `dict` and free it.
///
/// # Safety
/// `item` must be an item of `dict`, and both must be live. `item` is
/// freed, so the caller must not hold it afterwards.
pub unsafe fn tv_dict_item_remove(dict: *mut dict_T, item: *mut dictitem_T) {
    let hi = unsafe { hash_find(&raw mut (*dict).dv_hashtab, tv_dict_item_key(item)) };
    if unsafe { (*hi).is_kept() } {
        unsafe { hash_remove(&raw mut (*dict).dv_hashtab, hi) };
    } else {
        semsg_c!(tr_bytes(&e_intern2), c"tv_dict_item_remove()".as_ptr(),);
    }
    unsafe { tv_dict_item_free(item) };
}

/// Allocate an empty dictionary.  The caller owns the reference count.
///
/// # Safety
/// The caller must own the garbage collector's dictionary chain — the
/// editor's main thread — because the new dictionary is linked onto
/// `gc_first_dict`.
///
/// The result has a reference count of zero: the caller either raises it
/// or hands the dictionary somewhere that does.
pub unsafe fn tv_dict_alloc() -> *mut dict_T {
    let d = unsafe { xcalloc(1, ::core::mem::size_of::<dict_T>()) } as *mut dict_T;

    // Prepend the dictionary to the list of dictionaries for garbage
    // collection.
    if let Some(first) = unsafe { gc_first_dict.get().as_mut() } {
        first.dv_used_prev = d;
    }
    unsafe { (*d).dv_used_next = gc_first_dict.get() };
    unsafe { (*d).dv_used_prev = ::core::ptr::null_mut() };
    gc_first_dict.set(d);

    unsafe { hash_init(&raw mut (*d).dv_hashtab) };
    // SAFETY: freshly allocated just above.
    let mut dict = unsafe { Dt::new(d) };
    dict.dv_lock = VarLock::Unlocked;
    dict.dv_scope = VAR_NO_SCOPE;
    dict.dv_refcount = Refcount::ZERO;
    dict.dv_copyID = 0;
    unsafe { queue_init(&raw mut (*d).watchers) };
    dict.lua_table_ref = LUA_NOREF as LuaRef;
    d
}

/// Free every item and watcher of `d`, leaving the `dict_T` itself allocated
/// and empty.
///
/// # Safety
/// `d` must point at a live dictionary that nothing else is walking: the
/// hashtab is locked for the walk, so a re-entrant call through a watcher
/// callback would see a half-emptied dictionary.
pub unsafe fn tv_dict_free_contents(d: *mut dict_T) {
    // Lock the hashtab so `hash_remove` below cannot rehash it under the
    // walk.
    unsafe { hash_lock(&raw mut (*d).dv_hashtab) };
    // SAFETY: the caller's promise: a live dictionary.
    let mut dict = unsafe { Dt::new(d) };
    debug_assert!(dict.dv_hashtab.ht_locked > 0);
    for hi in tv_dict_iter(unsafe { &*d }) {
        // Remove the item before freeing it, so that a callback that
        // reaches this dictionary does not see a freed value.
        let di = unsafe { tv_dict_hi2di(hi) };
        unsafe { hash_remove(&raw mut (*d).dv_hashtab, hi) };
        unsafe { tv_dict_item_free(di) };
    }

    while !unsafe { queue_empty(&raw mut (*d).watchers) } {
        let w = dict.watchers.next;
        unsafe { queue_remove(w) };
        unsafe { tv_dict_watcher_free(tv_dict_watcher_node_data(w)) };
    }

    unsafe { hash_clear(&raw mut (*d).dv_hashtab) };
    dict.dv_hashtab.ht_locked -= 1;
    unsafe { hash_init(&raw mut (*d).dv_hashtab) };
}

/// Unlink `d` from the garbage collector's chain and free the `dict_T` itself.
///
/// # Safety
/// `d` must point at a live dictionary whose contents have already been
/// freed ([`tv_dict_free_contents`]), and the caller must own
/// `gc_first_dict`, which this unlinks it from. `d` is dangling
/// afterwards.
pub unsafe fn tv_dict_free_dict(d: *mut dict_T) {
    // Remove the dictionary from the list of dictionaries for garbage
    // collection.
    // SAFETY: the caller's promise: a live dictionary.
    let mut dict = unsafe { Dt::new(d) };
    match unsafe { (*d).dv_used_prev.as_mut() } {
        Some(prev) => prev.dv_used_next = dict.dv_used_next,
        None => gc_first_dict.set(dict.dv_used_next),
    }
    if let Some(next) = unsafe { (*d).dv_used_next.as_mut() } {
        next.dv_used_prev = dict.dv_used_prev;
    }

    // NLUA_CLEAR_REF
    if dict.lua_table_ref != LUA_NOREF {
        unsafe { api_free_luaref((*d).lua_table_ref) };
        dict.lua_table_ref = LUA_NOREF as LuaRef;
    }
    unsafe { xfree(d.cast()) };
}

/// Free `d` and everything in it.  A no-op while `free_unref_items()` is
/// walking, which frees the whole graph itself.
///
/// # Safety
/// `d` must point at a live dictionary that nothing still references.
pub unsafe fn tv_dict_free(d: *mut dict_T) {
    if tv_in_free_unref_items.get() {
        return;
    }
    unsafe { tv_dict_free_contents(d) };
    unsafe { tv_dict_free_dict(d) };
}

/// Drop a reference to `d`, freeing it when the last one goes.
///
/// # Safety
/// `d` is null, or points at a live dictionary of which the caller holds a
/// reference. That reference is given up here, so the caller must not use
/// `d` again.
pub unsafe fn tv_dict_unref(d: *mut dict_T) {
    if let Some(dict) = unsafe { d.as_mut() }
        && dict.dv_refcount.release() <= 0
    {
        unsafe { tv_dict_free(d) };
    }
}

/// Add `item` to `d`.  `FAIL` when the key is already there, or when it would
/// shadow a builtin function in a scope dictionary.
///
/// # Safety
/// `d` must point at a live dictionary and `item` at a fresh item that is
/// in no hashtab. On `OK` the dictionary owns `item`; on `FAIL` it is
/// still the caller's to free.
pub unsafe fn tv_dict_add(d: *mut dict_T, item: *mut dictitem_T) -> ::core::ffi::c_int {
    let key = tv_dict_item_key(item);
    if unsafe { tv_dict_wrong_func_name(d, &raw mut (*item).di_tv, key) } != 0 {
        return FAIL;
    }
    unsafe { hash_add(&raw mut (*d).dv_hashtab, key) }
}

/// Add `list` to `d` under `key`, taking a reference to it.
///
/// # Safety
/// `d` points at a live dictionary, `key` is readable for `key_len` bytes,
/// and `list` is null or a live list. A reference to `list` is taken on
/// success and dropped again on failure.
pub unsafe fn tv_dict_add_list(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    list: *mut list_T,
) -> ::core::ffi::c_int {
    let item = unsafe { tv_dict_item_alloc_len(key, key_len) };
    unsafe { (*item).di_tv = typval_T::list(list) };
    unsafe { tv_list_ref(list) };
    unsafe { add_or_free(d, item) }
}

/// Add a copy of `tv` to `d` under `key`.
///
/// # Safety
/// `d` points at a live dictionary, `key` is readable for `key_len` bytes,
/// and `tv` points at a value that is safe to copy — the copy takes its own
/// reference, so `tv` stays the caller's either way.
pub unsafe fn tv_dict_add_tv(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    tv: *mut typval_T,
) -> ::core::ffi::c_int {
    let item = unsafe { tv_dict_item_alloc_len(key, key_len) };
    unsafe { tv_copy(tv, &raw mut (*item).di_tv) };
    unsafe { add_or_free(d, item) }
}

/// Add `dict` to `d` under `key`, taking a reference to it.
///
/// # Safety
/// `d` points at a live dictionary, `key` is readable for `key_len` bytes,
/// and `dict` points at a live dictionary (not null: the reference count is
/// raised unconditionally).
pub unsafe fn tv_dict_add_dict(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    dict: *mut dict_T,
) -> ::core::ffi::c_int {
    let item = unsafe { tv_dict_item_alloc_len(key, key_len) };
    unsafe { (*item).di_tv = typval_T::dict(dict) };
    unsafe { (*dict).dv_refcount.retain() };
    unsafe { add_or_free(d, item) }
}

/// Add the number `nr` to `d` under `key`.
///
/// # Safety
/// `d` points at a live dictionary and `key` is readable for `key_len`
/// bytes.
pub unsafe fn tv_dict_add_nr(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    nr: varnumber_T,
) -> ::core::ffi::c_int {
    let item = unsafe { tv_dict_item_alloc_len(key, key_len) };
    unsafe { (*item).di_tv = typval_T::number(nr) };
    unsafe { add_or_free(d, item) }
}

/// Add the float `nr` to `d` under `key`.
///
/// # Safety
/// `d` points at a live dictionary and `key` is readable for `key_len`
/// bytes.
pub unsafe fn tv_dict_add_float(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    nr: float_T,
) -> ::core::ffi::c_int {
    let item = unsafe { tv_dict_item_alloc_len(key, key_len) };
    unsafe { (*item).di_tv = typval_T::float(nr) };
    unsafe { add_or_free(d, item) }
}

/// Add the boolean `val` to `d` under `key`.
///
/// # Safety
/// `d` points at a live dictionary and `key` is readable for `key_len`
/// bytes.
pub unsafe fn tv_dict_add_bool(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    val: BoolVarValue,
) -> ::core::ffi::c_int {
    let item = unsafe { tv_dict_item_alloc_len(key, key_len) };
    unsafe { (*item).di_tv = typval_T::boolean(val) };
    unsafe { add_or_free(d, item) }
}

/// Add a copy of the NUL-terminated string `val` to `d` under `key`.
///
/// # Safety
/// `d` points at a live dictionary, `key` is readable for `key_len` bytes,
/// and `val` is null or a NUL-terminated string. The string is copied.
pub unsafe fn tv_dict_add_str(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    val: *const ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    unsafe { tv_dict_add_str_len(d, key, key_len, val, -1) }
}

/// Add a copy of `val`'s first `len` bytes to `d` under `key`.  A negative
/// `len` means the whole NUL-terminated string; a NULL `val` stores NULL.
///
/// # Safety
/// `d` points at a live dictionary and `key` is readable for `key_len`
/// bytes. `val` is null, or readable for `len` bytes, or — when `len` is
/// negative — NUL-terminated. The bytes are copied.
pub unsafe fn tv_dict_add_str_len(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    val: *const ::core::ffi::c_char,
    len: ::core::ffi::c_int,
) -> ::core::ffi::c_int {
    let s = if val.is_null() {
        ::core::ptr::null_mut()
    } else if len < 0 {
        unsafe { xstrdup(val) }
    } else {
        unsafe { xstrndup(val, len as size_t) }
    };
    unsafe { tv_dict_add_allocated_str(d, key, key_len, s) }
}

/// Add `val` to `d` under `key`, taking ownership of the allocation.
///
/// # Safety
/// `d` points at a live dictionary and `key` is readable for `key_len`
/// bytes. `val` is null or an allocation from the `xmalloc` family, and
/// **this takes it over** whether the key was free or not — the caller must
/// not free it on either answer.
pub unsafe fn tv_dict_add_allocated_str(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    val: *mut ::core::ffi::c_char,
) -> ::core::ffi::c_int {
    let item = unsafe { tv_dict_item_alloc_len(key, key_len) };
    unsafe { (*item).di_tv = typval_T::string(val) };
    unsafe { add_or_free(d, item) }
}

/// Add a funcref to `fp` to `d` under `key`.
///
/// # Safety
/// `d` points at a live dictionary, `key` is readable for `key_len` bytes,
/// and `fp` points at a live `ufunc_T` whose `uf_name` is `uf_namelen`
/// readable bytes. Only the name is copied; the funcref counts as a use of
/// the function.
pub unsafe fn tv_dict_add_func(
    d: *mut dict_T,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    fp: *mut ufunc_T,
) -> ::core::ffi::c_int {
    let item = unsafe { tv_dict_item_alloc_len(key, key_len) };
    let name = unsafe { (&raw mut (*fp).uf_name).cast() };
    // SAFETY: the caller's promise: a live function.
    let func = unsafe { Live::<ufunc_T>::new(fp) };
    let namelen = func.uf_namelen;
    let owned = unsafe { xmemdupz(name, namelen) } as *mut ::core::ffi::c_char;
    unsafe {
        (*item).di_tv = typval_T {
            v_type: VAR_FUNC,
            ..typval_T::string(owned)
        }
    };
    if unsafe { tv_dict_add(d, item) } == FAIL {
        unsafe { tv_dict_item_free(item) };
        return FAIL;
    }
    unsafe { func_ref((*item).di_tv.vval.v_string) };
    OK
}

/// The tail every `tv_dict_add_*` shares: hand `item` to `d`, or free it again
/// when the key is taken.
///
/// # Safety
/// As [`tv_dict_add`], except that a taken key frees `item` rather than
/// handing it back — so the caller must not touch `item` after this returns
/// on either answer.
#[inline]
unsafe fn add_or_free(d: *mut dict_T, item: *mut dictitem_T) -> ::core::ffi::c_int {
    if unsafe { tv_dict_add(d, item) } == FAIL {
        unsafe { tv_dict_item_free(item) };
        return FAIL;
    }
    OK
}

/// Free every item of `d`, leaving it allocated and empty.
///
/// # Safety
/// `d` must point at a live dictionary that nothing else is walking; as
/// [`tv_dict_free_contents`], the hashtab is locked for the walk.
pub unsafe fn tv_dict_clear(d: *mut dict_T) {
    // Lock the hashtab so `hash_remove` below cannot rehash it under the
    // walk.
    unsafe { hash_lock(&raw mut (*d).dv_hashtab) };
    debug_assert!(unsafe { (*d).dv_hashtab.ht_locked } > 0);
    for hi in tv_dict_iter(unsafe { &*d }) {
        unsafe { tv_dict_item_free(tv_dict_hi2di(hi)) };
        unsafe { hash_remove(&raw mut (*d).dv_hashtab, hi) };
    }
    unsafe { hash_unlock(&raw mut (*d).dv_hashtab) };
}

/// `extend(d1, d2, action)`: fold `d2`'s items into `d1`.
///
/// `action` is `"keep"`, `"force"` or `"error"`, tested by its first byte —
/// plus the internal `"move"`, which takes each item out of `d2` rather than
/// copying it.
///
/// # Safety
/// `d1` and `d2` must point at live dictionaries and `action` at a string
/// of at least one byte. `"move"` empties `d2`, so it must not be the same
/// dictionary as `d1` and must not be locked against a walk.
pub unsafe fn tv_dict_extend(d1: *mut dict_T, d2: *mut dict_T, action: *const ::core::ffi::c_char) {
    let watched = unsafe { tv_dict_is_watched(d1) };
    let arg_errmsg = tr(c"extend() argument");
    let arg_errmsg_len = unsafe { strlen(arg_errmsg) };
    let action = unsafe { *action } as u8;

    if action == b'm' {
        unsafe { hash_lock(&raw mut (*d2).dv_hashtab) }; // don't rehash on hash_remove()
    }

    for hi2 in tv_dict_iter(unsafe { &*d2 }) {
        let di2 = unsafe { tv_dict_hi2di(hi2) };
        let di2_key = tv_dict_item_key(di2);
        let di1 = unsafe { tv_dict_find(d1, di2_key, -1) };
        // Check the key to be valid when adding to any scope.
        if unsafe { (*d1).dv_scope } != VAR_NO_SCOPE && !unsafe { valid_varname(di2_key) } {
            break;
        }
        if di1.is_null() {
            if action == b'm' {
                // Cheap way to move a dict item from "d2" to "d1".
                // If dict_add() fails then "d2" won't be empty.
                if unsafe { tv_dict_add(d1, di2) } == OK {
                    unsafe { hash_remove(&raw mut (*d2).dv_hashtab, hi2) };
                    // Note upstream does not gate this on `watched`, unlike
                    // the copying branch below.
                    let newtv = di_tv(di2);
                    let nul = ::core::ptr::null_mut();
                    unsafe { tv_dict_watcher_notify(d1, di2_key, newtv, nul) };
                }
            } else {
                let new_di = unsafe { tv_dict_item_copy(di2) };
                if unsafe { tv_dict_add(d1, new_di) } == FAIL {
                    unsafe { tv_dict_item_free(new_di) };
                } else if watched {
                    let key = tv_dict_item_key(new_di);
                    let newtv = di_tv(new_di);
                    let nul = ::core::ptr::null_mut();
                    unsafe { tv_dict_watcher_notify(d1, key, newtv, nul) };
                }
            }
        } else if action == b'e' {
            semsg_c!(tr(c"E737: Key already exists: %s"), di2_key);
            break;
        } else if action == b'f' && di2 != di1 {
            if unsafe { value_check_lock((*di1).di_tv.v_lock, arg_errmsg, arg_errmsg_len) } || {
                let flags = unsafe { (*di1).di_flags } as ::core::ffi::c_int;
                unsafe { var_check_ro(flags, arg_errmsg, arg_errmsg_len) }
            } {
                break;
            }
            // Disallow replacing a builtin function.
            if unsafe { tv_dict_wrong_func_name(d1, &raw mut (*di2).di_tv, di2_key) } != 0 {
                break;
            }

            let mut oldtv = TV_INITIAL_VALUE;
            if watched {
                unsafe { tv_copy(&raw mut (*di1).di_tv, &raw mut oldtv) };
            }

            unsafe { tv_clear(&raw mut (*di1).di_tv) };
            unsafe { tv_copy(&raw mut (*di2).di_tv, &raw mut (*di1).di_tv) };

            if watched {
                let key = tv_dict_item_key(di1);
                let newtv = di_tv(di1);
                unsafe { tv_dict_watcher_notify(d1, key, newtv, &raw mut oldtv) };
                unsafe { tv_clear(&raw mut oldtv) };
            }
        }
    }

    if action == b'm' {
        unsafe { hash_unlock(&raw mut (*d2).dv_hashtab) };
    }
}

/// Whether `d1` and `d2` hold the same keys with equal values.
///
/// # Safety
/// `d1` and `d2` are each null or a live dictionary. Comparing values can
/// recurse, so a cycle must already have been ruled out by the caller's
/// `copyID` bookkeeping.
pub unsafe fn tv_dict_equal(d1: *mut dict_T, d2: *mut dict_T, ic: bool) -> bool {
    if d1 == d2 {
        return true;
    }
    let len1 = unsafe { tv_dict_len(d1) };
    if len1 != unsafe { tv_dict_len(d2) } {
        return false;
    }
    if len1 == 0 {
        return true;
    }
    if d1.is_null() || d2.is_null() {
        return false;
    }

    for hi in tv_dict_iter(unsafe { &*d1 }) {
        let di1 = unsafe { tv_dict_hi2di(hi) };
        let di2 = unsafe { tv_dict_find(d2, tv_dict_item_key(di1), -1) };
        if di2.is_null() || !unsafe { tv_equal(&raw mut (*di1).di_tv, &raw mut (*di2).di_tv, ic) } {
            return false;
        }
    }
    true
}

/// Copy `orig`, deeply when `deep`, converting keys through `conv`.
///
/// `copyID` is the garbage collector's mark: non-zero records the copy on the
/// original so a self-referencing dictionary resolves to the same copy.
///
/// # Safety
/// `orig` is null or a live dictionary and `conv` is null or a live
/// converter. A non-zero `copyID` is written onto `orig`, so it must be one
/// the caller reserved from `get_copyID`; passing a stale one makes an
/// unrelated walk think this dictionary is already visited.
pub unsafe fn tv_dict_copy(
    conv: *const vimconv_T,
    orig: *mut dict_T,
    deep: bool,
    copyID: ::core::ffi::c_int,
) -> *mut dict_T {
    if orig.is_null() {
        return ::core::ptr::null_mut();
    }

    let mut copy = unsafe { tv_dict_alloc() };
    if copyID != 0 {
        // SAFETY: the caller's promise: a live dictionary.
        let mut from = unsafe { Dt::new(orig) };
        from.dv_copyID = copyID;
        from.dv_copydict = copy;
    }
    for hi in tv_dict_iter(unsafe { &*orig }) {
        let di = unsafe { tv_dict_hi2di(hi) };
        if got_int.get() {
            break;
        }
        let new_di = if conv.is_null() || unsafe { (*conv).vc_type } == CONV_NONE {
            unsafe { tv_dict_item_alloc(tv_dict_item_key(di)) }
        } else {
            let di_key = tv_dict_item_key(di);
            let mut len = unsafe { strlen(di_key) };
            let key = unsafe { string_convert(conv, di_key, &raw mut len) };
            if key.is_null() {
                // The conversion failed: keep the original key, but at the
                // length `string_convert` left behind.
                unsafe { tv_dict_item_alloc_len(di_key, len) }
            } else {
                let new_di = unsafe { tv_dict_item_alloc_len(key, len) };
                unsafe { xfree(key.cast()) };
                new_di
            }
        };
        if deep {
            let from = di_tv(di);
            let to = di_tv(new_di);
            if unsafe { var_item_copy(conv, from, to, deep, copyID) } == FAIL {
                unsafe { xfree(new_di.cast()) };
                break;
            }
        } else {
            unsafe { tv_copy(&raw mut (*di).di_tv, &raw mut (*new_di).di_tv) };
        }
        if unsafe { tv_dict_add(copy, new_di) } == FAIL {
            unsafe { tv_dict_item_free(new_di) };
            break;
        }
    }

    unsafe { (*copy).dv_refcount.retain() };
    if got_int.get() {
        unsafe { tv_dict_unref(copy) };
        copy = ::core::ptr::null_mut();
    }
    copy
}

/// Mark every key of `dict` read-only and fixed.
///
/// # Safety
/// `dict` must point at a live dictionary.
pub unsafe fn tv_dict_set_keys_readonly(dict: *mut dict_T) {
    for hi in tv_dict_iter(unsafe { &*dict }) {
        let di = unsafe { tv_dict_hi2di(hi) };
        unsafe { (*di).di_flags |= (DI_FLAGS_RO | DI_FLAGS_FIX) as uint8_t };
    }
}

/// Allocate an empty dictionary with the given lock status.
///
/// # Safety
/// As [`tv_dict_alloc`]: the caller owns `gc_first_dict`, and the result
/// arrives with a reference count of zero.
pub unsafe fn tv_dict_alloc_lock(lock: VarLock) -> *mut dict_T {
    let d = unsafe { tv_dict_alloc() };
    unsafe { (*d).dv_lock = lock };
    d
}

/// Allocate an empty dictionary and store it in `ret_tv` as the return value.
///
/// # Safety
/// `ret_tv` must point at a writable `typval_T` that holds no value yet —
/// whatever was there is overwritten, not cleared.
pub unsafe fn tv_dict_alloc_ret(ret_tv: *mut typval_T) {
    let d = unsafe { tv_dict_alloc_lock(VarLock::Unlocked) };
    unsafe { tv_dict_set_ret(ret_tv, d) };
}

/// `remove()` over a dictionary: move `argvars[0][argvars[1]]` into `rettv`.
///
/// # Safety
/// `argvars` must point at at least three values, the first a `VAR_DICT`
/// and the third the `VAR_UNKNOWN` terminator when there is no third
/// argument. `rettv` must be writable and hold no value yet, and
/// `arg_errmsg` must be a NUL-terminated string.
pub unsafe fn tv_dict_remove(
    argvars: *mut typval_T,
    rettv: *mut typval_T,
    arg_errmsg: *const ::core::ffi::c_char,
) {
    let mut numbuf = NumBuf::new();
    if unsafe { (*argvars.add(2)).v_type } != VAR_UNKNOWN {
        semsg_c!(tr_bytes(&e_toomanyarg), c"remove()".as_ptr(),);
        return;
    }

    let d = unsafe { (*argvars).vval.v_dict };
    if d.is_null() || unsafe { value_check_lock((*d).dv_lock, arg_errmsg, TV_TRANSLATE as size_t) }
    {
        return;
    }
    let key = unsafe { numbuf.string_chk(argvars.add(1)) };
    if key.is_null() {
        return;
    }
    let di = unsafe { tv_dict_find(d, key, -1) };
    if di.is_null() {
        semsg_c!(tr_bytes(&e_dictkey), key,);
        return;
    }
    // SAFETY: the item the lookup just found in `d`.
    let mut item = unsafe { Di::new(di) };
    let flags = item.di_flags as ::core::ffi::c_int;
    if unsafe { var_check_fixed(flags, arg_errmsg, TV_TRANSLATE as size_t) }
        || unsafe { var_check_ro(flags, arg_errmsg, TV_TRANSLATE as size_t) }
    {
        return;
    }

    // Move the value out rather than copying it: `rettv` takes the
    // reference the item held.
    unsafe { *rettv = item.di_tv };
    item.di_tv = TV_INITIAL_VALUE;
    unsafe { tv_dict_item_remove(d, di) };
    if unsafe { tv_dict_is_watched(d) } {
        unsafe { tv_dict_watcher_notify(d, key, ::core::ptr::null_mut(), rettv) };
    }
}
