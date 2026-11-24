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
//!    [`PartialRef`]: a `NonNull` to a [`Counted`] -- the reference count,
//!    the [`Lent`] word and, for a list or a dictionary, the registry slot
//!    and the deep copy's memo, beside the value -- plus the reference it
//!    owns. `Clone` retains, `Drop` releases and frees with the last one. A
//!    List, Dict or Blob value is borrowed only through `read()` ([`Read`])
//!    and `edit()` ([`Edit`]), counted in the `Lent` word as a `RefCell`
//!    counts its borrows, every build. Neither lends the words beside the
//!    value, so nothing a borrow can do -- `mem::swap` through two edits
//!    included -- reaches who holds an object. A partial is fixed once
//!    built: `Deref` only, and [`PartialRef::get_mut`] for its last holder.
//!    The raw bridges (`owning`, `retained`) stay `unsafe fn`s.
//! 2. **Allocation and release**: the four objects' allocations, the
//!    reference-count release that frees them, the scopes' seeds, and the
//!    collector's `TV_IN_FREE_UNREF_ITEMS` gate that makes a release free
//!    nothing while `free_unref_items` owns the whole graph.
//! 3. **The bit copy** ([`TypVal::bit_copy`]): a value duplicated without a
//!    reference, so that an argument frame can *name* the caller's values
//!    for the length of a call.
//!
//! The dictionary's item store is `store.rs`'s.
//!
//! Every other file of `eval/typval/` reaches these objects through this
//! API, and this file is a perimeter row under the raw-memory-primitive
//! rule (docs/perimeter.md): the handles are to the value graph what
//! `winlayer`'s are to the window graph.

#![deny(unsafe_op_in_unsafe_fn)]
// Unsafe perimeter: the `eval/typval/{handle,store}.rs` row in docs/perimeter.md.
#![allow(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use ::core::cell::{Cell, RefCell};
use ::core::marker::PhantomData;
use ::core::ptr::NonNull;
use ::core::sync::atomic::{AtomicU64, Ordering};

use super::edit::{Lent, Read};
use super::*;
use crate::types::Refcount;

/// What a value handle points at: the words the handle model keeps, beside
/// the value it lends (`Rc`'s shape).
///
/// `read()` and `edit()` lend `value` alone, so no borrow a handle hands out
/// reaches the reference count, the [`Lent`] word or -- for a list or a
/// dictionary -- the collector's registry slot and the deep copy's memo: a
/// `mem::swap`/`mem::replace` through two edits moves two values, and the
/// words saying who holds each object stay with the object. The fields are
/// this file's.
#[derive(Debug)]
pub struct Counted<T, H = ()> {
    count: Refcount,
    lent: Lent,
    head: H,
    value: T,
}

/// A list's or a dictionary's further words: the slot the collector's
/// registry holds it in, and the copy the deep copy still running made of it.
#[derive(Debug)]
pub struct Node<T> {
    root: Cell<RootId>,
    memo: Cell<Memo<T>>,
}

/// The deep copy's memo: the walk's id and the copy it made.
type Memo<T> = Option<(u64, NonNull<Counted<T, Node<T>>>)>;

/// A [`List`] and its words: what a [`ListRef`] points at.
pub type ListBox = Counted<List, Node<List>>;
/// A [`Dict`] and its words: what a [`DictRef`] points at.
pub type DictBox = Counted<Dict, Node<Dict>>;
/// A [`Blob`] and its words: what a [`BlobRef`] points at.
pub type BlobBox = Counted<Blob>;
/// A [`Partial`] and its words: what a [`PartialRef`] points at.
pub type PartialBox = Counted<Partial>;

impl<T, H> Counted<T, H> {
    /// A fresh object, owned by the handle about to be made over it.
    const fn new(value: T, head: H) -> Self {
        Counted {
            count: Refcount::one(),
            lent: Lent::new(),
            head,
            value,
        }
    }
}

/// The raw bridges between an object's address and its value's, for the
/// unit cases written against value pointers. Computed, never borrowed;
/// dereferencing either is the caller's own `unsafe`.
impl<T, H> Counted<T, H> {
    /// Where the value of the object at `at` sits.
    pub fn value_ptr(at: *mut Self) -> *mut T {
        at.wrapping_byte_add(::core::mem::offset_of!(Self, value))
            .cast()
    }

    /// The object whose value sits at `value`.
    pub fn from_value_ptr(value: *mut T) -> *mut Self {
        value
            .wrapping_byte_sub(::core::mem::offset_of!(Self, value))
            .cast()
    }
}

impl<T> Node<T> {
    const fn new() -> Self {
        Node {
            root: Cell::new(RootId::NONE),
            memo: Cell::new(None),
        }
    }
}

/// The reference count of the object `at` names, through its own bytes: a
/// clone or release must not claim the value, which an edit may hold.
#[inline(always)]
fn count<T, H>(at: &NonNull<Counted<T, H>>) -> &Refcount {
    // SAFETY: a handle's object is live while the handle is; the borrow
    // covers the count alone, which no edit or read lends.
    unsafe { &(*at.as_ptr()).count }
}

/// [`count`] for a list's or dictionary's further words.
#[inline(always)]
fn node<T>(at: &NonNull<Counted<T, Node<T>>>) -> &Node<T> {
    // SAFETY: as `count`'s.
    unsafe { &(*at.as_ptr()).head }
}

/// Where the value of the object `at` names sits, for comparing with a
/// borrow of one. Computed, not borrowed: an edit may hold the value.
#[inline(always)]
fn value_addr<T, H>(at: &NonNull<Counted<T, H>>) -> *const T {
    Counted::value_ptr(at.as_ptr()).cast_const()
}

/// Proof of being this file, for the collector's registry: a container
/// enters it here, at its allocation, and leaves it here, at its free, so
/// the registry names only live containers.
#[derive(Debug)]
pub(crate) struct RootKey(());

/// An owning reference to a heap-allocated [`List`].
///
/// This is the reference count, as a type: [`Clone`] takes a reference and
/// [`Drop`] gives one back, freeing the list when the last one goes.  What
/// it replaces is upstream's `tv_list_ref`/`list_unref` pair, which every
/// call site had to remember to write, in the right order, on every path.
///
/// **A fresh handle owns one reference.**  [`tv_list_alloc`] answers a
/// `ListRef` at a count of one, where upstream handed back a list at *zero*
/// and left the first storer to raise it. The count is exactly the number of
/// live holders, and there is no state in which a list is alive with none.
///
/// The list itself is borrowed only through [`ListRef::read`] and
/// [`ListRef::edit`].
///
/// The two constructors are the two things a raw pointer can mean.  A handle
/// is not null: `v:_null_list` is `TypVal::list(None)`, which
/// [`TypVal::list_ref`] answers as `None`.
#[repr(transparent)]
#[derive(Debug)]
pub struct ListRef(NonNull<ListBox>);

impl ListRef {
    /// Take over the caller's reference, or answer `None` for NULL.
    ///
    /// # Safety
    ///
    /// `at` is null or points at a live object, and the caller holds a
    /// reference to it -- one this handle now owns and eventually gives
    /// back.
    #[inline(always)]
    pub unsafe fn owning(at: *mut ListBox) -> Option<ListRef> {
        NonNull::new(at).map(ListRef)
    }

    /// Take *another* reference: the caller keeps its own. `None` for
    /// NULL, which counts nothing.
    ///
    /// # Safety
    ///
    /// `at` is null or points at a live object.
    #[inline(always)]
    pub unsafe fn retained(at: *mut ListBox) -> Option<ListRef> {
        let at = NonNull::new(at)?;
        count(&at).retain();
        Some(ListRef(at))
    }

    /// The object, as the pointer a callee outside the container
    /// model still takes. A **borrow**: live only while the handle is.
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut ListBox {
        self.0.as_ptr()
    }

    /// Give the reference up without releasing it: something else
    /// owns it now.
    #[inline(always)]
    pub fn into_raw(self) -> *mut ListBox {
        let at = self.0;
        ::core::mem::forget(self);
        at.as_ptr()
    }

    /// Whether the two handles name the same object.
    #[inline(always)]
    pub fn ptr_eq(&self, other: &ListRef) -> bool {
        self.0 == other.0
    }

    /// Whether `value` is this handle's list (a borrow taken through it, or
    /// a clone of it).
    #[inline(always)]
    pub fn holds(&self, value: &List) -> bool {
        ::core::ptr::eq(value_addr(&self.0), value)
    }

    /// The object, writable, from a shared handle, for the statement that
    /// asked: **never across a call that can run user code**, which reaches
    /// it through every other holder. Every build checks it: see [`Edit`].
    #[inline(always)]
    pub fn edit(&self) -> Edit<'_, List> {
        lend(&self.0)
    }

    /// The object, shared while the [`Read`] lives: an `edit()` through any
    /// handle meanwhile panics, so a body running user code lets it go.
    #[inline(always)]
    pub fn read(&self) -> Read<'_, List> {
        share(&self.0)
    }

    /// How many references the list has, for messages, tests and the
    /// scopes' seed comparisons. Not for arithmetic: nothing writes it back.
    #[inline(always)]
    pub fn refcount(&self) -> i32 {
        count(&self.0).get()
    }

    /// Hold `n` references as one ([`ListSeed`]).
    pub(crate) fn seed(&self, n: ::core::ffi::c_int) -> ListSeed {
        ListSeed(self.clone(), seed_extra(count(&self.0), n))
    }

    /// Take the list out of the collector's registry: the holder marks it
    /// itself (a funccall's `a:000`, a submatch list).
    pub(crate) fn unroot(&self) {
        let root = &node(&self.0).root;
        unroot_list(&RootKey(()), root.replace(RootId::NONE));
    }

    /// Record `copy` as this list's copy under `walk`: the deep copy's
    /// memo, which [`ListRef::copy_under`] reads back. The walk keeps a
    /// reference to the copy until it ends.
    pub(crate) fn remember_copy(&self, walk: &CopyWalk, copy: &ListRef) {
        node(&self.0).memo.set(Some((walk.id, copy.0)));
        walk.lists.borrow_mut().push(copy.clone());
    }

    /// The copy `walk` made of this list, with a reference of its own.
    pub(crate) fn copy_under(&self, walk: &CopyWalk) -> Option<ListRef> {
        let (id, at) = node(&self.0).memo.get()?;
        // SAFETY: a memo under `walk`'s id names a copy `walk` holds a
        // reference to (`remember_copy`); ids are never reused.
        (id == walk.id).then(|| unsafe { ListRef::retained(at.as_ptr()) })?
    }
}

impl Clone for ListRef {
    /// One more owner of the same object.
    #[inline(always)]
    fn clone(&self) -> ListRef {
        count(&self.0).retain();
        ListRef(self.0)
    }
}

impl Drop for ListRef {
    /// Give the reference back, freeing the object with the last one.
    #[inline(always)]
    fn drop(&mut self) {
        if count(&self.0).release() <= 0 {
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
    let list = List {
        lv_items: Vec::with_capacity(reserve),
        ..List::empty()
    };
    let at = NonNull::from(Box::leak(Box::new(Counted::new(list, Node::new()))));
    // The collector reaches every live list through its registry.
    node(&at).root.set(root_list(&RootKey(()), at));
    // The reference just seeded is the one this handle owns.
    ListRef(at)
}

/// Take `l` out of the garbage collector's registry and free the `List`,
/// with whatever items are still in its array.
///
/// # Safety
///
/// `l` must point at a live list [`tv_list_alloc`] made, unaliased for the
/// call and named by nothing afterwards.  Anything still in it is
/// **released**, so no caller may hold a reference to an item.
pub unsafe fn list_free_list(l: *mut ListBox) {
    // SAFETY: the caller's promise: a live list -- the `Box`
    // `tv_list_alloc` leaked -- which nothing names afterwards.
    let list = unsafe { Box::from_raw(l) };
    // Out of the collector's registry. A list the registry never held
    // carries `RootId::NONE`; the removal is a no-op for it.
    unroot_list(&RootKey(()), list.head.root.get());
    // NLUA_CLEAR_REF
    if list.value.lua_table_ref != LUA_NOREF {
        release_luaref(list.value.lua_table_ref);
    }
    drop(list);
}

/// An owning reference to a heap-allocated [`Dict`]: [`ListRef`]'s
/// counterpart, and the same statement about a reference count.
///
/// **The scope dictionaries are heap dictionaries too.** `b:`, `w:`, `t:`,
/// `g:`, `v:` and a funccall's `l:`/`a:` hold a [`DictSeed`] of
/// `DO_NOT_FREE_CNT` references while their scope lives, and `g:`, `v:`,
/// `s:`, `l:` and `a:` leave the collector's registry, which only marks
/// their items.
#[repr(transparent)]
#[derive(Debug)]
pub struct DictRef(NonNull<DictBox>);

/// The same API as [`ListRef`]'s, method for method.
impl DictRef {
    /// # Safety
    /// As [`ListRef::owning`].
    #[inline(always)]
    pub unsafe fn owning(at: *mut DictBox) -> Option<DictRef> {
        NonNull::new(at).map(DictRef)
    }

    /// # Safety
    /// As [`ListRef::retained`].
    #[inline(always)]
    pub unsafe fn retained(at: *mut DictBox) -> Option<DictRef> {
        let at = NonNull::new(at)?;
        count(&at).retain();
        Some(DictRef(at))
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *mut DictBox {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub fn into_raw(self) -> *mut DictBox {
        let at = self.0;
        ::core::mem::forget(self);
        at.as_ptr()
    }

    #[inline(always)]
    pub fn ptr_eq(&self, other: &DictRef) -> bool {
        self.0 == other.0
    }

    /// As [`ListRef::holds`].
    #[inline(always)]
    pub fn holds(&self, value: &Dict) -> bool {
        ::core::ptr::eq(value_addr(&self.0), value)
    }

    #[inline(always)]
    pub fn edit(&self) -> Edit<'_, Dict> {
        lend(&self.0)
    }

    /// As [`ListRef::read`].
    #[inline(always)]
    pub fn read(&self) -> Read<'_, Dict> {
        share(&self.0)
    }

    /// As [`ListRef::refcount`].
    #[inline(always)]
    pub fn refcount(&self) -> i32 {
        count(&self.0).get()
    }

    /// Hold `n` references as one ([`DictSeed`]).
    pub(crate) fn seed(&self, n: ::core::ffi::c_int) -> DictSeed {
        DictSeed(self.clone(), seed_extra(count(&self.0), n))
    }

    /// As [`ListRef::unroot`]: `g:`, `v:`, `s:` and the funccall scopes.
    pub(crate) fn unroot(&self) {
        let root = &node(&self.0).root;
        unroot_dict(&RootKey(()), root.replace(RootId::NONE));
    }

    /// As [`ListRef::remember_copy`].
    pub(crate) fn remember_copy(&self, walk: &CopyWalk, copy: &DictRef) {
        node(&self.0).memo.set(Some((walk.id, copy.0)));
        walk.dicts.borrow_mut().push(copy.clone());
    }

    /// As [`ListRef::copy_under`].
    pub(crate) fn copy_under(&self, walk: &CopyWalk) -> Option<DictRef> {
        let (id, at) = node(&self.0).memo.get()?;
        // SAFETY: as `ListRef::copy_under`'s.
        (id == walk.id).then(|| unsafe { DictRef::retained(at.as_ptr()) })?
    }

    /// Whether no borrow holds it: an `edit()` now would not be refused.
    pub(crate) fn is_free(&self) -> bool {
        let offset = ::core::mem::offset_of!(DictBox, lent);
        Claim::new(self.0, offset).word().is_free()
    }
}

impl Clone for DictRef {
    #[inline(always)]
    fn clone(&self) -> DictRef {
        count(&self.0).retain();
        DictRef(self.0)
    }
}

impl Drop for DictRef {
    #[inline(always)]
    fn drop(&mut self) {
        if count(&self.0).release() <= 0 {
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
    let dict = Dict {
        dv_lock: VarLock::Unlocked,
        dv_scope: VAR_NO_SCOPE,
        dv_copy_id: 0,
        dv_hashtab: DictTab::init(),
        watchers: Vec::new(),
        lua_table_ref: LUA_NOREF as LuaRef,
    };
    let at = NonNull::from(Box::leak(Box::new(Counted::new(dict, Node::new()))));
    // The collector reaches every live dictionary through its registry.
    node(&at).root.set(root_dict(&RootKey(()), at));
    // The reference just seeded is the one this handle owns.
    DictRef(at)
}

/// Unlink `d` from the garbage collector's chain and free the `Dict` itself.
///
/// # Safety
/// `d` must point at a live dictionary [`tv_dict_alloc`] made, whose
/// contents have already been freed ([`tv_dict_free_contents`]), and the
/// caller must own the collector's registry, which this takes it out of.
/// `d` is dangling afterwards.
pub unsafe fn tv_dict_free_dict(d: *mut DictBox) {
    // SAFETY: the caller's promise: a live dictionary -- the `Box`
    // `tv_dict_alloc` leaked -- which nothing names afterwards.
    let dict = unsafe { Box::from_raw(d) };
    // Out of the collector's registry.
    unroot_dict(&RootKey(()), dict.head.root.get());
    // NLUA_CLEAR_REF
    if dict.value.lua_table_ref != LUA_NOREF {
        release_luaref(dict.value.lua_table_ref);
    }
    drop(dict);
}

/// References taken as one and given back as one: what a scope holds on its
/// dictionary for its life -- `DO_NOT_FREE_CNT` of them, so that nothing
/// frees it mid-scope, and so that a count other than the seed plus the
/// scope's own holders says something else took one. Dropping the seed is
/// the release; nothing but a seed and a handle writes the count. (Zeroed,
/// an `Option<DictSeed>` is `None`: a zeroed buffer's scope entry.)
#[derive(Debug)]
pub(crate) struct DictSeed(DictRef, ::core::ffi::c_int);

/// [`DictSeed`] on a list: a funccall's `a:000`.
#[derive(Debug)]
pub(crate) struct ListSeed(ListRef, ::core::ffi::c_int);

/// Take the `n - 1` references a seed of `n` holds beside its handle.
fn seed_extra(count: &Refcount, n: ::core::ffi::c_int) -> ::core::ffi::c_int {
    assert!(n >= 1, "a seed holds its own handle");
    count.reset(count.get() + (n - 1));
    n - 1
}

/// Give back what [`seed_extra`] took. The handle's own reference stays, so
/// this cannot reach zero; the handle's drop is the release that may free.
fn unseed(count: &Refcount, extra: ::core::ffi::c_int) {
    count.reset(count.get() - extra);
}

impl Drop for DictSeed {
    fn drop(&mut self) {
        unseed(count(&(self.0).0), self.1);
    }
}

impl Drop for ListSeed {
    fn drop(&mut self) {
        unseed(count(&(self.0).0), self.1);
    }
}

/// One deep copy, for the memo that makes a self-referencing container
/// resolve to the same copy: each original copied under it records the copy
/// ([`ListRef::remember_copy`]), and the walk holds a reference to every
/// copy until it ends, so a memo under its id names a live object. Ids come
/// from a 64-bit counter and are never reused, so a memo left by an
/// earlier walk never matches.
#[derive(Debug)]
pub struct CopyWalk {
    id: u64,
    lists: RefCell<Vec<ListRef>>,
    dicts: RefCell<Vec<DictRef>>,
}

impl Default for CopyWalk {
    fn default() -> Self {
        CopyWalk::new()
    }
}

impl CopyWalk {
    pub fn new() -> CopyWalk {
        static NEXT: AtomicU64 = AtomicU64::new(1);
        CopyWalk {
            id: NEXT.fetch_add(1, Ordering::Relaxed),
            lists: RefCell::new(Vec::new()),
            dicts: RefCell::new(Vec::new()),
        }
    }
}

// ---------------------------------------------------------------------------
// The collector's free pass
//
// While `TV_IN_FREE_UNREF_ITEMS` is set no release frees anything, so every
// container the registry names stays live for the pass; these are the
// views and frees the pass works with, so that `eval/collect.rs` holds no
// pointer of its own.

/// The dictionary in the collector's registry slot `idx`, with a reference
/// of its own: `None` for a hole or a slot past the end.
pub(crate) fn registered_dict(idx: usize) -> Option<DictRef> {
    let at = crate::eval::gc::dict_at(idx)?;
    // SAFETY: a registered dictionary is live until it leaves the registry,
    // which only its free does (`RootKey`).
    unsafe { DictRef::retained(at.as_ptr()) }
}

/// [`registered_dict`] for the list registry.
pub(crate) fn registered_list(idx: usize) -> Option<ListRef> {
    let at = crate::eval::gc::list_at(idx)?;
    // SAFETY: as above.
    unsafe { ListRef::retained(at.as_ptr()) }
}

/// Free the structure of the dictionary in registry slot `idx`, taking it
/// out of the registry -- when nothing holds a reference to it any more.
/// Answers whether it did.
///
/// The collector's free pass empties every unreachable container first, so
/// what is left of a reference is a holder the marking never saw. Freeing
/// under it would leave that holder dangling; the dictionary stays (empty)
/// instead, and its last release frees it the ordinary way.
pub(crate) fn free_registered_dict(idx: usize) -> bool {
    let Some(dict) = registered_dict(idx) else {
        return false;
    };
    // The view's own reference is the only one left.
    if dict.refcount() > 1 {
        return false;
    }
    // SAFETY: registered, so `tv_dict_alloc` made it and the registry holds
    // it; no handle but the view names it (every handle owns a count), and
    // the view is given up unreleased here, so no borrow of it is out.
    unsafe { tv_dict_free_dict(dict.into_raw()) };
    true
}

/// [`free_registered_dict`] for a list.
pub(crate) fn free_registered_list(idx: usize) -> bool {
    let Some(list) = registered_list(idx) else {
        return false;
    };
    // As above.
    if list.refcount() > 1 {
        return false;
    }
    // SAFETY: as above.
    unsafe { list_free_list(list.into_raw()) };
    true
}

/// One reference to a [`Blob`], given back when the handle goes: the blob
/// half of [`ListRef`].
#[repr(transparent)]
#[derive(Debug)]
pub struct BlobRef(NonNull<BlobBox>);

/// The same API as [`ListRef`]'s, less the raw bridges: a blob is only
/// ever made by [`tv_blob_alloc`].
impl BlobRef {
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut BlobBox {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub fn ptr_eq(&self, other: &BlobRef) -> bool {
        self.0 == other.0
    }

    #[inline(always)]
    pub fn edit(&self) -> Edit<'_, Blob> {
        lend(&self.0)
    }

    #[inline(always)]
    pub fn read(&self) -> Read<'_, Blob> {
        share(&self.0)
    }

    /// As [`ListRef::refcount`].
    #[inline(always)]
    pub fn refcount(&self) -> i32 {
        count(&self.0).get()
    }
}

impl Clone for BlobRef {
    #[inline(always)]
    fn clone(&self) -> BlobRef {
        count(&self.0).retain();
        BlobRef(self.0)
    }
}

impl Drop for BlobRef {
    /// Give the reference back; the last one frees the blob.
    #[inline(always)]
    fn drop(&mut self) {
        if count(&self.0).release() <= 0 {
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
    let blob = Blob {
        bv_data: Vec::new(),
        bv_lock: VarLock::Unlocked,
    };
    BlobRef(NonNull::from(Box::leak(Box::new(Counted::new(blob, ())))))
}

/// One reference to a [`Partial`], given back when the handle goes: the
/// partial half of [`ListRef`].
#[repr(transparent)]
#[derive(Debug)]
pub struct PartialRef(NonNull<PartialBox>);

/// The same API as [`ListRef`]'s, less the raw bridges: a partial is only
/// ever made by [`PartialRef::new`].
impl PartialRef {
    /// A partial built in place, owned by the handle (a count of one).
    pub fn new(partial: Partial) -> PartialRef {
        PartialRef(NonNull::from(Box::leak(Box::new(Counted::new(
            partial,
            (),
        )))))
    }

    #[inline(always)]
    pub fn as_ptr(&self) -> *mut PartialBox {
        self.0.as_ptr()
    }

    #[inline(always)]
    pub fn ptr_eq(&self, other: &PartialRef) -> bool {
        self.0 == other.0
    }

    /// As [`ListRef::refcount`].
    #[inline(always)]
    pub fn refcount(&self) -> i32 {
        count(&self.0).get()
    }

    /// The partial, writable, while this handle is its only holder:
    /// `Rc::get_mut`'s answer, and the only `&mut` a partial ever lends
    /// (it is fixed once built, so every other borrow is shared).
    #[inline(always)]
    pub fn get_mut(&mut self) -> Option<&mut Partial> {
        if self.refcount() != 1 {
            return None;
        }
        // SAFETY: a count of one is this handle, and the exclusive borrow
        // of it rules out a shared borrow lent through it; the borrow is of
        // the value, not the count beside it.
        Some(unsafe { &mut (*self.0.as_ptr()).value })
    }
}

impl Clone for PartialRef {
    #[inline(always)]
    fn clone(&self) -> PartialRef {
        count(&self.0).retain();
        PartialRef(self.0)
    }
}

impl Drop for PartialRef {
    /// Give the reference back; the last one frees the partial.
    fn drop(&mut self) {
        if count(&self.0).release() <= 0 {
            // SAFETY: the last reference to the `Box` `new` leaked.
            partial_free(unsafe { Box::from_raw(self.as_ptr()) }.value);
        }
    }
}

impl ::core::ops::Deref for PartialRef {
    type Target = Partial;

    #[inline(always)]
    fn deref(&self) -> &Partial {
        // SAFETY: as `ListRef`'s.
        unsafe { &(*self.0.as_ptr()).value }
    }
}

/// The value of the object `at` names, writable for as long as the handle
/// holding `at` is borrowed: every `edit()`.
#[inline(always)]
fn lend<T, H>(at: &NonNull<Counted<T, H>>) -> Edit<'_, T> {
    let loan = Loan::new::<T, _>(*at, ::core::mem::offset_of!(Counted<T, H>, lent));
    Edit {
        // SAFETY: the handle keeps the object live; the place is the value
        // field, which nothing else lends while the loan is up.
        target: NonNull::from(unsafe { &mut (*at.as_ptr()).value }),
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

/// A borrow's claim on its object's [`Lent`] word: the word's address,
/// projected from the handle's pointer, never a borrow of the object. The
/// word sits beside the value an edit holds `&mut`, not inside it.
#[derive(Debug)]
struct Claim(*const Lent);

impl Claim {
    /// The word of the object at `at`, the field at `offset`.
    #[inline(always)]
    fn new<C>(at: NonNull<C>, offset: usize) -> Claim {
        Claim(at.as_ptr().cast_const().wrapping_byte_add(offset).cast())
    }

    #[inline(always)]
    fn word(&self) -> &Lent {
        // SAFETY: a `Lent` field of a live object a handle names, which the
        // borrow's handle keeps alive; the borrow covers the word alone,
        // which no edit or read lends.
        unsafe { &*self.0 }
    }
}

/// An [`Edit`]'s claim: the word raised to the edit's for its life. The
/// lifetime is the handle's borrow, which the destructor keeps alive to
/// the loan's end: a handle moved or dropped under it does not compile.
#[derive(Debug)]
pub(super) struct Loan<'a>(Claim, PhantomData<&'a ()>);

impl Loan<'_> {
    /// `T` names the value in a refusal's message.
    #[inline(always)]
    fn new<T, C>(at: NonNull<C>, offset: usize) -> Self {
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
    fn new<T, C>(at: NonNull<C>, offset: usize) -> Share {
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

/// The value of the object `at` names, shared for as long as the handle
/// holding `at` is borrowed: every `read()`.
#[inline(always)]
fn share<T, H>(at: &NonNull<Counted<T, H>>) -> Read<'_, T> {
    let share = Share::new::<T, _>(*at, ::core::mem::offset_of!(Counted<T, H>, lent));
    // SAFETY: the handle holds a reference, so the object is live; the
    // share just taken refuses an edit while this borrow lives.
    let target = unsafe { &(*at.as_ptr()).value };
    let share = Some(share);
    Read { target, share }
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
    unsafe fn bit_copy(&self) -> TypVal {
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
impl<'a, const N: usize> CallFrame<'a, N> {
    /// Append a bit copy of a value the caller keeps for the frame's life.
    pub(crate) fn push_borrowed(&mut self, tv: &'a TypVal) {
        // SAFETY: the duplicate is never released -- the slot's bit is
        // clear, nothing sets it again (the frame has no way to claim a
        // slot after the push), so `truncate` disowns it rather than
        // clearing it -- and it is never read after `tv` is gone: `'a` is
        // the frame's own lifetime, and the frame hands its slots out only
        // as shared borrows of itself.
        self.push_naming(unsafe { tv.bit_copy() });
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
        let d = tv_dict_alloc();
        d.edit().add_number(b"a", 0).expect("a key used once");
        d.edit().add_number(b"b", 1).expect("a key used once");
        let dp = d.as_ptr();
        TV_IN_FREE_UNREF_ITEMS.set(true);
        drop(d);
        // SAFETY: unreferenced but not freed; a view that owns nothing.
        let view = ::core::mem::ManuallyDrop::new(unsafe { DictRef::owning(dp) }.expect("live"));
        let refs = view.refcount();
        let b = view.read().find(b"b").map(|di| di.di_tv.number_or_zero());
        TV_IN_FREE_UNREF_ITEMS.set(false);
        assert_eq!(refs, 0);
        assert_eq!(b, Some(1));
        // SAFETY: as above. The handle takes the count below zero, which
        // frees it now that the flag is down.
        drop(unsafe { DictRef::owning(dp) });
    }

    /// A list of the numbers `0..len`, and a view of it that owns no
    /// reference: what the collector holds while it frees a list nobody
    /// references.
    fn list_and_view(len: VarNumber) -> (ListRef, ::core::mem::ManuallyDrop<ListRef>) {
        let list = tv_list_alloc(-1);
        for n in 0..len {
            list.edit().push_number(n);
        }
        // SAFETY: a live list; the view never gives the reference back.
        let view = unsafe { ListRef::owning(list.as_ptr()) }.expect("live");
        (list, ::core::mem::ManuallyDrop::new(view))
    }

    /// Give `view` a reference of its own and let it go: the release that
    /// frees whatever is left of the list.
    fn free_view(view: ::core::mem::ManuallyDrop<ListRef>) {
        drop(ListRef::clone(&view));
    }

    /// While the collector is freeing, the last reference going does not
    /// free the list: the collector frees the whole graph itself, and a
    /// free from inside it would free something twice.
    #[test]
    fn a_list_released_mid_collection_waits_for_the_collector() {
        let _serial = editor_state_lock();
        let (list, held) = list_and_view(3);
        assert_eq!(held.refcount(), 1);
        TV_IN_FREE_UNREF_ITEMS.set(true);
        drop(list);
        // Still allocated, still holding its items, at zero.
        let refs = held.refcount();
        let len = held.read().len();
        TV_IN_FREE_UNREF_ITEMS.set(false);
        assert_eq!((refs, len), (0, 3));
        // What `free_unref_items` does with it: contents, then the list.
        list_free_contents(&held);
        free_view(held);
    }

    /// A list that holds itself never reaches zero by itself; the
    /// collector's two passes are what free it. Pass 1 is
    /// `list_free_contents`, exactly as `free_unref_items` calls it.
    #[test]
    fn a_self_cycle_is_freed_by_the_collectors_two_passes() {
        let _serial = editor_state_lock();
        let (list, held) = list_and_view(1);
        list.edit().push_list(Some(list.clone()));
        assert_eq!(list.refcount(), 2);
        drop(list);
        assert_eq!(held.refcount(), 1, "the cycle keeps it alive");
        TV_IN_FREE_UNREF_ITEMS.set(true);
        // Pass 1: the contents. The self-reference goes to zero, and the
        // flag keeps that from freeing the list under the walk.
        list_free_contents(&held);
        let refs = held.refcount();
        TV_IN_FREE_UNREF_ITEMS.set(false);
        assert_eq!(refs, 0);
        // Pass 2: the structure.
        free_view(held);
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
