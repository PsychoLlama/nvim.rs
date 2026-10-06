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
        // SAFETY: this handle names a live object, and is giving up
        // the reference that kept it so.
        unsafe { list_unref(self.as_ptr()) };
    }
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

/// Allocate an empty list, **owned by the handle it answers**.
///
/// `len` is a capacity hint: a caller that knows how many items are coming
/// reserves them here rather than growing the array on the way.  A negative
/// one (`kListLenUnknown`) reserves nothing.
///
/// The list arrives at a reference count of **one**, held by the
/// [`ListRef`].  Upstream handed one back at zero and relied on the first
/// storer to raise it; a caller that stored it nowhere had to notice and
/// free it by hand.  Dropping the handle is that free.
pub fn tv_list_alloc(len: ptrdiff_t) -> ListRef {
    // Still the `xmalloc` family rather than a `Box`, because the allocation
    // log the unit cases assert against sees only that family -- and because
    // the tree hands `*mut List` around and frees it in `list_free_list`.
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

/// Initialise a `List` embedded in the caller's own storage: empty, locked
/// and carrying `DO_NOT_FREE_CNT`, so nothing frees it.
///
/// The caller's storage is what owns it -- `FuncCall`'s `a:000` list, a `\=`
/// expression's submatch list -- and dropping that storage drops the items.
///
/// # Safety
///
/// `l` must point at storage the caller owns and will not free through
/// [`list_free`]; whatever was there is overwritten without being
/// dropped, so it must hold no list yet.
pub unsafe fn list_init_static(l: *mut List) {
    // SAFETY: the caller's promise: storage holding no list yet.
    unsafe {
        l.write(List {
            lv_refcount: Refcount::new(DO_NOT_FREE_CNT.cast_signed()),
            lv_lock: VarLock::Fixed,
            ..List::empty()
        });
    }
}

/// Take `l` out of the garbage collector's registry and free the `List`.
///
/// Upstream freed the header and left whatever was still linked off it --
/// a leak the collector's two passes made unreachable.  The items are the
/// header's own array now, so they go with it.
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
    // The item array itself: `xfree` is the C's and runs no destructor.
    // SAFETY: as above.
    unsafe { ::core::ptr::drop_in_place(&raw mut (*l).lv_items) };
    unsafe { xfree(l.cast()) };
}

/// Free `l` and everything in it.  A no-op while `free_unref_items()` is
/// walking, which frees the whole graph itself.
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call.
pub unsafe fn list_free(l: *mut List) {
    if tv_in_free_unref_items.get() {
        return;
    }
    // SAFETY: the caller's promise: a live, unaliased list.
    list_free_contents(unsafe { &mut *l });
    unsafe { list_free_list(l) };
}

/// Drop a reference to `l`, freeing it when the last one goes.
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call.
pub unsafe fn list_unref(l: *mut List) {
    if let Some(list) = unsafe { l.as_mut() }
        && list.lv_refcount.release() <= 0
    {
        unsafe { list_free(l) };
    }
}

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

impl DictRef {
    /// See [`ListRef::from_owned`].
    ///
    /// # Safety
    ///
    /// As [`ListRef::from_owned`].
    #[inline(always)]
    pub unsafe fn from_owned(at: NonNull<Dict>) -> DictRef {
        DictRef(at)
    }

    /// See [`ListRef::owning`].
    ///
    /// # Safety
    ///
    /// As [`ListRef::owning`].
    #[inline(always)]
    pub unsafe fn owning(at: *mut Dict) -> Option<DictRef> {
        NonNull::new(at).map(DictRef)
    }

    /// See [`ListRef::retained`].
    ///
    /// # Safety
    ///
    /// As [`ListRef::retained`].
    #[inline(always)]
    pub unsafe fn retained(at: *mut Dict) -> Option<DictRef> {
        let at = NonNull::new(at)?;
        // SAFETY: the caller's promise: a live object.
        unsafe { (*at.as_ptr()).dv_refcount.retain() };
        Some(DictRef(at))
    }

    /// See [`ListRef::as_ptr`].
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut Dict {
        self.0.as_ptr()
    }

    /// See [`ListRef::into_raw`].
    #[inline(always)]
    pub fn into_raw(self) -> *mut Dict {
        let at = self.0;
        ::core::mem::forget(self);
        at.as_ptr()
    }

    /// See [`ListRef::ptr_eq`].
    #[inline(always)]
    pub fn ptr_eq(&self, other: &DictRef) -> bool {
        self.0 == other.0
    }

    /// See [`ListRef::edit`].
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
        // SAFETY: as `ListRef`'s.
        unsafe { tv_dict_unref(self.as_ptr()) };
    }
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
    unsafe { queue_init(&raw mut (*d).watchers) };
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

/// One reference to a [`Blob`], given back when the handle goes: the blob
/// half of [`ListRef`].
#[repr(transparent)]
pub struct BlobRef(NonNull<Blob>);

impl BlobRef {
    /// See [`ListRef::from_owned`].
    ///
    /// # Safety
    ///
    /// As [`ListRef::from_owned`].
    #[inline(always)]
    pub unsafe fn from_owned(at: NonNull<Blob>) -> BlobRef {
        BlobRef(at)
    }

    /// See [`ListRef::owning`].
    ///
    /// # Safety
    ///
    /// As [`ListRef::owning`].
    #[inline(always)]
    pub unsafe fn owning(at: *mut Blob) -> Option<BlobRef> {
        NonNull::new(at).map(BlobRef)
    }

    /// See [`ListRef::retained`].
    ///
    /// # Safety
    ///
    /// As [`ListRef::retained`].
    #[inline(always)]
    pub unsafe fn retained(at: *mut Blob) -> Option<BlobRef> {
        let at = NonNull::new(at)?;
        // SAFETY: the caller's promise: a live object.
        unsafe { (*at.as_ptr()).bv_refcount.retain() };
        Some(BlobRef(at))
    }

    /// See [`ListRef::as_ptr`].
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut Blob {
        self.0.as_ptr()
    }

    /// See [`ListRef::into_raw`].
    #[inline(always)]
    pub fn into_raw(self) -> *mut Blob {
        let at = self.0;
        ::core::mem::forget(self);
        at.as_ptr()
    }

    /// See [`ListRef::ptr_eq`].
    #[inline(always)]
    pub fn ptr_eq(&self, other: &BlobRef) -> bool {
        self.0 == other.0
    }

    /// See [`ListRef::edit`].
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
    #[inline(always)]
    fn drop(&mut self) {
        // SAFETY: as `ListRef`'s.
        unsafe { blob_unref(self.as_ptr()) };
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
    // SAFETY: the reference is this handle's own, and `blob_free` is the
    // `Box` it came from going back.
    unsafe { BlobRef::from_owned(NonNull::from(Box::leak(blob))) }
}

/// Free `b` and its bytes.
///
/// # Safety
///
/// `b` must point at a live blob, unaliased for the call.
pub unsafe fn blob_free(b: *mut Blob) {
    // SAFETY: the caller's promise: a live, unaliased blob, which
    // `tv_blob_alloc` made as a `Box`.
    drop(unsafe { Box::from_raw(b) });
}

/// Drop a reference to `b`, freeing it when the last one goes.
///
/// # Safety
///
/// `b` must point at a live blob, unaliased for the call.
pub unsafe fn blob_unref(b: *mut Blob) {
    if let Some(blob) = unsafe { b.as_mut() }
        && blob.bv_refcount.release() <= 0
    {
        unsafe { blob_free(b) };
    }
}

/// One reference to a [`Partial`], given back when the handle goes: the
/// partial half of [`ListRef`].
#[repr(transparent)]
pub struct PartialRef(NonNull<Partial>);

impl PartialRef {
    /// See [`ListRef::from_owned`].
    ///
    /// # Safety
    ///
    /// As [`ListRef::from_owned`].
    #[inline(always)]
    pub unsafe fn from_owned(at: NonNull<Partial>) -> PartialRef {
        PartialRef(at)
    }

    /// See [`ListRef::owning`].
    ///
    /// # Safety
    ///
    /// As [`ListRef::owning`].
    #[inline(always)]
    pub unsafe fn owning(at: *mut Partial) -> Option<PartialRef> {
        NonNull::new(at).map(PartialRef)
    }

    /// See [`ListRef::retained`].
    ///
    /// # Safety
    ///
    /// As [`ListRef::retained`].
    #[inline(always)]
    pub unsafe fn retained(at: *mut Partial) -> Option<PartialRef> {
        let at = NonNull::new(at)?;
        // SAFETY: the caller's promise: a live object.
        unsafe { (*at.as_ptr()).pt_refcount.retain() };
        Some(PartialRef(at))
    }

    /// See [`ListRef::as_ptr`].
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut Partial {
        self.0.as_ptr()
    }

    /// See [`ListRef::into_raw`].
    #[inline(always)]
    pub fn into_raw(self) -> *mut Partial {
        let at = self.0;
        ::core::mem::forget(self);
        at.as_ptr()
    }

    /// See [`ListRef::ptr_eq`].
    #[inline(always)]
    pub fn ptr_eq(&self, other: &PartialRef) -> bool {
        self.0 == other.0
    }

    /// See [`ListRef::edit`].
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
    #[inline(always)]
    fn drop(&mut self) {
        // SAFETY: as `ListRef`'s.
        unsafe { partial_unref(self.as_ptr()) };
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

/// Drop one reference to a partial, freeing it at zero.
///
/// # Safety
/// `pt` must be null or valid.
pub(crate) unsafe fn partial_unref(pt: *mut Partial) {
    if pt.is_null() {
        return;
    }
    // SAFETY: the caller's promise, and `pt` is not null.
    if unsafe { (*pt).pt_refcount.release() } <= 0 {
        unsafe { partial_free(pt) };
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
    if ::core::ffi::c_uint::from(unsafe { (*item).di_flags }) & DI_FLAGS_ALLOC != 0 {
        // The value and the key go with the item.
        // SAFETY: the caller's live item, which this allocated.
        drop(unsafe { Box::from_raw(item) });
    } else {
        unsafe { tv_clear(&mut (*item).di_tv) };
    }
}

impl Dict {
    /// The item under `key`, or `None` when there is none.
    ///
    /// The key is bytes, not a NUL-terminated string: the table hashes and
    /// compares exactly the bytes it is given, and the two spellings the C
    /// had (`hash_find` and `hash_find_len`) agree on every key a dictionary
    /// can hold, since a key that reaches the table is NUL-terminated and so
    /// has no NUL among its bytes.  A caller holding a `&CStr` says
    /// `key.to_bytes()`; one holding a `c"..."` literal pays nothing for it.
    ///
    /// **The answer borrows the dictionary, not the slot.** An item is its
    /// own allocation, so a rehash moves the slot and leaves the item where
    /// it was; what invalidates the borrow is the item being *removed*,
    /// which needs the exclusive borrow this one rules out.
    #[inline]
    pub fn find(&self, key: &[u8]) -> Option<&DictItem> {
        let at = self.find_ptr(key);
        // SAFETY: a non-null answer is one of this dictionary's own items,
        // which the borrow of the dictionary keeps alive.
        (!at.is_null()).then(|| unsafe { &*at })
    }

    /// [`Dict::find`] with the item writable.
    #[inline]
    pub fn find_mut(&mut self, key: &[u8]) -> Option<&mut DictItem> {
        let at = self.find_ptr(key);
        // SAFETY: a non-null answer is one of this dictionary's own items;
        // no two slots name the same one, and the exclusive borrow of the
        // dictionary keeps the set of them fixed.
        (!at.is_null()).then(|| unsafe { &mut *at })
    }

    /// The lookup both borrow forms are built on: the item under `key`, or
    /// null.
    ///
    /// **The answer is not derived from the borrow.** The table's slot holds
    /// a `*mut DictItem` that came from the item's own allocation, and this
    /// copies it out, so writing through it is sound where casting a shared
    /// borrow's address would not be.
    ///
    /// It is the escape hatch for the two bodies that must hold an item
    /// *while* they reach the dictionary again -- `extend()`'s overwrite
    /// branch, which re-enters through `value_check_lock`, and `remove()`,
    /// which takes the value out and then unlinks the item. Everything else
    /// wants [`Dict::find`].
    #[inline]
    pub(crate) fn find_ptr(&self, key: &[u8]) -> *mut DictItem {
        // SAFETY: a live table of this dictionary's own, and `key` is a
        // slice, so it is readable for its length. The two spellings the C
        // had agree here: a key that reaches the table is NUL-terminated, so
        // it has no NUL among its bytes, which is the only input the
        // length-taking hash treats differently.
        let hi = unsafe {
            hash_find_len(
                &raw const self.dv_hashtab,
                key.as_ptr().cast::<::core::ffi::c_char>(),
                key.len(),
            )
        };
        if hi.is_kept() {
            tv_dict_hi2di(hi)
        } else {
            ::core::ptr::null_mut()
        }
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

    /// Append a bit copy of each of the caller's values.
    pub(crate) fn extend_borrowed(&mut self, tvs: &[TypVal]) {
        for tv in tvs {
            self.push_borrowed(tv);
        }
    }

    /// Put a bit copy of `tv` in front of everything already in the frame,
    /// which is what makes `base->Method(a)` a call of `Method(base, a)`.
    pub(crate) fn insert_borrowed_front(&mut self, tv: &TypVal) {
        self.push_borrowed(tv);
        self.rotate_last_to_front();
    }
}
