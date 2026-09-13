//! Allocating, freeing and editing a `List` and the items it owns.
//!
//! [`tv_list_alloc`] and [`list_free`] are the reference-counted pair,
//! [`list_unref`] the one every caller actually uses.  The `ListWatch`
//! half ([`List::watch_add`], [`watch_shift`]) is how a `:for`
//! loop survives having the item it is standing on removed underneath it,
//! and [`List::remove_range`] / [`List::move_range_to`] are the two ways
//! items leave a list.
//!
//! # The item store
//!
//! A `List` owns its items in a `Vec<ListItem>`, so an item's identity *is*
//! its index and there is nothing to free per item.  Everything that used to
//! be a link walk is an index walk, everything that used to hold a
//! `*mut ListItem` across an edit holds an index instead, and the one such
//! cursor that outlives an edit -- a `:for` loop's [`ListWatch`] -- is
//! shifted by [`watch_shift`] at every insert and removal so that it
//! keeps naming the same *item*.
//!
//! A `*mut ListItem` still exists, and is still what most callers hold; it
//! is a **borrow of the array** and is invalidated by any edit, exactly as a
//! `&mut` into a `Vec` would be.

#![deny(unsafe_op_in_unsafe_fn)]
// Every entry point here dereferences the caller's list.
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
use crate::types::Refcount;

/// The items of `l` as a slice; a NULL list is empty.
#[inline(always)]
pub(crate) fn list_items(l: Option<&List>) -> &[ListItem] {
    l.map_or(&[], List::items)
}

/// The items of `l` as a mutable slice; a NULL list is empty.
#[inline(always)]
pub(crate) fn list_items_mut(l: Option<&mut List>) -> &mut [ListItem] {
    l.map_or(&mut [], List::items_mut)
}

/// The editing half of a [`List`]: what it holds, and what moves it.
///
/// Every entry point here took the list by pointer and was an `unsafe fn`
/// for no reason but that. A `List` is a `Vec<ListItem>` with a watcher
/// chain beside it, and a borrow says everything the pointer said -- except
/// where two of them would name the same list, which is the one thing the
/// pointer let happen silently and the borrow will not (see
/// [`List::extend_from_self`]).
impl List {
    /// The items this list owns.
    #[inline(always)]
    pub(crate) fn items(&self) -> &[ListItem] {
        &self.lv_items
    }

    /// The items this list owns, writable.
    #[inline(always)]
    pub(crate) fn items_mut(&mut self) -> &mut [ListItem] {
        &mut self.lv_items
    }

    /// How many items the list holds.
    #[inline(always)]
    pub(crate) fn len(&self) -> usize {
        self.lv_items.len()
    }

    /// Whether the list holds no items at all.
    #[inline(always)]
    pub(crate) fn is_empty(&self) -> bool {
        self.lv_items.is_empty()
    }

    /// The lock this list carries.
    #[inline(always)]
    pub(crate) fn lock(&self) -> VarLock {
        self.lv_lock
    }

    /// Lock or unlock the list.
    #[inline(always)]
    pub(crate) fn set_lock(&mut self, lock: VarLock) {
        self.lv_lock = lock;
    }

    /// The garbage collector's mark on this list.
    #[inline(always)]
    pub fn copy_id(&self) -> ::core::ffi::c_int {
        self.lv_copy_id
    }

    /// Mark this list with `copy_id`, which the caller reserved from
    /// `get_copyID`.
    #[inline(always)]
    pub fn set_copy_id(&mut self, copy_id: ::core::ffi::c_int) {
        self.lv_copy_id = copy_id;
    }

    /// Remove `self[at]`, clearing the value it held.
    ///
    /// Answers the index of the item that followed it, which is `at` again
    /// -- or `None` when the removed item was the last one.
    pub fn remove_at(&mut self, at: usize) -> Option<usize> {
        self.remove_range(at, at);
        (at < self.len()).then_some(at)
    }

    /// Push `lw` onto the watcher chain.
    ///
    /// # Safety
    ///
    /// `lw` must point at a watcher that outlives its registration: the
    /// list stores the address, so a watcher that moves or dies first
    /// leaves the chain naming freed storage.
    pub unsafe fn watch_add(&mut self, lw: *mut ListWatch) {
        // SAFETY: the caller's promise: a watcher that outlives this.
        unsafe { (*lw).lw_next = self.lv_watch };
        self.lv_watch = lw;
    }

    /// Unlink `lwrem` from the watcher chain.
    ///
    /// # Safety
    ///
    /// `lwrem` must point at an entry of this list's watcher chain.
    pub unsafe fn watch_remove(&mut self, lwrem: *mut ListWatch) {
        // `lwp` trails `lw` by one link so the match can be spliced out.
        let mut lwp = &raw mut self.lv_watch;
        let mut lw = self.lv_watch;
        while !lw.is_null() {
            if lw == lwrem {
                // SAFETY: `lwp` is the link that names `lw`, and `lw` an
                // entry of this chain.
                unsafe { *lwp = (*lw).lw_next };
                break;
            }
            // SAFETY: an entry of this list's watcher chain.
            let mut watch = unsafe { Lw::new(lw) };
            lwp = &raw mut watch.lw_next;
            lw = watch.lw_next;
        }
    }
}

/// Move every cursor on `l` so that it keeps naming the item it named, after
/// `count` items were inserted at (`count > 0`) or removed from (`count < 0`)
/// index `at`.
///
/// This is what keeps a `:for` loop walking a list edited underneath it.  The
/// three cases, and what each one is:
///
/// - **before the edit** -- the cursor's item did not move, so neither does
///   the cursor.
/// - **after the edit** -- the item moved by `count` places, so the cursor
///   follows it.
/// - **inside a removed run** -- the item is gone, so the cursor lands on
///   whatever followed the run, which is index `at` once everything has
///   shifted down.  That is upstream's `lw_item = item->li_next` walked to
///   the end of the run.
///
/// A cursor that lands past the last item is [`ENDED`](ListWatch::ENDED), and
/// stays ended however the list grows afterwards -- which is why a `:for`
/// loop whose body appends to the list it is walking still terminates.
///
/// `at` must be an index into the list as it now is.
pub(crate) fn watch_shift(l: &mut List, at: ::core::ffi::c_int, count: ::core::ffi::c_int) {
    let mut lw = l.lv_watch;
    if lw.is_null() {
        return;
    }
    let len = index_of(l.len());
    while !lw.is_null() {
        // SAFETY: an entry of `l`'s watcher chain.
        let watch = unsafe { Lw::new(lw) };
        let was = watch.lw_index;
        if was >= at {
            // Clamped at `at`: a cursor inside a removed run lands on
            // whatever followed the run.
            let moved = (was + count).max(at);
            let landed = if moved >= len {
                ListWatch::ENDED
            } else {
                moved
            };
            // SAFETY: as above.
            unsafe { (*lw).lw_index = landed };
        }
        lw = watch.lw_next;
    }
}

/// Move every cursor on `l` through a permutation of its items: whatever
/// stood at index `i` now stands at `moved[i]`.
///
/// The two callers are `sort()` and `reverse()`, which move items without
/// adding or removing any.  A cursor stands on an *item*, not on a place, so
/// it goes where the item went -- which is what upstream got for free by
/// relinking the items and leaving `lw_item` alone.
///
/// `moved` must be one index per item, as the list stood before.
pub(crate) fn watch_permute(l: &mut List, moved: &[::core::ffi::c_int]) {
    let mut lw = l.lv_watch;
    while !lw.is_null() {
        // SAFETY: an entry of `l`'s watcher chain.
        let watch = unsafe { Lw::new(lw) };
        let to = usize::try_from(watch.lw_index)
            .ok()
            .and_then(|at| moved.get(at));
        if let Some(&to) = to {
            unsafe { (*lw).lw_index = to };
        }
        lw = watch.lw_next;
    }
}

/// A length or index of a list, as the `int` the family counts in.
///
/// Lists are `int`-indexed the whole way down (`list_len`, `E684`, the
/// `[n1:n2]` arithmetic), so this is where the width changes, once.  A list
/// longer than `INT_MAX` cannot be built: every path that adds an item goes
/// through a length this saturates.
#[inline(always)]
pub(crate) fn index_of(n: usize) -> ::core::ffi::c_int {
    ::core::ffi::c_int::try_from(n).unwrap_or(::core::ffi::c_int::MAX)
}

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
    /// Take over a reference the caller already holds and will not release.
    ///
    /// # Safety
    ///
    /// `l` must point at a live list, and the caller must hold a reference
    /// to it -- one this handle now owns and eventually gives back.
    #[inline(always)]
    pub unsafe fn from_owned(at: NonNull<List>) -> ListRef {
        ListRef(at)
    }

    /// Take over the caller's reference to `l`, or answer `None` for a NULL
    /// list.  See [`ListRef::from_owned`].
    ///
    /// # Safety
    ///
    /// As [`ListRef::from_owned`], for a pointer that may be null.
    #[inline(always)]
    pub unsafe fn owning(l: *mut List) -> Option<ListRef> {
        NonNull::new(l).map(ListRef)
    }

    /// Take *another* reference to `l`: the caller keeps its own.
    ///
    /// This is `tv_list_ref`, with the handle that owes the matching release
    /// as its answer.  `None` for a NULL list, which is `v:_null_list` and
    /// counts nothing.
    ///
    /// # Safety
    ///
    /// `l` is null or points at a live list.
    #[inline(always)]
    pub unsafe fn retained(l: *mut List) -> Option<ListRef> {
        let at = NonNull::new(l)?;
        // SAFETY: the caller's promise: a live list.
        unsafe { Ls::new(l) }.lv_refcount.retain();
        Some(ListRef(at))
    }

    /// The list, as the pointer most of the family still takes.
    ///
    /// A **borrow**: it is live only while the handle is, and an edit to the
    /// list invalidates nothing about it, but a release does.
    #[inline(always)]
    pub fn as_ptr(&self) -> *mut List {
        self.0.as_ptr()
    }

    /// Give the reference up without releasing it: something else owns it
    /// now.
    ///
    /// The counterpart of [`ListRef::from_owned`], for the handful of places
    /// that hand a list on by pointer.
    #[inline(always)]
    pub fn into_raw(self) -> *mut List {
        let at = self.0;
        ::core::mem::forget(self);
        at.as_ptr()
    }
}

/// The `TypVal` readers and writers for the list arm.
///
/// Hand-written where the other nine are generated by
/// [`union_readers`](super::access) and
/// [`union_writers`](super::access), and living here rather than beside
/// them because the payload is a [`ListRef`]: it can be borrowed but never
/// handed out, since a copy of it would be a reference nobody took.
impl TypVal {
    /// The list, or `None` unless this is a `List` -- including the
    /// `v:_null_list` case, which answers `Some(NULL)` as the generated
    /// readers' `as_*` forms do for their own empty payloads.
    #[inline(always)]
    pub(crate) fn as_list(&self) -> Option<*mut List> {
        match self {
            TypVal::List(list) => Some(
                list.as_ref()
                    .map_or(::core::ptr::null_mut(), ListRef::as_ptr),
            ),
            _ => None,
        }
    }

    /// The list this value holds, or NULL unless it is a list holding one.
    ///
    /// A **borrow**: the answer is live as long as this value holds the
    /// reference, and a caller that keeps it past that owes a
    /// [`ListRef::retained`] of its own. This is the spelling the family
    /// reads a container in -- `list_len(NULL) == 0` and the rest of the
    /// null-tolerant entry points -- so the tag test and the `v:_null_list`
    /// case answer the same NULL, as the union read they replaced did.
    #[inline(always)]
    pub(crate) fn list_or_null(&self) -> *mut List {
        match self {
            TypVal::List(list) => list
                .as_ref()
                .map_or(::core::ptr::null_mut(), ListRef::as_ptr),
            _ => ::core::ptr::null_mut(),
        }
    }

    /// The list this value holds, borrowed -- `None` for every other kind
    /// and for `v:_null_list`.
    ///
    /// The safe spelling of [`TypVal::list_or_null`], and the one the
    /// `list_*` family reads its argument in: the borrow lasts as long as
    /// the value does, which is what the pointer never said.
    #[inline(always)]
    pub(crate) fn list_ref(&self) -> Option<&List> {
        match self {
            TypVal::List(list) => list.as_deref(),
            _ => None,
        }
    }

    /// The list this value holds, borrowed for writing.
    ///
    /// The exclusive borrow is the point: a caller holding one cannot also
    /// be reading the list through the value, which the raw pointer let it
    /// do.
    #[inline(always)]
    pub(crate) fn list_mut(&mut self) -> Option<&mut List> {
        match self {
            TypVal::List(list) => list.as_deref_mut(),
            _ => None,
        }
    }

    /// A list value over `list`, which the value takes over.
    ///
    /// The one place the [`ManuallyDrop`](::core::mem::ManuallyDrop) the
    /// variant carries is spelled: see [`TypVal`] for why it is there.
    #[inline(always)]
    pub(crate) const fn list(list: Option<ListRef>) -> TypVal {
        TypVal::List(::core::mem::ManuallyDrop::new(list))
    }

    /// Overwrite this slot with `list`, **releasing nothing**: see
    /// [`union_writers`].  The slot takes over whatever the handle owns.
    #[inline(always)]
    pub(crate) fn write_list(&mut self, list: Option<ListRef>) {
        self.overwrite(TypVal::list(list));
    }

    /// Move the list out of this slot, leaving `v:_null_list` behind.
    ///
    /// The caller owns the reference now: dropping the answer is what
    /// `list_unref` on the payload was, and keeping it is what a slot
    /// handing its container on by value wants.  Answers `None` for every
    /// other kind, which leaves the slot alone.
    #[inline(always)]
    pub(crate) fn take_list(&mut self) -> Option<ListRef> {
        match self {
            TypVal::List(list) => list.take(),
            _ => None,
        }
    }
}

impl Clone for ListRef {
    /// One more owner of the same list.
    #[inline(always)]
    fn clone(&self) -> ListRef {
        // SAFETY: this handle names a live list, since it holds a reference
        // to it.
        unsafe { Ls::new(self.as_ptr()) }.lv_refcount.retain();
        ListRef(self.0)
    }
}

impl Drop for ListRef {
    /// Give the reference back, freeing the list with the last one.
    #[inline(always)]
    fn drop(&mut self) {
        // SAFETY: this handle names a live list, and is giving up the
        // reference that kept it so.
        unsafe { list_unref(self.as_ptr()) };
    }
}

impl ::core::ops::Deref for ListRef {
    type Target = List;

    #[inline(always)]
    fn deref(&self) -> &List {
        // SAFETY: the handle holds a reference, so the list is live; the
        // borrow lasts only as long as the field access that asked for it.
        unsafe { self.0.as_ref() }
    }
}

impl ::core::ops::DerefMut for ListRef {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut List {
        // SAFETY: as [`ListRef::deref`].
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

/// Free every item in `l`, leaving the list itself allocated and empty.
pub fn list_free_contents(l: &mut List) {
    // Taken out before anything is cleared: releasing a value can re-enter
    // the evaluator, and what it must not find is a list half way through
    // being emptied.
    let items = ::core::mem::take(&mut l.lv_items);
    debug_assert!(l.lv_watch.is_null());
    // Dropping the array clears each value in turn, front to back.
    drop(items);
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

impl List {
    /// Take the items `self[first..=last]` out without releasing what they
    /// hold; the caller owns them now.  `first..=last` must be a run of the
    /// list's items.
    pub(crate) fn take_range(&mut self, first: usize, last: usize) -> Vec<ListItem> {
        let taken: Vec<ListItem> = self.lv_items.drain(first..=last).collect();
        // The items are gone, so the cursors move now.
        watch_shift(self, index_of(first), -index_of(taken.len()));
        taken
    }

    /// Remove the items `self[first..=last]`, releasing what they hold.
    pub fn remove_range(&mut self, first: usize, last: usize) {
        // Dropped after the cursors have moved: releasing a value can
        // re-enter the evaluator, which must not see a list whose watchers
        // still name items that are gone.
        drop(self.take_range(first, last));
    }

    /// Move the items `self[first..=last]` onto `target`'s tail.
    pub fn move_range_to(&mut self, first: usize, last: usize, target: &mut List) {
        let moved = self.take_range(first, last);
        target.lv_items.extend(moved);
    }

    /// Empty the list without releasing anything its items name.
    ///
    /// The one caller is a funccall's `a:000`, whose items *borrow* the
    /// caller's arguments for the length of the call and own nothing.
    /// Upstream spelled this `lv_first = NULL`, which threw away an array of
    /// values it had never owned; this is the same statement about an array
    /// that would otherwise release them.
    pub(crate) fn disown_items(&mut self) {
        for mut item in ::core::mem::take(&mut self.lv_items) {
            item.li_tv.disown();
        }
    }

    /// Upgrade every item to a value of its own.
    ///
    /// The counterpart of [`List::disown_items`]: a funccall that has to
    /// outlive the call that made it cannot keep naming the caller's
    /// arguments, so each item takes a real copy.
    pub(crate) fn own_items(&mut self) {
        for li in &mut self.lv_items {
            let slot = &raw mut li.li_tv;
            // SAFETY: source and destination are one slot, which `tv_copy`
            // reads before overwriting it with a value that owns what it
            // names.
            unsafe { tv_copy(&*slot, &mut *slot) };
        }
    }
}

/// Allocate an empty list and store it in `ret_tv` as the return value.
///
/// The answer is a **borrow** of the list `ret_tv` now owns, for the caller
/// to fill in; it is live as long as the return slot holds the list.
pub fn tv_list_alloc_ret(ret_tv: &mut TypVal, len: ptrdiff_t) -> &mut List {
    ret_tv.write_list(Some(tv_list_alloc(len)));
    ret_tv.list_mut().expect("the list just stored")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The safe layer these cases are written against.
    ///
    /// Every entry point below takes the caller's list by pointer; the
    /// helpers here promise what those signatures ask for once — the list is
    /// this module's own, the indexes are inside it — so that a case reads
    /// as ordinary code and says only what it is about.
    mod l {
        use super::*;

        /// Exclusive use of the collector's registries, which every case
        /// below allocates into. See [`crate::eval::gc::serial`].
        pub(super) fn serial() -> crate::eval::gc::serial::Held {
            crate::eval::gc::serial::lock()
        }

        /// A count the case wrote, as the `int` this family indexes by.
        fn index(n: usize) -> ::core::ffi::c_int {
            ::core::ffi::c_int::try_from(n).expect("a short list")
        }

        /// `[0, 1, ..., len - 1]`, the list every case below edits.
        ///
        /// The case owns the one reference the allocator handed out, and
        /// gives it back through [`done`].
        pub(super) fn counted(len: usize) -> *mut List {
            let list = tv_list_alloc(ptrdiff_t::try_from(len).expect("a short list"));
            let l = list.as_ptr();
            for n in 0..len {
                // SAFETY: the list just allocated.
                unsafe { (*l).push_number(VarNumber::try_from(n).expect("a small number")) };
            }
            list.into_raw()
        }

        /// The numbers `l` holds, so a case can say which items survived
        /// rather than how many.
        pub(super) fn numbers(l: *mut List) -> Vec<VarNumber> {
            // SAFETY: a list `counted` made, holding numbers.
            unsafe { list_iter(l.as_ref()).map(|li| li.li_tv.number_or_zero()) }.collect()
        }

        /// A watcher standing on `l[at]`, registered with `l`.
        ///
        /// Handed out as a raw pointer rather than a `Box`: the list stores
        /// the address, so moving the `Box` afterwards would invalidate it.
        /// [`done`] takes it back.
        pub(super) fn watch(l: *mut List, at: usize) -> *mut ListWatch {
            // SAFETY: a list `counted` made.
            assert!(
                at < list_items(unsafe { l.as_ref() }).len(),
                "no item at {at}"
            );
            let lw = Box::into_raw(Box::new(ListWatch {
                lw_index: index(at),
                lw_next: ::core::ptr::null_mut(),
            }));
            // SAFETY: as above, and the watcher outlives its registration.
            unsafe { (*l).watch_add(lw) };
            lw
        }

        /// Where a watcher is standing, as an index into `l` -- `None` once
        /// it has been pushed off the end.
        ///
        /// The watcher's own field is a pointer today and an index
        /// tomorrow; every case below is written in indexes so that it says
        /// the same thing either way.  That is the whole point of these
        /// cases: the identity a `:for` loop holds on to is *the item*, and
        /// what [`tv_list_watch_fix`] and its successors owe is that the
        /// item does not change under an edit somewhere else in the list.
        pub(super) fn watching(l: *mut List, lw: *mut ListWatch) -> Option<usize> {
            let _ = l;
            // SAFETY: a watcher `watch` registered with `l`.  `ENDED` is
            // negative, which is the NULL upstream pushed a cursor off the
            // end to.
            usize::try_from(unsafe { (*lw).lw_index }).ok()
        }

        /// Remove `l[at]`.
        pub(super) fn remove(l: *mut List, at: usize) {
            // SAFETY: a list `counted` made, and an index of it.
            unsafe { (*l).remove_at(at) };
        }

        /// Remove `l[first..=last]`.
        pub(super) fn remove_run(l: *mut List, first: usize, last: usize) {
            // SAFETY: as above, and a run of items of `l`.
            unsafe { (*l).remove_range(first, last) };
        }

        /// Move `l[first..=last]` onto `tgt`'s tail.
        pub(super) fn move_run(l: *mut List, first: usize, last: usize, tgt: *mut List) {
            // SAFETY: as above, plus a second list of this module's own.
            unsafe { (*l).move_range_to(first, last, &mut *tgt) };
        }

        /// Insert the number `n` in front of `l[at]`.
        pub(super) fn insert(l: *mut List, n: VarNumber, at: usize) {
            // SAFETY: as above, and a value the insert copies.
            unsafe { (*l).insert_copy(&TypVal::Number(n), Some(at)) };
        }

        /// Unregister every watcher and free `l`; the pair every case ends
        /// with.
        pub(super) fn done(l: *mut List, lws: &[*mut ListWatch]) {
            for &lw in lws {
                // SAFETY: a watcher `watch` registered with `l`, whose `Box`
                // is taken back here.
                unsafe { (*l).watch_remove(lw) };
                // SAFETY: the `Box` `watch` leaked, taken back here.
                drop(unsafe { Box::from_raw(lw) });
            }
            // SAFETY: a list `counted` made, now unwatched.
            unsafe { list_free(l) };
        }
    }

    /// A slot and its lock, and nothing else: the links are gone, so an
    /// item is the value plus the four bytes `:lockvar l[0]` sets.
    ///
    /// Twenty-four is what a `Vec` of them costs per item, against
    /// upstream's forty *plus* an `xmalloc` header per item -- which is
    /// where `tvbuild` and `tvlist` get their instructions back.
    ///
    /// Only the size and the alignment are pinned.  `ListItem` is
    /// `repr(Rust)` and nothing reads its fields by offset -- the two
    /// callers that need one ask `offset_of!` -- so where the compiler puts
    /// `li_tv` is the compiler's business, and `-Zrandomize-layout` says so
    /// by putting it somewhere else.
    #[test]
    #[cfg(not(randomized_layout))]
    fn an_item_is_a_value_and_a_lock() {
        assert_eq!(::core::mem::size_of::<ListItem>(), 24);
        assert_eq!(::core::mem::align_of::<ListItem>(), 8);
    }

    #[test]
    fn removing_an_item_after_the_watcher_leaves_it_where_it_was() {
        let _serial = l::serial();
        let list = l::counted(5);
        let lw = l::watch(list, 1);
        l::remove(list, 3);
        assert_eq!(l::numbers(list), [0, 1, 2, 4]);
        assert_eq!(l::watching(list, lw), Some(1));
        l::done(list, &[lw]);
    }

    #[test]
    fn removing_an_item_before_the_watcher_keeps_it_on_the_same_item() {
        let _serial = l::serial();
        let list = l::counted(5);
        let lw = l::watch(list, 3);
        l::remove(list, 1);
        assert_eq!(l::numbers(list), [0, 2, 3, 4]);
        // The item it stands on is still the one holding 3 -- which has
        // moved down one place.
        assert_eq!(l::watching(list, lw), Some(2));
        l::done(list, &[lw]);
    }

    #[test]
    fn removing_the_watched_item_lands_the_watcher_on_the_next_one() {
        let _serial = l::serial();
        let list = l::counted(5);
        let lw = l::watch(list, 2);
        l::remove(list, 2);
        assert_eq!(l::numbers(list), [0, 1, 3, 4]);
        // Index 2 again, but the item that *followed* the removed one.
        assert_eq!(l::watching(list, lw), Some(2));
        l::done(list, &[lw]);
    }

    #[test]
    fn removing_the_watched_last_item_pushes_the_watcher_off_the_end() {
        let _serial = l::serial();
        let list = l::counted(3);
        let lw = l::watch(list, 2);
        l::remove(list, 2);
        assert_eq!(l::numbers(list), [0, 1]);
        assert_eq!(l::watching(list, lw), None);
        l::done(list, &[lw]);
    }

    #[test]
    fn a_watcher_off_the_end_stays_off_it_when_the_list_grows_again() {
        // What ends a `:for` loop whose body appends to the list it is
        // walking: the cursor is already past the end, and nothing puts it
        // back.
        let _serial = l::serial();
        let list = l::counted(2);
        let lw = l::watch(list, 1);
        l::remove(list, 1);
        assert_eq!(l::watching(list, lw), None);
        // SAFETY: this module's own list.
        unsafe { (*list).push_number(9) };
        assert_eq!(l::watching(list, lw), None);
        l::done(list, &[lw]);
    }

    #[test]
    fn removing_a_run_around_the_watcher_lands_it_after_the_run() {
        let _serial = l::serial();
        let list = l::counted(7);
        let lws = [l::watch(list, 0), l::watch(list, 3), l::watch(list, 6)];
        l::remove_run(list, 2, 4);
        assert_eq!(l::numbers(list), [0, 1, 5, 6]);
        assert_eq!(l::watching(list, lws[0]), Some(0));
        // Was on 3, inside the run: now on what followed the run, 5.
        assert_eq!(l::watching(list, lws[1]), Some(2));
        // Was on 6, after the run: still on 6.
        assert_eq!(l::watching(list, lws[2]), Some(3));
        l::done(list, &lws);
    }

    #[test]
    fn inserting_before_the_watcher_keeps_it_on_the_same_item() {
        let _serial = l::serial();
        let list = l::counted(4);
        let lw = l::watch(list, 2);
        l::insert(list, 90, 1);
        assert_eq!(l::numbers(list), [0, 90, 1, 2, 3]);
        assert_eq!(l::watching(list, lw), Some(3));
        l::done(list, &[lw]);
    }

    #[test]
    fn inserting_after_the_watcher_leaves_it_where_it_was() {
        let _serial = l::serial();
        let list = l::counted(4);
        let lw = l::watch(list, 1);
        l::insert(list, 90, 3);
        assert_eq!(l::numbers(list), [0, 1, 2, 90, 3]);
        assert_eq!(l::watching(list, lw), Some(1));
        l::done(list, &[lw]);
    }

    #[test]
    fn inserting_at_the_watched_item_pushes_the_watcher_up() {
        let _serial = l::serial();
        let list = l::counted(4);
        let lw = l::watch(list, 1);
        l::insert(list, 90, 1);
        assert_eq!(l::numbers(list), [0, 90, 1, 2, 3]);
        // Still on the item holding 1, now one place further along.
        assert_eq!(l::watching(list, lw), Some(2));
        l::done(list, &[lw]);
    }

    #[test]
    fn moving_the_watched_run_to_another_list_lands_the_watcher_after_it() {
        let _serial = l::serial();
        let list = l::counted(6);
        let tgt = l::counted(0);
        let lws = [l::watch(list, 1), l::watch(list, 4)];
        l::move_run(list, 1, 2, tgt);
        assert_eq!(l::numbers(list), [0, 3, 4, 5]);
        assert_eq!(l::numbers(tgt), [1, 2]);
        // A watcher follows the list it is registered with, not the items
        // that left it.
        assert_eq!(l::watching(list, lws[0]), Some(1));
        assert_eq!(l::watching(list, lws[1]), Some(2));
        l::done(list, &lws);
        l::done(tgt, &[]);
    }
}
