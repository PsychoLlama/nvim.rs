//! A `Dict`, its hash table, and the items it owns.
//!
//! The table's slots name the items ([`DictEntry`]), where upstream's named
//! the item's *key* -- an item was over-allocated so its key sat in a
//! flexible tail, and the item was the key pointer minus a constant. That is
//! why an item could not own its key. Reading the key out of the entry costs
//! the probe nothing -- [`DictEntry::key`] answers the same NUL-terminated
//! bytes the slot used to hold -- and so leaves the hash, the probe
//! sequence, the resize thresholds and the slot every key lands in exactly
//! as they were. That matters: slot order is what `keys()`, `values()` and
//! `items()` show.
//!
//! The table does not free its items. Which of them it owns is
//! `DI_FLAGS_ALLOC`, and [`tv_dict_free_contents`] is what acts on it --
//! four kinds of item are embedded in something bigger (a funccall's fixed
//! variables, a scope's own entry, `b:changedtick`, a `v:` row) and outlive
//! every table they are in.
//!
//! [`tv_dict_alloc`] and [`tv_dict_unref`] are the reference-counted pair;
//! [`tv_dict_clear`] empties one without freeing it.  The `tv_dict_add_*`
//! family is the C header's overload set, each taking a key by pointer and
//! length and copying exactly that many bytes.  [`tv_dict_extend`] is
//! `extend()` with its three `action` modes, [`tv_dict_copy`] is
//! `copy()`/`deepcopy()` over a dictionary.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use ::core::ptr::NonNull;

use super::*;
use crate::cstr;
use crate::hashtab::removed_sentinel;
use crate::message_fmt::c_str;
use crate::semsg;
use crate::types::{CONV_NONE, DictKey, Failed, HashTab, Refcount, SlotEntry};

/// What a dictionary's hash table holds in an occupied slot: the item.
///
/// `Copy`, and a raw pointer, for the same reason the slots of every other
/// table are: the table names its items, and which of them it is responsible
/// for freeing is the item's own `DI_FLAGS_ALLOC`.
#[derive(Copy, Clone, PartialEq, Eq)]
pub struct DictEntry(*mut DictItem);

impl DictEntry {
    /// The entry for `item`.
    pub fn new(item: *mut DictItem) -> Self {
        DictEntry(item)
    }

    /// The item this entry names. Meaningless for an empty or removed slot.
    pub fn item(self) -> *mut DictItem {
        self.0
    }
}

// SAFETY: `EMPTY` is the null pointer and `is_empty` is the null test; the
// tombstone is the hash table's own private sentinel address, which no item
// can be allocated at and which is never dereferenced; `key` reads the key
// of an item the caller has promised is live, and a `DictItem` owns its key
// for as long as it is alive.
unsafe impl SlotEntry for DictEntry {
    const EMPTY: Self = DictEntry(::core::ptr::null_mut());

    fn is_empty(self) -> bool {
        self.0.is_null()
    }

    fn is_removed(self) -> bool {
        self.0 == removed_sentinel().cast::<DictItem>()
    }

    fn removed() -> Self {
        DictEntry(removed_sentinel().cast::<DictItem>())
    }

    /// # Safety
    ///
    /// As the trait's: a live entry, whose item is still alive.
    unsafe fn key(self) -> *const ::core::ffi::c_char {
        // SAFETY: the caller's promise -- a live item, which owns its key.
        unsafe { (*self.0).di_key.as_ptr() }
    }
}

/// A dictionary's hash table.
pub type DictTab = HashTab<DictEntry>;

/// One slot of a dictionary's hash table.
pub type ItemSlot = Slot<DictEntry>;

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

/// The dictionary already has an entry under that key — or, in a scope
/// dictionary, the key would shadow a builtin function — so nothing was
/// stored and the value the caller passed is still the caller's to free.
///
/// The `tv_dict_add_*` family answers the anonymous [`Failed`]; this is the
/// name that failure has for a caller who is filling a dictionary, and the
/// `From` below is what lets one `?` into the other.
#[derive(Clone, Copy, PartialEq, Eq, Debug, thiserror::Error)]
#[error("the key was already in the dictionary")]
pub struct KeyTaken;

impl From<Failed> for KeyTaken {
    fn from(_: Failed) -> Self {
        KeyTaken
    }
}

/// Allocate a `DictItem` holding a copy of `key`'s first `key_len` bytes.
///
/// The item owns its key: a short one -- which is nearly every key -- lives
/// in the item, a long one in its own allocation.  Upstream over-allocated
/// the item so the key sat in a flexible tail, because its hash table's slot
/// pointed at the key and found the item by subtracting an offset; the slot
/// names the item now.
///
/// # Safety
/// `key` must be readable for `key_len` bytes; it need not be
/// NUL-terminated, since the key terminates itself.
///
/// The item comes back owned by the caller, holding `VAR_UNKNOWN`. It has
/// to reach either [`tv_dict_add`] (which takes it on `OK` and leaves it on
/// `FAIL`) or [`tv_dict_item_free`]; nothing else frees it.
pub unsafe fn tv_dict_item_alloc_len(
    key: *const ::core::ffi::c_char,
    key_len: size_t,
) -> *mut DictItem {
    // SAFETY: the caller's promise -- `key_len` readable bytes.
    let bytes = unsafe { ::core::slice::from_raw_parts(key.cast::<u8>(), key_len) };
    Box::into_raw(Box::new(DictItem {
        di_tv: TypVal::Unknown,
        di_lock: VarLock::Unlocked,
        di_flags: DI_FLAGS_ALLOC as uint8_t,
        di_key: DictKey::new(bytes),
    }))
}

/// [`tv_dict_item_alloc_len`] for a NUL-terminated key.
///
/// # Safety
/// `key` must be a NUL-terminated string. Otherwise as
/// [`tv_dict_item_alloc_len`], including the caller's ownership of the
/// result.
pub unsafe fn tv_dict_item_alloc(key: *const ::core::ffi::c_char) -> *mut DictItem {
    unsafe { tv_dict_item_alloc_len(key, cstr::bytes_at(key).len()) }
}

/// Clear `item`'s value and free it, if it was allocated (rather than
/// embedded in a `FuncCall`'s fixed-variable array, a scope dictionary, a
/// buffer's `b:changedtick` or a `v:` row).
///
/// # Safety
/// `item` must be a live item that is **not** in any hashtab -- remove it
/// first, or the hashtab is left pointing at freed memory. An item with
/// `DI_FLAGS_ALLOC` is dangling afterwards; an embedded one is merely
/// emptied, keeping its key: the storage is not this call's to release.
pub unsafe fn tv_dict_item_free(item: *mut DictItem) {
    if unsafe { (*item).di_flags } as ::core::ffi::c_uint & DI_FLAGS_ALLOC != 0 {
        // The value and the key go with the item.
        // SAFETY: the caller's live item, which this allocated.
        drop(unsafe { Box::from_raw(item) });
    } else {
        unsafe { tv_clear(&mut (*item).di_tv) };
    }
}

/// A fresh item holding a copy of `di`'s key and value.
///
/// # Safety
/// `di` must be a live item. The copy is the caller's, with the same
/// obligation as [`tv_dict_item_alloc_len`]'s result.
pub unsafe fn tv_dict_item_copy(di: *mut DictItem) -> *mut DictItem {
    let new_di = unsafe { tv_dict_item_alloc(tv_dict_item_key(di)) };
    unsafe { tv_copy(&(*di).di_tv, &mut (*new_di).di_tv) };
    new_di
}

/// Remove `item` from `dict` and free it.
///
/// # Safety
/// `item` must be an item of `dict`, and both must be live. `item` is
/// freed, so the caller must not hold it afterwards.
pub unsafe fn tv_dict_item_remove(dict: *mut Dict, item: *mut DictItem) {
    let hi = unsafe { hash_find(&raw mut (*dict).dv_hashtab, tv_dict_item_key(item)) };
    if hi.is_kept() {
        unsafe { hash_remove(&raw mut (*dict).dv_hashtab, hi) };
    } else {
        let arg0 = "tv_dict_item_remove()";
        semsg!("E685: Internal error: {arg0}");
    }
    unsafe { tv_dict_item_free(item) };
}

/// An owning reference to a heap-allocated [`Dict`].
///
/// [`ListRef`]'s counterpart, and the same statement about a reference
/// count: [`Clone`] takes one, [`Drop`] gives one back and the last one
/// frees the dictionary. See [`ListRef`] for the whole of it -- the
/// refcount-zero idiom it retires, why `v:_null_dict` is
/// `TypVal::Dict(None)` rather than a null handle, and why [`Deref`] is
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

/// Add `item` to `d`.  `Err` when the key is already there, or when it would
/// shadow a builtin function in a scope dictionary.
///
/// # Safety
/// `d` must point at a live dictionary and `item` at a fresh item that is
/// in no hashtab. On `Ok` the dictionary owns `item`; on `Err` it is
/// still the caller's to free.
pub unsafe fn tv_dict_add(d: *mut Dict, item: *mut DictItem) -> Result<(), Failed> {
    let key = unsafe { tv_dict_item_key(item) };
    if unsafe { tv_dict_wrong_func_name(d, &mut (*item).di_tv, key) } != 0 {
        return Err(Failed);
    }
    unsafe { hash_add(&raw mut (*d).dv_hashtab, DictEntry::new(item)) }
}

/// Add `list` to `d` under `key`, which takes the handle over.
///
/// # Safety
/// `d` points at a live dictionary and `key` is readable for `key_len`
/// bytes. A failure releases the handle with the item.
pub unsafe fn tv_dict_add_list(
    d: *mut Dict,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    list: Option<ListRef>,
) -> Result<(), Failed> {
    let item = unsafe { tv_dict_item_alloc_len(key, key_len) };
    unsafe { (*item).di_tv.write_list(list) };
    unsafe { add_or_free(d, item) }
}

/// Add a copy of `tv` to `d` under `key`.
///
/// # Safety
/// `d` points at a live dictionary, `key` is readable for `key_len` bytes,
/// and `tv` points at a value that is safe to copy — the copy takes its own
/// reference, so `tv` stays the caller's either way.
pub unsafe fn tv_dict_add_tv(
    d: *mut Dict,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    tv: &mut TypVal,
) -> Result<(), Failed> {
    let item = unsafe { tv_dict_item_alloc_len(key, key_len) };
    unsafe { tv_copy(tv, &mut (*item).di_tv) };
    unsafe { add_or_free(d, item) }
}

/// Add `dict` to `d` under `key`, which takes the handle over.
///
/// # Safety
/// `d` points at a live dictionary and `key` is readable for `key_len`
/// bytes. A failure releases the handle with the item.
pub unsafe fn tv_dict_add_dict(
    d: *mut Dict,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    dict: Option<DictRef>,
) -> Result<(), Failed> {
    let item = unsafe { tv_dict_item_alloc_len(key, key_len) };
    unsafe { (*item).di_tv.write_dict(dict) };
    unsafe { add_or_free(d, item) }
}

/// Add the number `nr` to `d` under `key`.
///
/// # Safety
/// `d` points at a live dictionary and `key` is readable for `key_len`
/// bytes.
pub unsafe fn tv_dict_add_nr(
    d: *mut Dict,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    nr: VarNumber,
) -> Result<(), Failed> {
    let item = unsafe { tv_dict_item_alloc_len(key, key_len) };
    unsafe { (*item).di_tv.write_number(nr) };
    unsafe { add_or_free(d, item) }
}

/// Add the float `nr` to `d` under `key`.
///
/// # Safety
/// `d` points at a live dictionary and `key` is readable for `key_len`
/// bytes.
pub unsafe fn tv_dict_add_float(
    d: *mut Dict,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    nr: Float,
) -> Result<(), Failed> {
    let item = unsafe { tv_dict_item_alloc_len(key, key_len) };
    unsafe { (*item).di_tv.write_float(nr) };
    unsafe { add_or_free(d, item) }
}

/// Add the boolean `val` to `d` under `key`.
///
/// # Safety
/// `d` points at a live dictionary and `key` is readable for `key_len`
/// bytes.
pub unsafe fn tv_dict_add_bool(
    d: *mut Dict,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    val: BoolVarValue,
) -> Result<(), Failed> {
    let item = unsafe { tv_dict_item_alloc_len(key, key_len) };
    unsafe { (*item).di_tv.write_boolean(val) };
    unsafe { add_or_free(d, item) }
}

/// Add a copy of the NUL-terminated string `val` to `d` under `key`.
///
/// # Safety
/// `d` points at a live dictionary, `key` is readable for `key_len` bytes,
/// and `val` is null or a NUL-terminated string. The string is copied.
pub unsafe fn tv_dict_add_str(
    d: *mut Dict,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    val: *const ::core::ffi::c_char,
) -> Result<(), Failed> {
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
    d: *mut Dict,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    val: *const ::core::ffi::c_char,
    len: ::core::ffi::c_int,
) -> Result<(), Failed> {
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
    d: *mut Dict,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    val: *mut ::core::ffi::c_char,
) -> Result<(), Failed> {
    let item = unsafe { tv_dict_item_alloc_len(key, key_len) };
    unsafe { (*item).di_tv.write_string(val) };
    unsafe { add_or_free(d, item) }
}

/// Add a funcref to `func` to `d` under `key`.
///
/// # Safety
/// `d` points at a live dictionary, `key` is readable for `key_len` bytes,
/// and `func` points at a live `UserFunc` whose `uf_name` is `uf_namelen`
/// readable bytes. Only the name is copied; the funcref counts as a use of
/// the function.
pub unsafe fn tv_dict_add_func(
    d: *mut Dict,
    key: *const ::core::ffi::c_char,
    key_len: size_t,
    func: *mut UserFunc,
) -> Result<(), Failed> {
    let item = unsafe { tv_dict_item_alloc_len(key, key_len) };
    let name = unsafe { (&raw mut (*func).uf_name).cast() };
    // SAFETY: the caller's promise: a live function.
    let func = unsafe { Live::<UserFunc>::new(func) };
    let namelen = func.uf_namelen;
    let owned = unsafe { xmemdupz(name, namelen) } as *mut ::core::ffi::c_char;
    unsafe { (*item).di_tv.write_func_name(owned) };
    if unsafe { tv_dict_add(d, item) }.is_err() {
        unsafe { tv_dict_item_free(item) };
        return Err(Failed);
    }
    unsafe { func_ref((*item).di_tv.func_name_or_null()) };
    Ok(())
}

/// The tail every `tv_dict_add_*` shares: hand `item` to `d`, or free it again
/// when the key is taken.
///
/// # Safety
/// As [`tv_dict_add`], except that a taken key frees `item` rather than
/// handing it back — so the caller must not touch `item` after this returns
/// on either answer.
#[inline]
unsafe fn add_or_free(d: *mut Dict, item: *mut DictItem) -> Result<(), Failed> {
    if unsafe { tv_dict_add(d, item) }.is_err() {
        unsafe { tv_dict_item_free(item) };
        return Err(Failed);
    }
    Ok(())
}

/// Free every item of `d`, leaving it allocated and empty.
///
/// # Safety
/// `d` must point at a live dictionary that nothing else is walking; as
/// [`tv_dict_free_contents`], the hashtab is locked for the walk.
pub unsafe fn tv_dict_clear(d: *mut Dict) {
    // Lock the hashtab so `hash_remove` below cannot rehash it under the
    // walk.
    unsafe { hash_lock(&raw mut (*d).dv_hashtab) };
    debug_assert!(unsafe { (*d).dv_hashtab.ht_locked } > 0);
    for hi in unsafe { tv_dict_iter(d) } {
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
pub unsafe fn tv_dict_extend(d1: *mut Dict, d2: *mut Dict, action: *const ::core::ffi::c_char) {
    let watched = unsafe { tv_dict_is_watched(d1) };
    let arg_errmsg = tr(c"extend() argument");
    let arg_errmsg_len = unsafe { cstr::bytes_at(arg_errmsg) }.len();
    let action = unsafe { *action } as u8;

    if action == b'm' {
        unsafe { hash_lock(&raw mut (*d2).dv_hashtab) }; // don't rehash on hash_remove()
    }

    for hi2 in unsafe { tv_dict_iter(d2) } {
        let di2 = tv_dict_hi2di(hi2);
        let di2_key = unsafe { tv_dict_item_key(di2) };
        let di1 = unsafe { tv_dict_find(d1, di2_key, -1) };
        // Check the key to be valid when adding to any scope.
        if unsafe { (*d1).dv_scope } != VAR_NO_SCOPE && !unsafe { valid_varname(di2_key) } {
            break;
        }
        if di1.is_null() {
            if action == b'm' {
                // Cheap way to move a dict item from "d2" to "d1".
                // If dict_add() fails then "d2" won't be empty.
                if unsafe { tv_dict_add(d1, di2) }.is_ok() {
                    unsafe { hash_remove(&raw mut (*d2).dv_hashtab, hi2) };
                    // Note upstream does not gate this on `watched`, unlike
                    // the copying branch below.
                    // SAFETY: the item just moved into `d1`.
                    unsafe { tv_dict_watcher_notify(d1, di2_key, Some(&*di_tv(di2)), None) };
                }
            } else {
                let new_di = unsafe { tv_dict_item_copy(di2) };
                if unsafe { tv_dict_add(d1, new_di) }.is_err() {
                    unsafe { tv_dict_item_free(new_di) };
                } else if watched {
                    let key = unsafe { tv_dict_item_key(new_di) };
                    // SAFETY: the item just added to `d1`.
                    unsafe { tv_dict_watcher_notify(d1, key, Some(&*di_tv(new_di)), None) };
                }
            }
        } else if action == b'e' {
            // SAFETY: a message argument the caller holds as a NUL-terminated string.
            let di2_key = unsafe { c_str(di2_key) };
            semsg!("E737: Key already exists: {di2_key}");
            break;
        } else if action == b'f' && di2 != di1 {
            if unsafe { value_check_lock((*di1).di_lock, arg_errmsg, arg_errmsg_len) } || {
                let flags = unsafe { (*di1).di_flags } as ::core::ffi::c_int;
                unsafe { var_check_ro(flags, arg_errmsg, arg_errmsg_len) }
            } {
                break;
            }
            // Disallow replacing a builtin function.
            if unsafe { tv_dict_wrong_func_name(d1, &mut (*di2).di_tv, di2_key) } != 0 {
                break;
            }

            let mut oldtv = TV_INITIAL_VALUE;
            if watched {
                unsafe { tv_copy(&(*di1).di_tv, &mut oldtv) };
            }

            unsafe { tv_clear(&mut (*di1).di_tv) };
            unsafe { tv_copy(&(*di2).di_tv, &mut (*di1).di_tv) };

            if watched {
                let key = unsafe { tv_dict_item_key(di1) };
                // SAFETY: the item just overwritten in `d1`.
                let new = Some(unsafe { &*di_tv(di1) });
                unsafe { tv_dict_watcher_notify(d1, key, new, Some(&oldtv)) };
                unsafe { tv_clear(&mut oldtv) };
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
/// `copy_id` bookkeeping.
pub unsafe fn tv_dict_equal(d1: *mut Dict, d2: *mut Dict, ic: bool) -> bool {
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

    for hi in unsafe { tv_dict_iter(d1) } {
        let di1 = tv_dict_hi2di(hi);
        let di2 = unsafe { tv_dict_find(d2, tv_dict_item_key(di1), -1) };
        if di2.is_null() || !unsafe { tv_equal(&(*di1).di_tv, &(*di2).di_tv, ic) } {
            return false;
        }
    }
    true
}

/// Copy `orig`, deeply when `deep`, converting keys through `conv`.
///
/// `copy_id` is the garbage collector's mark: non-zero records the copy on the
/// original so a self-referencing dictionary resolves to the same copy.
///
/// # Safety
/// `orig` is null or a live dictionary and `conv` is null or a live
/// converter. A non-zero `copy_id` is written onto `orig`, so it must be one
/// the caller reserved from `get_copyID`; passing a stale one makes an
/// unrelated walk think this dictionary is already visited.
pub unsafe fn tv_dict_copy(
    conv: *const VimConv,
    orig: *mut Dict,
    deep: bool,
    copy_id: ::core::ffi::c_int,
) -> Option<DictRef> {
    if orig.is_null() {
        return None;
    }

    let copy = tv_dict_alloc();
    // A borrow of the dictionary the handle owns, for the items to go into.
    let into = copy.as_ptr();
    if copy_id != 0 {
        // SAFETY: the caller's promise: a live dictionary.
        let mut from = unsafe { Dt::new(orig) };
        from.dv_copy_id = copy_id;
        from.dv_copydict = into;
    }
    for hi in unsafe { tv_dict_iter(orig) } {
        let di = tv_dict_hi2di(hi);
        if got_int.get() {
            break;
        }
        let new_di = if conv.is_null() || unsafe { (*conv).vc_type } == CONV_NONE {
            unsafe { tv_dict_item_alloc(tv_dict_item_key(di)) }
        } else {
            let di_key = unsafe { tv_dict_item_key(di) };
            let mut len = unsafe { cstr::bytes_at(di_key) }.len();
            let key = unsafe { string_convert(conv, di_key.cast_mut(), &raw mut len) };
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
            if unsafe { var_item_copy(conv, &*from, &mut *to, deep, copy_id) }.is_err() {
                unsafe { xfree(new_di.cast()) };
                break;
            }
        } else {
            unsafe { tv_copy(&(*di).di_tv, &mut (*new_di).di_tv) };
        }
        if unsafe { tv_dict_add(into, new_di) }.is_err() {
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

/// Mark every key of `dict` read-only and fixed.
///
/// # Safety
/// `dict` must point at a live dictionary.
pub unsafe fn tv_dict_set_keys_readonly(dict: *mut Dict) {
    for hi in unsafe { tv_dict_iter(dict) } {
        let di = tv_dict_hi2di(hi);
        unsafe { (*di).di_flags |= (DI_FLAGS_RO | DI_FLAGS_FIX) as uint8_t };
    }
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
    let key = unsafe { numbuf.string_chk(&args[1]) };
    if key.is_null() {
        return;
    }
    let di = unsafe { tv_dict_find(d, key, -1) };
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
    unsafe { tv_dict_item_remove(d, di) };
    if unsafe { tv_dict_is_watched(d) } {
        unsafe { tv_dict_watcher_notify(d, key, None, Some(result)) };
    }
}
