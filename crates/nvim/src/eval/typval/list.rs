//! Allocating, freeing and editing a `List` and the items it owns.
//!
//! [`tv_list_alloc`] and [`tv_list_free`] are the reference-counted pair,
//! [`tv_list_unref`] the one every caller actually uses.  The `ListWatch`
//! half ([`tv_list_watch_add`], [`tv_list_watch_shift`]) is how a `:for`
//! loop survives having the item it is standing on removed underneath it,
//! and [`tv_list_remove_range`] / [`tv_list_move_range`] are the two ways
//! items leave a list.
//!
//! # The item store
//!
//! A `List` owns its items in a `Vec<ListItem>`, so an item's identity *is*
//! its index and there is nothing to free per item.  Everything that used to
//! be a link walk is an index walk, everything that used to hold a
//! `*mut ListItem` across an edit holds an index instead, and the one such
//! cursor that outlives an edit -- a `:for` loop's [`ListWatch`] -- is
//! shifted by [`tv_list_watch_shift`] at every insert and removal so that it
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

use super::*;
use crate::types::Refcount;

/// The items of `l` as a slice; a NULL list is empty.
///
/// # Safety
/// `l` is null or points at a live list, and the slice borrows it: any edit
/// to the list invalidates it.
#[inline(always)]
pub(crate) unsafe fn tv_list_items<'a>(l: *const List) -> &'a [ListItem] {
    match unsafe { l.as_ref() } {
        Some(l) => &l.lv_items,
        None => &[],
    }
}

/// The items of `l` as a mutable slice; a NULL list is empty.
///
/// # Safety
/// As [`tv_list_items`], and the caller must hold no other borrow of the
/// list for the life of the slice.
#[inline(always)]
pub(crate) unsafe fn tv_list_items_mut<'a>(l: *mut List) -> &'a mut [ListItem] {
    match unsafe { l.as_mut() } {
        Some(l) => &mut l.lv_items,
        None => &mut [],
    }
}

/// The item store of `l`, for the handful of places that edit it.
///
/// # Safety
/// `l` must point at a live list, unaliased for the borrow.
#[inline(always)]
unsafe fn items_of<'a>(l: *mut List) -> &'a mut Vec<ListItem> {
    // SAFETY: the caller's promise: a live, unaliased list.
    unsafe { &mut (*l).lv_items }
}

/// Remove `l[at]`, clearing the value it held.
///
/// Answers the index of the item that followed it, which is `at` again --
/// or `None` when the removed item was the last one.
///
/// # Safety
/// `l` must point at a live list, unaliased for the call, and `at` must be
/// an index into it.
pub unsafe fn tv_list_remove_at(l: *mut List, at: usize) -> Option<usize> {
    unsafe { tv_list_remove_range(l, at, at) };
    // SAFETY: as above.
    (at < unsafe { tv_list_items(l) }.len()).then_some(at)
}

/// Push `lw` onto `l`'s watcher chain.
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call. `lw` must point at
/// a watcher that outlives its registration.
pub unsafe fn tv_list_watch_add(l: *mut List, lw: *mut ListWatch) {
    unsafe { (*lw).lw_next = (*l).lv_watch };
    unsafe { (*l).lv_watch = lw };
}

/// Unlink `lwrem` from `l`'s watcher chain.
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call. `lwrem` must point
/// at an entry of `l`'s watcher chain.
pub unsafe fn tv_list_watch_remove(l: *mut List, lwrem: *mut ListWatch) {
    // `lwp` trails `lw` by one link so the match can be spliced out.
    let mut lwp = lv_watch(l);
    let mut lw = unsafe { (*l).lv_watch };
    while !lw.is_null() {
        if lw == lwrem {
            unsafe { *lwp = (*lw).lw_next };
            break;
        }
        // SAFETY: an entry of `l`'s watcher chain.
        let mut watch = unsafe { Lw::new(lw) };
        lwp = &raw mut watch.lw_next;
        lw = watch.lw_next;
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
/// # Safety
///
/// `l` must point at a live list whose items have already been edited, and
/// `at` must be an index into the list as it now is.
pub(crate) unsafe fn tv_list_watch_shift(
    l: *mut List,
    at: ::core::ffi::c_int,
    count: ::core::ffi::c_int,
) {
    let mut lw = unsafe { (*l).lv_watch };
    if lw.is_null() {
        return;
    }
    // SAFETY: the caller's promise: a live list.
    let len = index_of(unsafe { tv_list_items(l) }.len());
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
pub(crate) fn tv_list_watch_permute(l: &mut List, moved: &[::core::ffi::c_int]) {
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
/// Lists are `int`-indexed the whole way down (`tv_list_len`, `E684`, the
/// `[n1:n2]` arithmetic), so this is where the width changes, once.  A list
/// longer than `INT_MAX` cannot be built: every path that adds an item goes
/// through a length this saturates.
#[inline(always)]
pub(crate) fn index_of(n: usize) -> ::core::ffi::c_int {
    ::core::ffi::c_int::try_from(n).unwrap_or(::core::ffi::c_int::MAX)
}

/// Allocate an empty list.  The caller owns the reference count.
///
/// `len` is a capacity hint: a caller that knows how many items are coming
/// reserves them here rather than growing the array on the way.  A negative
/// one (`kListLenUnknown`) reserves nothing.
pub fn tv_list_alloc(len: ptrdiff_t) -> *mut List {
    // Still the `xmalloc` family rather than a `Box`, because the allocation
    // log the unit cases assert against sees only that family -- and because
    // the tree hands `*mut List` around and frees it in `tv_list_free_list`.
    let list = unsafe { xcalloc(1, ::core::mem::size_of::<List>()) }.cast::<List>();
    // Written, not assigned: a zeroed `List` is not a valid one (`Vec` never
    // holds a null pointer), so there is nothing there to drop.
    // SAFETY: the allocation just made, of exactly this size.
    unsafe { list.write(List::empty()) };
    if let Ok(len) = usize::try_from(len) {
        unsafe { items_of(list) }.reserve_exact(len);
    }

    // Prepend the list to the list of lists for garbage collection.
    if let Some(first) = unsafe { gc_first_list.get().as_mut() } {
        first.lv_used_prev = list;
    }
    unsafe { (*list).lv_used_next = gc_first_list.get() };
    gc_first_list.set(list);
    list
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
/// [`tv_list_free`]; whatever was there is overwritten without being
/// dropped, so it must hold no list yet.
pub unsafe fn tv_list_init_static(l: *mut List) {
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
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call.
pub unsafe fn tv_list_free_contents(l: *mut List) {
    // Taken out before anything is cleared: releasing a value can re-enter
    // the evaluator, and what it must not find is a list half way through
    // being emptied.
    // SAFETY: the caller's promise: a live list.
    let items = ::core::mem::take(unsafe { items_of(l) });
    debug_assert!(unsafe { (*l).lv_watch }.is_null());
    // Dropping the array clears each value in turn, front to back.
    drop(items);
}

/// Unlink `l` from the garbage collector's chain and free the `List` itself.
///
/// Upstream freed the header and left whatever was still linked off it --
/// a leak the collector's two passes made unreachable.  The items are the
/// header's own array now, so they go with it.
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call.  Anything still
/// in it is **released**, so no caller may hold a reference to an item.
pub unsafe fn tv_list_free_list(l: *mut List) {
    // Remove the list from the list of lists for garbage collection.
    // SAFETY: the caller's promise: a live list.
    let mut list = unsafe { Ls::new(l) };
    match unsafe { (*l).lv_used_prev.as_mut() } {
        Some(prev) => prev.lv_used_next = list.lv_used_next,
        None => gc_first_list.set(list.lv_used_next),
    }
    if let Some(next) = unsafe { (*l).lv_used_next.as_mut() } {
        next.lv_used_prev = list.lv_used_prev;
    }

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
pub unsafe fn tv_list_free(l: *mut List) {
    if tv_in_free_unref_items.get() {
        return;
    }
    unsafe { tv_list_free_contents(l) };
    unsafe { tv_list_free_list(l) };
}

/// Drop a reference to `l`, freeing it when the last one goes.
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call.
pub unsafe fn tv_list_unref(l: *mut List) {
    if let Some(list) = unsafe { l.as_mut() }
        && list.lv_refcount.release() <= 0
    {
        unsafe { tv_list_free(l) };
    }
}

/// Take the items `l[first..=last]` out of `l` without releasing what they
/// hold; the caller owns them now.
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call, and
/// `first..=last` must be a run of its items.
pub(crate) unsafe fn tv_list_take_range(l: *mut List, first: usize, last: usize) -> Vec<ListItem> {
    // SAFETY: the caller's promise: a live list and a run of its items.
    let taken: Vec<ListItem> = unsafe { items_of(l) }.drain(first..=last).collect();
    // SAFETY: as above; the items are gone, so the cursors move now.
    unsafe { tv_list_watch_shift(l, index_of(first), -index_of(taken.len())) };
    taken
}

/// Remove the items `l[first..=last]` from `l`, releasing what they hold.
///
/// # Safety
///
/// As [`tv_list_take_range`].
pub unsafe fn tv_list_remove_range(l: *mut List, first: usize, last: usize) {
    // Dropped after the cursors have moved: releasing a value can re-enter
    // the evaluator, which must not see a list whose watchers still name
    // items that are gone.
    drop(unsafe { tv_list_take_range(l, first, last) });
}

/// Move the items `l[first..=last]` onto `tgt_l`'s tail.
///
/// # Safety
///
/// As [`tv_list_take_range`], and `tgt_l` must point at a live list,
/// unaliased for the call and not `l` itself.
pub unsafe fn tv_list_move_range(l: *mut List, first: usize, last: usize, tgt_l: *mut List) {
    debug_assert!(l != tgt_l);
    let moved = unsafe { tv_list_take_range(l, first, last) };
    // SAFETY: the caller's promise: a live target list, which is not `l`.
    unsafe { items_of(tgt_l) }.extend(moved);
}

/// Empty `l` without releasing anything its items name.
///
/// The one caller is a funccall's `a:000`, whose items *borrow* the caller's
/// arguments for the length of the call and own nothing.  Upstream spelled
/// this `lv_first = NULL`, which threw away an array of values it had never
/// owned; this is the same statement about an array that would otherwise
/// release them.
pub(crate) fn tv_list_disown_items(l: &mut List) {
    for mut item in ::core::mem::take(&mut l.lv_items) {
        item.li_tv.disown();
    }
}

/// Upgrade every item of `l` to a value of its own.
///
/// The counterpart of [`tv_list_disown_items`]: a funccall that has to
/// outlive the call that made it cannot keep naming the caller's arguments,
/// so each item takes a real copy.
pub(crate) fn tv_list_own_items(l: &mut List) {
    for li in &mut l.lv_items {
        let slot = &raw mut li.li_tv;
        // SAFETY: source and destination are one slot, which `tv_copy` reads
        // before overwriting it with a value that owns what it names.
        unsafe { tv_copy(&*slot, &mut *slot) };
    }
}

/// Allocate an empty list and store it in `ret_tv` as the return value.
///
/// # Safety
///
/// `ret_tv` must point at the caller's return slot: an initialized typval it
/// owns and will clear.
pub unsafe fn tv_list_alloc_ret(ret_tv: &mut TypVal, len: ptrdiff_t) -> *mut List {
    let l = tv_list_alloc(len);
    unsafe { tv_list_set_ret(ret_tv, l) };
    l
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

        /// A count the case wrote, as the `int` this family indexes by.
        fn index(n: usize) -> ::core::ffi::c_int {
            ::core::ffi::c_int::try_from(n).expect("a short list")
        }

        /// `[0, 1, ..., len - 1]`, the list every case below edits.
        pub(super) fn counted(len: usize) -> *mut List {
            let l = tv_list_alloc(ptrdiff_t::try_from(len).expect("a short list"));
            for n in 0..len {
                // SAFETY: the list just allocated.
                unsafe {
                    tv_list_append_number(l, VarNumber::try_from(n).expect("a small number"))
                };
            }
            l
        }

        /// The numbers `l` holds, so a case can say which items survived
        /// rather than how many.
        pub(super) fn numbers(l: *mut List) -> Vec<VarNumber> {
            // SAFETY: a list `counted` made, holding numbers.
            unsafe { tv_list_iter(l.as_ref()).map(|li| li.li_tv.number_or_zero()) }.collect()
        }

        /// A watcher standing on `l[at]`, registered with `l`.
        ///
        /// Handed out as a raw pointer rather than a `Box`: the list stores
        /// the address, so moving the `Box` afterwards would invalidate it.
        /// [`done`] takes it back.
        pub(super) fn watch(l: *mut List, at: usize) -> *mut ListWatch {
            // SAFETY: a list `counted` made.
            assert!(at < unsafe { tv_list_items(l) }.len(), "no item at {at}");
            let lw = Box::into_raw(Box::new(ListWatch {
                lw_index: index(at),
                lw_next: ::core::ptr::null_mut(),
            }));
            // SAFETY: as above, and the watcher outlives its registration.
            unsafe { tv_list_watch_add(l, lw) };
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
            unsafe { tv_list_remove_at(l, at) };
        }

        /// Remove `l[first..=last]`.
        pub(super) fn remove_run(l: *mut List, first: usize, last: usize) {
            // SAFETY: as above, and a run of items of `l`.
            unsafe { tv_list_remove_range(l, first, last) };
        }

        /// Move `l[first..=last]` onto `tgt`'s tail.
        pub(super) fn move_run(l: *mut List, first: usize, last: usize, tgt: *mut List) {
            // SAFETY: as above, plus a second list of this module's own.
            unsafe { tv_list_move_range(l, first, last, tgt) };
        }

        /// Insert the number `n` in front of `l[at]`.
        pub(super) fn insert(l: *mut List, n: VarNumber, at: usize) {
            // SAFETY: as above, and a value the insert copies.
            unsafe { tv_list_insert_tv(l, &TypVal::Number(n), Some(at)) };
        }

        /// Unregister every watcher and free `l`; the pair every case ends
        /// with.
        pub(super) fn done(l: *mut List, lws: &[*mut ListWatch]) {
            for &lw in lws {
                // SAFETY: a watcher `watch` registered with `l`, whose `Box`
                // is taken back here.
                drop(unsafe {
                    tv_list_watch_remove(l, lw);
                    Box::from_raw(lw)
                });
            }
            // SAFETY: a list `counted` made, now unwatched.
            unsafe { tv_list_free(l) };
        }
    }

    /// A slot and its lock, and nothing else: the links are gone, so an
    /// item is the value plus the four bytes `:lockvar l[0]` sets.
    ///
    /// Twenty-four is what a `Vec` of them costs per item, against
    /// upstream's forty *plus* an `xmalloc` header per item -- which is
    /// where `tvbuild` and `tvlist` get their instructions back.
    #[test]
    fn an_item_is_a_value_and_a_lock() {
        assert_eq!(::core::mem::size_of::<ListItem>(), 24);
        assert_eq!(::core::mem::align_of::<ListItem>(), 8);
        assert_eq!(::core::mem::offset_of!(ListItem, li_tv), 0);
    }

    #[test]
    fn removing_an_item_after_the_watcher_leaves_it_where_it_was() {
        let list = l::counted(5);
        let lw = l::watch(list, 1);
        l::remove(list, 3);
        assert_eq!(l::numbers(list), [0, 1, 2, 4]);
        assert_eq!(l::watching(list, lw), Some(1));
        l::done(list, &[lw]);
    }

    #[test]
    fn removing_an_item_before_the_watcher_keeps_it_on_the_same_item() {
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
        let list = l::counted(2);
        let lw = l::watch(list, 1);
        l::remove(list, 1);
        assert_eq!(l::watching(list, lw), None);
        // SAFETY: this module's own list.
        unsafe { tv_list_append_number(list, 9) };
        assert_eq!(l::watching(list, lw), None);
        l::done(list, &[lw]);
    }

    #[test]
    fn removing_a_run_around_the_watcher_lands_it_after_the_run() {
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
        let list = l::counted(4);
        let lw = l::watch(list, 2);
        l::insert(list, 90, 1);
        assert_eq!(l::numbers(list), [0, 90, 1, 2, 3]);
        assert_eq!(l::watching(list, lw), Some(3));
        l::done(list, &[lw]);
    }

    #[test]
    fn inserting_after_the_watcher_leaves_it_where_it_was() {
        let list = l::counted(4);
        let lw = l::watch(list, 1);
        l::insert(list, 90, 3);
        assert_eq!(l::numbers(list), [0, 1, 2, 90, 3]);
        assert_eq!(l::watching(list, lw), Some(1));
        l::done(list, &[lw]);
    }

    #[test]
    fn inserting_at_the_watched_item_pushes_the_watcher_up() {
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
