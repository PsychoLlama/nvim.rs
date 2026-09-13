//! A `Dict` and the operations over a whole one.
//!
//! The items themselves, and the slots of the table that names them, are
//! [`dictitem`](super::dictitem)'s; the walk order they give is
//! user-visible, which is what [`Dict::items`] promises and what this
//! module's tests pin.
//!
//! [`tv_dict_alloc`] and [`tv_dict_unref`] are the reference-counted pair;
//! [`Dict::clear`] empties one without freeing it.  The `Dict::add_*` family
//! is the C header's overload set, each taking the key as bytes and copying
//! exactly those.  [`dict_extend`] is `extend()` with its three `action`
//! modes, [`dict_copy`] is `copy()`/`deepcopy()` over a dictionary.
//!
//! Three entry points keep a raw pointer, each because the operation reaches
//! the same dictionary again while it is running: [`dict_extend`] (the two
//! arguments may be one dictionary), [`dict_copy`] (a cycle is read back
//! through the mark this call writes) and
//! [`dict_watcher_notify`](super::dict_watcher_notify) (the callbacks are
//! user code). The allocation pair -- `tv_dict_item_*` -- keeps one for the
//! reason `tv_list_free` does.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use ::core::ffi::CStr;
use ::core::ptr::NonNull;

use super::*;
use crate::cstr;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::types::{CONV_NONE, Failed, Refcount};

impl Dict {
    /// How many entries the dictionary holds.
    pub fn len(&self) -> usize {
        self.dv_hashtab.ht_used
    }

    /// Whether the dictionary holds no entries.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Every entry, in slot order -- which is the order Vim shows.
    ///
    /// The walk borrows the dictionary, so nothing can add to or remove from
    /// it while the walk is live. A body that edits the table wants
    /// [`tv_dict_iter`](super::tv_dict_iter), which is a slot index.
    pub fn items(&self) -> impl Iterator<Item = &DictItem> {
        // SAFETY: a kept slot of a live dictionary names a live item, and
        // the borrow of the dictionary keeps it alive for the walk.
        self.dv_hashtab
            .items()
            .map(|hi| unsafe { &*hi.hi_key.item() })
    }

    /// Every entry, writable, in slot order.
    ///
    /// The items are not in the table's storage, so handing out `&mut` to
    /// each of them in turn does not alias the table itself; what the
    /// exclusive borrow of the dictionary rules out is a body that adds or
    /// removes entries, which is the same rule the table has always had.
    pub fn items_mut(&mut self) -> impl Iterator<Item = &mut DictItem> {
        self.dv_hashtab
            .slots()
            .iter()
            .filter(|hi| hi.is_kept())
            .map(|hi| hi.hi_key.item())
            .collect::<Vec<_>>()
            .into_iter()
            // SAFETY: a kept slot names a live item, no two slots name the
            // same one, and the exclusive borrow keeps the set fixed.
            .map(|di| unsafe { &mut *di })
    }
}

/// An owning reference to a heap-allocated [`Dict`].
///
/// [`ListRef`]'s counterpart, and the same statement about a reference
/// count: [`Clone`] takes one, [`Drop`] gives one back and the last one
/// frees the dictionary. See [`ListRef`] for the whole of it -- the
/// refcount-zero idiom it retires, why `v:_null_dict` is
/// `TypVal::dict(None)` rather than a null handle, and why [`Deref`] is
/// [`Live`](crate::winlayer::Live)'s rather than a borrow held across a
/// call.
///
/// **The scope dictionaries are not heap dictionaries.** `b:`, `w:`, `t:`,
/// `g:`, `v:` and a funccall's `l:`/`a:` live inside the structure that owns
/// them, are seeded with `DO_NOT_FREE_CNT`, and carry `RootId::NONE` because
/// the allocator never handed them out. A handle over one of those is a
/// *borrow* spelled as a handle -- [`DictRef::owning`] -- and the count it
/// takes over is one of the sentinel's; nothing can drive it to zero, and
/// `unref_var_dict` is what gives the whole block back.
///
/// [`ListRef`]: crate::eval::typval::ListRef
/// [`Deref`]: core::ops::Deref
#[repr(transparent)]
pub struct DictRef(NonNull<Dict>);

impl DictRef {
    /// Take over a reference the caller already holds and will not release.
    ///
    /// # Safety
    ///
    /// `d` must point at a live dictionary, and the caller must hold a
    /// reference to it -- one this handle now owns and eventually gives
    /// back.
    #[inline(always)]
    pub unsafe fn from_owned(at: NonNull<Dict>) -> DictRef {
        DictRef(at)
    }

    /// Take over the caller's reference to `d`, or answer `None` for a NULL
    /// dictionary.  See [`DictRef::from_owned`].
    ///
    /// # Safety
    ///
    /// As [`DictRef::from_owned`], for a pointer that may be null.
    #[inline(always)]
    pub unsafe fn owning(d: *mut Dict) -> Option<DictRef> {
        NonNull::new(d).map(DictRef)
    }

    /// Take *another* reference to `d`: the caller keeps its own.
    ///
    /// This is `tv_dict_ref`, with the handle that owes the matching release
    /// as its answer.  `None` for a NULL dictionary, which is
    /// `v:_null_dict` and counts nothing.
    ///
    /// # Safety
    ///
    /// `d` is null or points at a live dictionary.
    #[inline(always)]
    pub unsafe fn retained(d: *mut Dict) -> Option<DictRef> {
        let at = NonNull::new(d)?;
        // SAFETY: the caller's promise: a live dictionary.
        unsafe { Dt::new(d) }.dv_refcount.retain();
        Some(DictRef(at))
    }

    /// The dictionary, as the pointer most of the family still takes.
    ///
    /// A **borrow**: it is live only while the handle is.
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut Dict {
        self.0.as_ptr()
    }

    /// Give the reference up without releasing it: something else owns it
    /// now.
    #[inline(always)]
    pub fn into_raw(self) -> *mut Dict {
        let at = self.0;
        ::core::mem::forget(self);
        at.as_ptr()
    }
}

impl Clone for DictRef {
    /// One more owner of the same dictionary.
    #[inline(always)]
    fn clone(&self) -> DictRef {
        // SAFETY: this handle names a live dictionary, since it holds a
        // reference to it.
        unsafe { Dt::new(self.as_ptr()) }.dv_refcount.retain();
        DictRef(self.0)
    }
}

impl Drop for DictRef {
    /// Give the reference back, freeing the dictionary with the last one.
    #[inline(always)]
    fn drop(&mut self) {
        // SAFETY: this handle names a live dictionary, and is giving up the
        // reference that kept it so.
        unsafe { tv_dict_unref(self.as_ptr()) };
    }
}

impl ::core::ops::Deref for DictRef {
    type Target = Dict;

    #[inline(always)]
    fn deref(&self) -> &Dict {
        // SAFETY: the handle holds a reference, so the dictionary is live;
        // the borrow lasts only as long as the field access that asked for
        // it.
        unsafe { self.0.as_ref() }
    }
}

impl ::core::ops::DerefMut for DictRef {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Dict {
        // SAFETY: as [`DictRef::deref`].
        unsafe { self.0.as_mut() }
    }
}

/// Allocate an empty dictionary, **owned by the handle it answers**.
///
/// The dictionary arrives at a reference count of **one**, held by the
/// [`DictRef`]; upstream handed one back at zero and relied on the first
/// storer to raise it.
///
/// Safe, as [`tv_list_alloc`](super::tv_list_alloc) is: the collector's
/// registry is a `GlobalCell`, so the editor's own thread is the only
/// caller by construction.
pub fn tv_dict_alloc() -> DictRef {
    let d = unsafe { xcalloc(1, ::core::mem::size_of::<Dict>()) } as *mut Dict;

    let at = NonNull::new(d).expect("xcalloc never answers null");
    // The collector reaches every live dictionary through its registry.
    let root = root_dict(at);

    unsafe { hash_init(&raw mut (*d).dv_hashtab) };
    // SAFETY: freshly allocated just above.
    let mut dict = unsafe { Dt::new(d) };
    dict.dv_lock = VarLock::Unlocked;
    dict.dv_scope = VAR_NO_SCOPE;
    dict.dv_refcount = Refcount::ONE;
    dict.dv_copy_id = 0;
    dict.dv_root = root;
    unsafe { queue_init(&raw mut (*d).watchers) };
    dict.lua_table_ref = LUA_NOREF as LuaRef;
    // SAFETY: the reference just seeded is the one this handle owns.
    unsafe { DictRef::from_owned(at) }
}

/// Free every item and watcher of `d`, leaving the `Dict` itself allocated
/// and empty.
///
/// # Safety
/// `d` must point at a live dictionary that nothing else is walking: the
/// hashtab is locked for the walk, so a re-entrant call through a watcher
/// callback would see a half-emptied dictionary.
pub unsafe fn tv_dict_free_contents(d: *mut Dict) {
    // Lock the hashtab so `hash_remove` below cannot rehash it under the
    // walk.
    unsafe { hash_lock(&raw mut (*d).dv_hashtab) };
    // SAFETY: the caller's promise: a live dictionary.
    let mut dict = unsafe { Dt::new(d) };
    debug_assert!(dict.dv_hashtab.ht_locked > 0);
    for hi in unsafe { tv_dict_iter(d) } {
        // Remove the item before freeing it, so that a callback that
        // reaches this dictionary does not see a freed value.
        let di = tv_dict_hi2di(hi);
        unsafe { hash_remove(&raw mut (*d).dv_hashtab, hi) };
        unsafe { tv_dict_item_free(di) };
    }

    while !unsafe { queue_empty(&raw mut (*d).watchers) } {
        let w = dict.watchers.next;
        unsafe { queue_remove(w) };
        unsafe { tv_dict_watcher_free(tv_dict_watcher_node_data(w)) };
    }

    dict.dv_hashtab.ht_locked -= 1;
    // SAFETY: the caller's dictionary, now empty of items.
    hash_reset(unsafe { &mut (*d).dv_hashtab });
}

/// Unlink `d` from the garbage collector's chain and free the `Dict` itself.
///
/// # Safety
/// `d` must point at a live dictionary whose contents have already been
/// freed ([`tv_dict_free_contents`]), and the caller must own the
/// collector's registry, which this takes it out of. `d` is dangling
/// afterwards.
pub unsafe fn tv_dict_free_dict(d: *mut Dict) {
    // Out of the collector's registry. A scope dictionary initialised in
    // place was never in it, and carries `RootId::NONE`.
    // SAFETY: the caller's promise: a live dictionary.
    let mut dict = unsafe { Dt::new(d) };
    unroot_dict(dict.dv_root);
    dict.dv_root = RootId::NONE;

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
pub unsafe fn tv_dict_free(d: *mut Dict) {
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
pub unsafe fn tv_dict_unref(d: *mut Dict) {
    if let Some(dict) = unsafe { d.as_mut() }
        && dict.dv_refcount.release() <= 0
    {
        unsafe { tv_dict_free(d) };
    }
}

impl Dict {
    /// Add `item`, which the dictionary takes over.  `Err` when the key is
    /// already there, or when it would shadow a builtin function in a scope
    /// dictionary — and then `item` is still the caller's to free.
    ///
    /// # Safety
    /// `item` must point at a live item that is in no hashtab.
    pub unsafe fn add_item(&mut self, item: *mut DictItem) -> Result<(), Failed> {
        // SAFETY: the caller's fresh item.
        if dict_wrong_func_name(self, unsafe { &(*item).di_tv }, unsafe { (*item).key() }) {
            return Err(Failed);
        }
        // SAFETY: this dictionary's own table, and an item that is in none.
        unsafe { hash_add(&raw mut self.dv_hashtab, DictEntry::new(item)) }
    }

    /// The tail every `add_*` below shares: hand `item` over, or free it
    /// again when the key is taken.
    ///
    /// # Safety
    /// As [`Dict::add_item`], except that a taken key frees `item` rather
    /// than handing it back — so the caller must not touch it on either
    /// answer.
    #[inline]
    unsafe fn add_or_free(&mut self, item: *mut DictItem) -> Result<(), Failed> {
        // SAFETY: the caller's fresh item.
        if unsafe { self.add_item(item) }.is_err() {
            // SAFETY: the item the add refused, which is in no table.
            unsafe { tv_dict_item_free(item) };
            return Err(Failed);
        }
        Ok(())
    }

    /// A fresh item under `key`, holding `VAR_UNKNOWN` and owned by the
    /// caller.
    #[inline]
    fn fresh_item(key: &[u8]) -> *mut DictItem {
        // SAFETY: `key` is a slice, so it is readable for its own length.
        unsafe { tv_dict_item_alloc_len(key.as_ptr().cast::<::core::ffi::c_char>(), key.len()) }
    }

    /// Add a copy of `tv` under `key`.
    ///
    /// The copy takes its own reference, so `tv` stays the caller's either
    /// way — and the exclusive borrow is what rules out its being a value of
    /// this same dictionary.
    pub fn add_tv(&mut self, key: &[u8], tv: &TypVal) -> Result<(), Failed> {
        let item = Self::fresh_item(key);
        // SAFETY: the item just allocated, which holds `VAR_UNKNOWN`.
        unsafe { tv_copy(tv, &mut (*item).di_tv) };
        // SAFETY: as above — a fresh item in no table.
        unsafe { self.add_or_free(item) }
    }

    /// Add `list` under `key`, taking the handle over.  A failure releases
    /// it with the item.
    pub fn add_list(&mut self, key: &[u8], list: Option<ListRef>) -> Result<(), Failed> {
        let item = Self::fresh_item(key);
        // SAFETY: the item just allocated.
        unsafe { (*item).di_tv.write_list(list) };
        // SAFETY: as above.
        unsafe { self.add_or_free(item) }
    }

    /// Add `dict` under `key`, taking the handle over.  A failure releases
    /// it with the item.
    ///
    /// The handle may name *this* dictionary: it is a reference count and
    /// nothing reads through it here, which is what makes `let d.self = d`
    /// work.
    pub fn add_dict(&mut self, key: &[u8], dict: Option<DictRef>) -> Result<(), Failed> {
        let item = Self::fresh_item(key);
        // SAFETY: the item just allocated.
        unsafe { (*item).di_tv.write_dict(dict) };
        // SAFETY: as above.
        unsafe { self.add_or_free(item) }
    }

    /// Add the number `nr` under `key`.
    pub fn add_number(&mut self, key: &[u8], nr: VarNumber) -> Result<(), Failed> {
        let item = Self::fresh_item(key);
        // SAFETY: the item just allocated.
        unsafe { (*item).di_tv.write_number(nr) };
        // SAFETY: as above.
        unsafe { self.add_or_free(item) }
    }

    /// Add the float `nr` under `key`.
    pub fn add_float(&mut self, key: &[u8], nr: Float) -> Result<(), Failed> {
        let item = Self::fresh_item(key);
        // SAFETY: the item just allocated.
        unsafe { (*item).di_tv.write_float(nr) };
        // SAFETY: as above.
        unsafe { self.add_or_free(item) }
    }

    /// Add the boolean `val` under `key`.
    pub fn add_bool(&mut self, key: &[u8], val: BoolVarValue) -> Result<(), Failed> {
        let item = Self::fresh_item(key);
        // SAFETY: the item just allocated.
        unsafe { (*item).di_tv.write_boolean(val) };
        // SAFETY: as above.
        unsafe { self.add_or_free(item) }
    }

    /// Add a copy of the NUL-terminated string `val` under `key`.
    ///
    /// # Safety
    /// `val` is null or a NUL-terminated string. The string is copied.
    pub unsafe fn add_str(
        &mut self,
        key: &[u8],
        val: *const ::core::ffi::c_char,
    ) -> Result<(), Failed> {
        // SAFETY: the caller's promise about `val`.
        unsafe { self.add_str_len(key, val, -1) }
    }

    /// Add a copy of `val`'s first `len` bytes under `key`.  A negative
    /// `len` means the whole NUL-terminated string; a NULL `val` stores
    /// NULL.
    ///
    /// # Safety
    /// `val` is null, or readable for `len` bytes, or — when `len` is
    /// negative — NUL-terminated. The bytes are copied.
    pub unsafe fn add_str_len(
        &mut self,
        key: &[u8],
        val: *const ::core::ffi::c_char,
        len: ::core::ffi::c_int,
    ) -> Result<(), Failed> {
        let s = if val.is_null() {
            ::core::ptr::null_mut()
        } else if len < 0 {
            // SAFETY: the caller's NUL-terminated string.
            unsafe { xstrdup(val) }
        } else {
            // SAFETY: the caller's `len` readable bytes.
            unsafe { xstrndup(val, len as size_t) }
        };
        // SAFETY: a fresh allocation this call owns and hands on.
        unsafe { self.add_allocated_str(key, s) }
    }

    /// Add `val` under `key`, taking ownership of the allocation.
    ///
    /// # Safety
    /// `val` is null or an allocation from the `xmalloc` family, and **this
    /// takes it over** whether the key was free or not — the caller must not
    /// free it on either answer.
    pub unsafe fn add_allocated_str(
        &mut self,
        key: &[u8],
        val: *mut ::core::ffi::c_char,
    ) -> Result<(), Failed> {
        let item = Self::fresh_item(key);
        // SAFETY: the item just allocated, which takes `val` over.
        unsafe { (*item).di_tv.write_string(val) };
        // SAFETY: as above.
        unsafe { self.add_or_free(item) }
    }

    /// Add a funcref to `func` under `key`.
    ///
    /// Only the name is copied; the funcref counts as a use of the function.
    ///
    /// # Safety
    /// `func` must point at a live `UserFunc` whose `uf_name` is
    /// `uf_namelen` readable bytes.
    pub unsafe fn add_func(&mut self, key: &[u8], func: *mut UserFunc) -> Result<(), Failed> {
        let item = Self::fresh_item(key);
        let name = unsafe { (&raw mut (*func).uf_name).cast() };
        // SAFETY: the caller's promise: a live function.
        let func = unsafe { Live::<UserFunc>::new(func) };
        let namelen = func.uf_namelen;
        // SAFETY: the function's own name, `namelen` bytes of it.
        let owned = unsafe { xmemdupz(name, namelen) } as *mut ::core::ffi::c_char;
        // SAFETY: the item just allocated, which takes the name over.
        unsafe { (*item).di_tv.write_func_name(owned) };
        // SAFETY: a fresh item in no table.
        if unsafe { self.add_item(item) }.is_err() {
            // SAFETY: the item the add refused.
            unsafe { tv_dict_item_free(item) };
            return Err(Failed);
        }
        // SAFETY: the name the item now holds.
        unsafe { func_ref((*item).di_tv.func_name_or_null()) };
        Ok(())
    }
}

impl Dict {
    /// Free every item, leaving the dictionary allocated and empty.
    ///
    /// The exclusive borrow is the whole contract the pointer form spelled
    /// out: nothing else may be walking the table. The hashtab is still
    /// locked for the walk, because *this* walk removes as it goes.
    pub fn clear(&mut self) {
        // SAFETY: this dictionary's own table; the lock is released below.
        unsafe { hash_lock(&raw mut self.dv_hashtab) };
        debug_assert!(self.dv_hashtab.ht_locked > 0);
        // SAFETY: a live dictionary, locked for the walk.
        for hi in unsafe { tv_dict_iter(&raw const *self) } {
            // SAFETY: the walk's own item, unlinked immediately below.
            unsafe { tv_dict_item_free(tv_dict_hi2di(hi)) };
            // SAFETY: the slot the walk is standing on.
            unsafe { hash_remove(&raw mut self.dv_hashtab, hi) };
        }
        // SAFETY: the lock taken above.
        unsafe { hash_unlock(&raw mut self.dv_hashtab) };
    }

    /// Mark every key read-only and fixed.
    pub fn set_keys_readonly(&mut self) {
        for di in self.items_mut() {
            di.di_flags |= (DI_FLAGS_RO | DI_FLAGS_FIX) as uint8_t;
        }
    }

    /// `extend(self, other, action)` for two *different* dictionaries: fold
    /// `other`'s items in.
    ///
    /// `action` is the first byte of `"keep"`, `"force"` or `"error"`, plus
    /// the internal `"move"`, which takes each item out of `other` rather
    /// than copying it — and so needs it writable.
    pub fn extend_from(&mut self, other: &mut Dict, action: u8) {
        // SAFETY: two live dictionaries the borrows name, and both borrows
        // are given up for the call: the body re-enters through
        // `value_check_lock` and through the watcher callbacks, either of
        // which can reach either dictionary again.
        unsafe { dict_extend(&raw mut *self, &raw mut *other, action) };
    }

    /// `extend(d, d, action)`: the case where the two dictionaries are one.
    ///
    /// Every key the walk finds is already there, so the `"keep"`,
    /// `"force"` and `"move"` branches are all no-ops — `"force"` by
    /// upstream's own `di2 != di1` guard — and what is left is `"error"`
    /// reporting the first key, plus the scope check that runs ahead of it.
    pub fn extend_from_self(&mut self, action: u8) {
        let scoped = self.dv_scope != VAR_NO_SCOPE;
        for di in self.items() {
            let key = di.key();
            // SAFETY: the item's own NUL-terminated key.
            if scoped && !unsafe { valid_varname(key.as_ptr()) } {
                break;
            }
            if action == b'e' {
                // SAFETY: as above.
                let key = unsafe { c_str(key.as_ptr()) };
                semsg!("E737: Key already exists: {key}");
                break;
            }
        }
    }
}

/// `extend(d1, d2, action)`: fold `d2`'s items into `d1`.
///
/// The one entry point that still takes pointers, and for the same reason
/// the list's does: **`d1` and `d2` may be the same dictionary**, and the
/// body reaches each of them again while holding an item of the other.
/// [`Dict::extend_from`] and [`Dict::extend_from_self`] are the two cases
/// spelled as borrows; this is what branches between them.
///
/// # Safety
/// `d1` and `d2` must point at live dictionaries. `"move"` empties `d2`, so
/// it must not be the same dictionary as `d1` and must not be locked against
/// a walk.
pub unsafe fn dict_extend(d1: *mut Dict, d2: *mut Dict, action: u8) {
    // SAFETY: the caller's live dictionary.
    let watched = dict_is_watched(unsafe { d1.as_ref() });
    let arg_errmsg = tr(c"extend() argument");
    // SAFETY: a NUL-terminated message from the translation table.
    let arg_errmsg_len = unsafe { cstr::bytes_at(arg_errmsg) }.len();

    if action == b'm' {
        // don't rehash on hash_remove()
        // SAFETY: the caller's live dictionary; unlocked below.
        unsafe { hash_lock(&raw mut (*d2).dv_hashtab) };
    }

    // SAFETY: the caller's live dictionary, which the body writes through.
    for hi2 in unsafe { tv_dict_iter(d2) } {
        let di2 = tv_dict_hi2di(hi2);
        // SAFETY: the walk's own item.
        let di2_key = unsafe { (*di2).key() };
        // SAFETY: the caller's live dictionary. The pointer form is what
        // this branch needs: the item is held across `value_check_lock`,
        // which re-enters, and it is an item of `d1` itself when the two
        // dictionaries are one.
        let di1 = unsafe { (*d1).find_ptr(di2_key.to_bytes()) };
        // Check the key to be valid when adding to any scope.
        // SAFETY: the caller's live dictionary and the item's own key.
        if unsafe { (*d1).dv_scope } != VAR_NO_SCOPE && !unsafe { valid_varname(di2_key.as_ptr()) }
        {
            break;
        }
        if di1.is_null() {
            if action == b'm' {
                // Cheap way to move a dict item from "d2" to "d1".
                // If dict_add() fails then "d2" won't be empty.
                // SAFETY: an item the table is about to give up.
                if unsafe { (*d1).add_item(di2) }.is_ok() {
                    // SAFETY: the slot the walk is standing on.
                    unsafe { hash_remove(&raw mut (*d2).dv_hashtab, hi2) };
                    // Note upstream does not gate this on `watched`, unlike
                    // the copying branch below.
                    // SAFETY: the item just moved into `d1`.
                    unsafe { dict_watcher_notify(d1, di2_key, Some(&*di_tv(di2)), None) };
                }
            } else {
                // SAFETY: the walk's own item.
                let new_di = unsafe { tv_dict_item_copy(di2) };
                // SAFETY: a fresh item in no table.
                if unsafe { (*d1).add_item(new_di) }.is_err() {
                    // SAFETY: the item the add refused.
                    unsafe { tv_dict_item_free(new_di) };
                } else if watched {
                    // SAFETY: the item just added to `d1`.
                    unsafe {
                        let key = (*new_di).key();
                        dict_watcher_notify(d1, key, Some(&*di_tv(new_di)), None);
                    }
                }
            }
        } else if action == b'e' {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let di2_key = unsafe { c_str(di2_key.as_ptr()) };
            semsg!("E737: Key already exists: {di2_key}");
            break;
        } else if action == b'f' && di2 != di1 {
            // SAFETY: the item the lookup found in `d1`.
            if unsafe { value_check_lock((*di1).di_lock, arg_errmsg, arg_errmsg_len) } || {
                // SAFETY: as above.
                let flags = unsafe { (*di1).di_flags } as ::core::ffi::c_int;
                // SAFETY: a NUL-terminated message.
                unsafe { var_check_ro(flags, arg_errmsg, arg_errmsg_len) }
            } {
                break;
            }
            // Disallow replacing a builtin function.
            // SAFETY: the caller's live dictionary and the source item.
            if unsafe { dict_wrong_func_name(&*d1, &(*di2).di_tv, di2_key) } {
                break;
            }

            let mut oldtv = TV_INITIAL_VALUE;
            if watched {
                // SAFETY: the item being overwritten.
                unsafe { tv_copy(&(*di1).di_tv, &mut oldtv) };
            }

            // SAFETY: the two items, each of a live dictionary.
            unsafe {
                tv_clear(&mut (*di1).di_tv);
                tv_copy(&(*di2).di_tv, &mut (*di1).di_tv);
            }

            if watched {
                // SAFETY: the item just overwritten in `d1`.
                unsafe {
                    let key = (*di1).key();
                    dict_watcher_notify(d1, key, Some(&*di_tv(di1)), Some(&oldtv));
                }
                tv_clear(&mut oldtv);
            }
        }
    }

    if action == b'm' {
        // SAFETY: the lock taken above.
        unsafe { hash_unlock(&raw mut (*d2).dv_hashtab) };
    }
}

/// Whether `d1` and `d2` hold the same keys with equal values.
///
/// The two borrows may name one dictionary — comparing a value to itself is
/// a read on both sides — and the identity test is the fast path for that.
/// Comparing values can recurse, so a cycle must already have been ruled out
/// by the caller's `copy_id` bookkeeping.
pub fn dict_equal(d1: Option<&Dict>, d2: Option<&Dict>, ic: bool) -> bool {
    let at = |d: Option<&Dict>| d.map_or(::core::ptr::null(), ::core::ptr::from_ref);
    if at(d1) == at(d2) {
        return true;
    }
    let len1 = dict_len(d1);
    if len1 != dict_len(d2) {
        return false;
    }
    if len1 == 0 {
        return true;
    }
    let (Some(d1), Some(d2)) = (d1, d2) else {
        return false;
    };

    for di1 in d1.items() {
        let Some(di2) = d2.find(di1.key_bytes()) else {
            return false;
        };
        if !tv_equal(&di1.di_tv, &di2.di_tv, ic) {
            return false;
        }
    }
    true
}

/// Copy `orig`, deeply when `deep`, converting keys through `conv`.
///
/// `copy_id` is the garbage collector's mark: non-zero records the copy on
/// the original so a self-referencing dictionary resolves to the same copy.
///
/// **The source stays a pointer**, where every other read of a dictionary in
/// this file is a borrow. A deep copy re-enters through `var_item_copy`, and
/// a dictionary that holds itself is read again from in there — through the
/// `dv_copydict` this call has just written onto it. Neither a shared borrow
/// (which could not write the mark) nor an exclusive one (which the
/// re-entrant read would alias) describes that, and a counted handle would
/// cost a retain and a release on every `copy()` — which is what the list
/// side paid for the same answer.
///
/// # Safety
/// `orig` is null or a live dictionary and `conv` is null or a live
/// converter. A non-zero `copy_id` is written onto `orig`, so it must be one
/// the caller reserved from `get_copyID`; passing a stale one makes an
/// unrelated walk think this dictionary is already visited.
pub unsafe fn dict_copy(
    conv: *const VimConv,
    orig: *mut Dict,
    deep: bool,
    copy_id: ::core::ffi::c_int,
) -> Option<DictRef> {
    if orig.is_null() {
        return None;
    }

    let mut copy = tv_dict_alloc();
    // A borrow of the dictionary the handle owns, for the items to go into.
    let into = copy.as_ptr();
    if copy_id != 0 {
        // SAFETY: the caller's promise: a live dictionary.
        let mut from = unsafe { Dt::new(orig) };
        from.dv_copy_id = copy_id;
        from.dv_copydict = into;
    }
    // SAFETY: the caller's live dictionary; the walk re-enters through
    // `var_item_copy`, so it is driven off the pointer and not a borrow.
    for hi in unsafe { tv_dict_iter(orig) } {
        let di = tv_dict_hi2di(hi);
        if got_int.get() {
            break;
        }
        // SAFETY: the walk's own item.
        let di_key = unsafe { (*di).key() };
        // SAFETY: the caller's converter, read only for its kind.
        let new_di = if conv.is_null() || unsafe { (*conv).vc_type } == CONV_NONE {
            // SAFETY: the item's own NUL-terminated key.
            unsafe { tv_dict_item_alloc(di_key.as_ptr()) }
        } else {
            let mut len = di_key.count_bytes();
            // SAFETY: the caller's converter and the item's own key.
            let key = unsafe { string_convert(conv, di_key.as_ptr().cast_mut(), &raw mut len) };
            if key.is_null() {
                // The conversion failed: keep the original key, but at the
                // length `string_convert` left behind.
                // SAFETY: the item's own key, which is at least that long.
                unsafe { tv_dict_item_alloc_len(di_key.as_ptr(), len) }
            } else {
                // SAFETY: the converted key, `len` bytes of it.
                let new_di = unsafe { tv_dict_item_alloc_len(key, len) };
                // SAFETY: the conversion's own allocation.
                unsafe { xfree(key.cast()) };
                new_di
            }
        };
        if deep {
            let from = di_tv(di);
            let to = di_tv(new_di);
            // SAFETY: the source item's value and the fresh item's slot.
            if unsafe { var_item_copy(conv, &*from, &mut *to, deep, copy_id) }.is_err() {
                // SAFETY: the fresh item, which is in no table.
                unsafe { xfree(new_di.cast()) };
                break;
            }
        } else {
            // SAFETY: as above.
            unsafe { tv_copy(&(*di).di_tv, &mut (*new_di).di_tv) };
        }
        // SAFETY: a fresh item in no table.
        if unsafe { copy.add_item(new_di) }.is_err() {
            // SAFETY: the item the add refused.
            unsafe { tv_dict_item_free(new_di) };
            break;
        }
    }

    if got_int.get() {
        // The partial copy goes with the handle, which is its only
        // reference.
        return None;
    }
    Some(copy)
}

/// Allocate an empty dictionary with the given lock status.
///
/// The handle owns the one reference the dictionary arrives with; see
/// [`tv_dict_alloc`].
pub fn tv_dict_alloc_lock(lock: VarLock) -> DictRef {
    let mut d = tv_dict_alloc();
    d.dv_lock = lock;
    d
}

/// Allocate an empty dictionary and store it in `ret_tv` as the return value.
///
/// # Safety
/// `ret_tv` must point at a writable `TypVal` that holds no value yet —
/// whatever was there is overwritten, not cleared.
pub fn tv_dict_alloc_ret(ret_tv: &mut TypVal) {
    // SAFETY: the allocator is the editor's own, on its own thread.
    ret_tv.write_dict(Some(tv_dict_alloc_lock(VarLock::Unlocked)));
}

/// `remove()` over a dictionary: move `argvars[0][argvars[1]]` into `result`.
///
/// # Safety
/// `args` must point at at least three values, the first a `VAR_DICT`
/// and the third the `VAR_UNKNOWN` terminator when there is no third
/// argument. `result` must be writable and hold no value yet, and
/// `arg_errmsg` must be a NUL-terminated string.
pub unsafe fn tv_dict_remove(
    args: &[TypVal],
    result: &mut TypVal,
    arg_errmsg: *const ::core::ffi::c_char,
) {
    let mut numbuf = NumBuf::new();
    if args.len() > 2 {
        let arg0 = "remove()";
        semsg!("E118: Too many arguments for function: {arg0}");
        return;
    }

    let d = args[0].dict_or_null();
    if d.is_null() || unsafe { value_check_lock((*d).dv_lock, arg_errmsg, TV_TRANSLATE as size_t) }
    {
        return;
    }
    let key = numbuf.string_ptr_chk(&args[1]);
    if key.is_null() {
        return;
    }
    // SAFETY: the dictionary the argument holds, and a NUL-terminated key
    // from the scratch buffer. The pointer form is what this needs: the
    // value is moved out of the item and the item is then unlinked, which
    // reaches the dictionary again.
    let di = unsafe { (*d).find_ptr(cstr::bytes_at(key)) };
    if di.is_null() {
        // SAFETY: a message argument the caller holds as a NUL-terminated string.
        let key = unsafe { c_str(key) };
        semsg!("E716: Key not present in Dictionary: \"{key}\"");
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

    // Move the value out rather than copying it: `result` takes the
    // reference the item held.
    *result = item.di_tv.take();
    // SAFETY: the dictionary the argument holds, and its own item.
    unsafe { tv_dict_item_remove(d, di) };
    // SAFETY: the dictionary the argument holds.
    if dict_is_watched(unsafe { d.as_ref() }) {
        // SAFETY: as above, and a NUL-terminated key from the scratch.
        unsafe { dict_watcher_notify(d, CStr::from_ptr(key), None, Some(result)) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exclusive use of the collector's registries, which every dictionary
    /// allocated below is entered in. See [`crate::eval::gc::serial`].
    fn serial() -> crate::eval::gc::serial::Held {
        crate::eval::gc::serial::lock()
    }

    /// A dictionary holding `keys`, each under its own position as a number.
    fn dict_of(keys: &[&str]) -> DictRef {
        let mut d = tv_dict_alloc();
        for (n, key) in keys.iter().enumerate() {
            let nr = VarNumber::try_from(n).expect("a short dict");
            d.add_number(key.as_bytes(), nr).expect("a key used once");
        }
        d
    }

    /// The keys of `d` in slot order -- the order `keys()` shows.
    fn slot_order(d: &Dict) -> Vec<String> {
        d.items()
            .map(|di| String::from_utf8(di.key_bytes().to_vec()).expect("an ASCII key"))
            .collect()
    }

    /// The number `d[key]` holds, or `None` when there is no such key.
    fn number_at(d: &Dict, key: &[u8]) -> Option<VarNumber> {
        d.find(key).map(|di| di.di_tv.number_or_zero())
    }

    /// **The slot order is user-visible**, so it is pinned here rather than
    /// described: `keys()` shows this, and a change to the hash, the probe
    /// sequence or the resize thresholds would change it.
    ///
    /// The keys straddle the first two rehashes, and are of several shapes
    /// and lengths, which is what makes this more than a spelling of
    /// insertion order -- the answer is neither that nor sorted.
    #[test]
    fn the_slot_order_survives_growth() {
        let _held = serial();
        const KEYS: [&str; 14] = [
            "a",
            "bb",
            "ccc",
            "dddd",
            "k0",
            "k1",
            "k2",
            "k3",
            "k4",
            "k5",
            "zz",
            "Z",
            "_",
            "a_long_key_past_the_inline_cap",
        ];
        let d = dict_of(&KEYS);
        assert_eq!(
            slot_order(&d),
            [
                "dddd",
                "a",
                "zz",
                "a_long_key_past_the_inline_cap",
                "Z",
                "k5",
                "k0",
                "k1",
                "k2",
                "k3",
                "k4",
                "bb",
                "ccc",
                "_",
            ]
        );
        assert_eq!(d.len(), KEYS.len());
    }

    /// A rehash moves *slots*, not items: an item is its own allocation, so
    /// the address a lookup answered before a growth is the address it
    /// answers after one.
    ///
    /// This is what lets [`Dict::find`] hand back a borrow of the item at
    /// all. What the borrow does **not** survive is the item being removed
    /// -- that frees it -- which is why it is a borrow of the dictionary.
    #[test]
    fn an_item_outlives_the_rehash_that_moves_its_slot() {
        let _held = serial();
        let mut d = dict_of(&["first"]);
        let before = d.find_ptr(b"first");
        let slots_before = d.dv_hashtab.size();
        for n in 0..40 {
            let key = format!("k{n}");
            d.add_number(key.as_bytes(), VarNumber::from(n))
                .expect("a key used once");
        }
        assert!(d.dv_hashtab.size() > slots_before, "the table never grew");
        assert_eq!(before, d.find_ptr(b"first"));
        assert_eq!(number_at(&d, b"first"), Some(0));
        assert_eq!(number_at(&d, b"k39"), Some(39));
        assert_eq!(number_at(&d, b"absent"), None);
    }

    /// The empty string is a key like any other: `{'': 1}` holds one entry.
    ///
    /// It is also the one input on which the two hashes the C had disagree
    /// -- and they agree here, because a key that reaches the table is
    /// NUL-terminated and so carries no NUL of its own.
    #[test]
    fn the_empty_key_is_a_key() {
        let _held = serial();
        let d = dict_of(&["", "a"]);
        assert_eq!(d.len(), 2);
        assert_eq!(number_at(&d, b""), Some(0));
        assert_eq!(number_at(&d, b"a"), Some(1));
        assert!(d.has_key(b""));
        assert!(!d.has_key(b"b"));
    }

    /// `extend(d, d)` walks the dictionary it is adding to. Every key it
    /// finds is already there, so nothing is added and nothing overwritten.
    #[test]
    fn extending_a_dictionary_with_itself_is_the_identity() {
        let _held = serial();
        let mut d = dict_of(&["a", "b", "c"]);
        let order = slot_order(&d);
        d.extend_from_self(b'f');
        assert_eq!(slot_order(&d), order);
        assert_eq!(number_at(&d, b"a"), Some(0));
        assert_eq!(number_at(&d, b"c"), Some(2));
    }

    /// And the pointer form still branches to it: the two arguments may be
    /// one dictionary, which is the whole reason it takes pointers.
    #[test]
    fn the_branching_extend_reaches_both_cases() {
        let _held = serial();
        let into = dict_of(&["a"]);
        let from = dict_of(&["b", "c"]);
        // SAFETY: two live dictionaries this case owns.
        unsafe { dict_extend(into.as_ptr(), from.as_ptr(), b'f') };
        assert_eq!(into.len(), 3);
        assert_eq!(number_at(&into, b"b"), Some(0));
        // SAFETY: one live dictionary, named twice.
        unsafe { dict_extend(into.as_ptr(), into.as_ptr(), b'f') };
        assert_eq!(into.len(), 3);
        assert_eq!(number_at(&into, b"a"), Some(0));
    }

    /// A dictionary equals itself without walking into the comparison, and
    /// two dictionaries with the same keys and values are equal whatever
    /// order they were built in.
    #[test]
    fn equality_is_by_key_not_by_slot() {
        let _held = serial();
        let d1 = dict_of(&["a", "b"]);
        assert!(dict_equal(Some(&d1), Some(&d1), false));
        let mut d2 = tv_dict_alloc();
        for (n, key) in [(1, b"b"), (0, b"a")] {
            d2.add_number(key, VarNumber::from(n))
                .expect("a key used once");
        }
        assert!(dict_equal(Some(&d1), Some(&d2), false));
        // `v:_null_dict` is not a two-entry dictionary, but it is an empty
        // one.
        assert!(!dict_equal(Some(&d1), None, false));
        assert!(dict_equal(None, None, false));
    }

    /// A deep copy of a dictionary that holds itself resolves to the *copy*,
    /// not to the original: `copy_id` is what records the answer on the way
    /// down, and the cycle is what would otherwise recurse forever.
    #[test]
    fn a_deep_copy_of_a_cycle_points_at_the_copy() {
        let _held = serial();
        let mut d = dict_of(&["n"]);
        // The value is a second reference to the dictionary itself.
        // SAFETY: a live dictionary, which the handle takes a reference to.
        let held = unsafe { DictRef::retained(d.as_ptr()) };
        d.add_dict(b"self", held).expect("a key used once");

        let copy_id = crate::eval::get_copy_id();
        // SAFETY: a live dictionary, no conversion, and a fresh copy id.
        let mut copy = unsafe { dict_copy(::core::ptr::null(), d.as_ptr(), true, copy_id) }
            .expect("the copy was not interrupted");
        assert_eq!(copy.len(), 2);
        assert_eq!(number_at(&copy, b"n"), Some(0));
        let inner = copy.find(b"self").expect("the copy kept the key");
        assert_eq!(
            inner.di_tv.dict_or_null(),
            copy.as_ptr(),
            "the cycle followed the original"
        );

        // Break the cycles so both dictionaries actually go away.
        d.clear();
        copy.clear();
    }

    /// A walk may remove the entry it is standing on, but only with the
    /// table locked: an unlocked `hash_remove` may rehash and renumber the
    /// slots the cursor is counting through.
    #[test]
    fn a_locked_walk_may_remove_as_it_goes() {
        let _held = serial();
        let d = dict_of(&["a", "b", "c", "d"]);
        // SAFETY: a live dictionary; the lock is released below.
        unsafe { hash_lock(&raw mut (*d.as_ptr()).dv_hashtab) };
        // SAFETY: a live dictionary, locked for the walk.
        for hi in unsafe { tv_dict_iter(d.as_ptr()) } {
            let di = tv_dict_hi2di(hi);
            // SAFETY: one of the dictionary's own items.
            if unsafe { (*di).di_tv.number_or_zero() } % 2 == 0 {
                // SAFETY: the slot the walk is standing on, and its item.
                unsafe { hash_remove(&raw mut (*d.as_ptr()).dv_hashtab, hi) };
                // SAFETY: as above, now out of the table.
                unsafe { tv_dict_item_free(di) };
            }
        }
        // SAFETY: the lock taken above.
        unsafe { hash_unlock(&raw mut (*d.as_ptr()).dv_hashtab) };
        assert_eq!(slot_order(&d), ["b", "d"]);
        assert_eq!(number_at(&d, b"a"), None);
        assert_eq!(number_at(&d, b"d"), Some(3));
    }

    /// [`Dict::clear`] empties a dictionary without freeing it, and the
    /// table it leaves behind takes new keys in the order a fresh one does.
    #[test]
    fn clearing_leaves_a_usable_table() {
        let _held = serial();
        let mut d = dict_of(&["a", "b", "c"]);
        d.clear();
        assert_eq!(d.len(), 0);
        assert!(d.is_empty());
        assert_eq!(number_at(&d, b"a"), None);
        let refilled = dict_of(&["x", "y"]);
        for (n, key) in [b"x", b"y"].iter().enumerate() {
            let nr = VarNumber::try_from(n).expect("a short dict");
            d.add_number(*key, nr).expect("a key used once");
        }
        assert_eq!(slot_order(&d), slot_order(&refilled));
    }
}
