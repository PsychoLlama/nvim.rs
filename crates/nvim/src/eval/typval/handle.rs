//! The value-handle core: the one file of the container model that still
//! allows `unsafe`, and everything in it.
//!
//! A Vimscript List, Dict, Blob or partial is a heap object shared by
//! reference count and reached from many places at once -- a variable, an
//! argument frame, a `:for` loop, a watcher's argument, the collector's
//! registry, Lua. The evaluator re-enters at every call, so two of those
//! holders can be live on one object while one of them edits it. Nothing in
//! Rust's safe vocabulary says that, so what does is concentrated here, where
//! one reading covers it:
//!
//! 1. **The four handles** -- [`ListRef`], [`DictRef`], [`BlobRef`],
//!    [`PartialRef`]: a `NonNull` plus the reference it owns. `Clone`
//!    retains, `Drop` releases and frees with the last one, and
//!    `Deref`/`DerefMut` (and `edit`, the same thing from a shared handle)
//!    hand out a borrow **that must not span a call that can run user code**.
//!    That promise is [`Live`](crate::winlayer::Live)'s, and it is the one
//!    this file cannot check: a second holder can always reach the same
//!    object. The raw bridges (`owning`, `retained`, `from_owned`) stay
//!    `unsafe fn`s for the callers outside the container model that still
//!    hold a pointer.
//! 2. **Allocation and release**: the four objects' allocations, the
//!    reference-count release that frees them, and the collector's
//!    `tv_in_free_unref_items` gate that makes a release free nothing while
//!    `free_unref_items` owns the whole graph.
//! 3. **The dictionary's item store**: a hash table slot names a
//!    `*mut DictItem`, either a `Box` the dictionary owns (`DI_FLAGS_ALLOC`)
//!    or an item embedded in a structure that outlives the table (a
//!    funccall's fixed variables, a scope's own entry, `b:changedtick`, a
//!    `v:` row). Reading an item out of a slot, and freeing an allocated one,
//!    happen here.
//! 4. **The bit copy** ([`TypVal::bit_copy`]): a value duplicated without a
//!    reference, so that an argument frame can *name* the caller's values
//!    for the length of a call.
//!
//! Every other file of `eval/typval/` reaches these objects through this
//! API. Whether these stay a perimeter row under the raw-memory-primitive
//! rule or get a safe form is a decision taken once, over this file.

#![deny(unsafe_op_in_unsafe_fn)]
// The value-handle core (see the module docs): left allowing unsafe on purpose.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use ::core::ptr::NonNull;

use super::*;
use crate::hashtab::removed_sentinel;
use crate::types::{Refcount, SlotEntry};

/// An owning reference to a heap-allocated [`List`].
///
/// This is the reference count, as a type: [`Clone`] takes a reference and
/// [`Drop`] gives one back, freeing the list when the last one goes.  What
/// it replaces is upstream's `tv_list_ref`/`list_unref` pair, which every
/// call site had to remember to write, in the right order, on every path.
///
/// **A fresh handle owns one reference.**  [`tv_list_alloc`] answers a
/// `ListRef` at a count of one, where upstream handed back a list at *zero*
/// and left the first storer to raise it -- an idiom whose whole purpose was
/// to let an error path free a list nobody had claimed, which is what a
/// destructor does by itself.  The count is now exactly the number of live
/// holders, and there is no state in which a list is alive with none.
///
/// [`Deref`](core::ops::Deref) is [`Live`](crate::winlayer::Live)'s: the
/// borrow lasts as long as the field access that asked for it and never
/// spans a call, because the evaluator re-enters and the same list is
/// reachable through a `*mut List` somewhere else at the same time.
///
/// The two constructors are the two things a raw pointer can mean.  A handle
/// is not null: `v:_null_list` is `TypVal::list(None)`, and every reader that
/// wants the old spelling asks [`TypVal::list_or_null`].
#[repr(transparent)]
pub struct ListRef(NonNull<List>);

impl ListRef {
    /// Take over a reference the caller already holds and will not
    /// release.
    ///
    /// # Safety
    ///
    /// `at` must point at a live object, and the caller must hold a
    /// reference to it -- one this handle now owns and eventually
    /// gives back.
    #[inline(always)]
    pub unsafe fn from_owned(at: NonNull<List>) -> ListRef {
        ListRef(at)
    }

    /// Take over the caller's reference, or answer `None` for NULL.
    ///
    /// # Safety
    ///
    /// As `from_owned`, for a pointer that may be null.
    #[inline(always)]
    pub unsafe fn owning(at: *mut List) -> Option<ListRef> {
        NonNull::new(at).map(ListRef)
    }

    /// Take *another* reference: the caller keeps its own. `None` for
    /// NULL, which counts nothing.
    ///
    /// # Safety
    ///
    /// `at` is null or points at a live object.
    #[inline(always)]
    pub unsafe fn retained(at: *mut List) -> Option<ListRef> {
        let at = NonNull::new(at)?;
        // SAFETY: the caller's promise: a live object.
        unsafe { (*at.as_ptr()).lv_refcount.retain() };
        Some(ListRef(at))
    }

    /// The object, as the pointer a callee outside the container
    /// model still takes. A **borrow**: live only while the handle is.
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut List {
        self.0.as_ptr()
    }

    /// Give the reference up without releasing it: something else
    /// owns it now.
    #[inline(always)]
    pub fn into_raw(self) -> *mut List {
        let at = self.0;
        ::core::mem::forget(self);
        at.as_ptr()
    }

    /// Whether the two handles name the same object.
    #[inline(always)]
    pub fn ptr_eq(&self, other: &ListRef) -> bool {
        self.0 == other.0
    }

    /// The object, writable, from a shared handle.
    ///
    /// The `Live` promise, spelled out: the borrow lasts for the
    /// statement that asked for it and **never spans a call that can
    /// run user code**, because every other holder of this object can
    /// reach it from there. Nothing checks it; this is the one place
    /// the container model says so.
    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn edit(&self) -> &mut List {
        // SAFETY: the handle holds a reference, so the object is
        // live; exclusivity is the caller's promise above.
        unsafe { &mut *self.0.as_ptr() }
    }
}

impl Clone for ListRef {
    /// One more owner of the same object.
    #[inline(always)]
    fn clone(&self) -> ListRef {
        self.edit().lv_refcount.retain();
        ListRef(self.0)
    }
}

impl Drop for ListRef {
    /// Give the reference back, freeing the object with the last one.
    #[inline(always)]
    fn drop(&mut self) {
        if self.edit().lv_refcount.release() <= 0 {
            free_list(self);
        }
    }
}

/// Free `list` and everything in it: its last reference has just gone.
/// A no-op while `free_unref_items()` is walking, which frees the whole
/// graph itself.
fn free_list(list: &ListRef) {
    if tv_in_free_unref_items.get() {
        return;
    }
    // The handle is the view the contents are freed through: it owns the
    // reference that was the last, and gives it back by freeing.
    list_free_contents(list);
    // SAFETY: a live list, which nothing references any more.
    unsafe { list_free_list(list.as_ptr()) };
}

impl ::core::ops::Deref for ListRef {
    type Target = List;

    #[inline(always)]
    fn deref(&self) -> &List {
        // SAFETY: the handle holds a reference, so the object is
        // live; the borrow lasts only as long as the access that
        // asked for it.
        unsafe { self.0.as_ref() }
    }
}

impl ::core::ops::DerefMut for ListRef {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut List {
        // SAFETY: as `deref`.
        unsafe { self.0.as_mut() }
    }
}

/// Allocate an empty list, **owned by the handle it answers** (a count of
/// one, where upstream answered zero). `len` is a capacity hint; a negative
/// one (`kListLenUnknown`) reserves nothing.
pub fn tv_list_alloc(len: ptrdiff_t) -> ListRef {
    // The `xmalloc` family, not a `Box`: the unit cases' allocation log sees
    // only that family.
    let list = unsafe { xcalloc(1, ::core::mem::size_of::<List>()) }.cast::<List>();
    // Written, not assigned: a zeroed `List` is not a valid one (`Vec` never
    // holds a null pointer), so there is nothing there to drop.
    // SAFETY: the allocation just made, of exactly this size.
    unsafe { list.write(List::empty()) };
    if let Ok(len) = usize::try_from(len) {
        // SAFETY: the allocation just written.
        unsafe { &mut *list }.lv_items.reserve_exact(len);
    }

    let at = NonNull::new(list).expect("xcalloc never answers null");
    // The collector reaches every live list through its registry.
    let root = root_list(at);
    // SAFETY: the allocation just made and written.
    let mut live = unsafe { Ls::new(list) };
    live.lv_root = root;
    live.lv_refcount = Refcount::ONE;
    // SAFETY: the reference just seeded is the one this handle owns.
    unsafe { ListRef::from_owned(at) }
}

/// Take `l` out of the garbage collector's registry and free the `List`,
/// with whatever items are still in its array.
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call.  Anything still
/// in it is **released**, so no caller may hold a reference to an item.
pub unsafe fn list_free_list(l: *mut List) {
    // Out of the collector's registry. A list the allocator never handed
    // out -- `a:000`, a submatch list -- carries `RootId::NONE` and is not
    // in it; the removal is a no-op for those.
    // SAFETY: the caller's promise: a live list.
    let mut list = unsafe { Ls::new(l) };
    unroot_list(list.lv_root);
    list.lv_root = RootId::NONE;

    // NLUA_CLEAR_REF
    if list.lua_table_ref != LUA_NOREF {
        unsafe { api_free_luaref((*l).lua_table_ref) };
        list.lua_table_ref = LUA_NOREF as LuaRef;
    }
    // The item array and the cursor array: `xfree` is the C's and runs no
    // destructor, so the fields go first.
    // SAFETY: as above.
    unsafe { ::core::ptr::drop_in_place(l) };
    unsafe { xfree(l.cast()) };
}

impl Dict {
    /// Every entry, in slot order -- which is the order Vim shows.
    ///
    /// The walk borrows the dictionary, so nothing can add to or remove from
    /// it while the walk is live. A body that edits the table wants
    /// [`DictCursor`](super::DictCursor), which is a slot index.
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

/// An owning reference to a heap-allocated [`Dict`]: [`ListRef`]'s
/// counterpart, and the same statement about a reference count.
///
/// **The scope dictionaries are not heap dictionaries.** `b:`, `w:`, `t:`,
/// `g:`, `v:` and a funccall's `l:`/`a:` live inside the structure that owns
/// them, are seeded with `DO_NOT_FREE_CNT` and carry `RootId::NONE`; a
/// handle over one is a *borrow* spelled as a handle ([`DictRef::owning`]),
/// and `unref_var_dict` is what gives the whole block back.
#[repr(transparent)]
pub struct DictRef(NonNull<Dict>);

/// The same API as [`ListRef`]'s, method for method.
impl DictRef {
    /// # Safety
    /// As [`ListRef::from_owned`].
    #[inline(always)]
    pub unsafe fn from_owned(at: NonNull<Dict>) -> DictRef {
        DictRef(at)
    }

    /// # Safety
    /// As [`ListRef::owning`].
    #[inline(always)]
    pub unsafe fn owning(at: *mut Dict) -> Option<DictRef> {
        NonNull::new(at).map(DictRef)
    }

    /// # Safety
    /// As [`ListRef::retained`].
    #[inline(always)]
    pub unsafe fn retained(at: *mut Dict) -> Option<DictRef> {
        let at = NonNull::new(at)?;
        // SAFETY: the caller's promise: a live object.
        unsafe { (*at.as_ptr()).dv_refcount.retain() };
        Some(DictRef(at))
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *mut Dict {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub fn into_raw(self) -> *mut Dict {
        let at = self.0;
        ::core::mem::forget(self);
        at.as_ptr()
    }

    #[inline(always)]
    pub fn ptr_eq(&self, other: &DictRef) -> bool {
        self.0 == other.0
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn edit(&self) -> &mut Dict {
        // SAFETY: the handle holds a reference, so the object is
        // live; exclusivity is the caller's promise above.
        unsafe { &mut *self.0.as_ptr() }
    }
}

impl Clone for DictRef {
    #[inline(always)]
    fn clone(&self) -> DictRef {
        self.edit().dv_refcount.retain();
        DictRef(self.0)
    }
}

impl Drop for DictRef {
    #[inline(always)]
    fn drop(&mut self) {
        if self.edit().dv_refcount.release() <= 0 {
            free_dict(self);
        }
    }
}

/// [`free_list`]'s counterpart: free `dict` and everything in it.
fn free_dict(dict: &DictRef) {
    if tv_in_free_unref_items.get() {
        return;
    }
    tv_dict_free_contents(dict);
    // SAFETY: a live dictionary whose contents are gone, which nothing
    // references any more.
    unsafe { tv_dict_free_dict(dict.as_ptr()) };
}

impl ::core::ops::Deref for DictRef {
    type Target = Dict;

    #[inline(always)]
    fn deref(&self) -> &Dict {
        // SAFETY: as `ListRef`'s.
        unsafe { self.0.as_ref() }
    }
}

impl ::core::ops::DerefMut for DictRef {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Dict {
        // SAFETY: as `ListRef`'s.
        unsafe { self.0.as_mut() }
    }
}

/// Allocate an empty dictionary, **owned by the handle it answers**; see
/// [`tv_list_alloc`].
pub fn tv_dict_alloc() -> DictRef {
    let d = unsafe { xcalloc(1, ::core::mem::size_of::<Dict>()) }.cast::<Dict>();

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
    // Written, not assigned: zeroed storage is not a valid `Vec`.
    // SAFETY: the allocation just made.
    unsafe { (&raw mut (*d).watchers).write(Vec::new()) };
    dict.lua_table_ref = LUA_NOREF as LuaRef;
    // SAFETY: the reference just seeded is the one this handle owns.
    unsafe { DictRef::from_owned(at) }
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

/// One reference to a [`Blob`], given back when the handle goes: the blob
/// half of [`ListRef`].
#[repr(transparent)]
pub struct BlobRef(NonNull<Blob>);

/// The same API as [`ListRef`]'s, less the raw bridges: a blob is only
/// ever made by [`tv_blob_alloc`].
impl BlobRef {
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut Blob {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub fn ptr_eq(&self, other: &BlobRef) -> bool {
        self.0 == other.0
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn edit(&self) -> &mut Blob {
        // SAFETY: the handle holds a reference, so the object is
        // live; exclusivity is the caller's promise above.
        unsafe { &mut *self.0.as_ptr() }
    }
}

impl Clone for BlobRef {
    #[inline(always)]
    fn clone(&self) -> BlobRef {
        self.edit().bv_refcount.retain();
        BlobRef(self.0)
    }
}

impl Drop for BlobRef {
    /// Give the reference back; the last one frees the blob.
    #[inline(always)]
    fn drop(&mut self) {
        if self.edit().bv_refcount.release() <= 0 {
            // SAFETY: the last reference to the `Box` `tv_blob_alloc` leaked.
            drop(unsafe { Box::from_raw(self.as_ptr()) });
        }
    }
}

impl ::core::ops::Deref for BlobRef {
    type Target = Blob;

    #[inline(always)]
    fn deref(&self) -> &Blob {
        // SAFETY: as `ListRef`'s.
        unsafe { self.0.as_ref() }
    }
}

impl ::core::ops::DerefMut for BlobRef {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Blob {
        // SAFETY: as `ListRef`'s.
        unsafe { self.0.as_mut() }
    }
}

/// Allocate an empty blob, **owned by the handle it answers**.
///
/// The blob arrives at a reference count of **one**, as
/// [`tv_list_alloc`](super::tv_list_alloc) does and where upstream answered
/// zero: a caller that stores it nowhere drops the handle, and that is the
/// free.
pub fn tv_blob_alloc() -> BlobRef {
    let mut blob = Box::new(Blob {
        bv_data: Vec::new(),
        bv_refcount: Refcount::ZERO,
        bv_lock: VarLock::Unlocked,
    });
    // The count starts at the handle's one.
    blob.bv_refcount.retain();
    BlobRef(NonNull::from(Box::leak(blob)))
}

/// One reference to a [`Partial`], given back when the handle goes: the
/// partial half of [`ListRef`].
#[repr(transparent)]
pub struct PartialRef(NonNull<Partial>);

/// The same API as [`ListRef`]'s, less the raw bridges: a partial is only
/// ever made by [`PartialRef::new`].
impl PartialRef {
    /// A partial built in place, owned by the handle (a count of one).
    pub fn new(mut partial: Partial) -> PartialRef {
        partial.pt_refcount = Refcount::ONE;
        PartialRef(NonNull::from(Box::leak(Box::new(partial))))
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *mut Partial {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub fn ptr_eq(&self, other: &PartialRef) -> bool {
        self.0 == other.0
    }

    #[inline(always)]
    #[allow(clippy::mut_from_ref)]
    pub fn edit(&self) -> &mut Partial {
        // SAFETY: the handle holds a reference, so the object is
        // live; exclusivity is the caller's promise above.
        unsafe { &mut *self.0.as_ptr() }
    }
}

impl Clone for PartialRef {
    #[inline(always)]
    fn clone(&self) -> PartialRef {
        self.edit().pt_refcount.retain();
        PartialRef(self.0)
    }
}

impl Drop for PartialRef {
    /// Give the reference back; the last one frees the partial.
    fn drop(&mut self) {
        if self.edit().pt_refcount.release() <= 0 {
            // SAFETY: the last reference to the `Box` `new` leaked.
            partial_free(*unsafe { Box::from_raw(self.as_ptr()) });
        }
    }
}

impl ::core::ops::Deref for PartialRef {
    type Target = Partial;

    #[inline(always)]
    fn deref(&self) -> &Partial {
        // SAFETY: as `ListRef`'s.
        unsafe { self.0.as_ref() }
    }
}

impl ::core::ops::DerefMut for PartialRef {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut Partial {
        // SAFETY: as `ListRef`'s.
        unsafe { self.0.as_mut() }
    }
}

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

/// Release every item of a table already taken out of its dictionary, in
/// slot order: the whole-dictionary free, without a removal per slot. An
/// embedded item (not `DI_FLAGS_ALLOC`) is only emptied.
pub(crate) fn release_table(table: DictTab) {
    for hi in table.items() {
        let item = hi.hi_key.item();
        // SAFETY: a kept slot of a table nothing else can reach any more
        // names a live item, released once here; `DictItem::boxed` made it
        // when it carries `DI_FLAGS_ALLOC`.
        unsafe {
            if ::core::ffi::c_uint::from((*item).di_flags) & DI_FLAGS_ALLOC != 0 {
                drop(Box::from_raw(item));
            } else {
                tv_clear(&mut (*item).di_tv);
            }
        }
    }
}

impl Dict {
    /// The item in slot `slot`, or `None` when the slot holds none.
    #[inline]
    pub(crate) fn item_at(&self, slot: usize) -> Option<&DictItem> {
        let hi = self.dv_hashtab.slot(slot);
        // SAFETY: a kept slot of this dictionary names a live item, which the
        // borrow of the dictionary keeps alive.
        hi.is_kept().then(|| unsafe { &*hi.hi_key.item() })
    }

    /// [`Dict::item_at`] with the item writable.
    #[inline]
    pub(crate) fn item_at_mut(&mut self, slot: usize) -> Option<&mut DictItem> {
        let hi = self.dv_hashtab.slot(slot);
        // SAFETY: as `item_at`; no two slots name one item, and the exclusive
        // borrow keeps the set of them fixed.
        hi.is_kept().then(|| unsafe { &mut *hi.hi_key.item() })
    }

    /// The slot holding `key`, if any.
    #[inline]
    pub(crate) fn slot_of(&self, key: &[u8]) -> Option<usize> {
        // SAFETY: this dictionary's own table, and a slice key.
        let hi = unsafe {
            hash_find_len(
                &raw const self.dv_hashtab,
                key.as_ptr().cast::<::core::ffi::c_char>(),
                key.len(),
            )
        };
        hi.is_kept().then(|| hi.index())
    }

    /// Put `item` in the table, which takes it over. When its key is
    /// already there the item comes back, untouched.
    pub(crate) fn insert(&mut self, item: Box<DictItem>) -> Result<(), Box<DictItem>> {
        let at = Box::into_raw(item);
        // SAFETY: this dictionary's own table, and an item in no table whose
        // key lives as long as it does.
        match unsafe { hash_add(&raw mut self.dv_hashtab, DictEntry::new(at)) } {
            Ok(()) => Ok(()),
            // SAFETY: the table refused it, so the `Box` is the caller's again.
            Err(_) => Err(unsafe { Box::from_raw(at) }),
        }
    }

    /// Take the item in slot `slot` out of the table.
    ///
    /// Nothing is released here: the answer owns what the item held, and
    /// dropping it after the borrow of the dictionary has ended is the
    /// release -- which matters, because a value can name the dictionary it
    /// was in, and releasing it reads that dictionary again.
    pub(crate) fn remove_at(&mut self, slot: usize) -> RemovedItem {
        let hi = self.dv_hashtab.slot(slot);
        debug_assert!(hi.is_kept());
        let item = hi.hi_key.item();
        // SAFETY: a kept slot of this dictionary's own table.
        unsafe { hash_remove(&raw mut self.dv_hashtab, hi) };
        // SAFETY: the item the slot named, out of the table now. An allocated
        // one is the `Box` `insert` (or `DictItem::boxed`) made; an embedded
        // one belongs to the structure it lives in, which keeps it.
        unsafe {
            if ::core::ffi::c_uint::from((*item).di_flags) & DI_FLAGS_ALLOC != 0 {
                RemovedItem::Allocated(Box::from_raw(item))
            } else {
                RemovedItem::Embedded((*item).di_tv.take())
            }
        }
    }

    /// Stop the table resizing while a walk removes as it goes.
    #[inline]
    pub(crate) fn lock_table(&mut self) {
        self.dv_hashtab.ht_locked += 1;
    }

    /// Undo one [`Dict::lock_table`], resizing now if the table wanted to.
    #[inline]
    pub(crate) fn unlock_table(&mut self) {
        // SAFETY: this dictionary's own table, locked by the caller.
        unsafe { hash_unlock(&raw mut self.dv_hashtab) };
    }
}

impl TypVal {
    /// Duplicate the value's bits, **sharing** whatever it points at.
    ///
    /// This is not a copy of the *value*: no string is duplicated and no
    /// reference count moves, so the payload now has two holders — and, with
    /// `Drop` live, two would-be releasers.  Every use of this is a place
    /// where upstream relies on two typvals naming one object for a bounded
    /// window: an argument vector that borrows the caller's values for the
    /// length of a call, a slot packed for output while the original is still
    /// the owner.
    ///
    /// A real copy — one that duplicates the string and takes the
    /// reference — is [`Clone`].
    ///
    /// # Safety
    /// The duplicate must not be released: exactly one of the two holders
    /// may, and it is the original. In practice the duplicate goes into a
    /// [`ManuallyDrop`](core::mem::ManuallyDrop) frame that outlives nothing.
    #[inline(always)]
    pub(crate) unsafe fn bit_copy(&self) -> TypVal {
        // SAFETY: the caller's promise above -- the duplicate is not released,
        // so the payload keeps its one owner.
        unsafe { ::core::ptr::read(self) }
    }

    /// The [`bit_copy`](TypVal::bit_copy) an `a:` or `a:000` item holds: it
    /// *names* the caller's argument for the call. Contract: the caller keeps
    /// `self` until the frame gives the duplicate up unreleased (or upgrades
    /// it to a copy when the scope outlives the call).
    pub(crate) fn named_for_call(&self) -> TypVal {
        // SAFETY: the contract above -- the payload keeps its one owner.
        unsafe { self.bit_copy() }
    }
}

/// The one step of a [`CallFrame`] that is not the frame's own business:
/// duplicating a value the caller keeps, so that the frame can *name* it.
///
/// It lives here rather than beside the type because `bit_copy` is this
/// module's, and a frame that never takes one is safe code end to end.
impl<const N: usize> CallFrame<N> {
    /// Append a bit copy of a value the caller keeps.
    pub(crate) fn push_borrowed(&mut self, tv: &TypVal) {
        // SAFETY: the duplicate is never released -- the slot's bit is
        // clear, so `truncate` disowns it rather than clearing it.
        self.push_naming(unsafe { tv.bit_copy() });
    }
}

/// The deep copy's memo: what a cycle back to a container resolves to.
///
/// A deep copy marks each original with its `copy_id` and the copy it made
/// before copying the items, so that an item naming the original again gets
/// the copy. The copy is held by the walk that made it for as long as that
/// `copy_id` is the one asked about, which is what the reads rest on.
impl ListRef {
    /// Mark this list as copied to `copy` under `copy_id`.
    pub(crate) fn remember_copy(&self, copy_id: ::core::ffi::c_int, copy: &ListRef) {
        let this = self.edit();
        this.lv_copy_id = copy_id;
        this.lv_copylist = copy.as_ptr();
    }
}

impl List {
    /// The copy this list was given under `copy_id` by the walk still
    /// running, with a reference of its own.
    pub(crate) fn copy_under(&self, copy_id: ::core::ffi::c_int) -> Option<ListRef> {
        if copy_id == 0 || self.lv_copy_id != copy_id {
            return None;
        }
        // SAFETY: written by `remember_copy` under this id, by the walk that
        // still holds the copy.
        unsafe { ListRef::retained(self.lv_copylist) }
    }
}

impl DictRef {
    /// Mark this dictionary as copied to `copy` under `copy_id`.
    pub(crate) fn remember_copy(&self, copy_id: ::core::ffi::c_int, copy: &DictRef) {
        let this = self.edit();
        this.dv_copy_id = copy_id;
        this.dv_copydict = copy.as_ptr();
    }
}

impl Dict {
    /// The copy this dictionary was given under `copy_id` by the walk still
    /// running, with a reference of its own.
    pub(crate) fn copy_under(&self, copy_id: ::core::ffi::c_int) -> Option<DictRef> {
        if copy_id == 0 || self.dv_copy_id != copy_id {
            return None;
        }
        // SAFETY: written by `remember_copy` under this id, by the walk that
        // still holds the copy.
        unsafe { DictRef::retained(self.dv_copydict) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_cell::editor_state_lock;

    /// A dictionary released to zero under the flag is left for the
    /// collector, which frees it explicitly afterwards. Here rather than
    /// beside the dictionary operations because reaching a dictionary nobody
    /// holds is a raw read.
    #[test]
    fn a_dict_released_mid_collection_waits_for_the_collector() {
        let _held = editor_state_lock();
        let mut d = tv_dict_alloc();
        d.add_number(b"a", 0).expect("a key used once");
        d.add_number(b"b", 1).expect("a key used once");
        let dp = d.as_ptr();
        tv_in_free_unref_items.set(true);
        drop(d);
        // SAFETY: unreferenced but not freed.
        let (refs, b) = unsafe {
            let dict = &*dp;
            (
                dict.dv_refcount.get(),
                dict.find(b"b").map(|di| di.di_tv.number_or_zero()),
            )
        };
        tv_in_free_unref_items.set(false);
        assert_eq!(refs, 0);
        assert_eq!(b, Some(1));
        // SAFETY: as above. The handle takes the count below zero, which
        // frees it now that the flag is down.
        drop(unsafe { DictRef::owning(dp) });
    }

    /// The discriminant is the `VarType` code at offset zero: what
    /// `#[repr(C, u32)]` promises and what the generated `ffi.cdef` chunk
    /// describes to the unit fixtures.
    #[test]
    fn a_value_is_tagged_by_its_var_type_at_offset_zero() {
        for tv in [
            TypVal::Unknown,
            TypVal::Number(1),
            TypVal::string(None),
            TypVal::func(None),
            TypVal::list(None),
            TypVal::dict(None),
            TypVal::Float(1.0),
            TypVal::Bool(kBoolVarTrue),
            TypVal::Special(kSpecialVarNull),
            TypVal::partial(None),
            TypVal::blob(None),
        ] {
            // SAFETY: `repr(C, u32)` puts the discriminant first, and it is
            // a `u32`.
            let tag = unsafe { *(&raw const tv).cast::<crate::types::VarType>() };
            assert_eq!(tag, tv.v_type());
        }
    }
}
