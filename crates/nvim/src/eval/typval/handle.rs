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
//!    retains, `Drop` releases and frees with the last one. A List, Dict or
//!    Blob is borrowed only through `read()` ([`Read`]) and `edit()`
//!    ([`Edit`]), counted in the object's [`Lent`] word as a `RefCell` counts
//!    its borrows: any number of reads or one edit, every build, so a clone's
//!    edit under a live read (or a read under an edit) panics where it
//!    happens instead of reallocating under it. A partial is fixed once
//!    built: `Deref` only, and [`PartialRef::get_mut`] for its last holder.
//!    The count is a `Cell`, so cloning and dropping touch no borrow. The raw
//!    bridges (`owning`, `retained`) stay `unsafe fn`s for the callers
//!    outside the container model that still hold a pointer.
//! 2. **Allocation and release**: the four objects' allocations, the
//!    reference-count release that frees them, and the collector's
//!    `TV_IN_FREE_UNREF_ITEMS` gate that makes a release free nothing while
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
//! API, and this file is a perimeter row under the raw-memory-primitive
//! rule (docs/perimeter.md): the handles are to the value graph what
//! `winlayer`'s are to the window graph.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `eval/typval/handle.rs` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use ::core::marker::PhantomData;
use ::core::ptr::NonNull;

use super::edit::{Lent, Read};
use super::*;
use crate::hashtab::{HashValue, SlotEntry};
use crate::types::Refcount;

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
/// The list itself is borrowed only through [`ListRef::read`] and
/// [`ListRef::edit`].
///
/// The two constructors are the two things a raw pointer can mean.  A handle
/// is not null: `v:_null_list` is `TypVal::list(None)`, which
/// [`TypVal::list_ref`] answers as `None`.
#[repr(transparent)]
#[derive(Debug)]
pub struct ListRef(NonNull<List>);

impl ListRef {
    /// Take over the caller's reference, or answer `None` for NULL.
    ///
    /// # Safety
    ///
    /// `at` is null or points at a live object, and the caller holds a
    /// reference to it -- one this handle now owns and eventually gives
    /// back.
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

    /// The object, writable, from a shared handle, for the statement that
    /// asked: **never across a call that can run user code**, which reaches
    /// it through every other holder. Every build checks it: see [`Edit`].
    #[inline(always)]
    pub fn edit(&self) -> Edit<'_, List> {
        lend(&self.0, ::core::mem::offset_of!(List, lv_lent))
    }

    /// The object, shared while the [`Read`] lives: an `edit()` through any
    /// handle meanwhile panics, so a body running user code lets it go.
    #[inline(always)]
    pub fn read(&self) -> Read<'_, List> {
        share(&self.0, ::core::mem::offset_of!(List, lv_lent))
    }

    /// The reference count, through its own bytes only: a clone or
    /// release must not claim the whole object, which a walk over it
    /// (a dictionary's inline slots) may be borrowing at the time. Shared,
    /// because the count is a `Cell`: a holder reading the object meanwhile
    /// stays valid.
    #[inline(always)]
    pub(super) fn count(&self) -> &Refcount {
        // SAFETY: the handle holds a reference, so the object is live;
        // the borrow covers the count alone.
        unsafe { &(*self.0.as_ptr()).lv_refcount }
    }
}

impl Clone for ListRef {
    /// One more owner of the same object.
    #[inline(always)]
    fn clone(&self) -> ListRef {
        self.count().retain();
        ListRef(self.0)
    }
}

impl Drop for ListRef {
    /// Give the reference back, freeing the object with the last one.
    #[inline(always)]
    fn drop(&mut self) {
        if self.count().release() <= 0 {
            free_list(self);
        }
    }
}

/// Free `list` and everything in it: its last reference has just gone.
/// A no-op while `free_unref_items()` is walking, which frees the whole
/// graph itself.
fn free_list(list: &ListRef) {
    if TV_IN_FREE_UNREF_ITEMS.get() {
        return;
    }
    // The handle is the view the contents are freed through: it owns the
    // reference that was the last, and gives it back by freeing.
    list_free_contents(list);
    // SAFETY: a live list, which nothing references any more.
    unsafe { list_free_list(list.as_ptr()) };
}

/// The most [`tv_list_alloc`] reserves up front.
///
/// A caller's length is a hint, and not every caller can prove it: a
/// decoder's container header or a Lua table's largest index is only a
/// claim. Past this the list grows as items really arrive, as the msgpack
/// unpacker's own cap has it.
const LIST_RESERVE_MAX: usize = 1 << 16;

/// Allocate an empty list, **owned by the handle it answers** (a count of
/// one, where upstream answered zero). `len` is a capacity hint, capped at
/// [`LIST_RESERVE_MAX`]; a negative one (`LIST_LEN_UNKNOWN`) reserves
/// nothing.
pub fn tv_list_alloc(len: ptrdiff_t) -> ListRef {
    let reserve = usize::try_from(len).map_or(0, |len| len.min(LIST_RESERVE_MAX));
    // The `xmalloc` family, not a `Box`: the unit cases' allocation log sees
    // only that family.
    let raw = unsafe { xcalloc(1, ::core::mem::size_of::<List>()) }.cast::<List>();
    let at = NonNull::new(raw).expect("xcalloc never answers null");
    // Written, not assigned: a zeroed `List` is not a valid one (`Vec` never
    // holds a null pointer), so there is nothing there to drop.
    // SAFETY: the allocation just made, of exactly this size.
    unsafe {
        raw.write(List {
            lv_items: Vec::with_capacity(reserve),
            lv_refcount: Refcount::one(),
            ..List::empty()
        });
    }
    // The reference just seeded is the one this handle owns.
    let list = ListRef(at);
    // The collector reaches every live list through its registry.
    list.edit().lv_root = root_list(at);
    list
}

/// Take `l` out of the garbage collector's registry and free the `List`,
/// with whatever items are still in its array.
///
/// # Safety
///
/// `l` must point at a live list [`tv_list_alloc`] made, unaliased for the
/// call and named by nothing afterwards.  Anything still in it is
/// **released**, so no caller may hold a reference to an item.
pub unsafe fn list_free_list(l: *mut List) {
    // SAFETY: the caller's promise: a live list, which this moves out of
    // its allocation; nothing reads the allocation again but the free.
    let list = unsafe { l.read() };
    // Out of the collector's registry. A list the registry never held
    // carries `RootId::NONE`; the removal is a no-op for it.
    unroot_list(list.lv_root);
    // NLUA_CLEAR_REF
    if list.lua_table_ref != LUA_NOREF {
        release_luaref(list.lua_table_ref);
    }
    // The item array and the cursor array first, then the header: the
    // order the allocation log pins.
    drop(list);
    // SAFETY: the `xcalloc` `tv_list_alloc` made, its contents moved out.
    unsafe { xfree(l.cast()) };
}

impl Dict {
    /// Every entry, in slot order -- which is the order Vim shows.
    ///
    /// The walk borrows the dictionary, so nothing can add to or remove from
    /// it while the walk is live. A body that edits the table wants
    /// [`DictCursor`], which is a slot index.
    pub fn items(&self) -> impl Iterator<Item = &DictItem> {
        // Not `item_at` per index: the walk over the slots themselves skips
        // a bounds check and a slot copy per slot, which `tvequal` reads as
        // +0.3 % Ir.
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
#[derive(Debug)]
pub struct DictRef(NonNull<Dict>);

/// The same API as [`ListRef`]'s, method for method.
impl DictRef {
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
    pub fn edit(&self) -> Edit<'_, Dict> {
        lend(&self.0, ::core::mem::offset_of!(Dict, dv_lent))
    }

    /// As [`ListRef::read`].
    #[inline(always)]
    pub fn read(&self) -> Read<'_, Dict> {
        share(&self.0, ::core::mem::offset_of!(Dict, dv_lent))
    }

    /// As [`ListRef::count`].
    #[inline(always)]
    pub(super) fn count(&self) -> &Refcount {
        // SAFETY: the handle holds a reference, so the object is live;
        // the borrow covers the count alone.
        unsafe { &(*self.0.as_ptr()).dv_refcount }
    }

    /// Whether no borrow holds it: an `edit()` now would not be refused.
    pub(crate) fn is_free(&self) -> bool {
        let offset = ::core::mem::offset_of!(Dict, dv_lent);
        Claim::new(self.0, offset).word().is_free()
    }
}

impl Clone for DictRef {
    #[inline(always)]
    fn clone(&self) -> DictRef {
        self.count().retain();
        DictRef(self.0)
    }
}

impl Drop for DictRef {
    #[inline(always)]
    fn drop(&mut self) {
        if self.count().release() <= 0 {
            free_dict(self);
        }
    }
}

/// [`free_list`]'s counterpart: free `dict` and everything in it.
fn free_dict(dict: &DictRef) {
    if TV_IN_FREE_UNREF_ITEMS.get() {
        return;
    }
    tv_dict_free_contents(dict);
    // SAFETY: a live dictionary whose contents are gone, which nothing
    // references any more.
    unsafe { tv_dict_free_dict(dict.as_ptr()) };
}

/// Allocate an empty dictionary, **owned by the handle it answers**; see
/// [`tv_list_alloc`].
pub fn tv_dict_alloc() -> DictRef {
    // The `xmalloc` family, for the allocation log, as a list's.
    let raw = unsafe { xcalloc(1, ::core::mem::size_of::<Dict>()) }.cast::<Dict>();
    let at = NonNull::new(raw).expect("xcalloc never answers null");
    // Written, not assigned: zeroed storage is not a valid `Dict`.
    // SAFETY: the allocation just made, of exactly this size.
    unsafe {
        raw.write(Dict {
            dv_lock: VarLock::Unlocked,
            dv_lent: Lent::new(),
            dv_scope: VAR_NO_SCOPE,
            dv_refcount: Refcount::one(),
            dv_copy_id: 0,
            dv_hashtab: DictTab::init(),
            dv_copydict: ::core::ptr::null_mut(),
            dv_root: RootId::NONE,
            watchers: Vec::new(),
            lua_table_ref: LUA_NOREF as LuaRef,
        });
    }
    // The reference just seeded is the one this handle owns.
    let dict = DictRef(at);
    // The collector reaches every live dictionary through its registry.
    dict.edit().dv_root = root_dict(at);
    dict
}

/// Unlink `d` from the garbage collector's chain and free the `Dict` itself.
///
/// # Safety
/// `d` must point at a live dictionary [`tv_dict_alloc`] made, whose
/// contents have already been freed ([`tv_dict_free_contents`]), and the
/// caller must own the collector's registry, which this takes it out of.
/// `d` is dangling afterwards.
pub unsafe fn tv_dict_free_dict(d: *mut Dict) {
    // SAFETY: the caller's promise: a live dictionary, which this moves out
    // of its allocation; nothing reads the allocation again but the free.
    let dict = unsafe { d.read() };
    // Out of the collector's registry.
    unroot_dict(dict.dv_root);
    // NLUA_CLEAR_REF
    if dict.lua_table_ref != LUA_NOREF {
        release_luaref(dict.lua_table_ref);
    }
    drop(dict);
    // SAFETY: the `xcalloc` `tv_dict_alloc` made, its contents moved out.
    unsafe { xfree(d.cast()) };
}

// ---------------------------------------------------------------------------
// The collector's free pass
//
// While `TV_IN_FREE_UNREF_ITEMS` is set no release frees anything, so every
// container the registry names stays live for the pass; these are the
// views and frees the pass works with, so that `eval/collect.rs` holds no
// pointer of its own.

/// A view of the dictionary in the collector's registry slot `idx`, taking
/// no reference: `None` for a hole or a slot past the end.
pub(crate) fn registered_dict(idx: usize) -> Option<::core::mem::ManuallyDrop<DictRef>> {
    let at = crate::eval::gc::dict_at(idx)?;
    // A registered dictionary is live until it leaves the registry, and the
    // view takes no reference, so dropping it changes nothing.
    Some(::core::mem::ManuallyDrop::new(DictRef(at)))
}

/// [`registered_dict`] for the list registry.
pub(crate) fn registered_list(idx: usize) -> Option<::core::mem::ManuallyDrop<ListRef>> {
    let at = crate::eval::gc::list_at(idx)?;
    // As above.
    Some(::core::mem::ManuallyDrop::new(ListRef(at)))
}

/// Free the structure of a dictionary the pass has already emptied, taking
/// it out of the registry.
pub(crate) fn free_registered_dict(view: ::core::mem::ManuallyDrop<DictRef>) {
    // SAFETY: an emptied, registered dictionary; the pass owns the registry
    // and the view is the last thing naming it.
    unsafe { tv_dict_free_dict(view.as_ptr()) };
}

/// [`free_registered_dict`] for a list.
pub(crate) fn free_registered_list(view: ::core::mem::ManuallyDrop<ListRef>) {
    // SAFETY: as above.
    unsafe { list_free_list(view.as_ptr()) };
}

/// One reference to a [`Blob`], given back when the handle goes: the blob
/// half of [`ListRef`].
#[repr(transparent)]
#[derive(Debug)]
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
    pub fn edit(&self) -> Edit<'_, Blob> {
        lend(&self.0, ::core::mem::offset_of!(Blob, bv_lent))
    }

    #[inline(always)]
    pub fn read(&self) -> Read<'_, Blob> {
        share(&self.0, ::core::mem::offset_of!(Blob, bv_lent))
    }

    /// As [`ListRef::count`].
    #[inline(always)]
    pub(super) fn count(&self) -> &Refcount {
        // SAFETY: the handle holds a reference, so the object is live;
        // the borrow covers the count alone.
        unsafe { &(*self.0.as_ptr()).bv_refcount }
    }
}

impl Clone for BlobRef {
    #[inline(always)]
    fn clone(&self) -> BlobRef {
        self.count().retain();
        BlobRef(self.0)
    }
}

impl Drop for BlobRef {
    /// Give the reference back; the last one frees the blob.
    #[inline(always)]
    fn drop(&mut self) {
        if self.count().release() <= 0 {
            // SAFETY: the last reference to the `Box` `tv_blob_alloc` leaked.
            drop(unsafe { Box::from_raw(self.as_ptr()) });
        }
    }
}

/// Allocate an empty blob, **owned by the handle it answers**.
///
/// The blob arrives at a reference count of **one**, as
/// [`tv_list_alloc`](super::tv_list_alloc) does and where upstream answered
/// zero: a caller that stores it nowhere drops the handle, and that is the
/// free.
pub fn tv_blob_alloc() -> BlobRef {
    let blob = Box::new(Blob {
        bv_data: Vec::new(),
        bv_refcount: Refcount::zero(),
        bv_lock: VarLock::Unlocked,
        bv_lent: Lent::new(),
    });
    // The count starts at the handle's one.
    blob.bv_refcount.retain();
    BlobRef(NonNull::from(Box::leak(blob)))
}

/// One reference to a [`Partial`], given back when the handle goes: the
/// partial half of [`ListRef`].
#[repr(transparent)]
#[derive(Debug)]
pub struct PartialRef(NonNull<Partial>);

/// The same API as [`ListRef`]'s, less the raw bridges: a partial is only
/// ever made by [`PartialRef::new`].
impl PartialRef {
    /// A partial built in place, owned by the handle (a count of one).
    pub fn new(mut partial: Partial) -> PartialRef {
        partial.pt_refcount = Refcount::one();
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

    /// The partial, writable, while this handle is its only holder:
    /// `Rc::get_mut`'s answer, and the only `&mut` a partial ever lends
    /// (it is fixed once built, so every other borrow is shared).
    #[inline(always)]
    pub fn get_mut(&mut self) -> Option<&mut Partial> {
        if self.count().get() != 1 {
            return None;
        }
        // SAFETY: a count of one is this handle, and the exclusive borrow
        // of it rules out a shared borrow lent through it.
        Some(unsafe { self.0.as_mut() })
    }

    /// As [`ListRef::count`].
    #[inline(always)]
    pub(super) fn count(&self) -> &Refcount {
        // SAFETY: the handle holds a reference, so the object is live;
        // the borrow covers the count alone.
        unsafe { &(*self.0.as_ptr()).pt_refcount }
    }
}

impl Clone for PartialRef {
    #[inline(always)]
    fn clone(&self) -> PartialRef {
        self.count().retain();
        PartialRef(self.0)
    }
}

impl Drop for PartialRef {
    /// Give the reference back; the last one frees the partial.
    fn drop(&mut self) {
        if self.count().release() <= 0 {
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

/// The object `at` names, writable for as long as the handle holding `at`
/// is borrowed: every `edit()`. `lent` is `offset_of!` its [`Lent`] flag.
#[inline(always)]
fn lend<T>(at: &NonNull<T>, lent: usize) -> Edit<'_, T> {
    let loan = Loan::new(*at, lent);
    Edit {
        target: *at,
        loan,
        borrow: PhantomData,
    }
}

impl<'a, T> Edit<'a, T> {
    /// A borrow of part of the object, under the same loan.
    #[inline(always)]
    pub fn map<U>(self, part: impl FnOnce(&'a mut T) -> &'a mut U) -> Edit<'a, U> {
        let Edit { target, loan, .. } = self;
        // SAFETY: as `deref_mut`'s.
        let part = part(unsafe { &mut *target.as_ptr() });
        Edit {
            target: NonNull::from(part),
            loan,
            borrow: PhantomData,
        }
    }

    /// [`Edit::map`] for a part that may be absent.
    #[inline(always)]
    pub fn filter_map<U>(
        self,
        part: impl FnOnce(&'a mut T) -> Option<&'a mut U>,
    ) -> Option<Edit<'a, U>> {
        let Edit { target, loan, .. } = self;
        // SAFETY: as `deref_mut`'s.
        let part = part(unsafe { &mut *target.as_ptr() })?;
        Some(Edit {
            target: NonNull::from(part),
            loan,
            borrow: PhantomData,
        })
    }
}

impl<T> ::core::ops::Deref for Edit<'_, T> {
    type Target = T;

    #[inline(always)]
    fn deref(&self) -> &T {
        // SAFETY: as `deref_mut`'s, shared.
        unsafe { self.target.as_ref() }
    }
}

impl<T> ::core::ops::DerefMut for Edit<'_, T> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: the handle the edit borrows keeps the object live, and
        // the loan keeps every other borrow of it out until the edit ends.
        unsafe { self.target.as_mut() }
    }
}

/// A borrow's claim on its object's [`Lent`] word: the word's address, not
/// a borrow of it. An edit holds the whole object `&mut`, word included, so
/// the word is reached only through the handle's pointer, projected (never
/// re-borrowed), before the edit is made and after its last use.
#[derive(Debug)]
struct Claim(*const Lent);

impl Claim {
    /// The word of the object at `at`, the field at `offset`.
    #[inline(always)]
    fn new<T>(at: NonNull<T>, offset: usize) -> Claim {
        Claim(at.as_ptr().cast_const().wrapping_byte_add(offset).cast())
    }

    #[inline(always)]
    fn word(&self) -> &Lent {
        // SAFETY: a `Lent` field of a live object a handle names, which the
        // borrow's handle keeps alive; the borrow covers the word alone, and
        // an edit's own borrow of the object is not live across it.
        unsafe { &*self.0 }
    }
}

/// An [`Edit`]'s claim: the word raised to the edit's for its life. The
/// lifetime is the handle's borrow, which the destructor keeps alive to
/// the loan's end: a handle moved or dropped under it does not compile.
#[derive(Debug)]
pub(super) struct Loan<'a>(Claim, PhantomData<&'a ()>);

impl Loan<'_> {
    #[inline(always)]
    fn new<T>(at: NonNull<T>, offset: usize) -> Self {
        let claim = Claim::new(at, offset);
        claim.word().lend::<T>();
        Loan(claim, PhantomData)
    }
}

impl Drop for Loan<'_> {
    #[inline(always)]
    fn drop(&mut self) {
        self.0.word().give_back();
    }
}

/// A [`Read`]'s claim: one count in the word for its life.
#[derive(Debug)]
pub(super) struct Share(Claim);

impl Share {
    #[inline(always)]
    fn new<T>(at: NonNull<T>, offset: usize) -> Share {
        let claim = Claim::new(at, offset);
        claim.word().share::<T>();
        Share(claim)
    }
}

impl Drop for Share {
    #[inline(always)]
    fn drop(&mut self) {
        self.0.word().unshare();
    }
}

/// The object `at` names, shared for as long as the handle holding `at` is
/// borrowed: every `read()`. `lent` is `offset_of!` its [`Lent`] word.
#[inline(always)]
fn share<T>(at: &NonNull<T>, lent: usize) -> Read<'_, T> {
    let share = Share::new(*at, lent);
    // SAFETY: the handle holds a reference, so the object is live; the
    // share just taken refuses an edit while this borrow lives.
    let target = unsafe { at.as_ref() };
    let share = Some(share);
    Read { target, share }
}

/// What a dictionary's hash table holds in an occupied slot: the item.
///
/// `Copy`, and a raw pointer, for the same reason the slots of every other
/// table are: the table names its items, and which of them it is responsible
/// for freeing is the item's own `DI_FLAGS_ALLOC`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct DictEntry(*mut DictItem);

impl DictEntry {
    /// The item this entry names. Meaningless for an empty or removed slot.
    pub fn item(self) -> *mut DictItem {
        self.0
    }
}

/// The tombstone: `NonNull::dangling()`'s address, the item type's
/// alignment, which no allocation can be at and nothing dereferences.
impl SlotEntry for DictEntry {
    const EMPTY: Self = DictEntry(::core::ptr::null_mut());

    fn is_empty(self) -> bool {
        self.0.is_null()
    }

    fn is_removed(self) -> bool {
        self.0 == NonNull::<DictItem>::dangling().as_ptr()
    }

    fn removed() -> Self {
        DictEntry(NonNull::dangling().as_ptr())
    }
}

/// Release every item of a table already taken out of its dictionary, in
/// slot order: the whole-dictionary free, without a removal per slot. An
/// embedded item (not `DI_FLAGS_ALLOC`) is only emptied.
pub(crate) fn release_table(table: DictTab) {
    for hi in table.items() {
        // SAFETY: a kept slot of a table nothing else can reach any more
        // names a live item, released once here.
        drop(unsafe { take_item(hi.hi_key.item()) });
    }
}

/// What `item` holds comes out: an allocated item whole, an embedded one's
/// value.
///
/// # Safety
///
/// `item` is a live item that no table holds any more, and nothing else
/// takes it. An allocated one (`DI_FLAGS_ALLOC`) is the `Box`
/// `DictItem::boxed` made; an embedded one belongs to the structure it
/// lives in, which keeps it.
unsafe fn take_item(item: *mut DictItem) -> RemovedItem {
    // SAFETY: the caller's promise above.
    unsafe {
        if ::core::ffi::c_uint::from((*item).di_flags) & DI_FLAGS_ALLOC != 0 {
            RemovedItem::Allocated(Box::from_raw(item))
        } else {
            RemovedItem::Embedded((*item).di_tv.take())
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

    /// The slot `key` (hashing to `hash`) occupies, or the one it belongs in.
    #[inline]
    pub(super) fn probe_key(&self, key: &[u8], hash: HashValue) -> ItemSlot {
        // SAFETY: the probe hands the test only this dictionary's kept
        // entries, each naming a live item -- which owns its key -- that the
        // borrow of the dictionary keeps alive for the walk.
        self.dv_hashtab
            .probe(hash, |entry| unsafe { (*entry.0).di_key.bytes() } == key)
    }

    /// Put `item` in the table, which takes it over. When its key is
    /// already there the item comes back, untouched.
    pub(crate) fn insert(&mut self, item: Box<DictItem>) -> Result<(), Box<DictItem>> {
        let Ok((hi, hash)) = self.vacant_slot(&item.di_key) else {
            return Err(item);
        };
        self.dv_hashtab
            .insert_at(hi, DictEntry(Box::into_raw(item)), hash);
        Ok(())
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
        self.dv_hashtab.remove_at(hi);
        // SAFETY: the item the slot named, out of the table now.
        unsafe { take_item(hi.hi_key.item()) }
    }
}

/// Link `b:changedtick` -- the item embedded in `buffer` itself -- into the
/// buffer's `b:` dictionary, which only points at it.
///
/// The buffer owns both, and [`unlink_buf_changedtick`] takes the item out
/// of the table before either goes, which is what makes the link sound. `b:`
/// is neither scope the function-name check guards.
pub(crate) fn link_buf_changedtick(buffer: crate::winlayer::Buf) {
    let item = buffer.changedtick_item();
    let key = DictKey::new(b"changedtick");
    let mut held = buffer.edit();
    let Some(mut dict) = held.b_bufvar.di_tv.dict_mut() else {
        return;
    };
    // The buffer's own item, which outlives its stay in the table as above.
    if let Ok((hi, hash)) = dict.vacant_slot(&key) {
        dict.dv_hashtab.insert_at(hi, DictEntry(item), hash);
    }
}

/// Take `b:changedtick` out of `buffer`'s `b:` table, leaving the item --
/// and the counter in it -- alone: clearing it would zero the counter.
pub(crate) fn unlink_buf_changedtick(buffer: crate::winlayer::Buf) {
    let mut held = buffer.edit();
    let Some(mut dict) = held.b_bufvar.di_tv.dict_mut() else {
        return;
    };
    let slot = dict.slot_of(b"changedtick");
    debug_assert!(slot.is_some(), "changedtick is in the table");
    if let Some(slot) = slot {
        // The item the slot names is the buffer's, which this does not touch.
        let hi = dict.dv_hashtab.slot(slot);
        dict.dv_hashtab.remove_at(hi);
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
/// (`remember_copy`, the marking, is safe code: `list.rs`, `dict.rs`.)
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
    use crate::buffer::{bare_buffer, buf_init_changedtick, free_bare_buffer};
    use crate::eval::vars::{new_var_scope, release_var_scope, vars_clear};
    use crate::global_cell::editor_state_lock;
    use crate::types::VAR_SCOPE;

    /// `b:changedtick` is an item embedded in the buffer that the buffer's
    /// `b:` table points at: the table's pointer stays good while the buffer
    /// is written around it, and reads the counter the buffer keeps.
    #[test]
    fn b_changedtick_is_the_buffer_own_item() {
        let _held = editor_state_lock();
        let buffer = bare_buffer();
        buffer.edit().b_bufvar = new_var_scope(VAR_SCOPE);
        buf_init_changedtick(buffer);
        buffer.set_b_changed(1);
        buffer.edit().changedtick_di.di_tv.write_number(7);
        let vars = buffer
            .read()
            .b_bufvar
            .di_tv
            .dict_handle()
            .expect("a b: scope");
        let tick = vars
            .read()
            .find(b"changedtick")
            .map(|item| item.di_tv.number_or_zero());
        assert_eq!(tick, Some(7));
        unlink_buf_changedtick(buffer);
        assert!(vars.read().find(b"changedtick").is_none());
        vars_clear(&vars);
        drop(vars);
        release_var_scope(&mut buffer.edit().b_bufvar);
        free_bare_buffer(buffer);
    }

    /// A dictionary released to zero under the flag is left for the
    /// collector, which frees it explicitly afterwards. Here rather than
    /// beside the dictionary operations because reaching a dictionary nobody
    /// holds is a raw read.
    #[test]
    fn a_dict_released_mid_collection_waits_for_the_collector() {
        let _held = editor_state_lock();
        let d = tv_dict_alloc();
        d.edit().add_number(b"a", 0).expect("a key used once");
        d.edit().add_number(b"b", 1).expect("a key used once");
        let dp = d.as_ptr();
        TV_IN_FREE_UNREF_ITEMS.set(true);
        drop(d);
        // SAFETY: unreferenced but not freed.
        let (refs, b) = unsafe {
            let dict = &*dp;
            (
                dict.dv_refcount.get(),
                dict.find(b"b").map(|di| di.di_tv.number_or_zero()),
            )
        };
        TV_IN_FREE_UNREF_ITEMS.set(false);
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
            TypVal::Bool(BOOL_VAR_TRUE),
            TypVal::Special(SPECIAL_VAR_NULL),
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
