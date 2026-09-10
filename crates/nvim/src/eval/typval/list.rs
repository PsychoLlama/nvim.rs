//! Allocating, freeing and unlinking a `List` and its `ListItem`s.
//!
//! [`tv_list_alloc`] and [`tv_list_free`] are the reference-counted pair,
//! [`tv_list_unref`] the one every caller actually uses.  The `ListWatch`
//! half ([`tv_list_watch_add`], [`tv_list_watch_fix`]) is how a `:for` loop
//! survives having the item it is standing on removed underneath it, and
//! [`tv_list_drop_items`] / [`tv_list_move_items`] are the two ways items
//! leave a list.

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

/// Safe: it takes nothing and reads nothing.  `xmalloc` either answers an
/// allocation or aborts, so the only obligation left is the ambient one every
/// allocation in the editor carries — being on the main thread — and *using*
/// what comes back is the caller's business, not this call's.
///
/// The three links and the value are the caller's to fill in; `li_lock` is
/// not, because an item is born unlocked and no caller says so.  Upstream got
/// it for free — the lock lived in the value being assigned — and here it is
/// one store into an `xmalloc`'d slot that would otherwise stay uninitialised.
pub(crate) fn tv_list_item_alloc() -> *mut ListItem {
    let li = unsafe { xmalloc(::core::mem::size_of::<ListItem>()).cast::<ListItem>() };
    // SAFETY: the allocation just made, whose lock nothing has read yet.
    unsafe { li_lock(li).write(VarLock::Unlocked) };
    li
}

/// Remove `item` from `l`, clear its value and free it.
///
/// Answers the item that followed it, or NULL when it was the last one.
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call. `item` must point
/// at an item of `l`.
pub unsafe fn tv_list_item_remove(l: *mut List, item: *mut ListItem) -> *mut ListItem {
    let next_item = unsafe { (*item).li_next };
    unsafe { tv_list_drop_items(l, item, item) };
    unsafe { tv_clear(&mut (*item).li_tv) };
    unsafe { xfree(item.cast()) };
    next_item
}

/// Push `lw` onto `l`'s watcher chain.
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call. `lw` must point at
/// an entry of `l`'s watcher chain.
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

/// Advance any watcher standing on `item` to the item after it.
///
/// This is what keeps a `:for` loop walking a list whose current item is
/// removed underneath it.
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call. `item` must point
/// at an item of `l`.
pub(crate) unsafe fn tv_list_watch_fix(l: *mut List, item: *const ListItem) {
    let mut lw = unsafe { (*l).lv_watch };
    while !lw.is_null() {
        // SAFETY: an entry of `l`'s watcher chain.
        let watch = unsafe { Lw::new(lw) };
        if watch.lw_item.cast_const() == item {
            unsafe { (*lw).lw_item = (*item).li_next };
        }
        lw = watch.lw_next;
    }
}

/// Allocate an empty list.  The caller owns the reference count.
///
/// `len` is upstream's hint for a future array-backed list; nothing reads it.
pub fn tv_list_alloc(_len: ptrdiff_t) -> *mut List {
    let list = unsafe { xcalloc(1, ::core::mem::size_of::<List>()) }.cast::<List>();

    // Prepend the list to the list of lists for garbage collection.
    if let Some(first) = unsafe { gc_first_list.get().as_mut() } {
        first.lv_used_prev = list;
    }
    unsafe { (*list).lv_used_prev = ::core::ptr::null_mut() };
    unsafe { (*list).lv_used_next = gc_first_list.get() };
    gc_first_list.set(list);
    unsafe { (*list).lua_table_ref = LUA_NOREF as LuaRef };
    list
}

/// Initialise a stack-allocated ten-item list, all items zeroed and linked.
///
/// The list is `VarLock::Fixed` and carries `DO_NOT_FREE_CNT`, so nothing frees it.
///
/// # Safety
///
/// `sl` must point at a `StaticList10` the caller owns for as long as the
/// list is used; the list is *not* heap-allocated and must never be freed.
pub unsafe fn tv_list_init_static10(sl: *mut StaticList10) {
    // No `Live<StaticList10>` here: the list this builds points at the item
    // array in the *same* struct, and a `DerefMut` that reborrows the whole
    // struct pops those interior pointers under Stacked and Tree Borrows.
    unsafe { sl.write_bytes(0, 1) };
    let l = unsafe { &raw mut (*sl).sl_list };
    let items = unsafe { &raw mut (*sl).sl_items }.cast::<ListItem>();

    unsafe { (*l).lv_first = items };
    unsafe { (*l).lv_last = items.add(SL_SIZE - 1) };
    unsafe { (*l).lv_refcount = Refcount::new(DO_NOT_FREE_CNT.cast_signed()) };
    unsafe { tv_list_set_lock(l, VarLock::Fixed) };
    unsafe { (*l).lv_len = 10 };

    unsafe { (*items).li_prev = ::core::ptr::null_mut() };
    unsafe { (*items).li_next = items.add(1) };
    unsafe { (*items.add(SL_SIZE - 1)).li_prev = items.add(SL_SIZE - 2) };
    unsafe { (*items.add(SL_SIZE - 1)).li_next = ::core::ptr::null_mut() };

    for i in 1..SL_SIZE - 1 {
        let li = unsafe { items.add(i) };
        unsafe { (*li).li_prev = li.sub(1) };
        unsafe { (*li).li_next = li.add(1) };
    }
}

/// Initialise a stack-allocated empty list that nothing may free.
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call.
pub unsafe fn tv_list_init_static(l: *mut List) {
    unsafe { l.write_bytes(0, 1) };
    unsafe { (*l).lv_refcount = Refcount::new(DO_NOT_FREE_CNT.cast_signed()) };
}

/// Free every item in `l`, leaving the list itself allocated and empty.
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call.
pub unsafe fn tv_list_free_contents(l: *mut List) {
    // Unlink each item before clearing it: `tv_clear` can re-enter.
    // SAFETY: the caller's promise: a live list.
    let mut list = unsafe { Ls::new(l) };
    let mut item = list.lv_first;
    while !item.is_null() {
        unsafe { (*l).lv_first = (*item).li_next };
        unsafe { tv_clear(&mut (*item).li_tv) };
        unsafe { xfree(item.cast()) };
        item = list.lv_first;
    }
    list.lv_len = 0;
    unsafe { (*l).lv_idx_item = ::core::ptr::null_mut() };
    unsafe { (*l).lv_last = ::core::ptr::null_mut() };
    debug_assert!(list.lv_watch.is_null());
}

/// Unlink `l` from the garbage collector's chain and free the `List` itself.
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call.
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

/// Unlink the items `item..=item2` from `l` without freeing them.
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call. `item` must point
/// at an item of `l`. `item2` must point at an item of `l`.
pub unsafe fn tv_list_drop_items(l: *mut List, item: *mut ListItem, item2: *mut ListItem) {
    // Notify watchers.
    let mut ip = item;
    // SAFETY: the caller's promise: an item of `l`.
    let last = unsafe { Li::new(item2) };
    while ip != last.li_next {
        unsafe { (*l).lv_len -= 1 };
        unsafe { tv_list_watch_fix(l, ip) };
        ip = unsafe { (*ip).li_next };
    }

    // SAFETY: the caller's promise: an item of `l`.
    let first = unsafe { Li::new(item) };
    match unsafe { (*item2).li_next.as_mut() } {
        Some(after) => after.li_prev = first.li_prev,
        None => unsafe { (*l).lv_last = (*item).li_prev },
    }
    match unsafe { (*item).li_prev.as_mut() } {
        Some(before) => before.li_next = last.li_next,
        None => unsafe { (*l).lv_first = (*item2).li_next },
    }
    unsafe { (*l).lv_idx_item = ::core::ptr::null_mut() };
}

/// Unlink the items `item..=item2` from `l` and free them.
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call. `item` must point
/// at an item of `l`. `item2` must point at an item of `l`.
pub unsafe fn tv_list_remove_items(l: *mut List, item: *mut ListItem, item2: *mut ListItem) {
    unsafe { tv_list_drop_items(l, item, item2) };
    let mut li = item;
    loop {
        unsafe { tv_clear(&mut (*li).li_tv) };
        // Read the link before the free, not after.
        let nli = unsafe { (*li).li_next };
        unsafe { xfree(li.cast()) };
        if li == item2 {
            break;
        }
        li = nli;
    }
}

/// Move the items `item..=item2` (`cnt` of them) from `l` onto `tgt_l`'s tail.
///
/// # Safety
///
/// `l` must point at a live list, unaliased for the call. `item` must point
/// at an item of `l`. `item2` must point at an item of `l`. `tgt_l` must
/// point at a live list, unaliased for the call.
pub unsafe fn tv_list_move_items(
    l: *mut List,
    item: *mut ListItem,
    item2: *mut ListItem,
    tgt_l: *mut List,
    cnt: ::core::ffi::c_int,
) {
    unsafe { tv_list_drop_items(l, item, item2) };
    unsafe { (*item).li_prev = (*tgt_l).lv_last };
    unsafe { (*item2).li_next = ::core::ptr::null_mut() };
    // SAFETY: the caller's promise: a live target list.
    let mut tgt = unsafe { Ls::new(tgt_l) };
    match unsafe { (*tgt_l).lv_last.as_mut() } {
        Some(last) => last.li_next = item,
        None => tgt.lv_first = item,
    }
    tgt.lv_last = item2;
    tgt.lv_len += cnt;
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
            unsafe { tv_list_iter(l.as_ref()).map(|li| (*li).li_tv.number_or_zero()) }.collect()
        }

        /// The item at `at`, which must be there.
        pub(super) fn at(l: *mut List, at: usize) -> *mut ListItem {
            // SAFETY: a list `counted` made.
            let item = unsafe { tv_list_find(l, index(at)) };
            assert!(!item.is_null(), "no item at {at}");
            item
        }

        /// A watcher standing on `l[index]`, registered with `l`.
        ///
        /// Handed out as a raw pointer rather than a `Box`: the list stores
        /// the address, so moving the `Box` afterwards would invalidate it.
        /// [`done`] takes it back.
        pub(super) fn watch(l: *mut List, index: usize) -> *mut ListWatch {
            let lw = Box::into_raw(Box::new(ListWatch {
                lw_item: at(l, index),
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
            // SAFETY: a watcher `watch` registered with `l`.
            let at = unsafe { tv_list_idx_of_item(l, (*lw).lw_item) };
            // -1 is "not an item of this list", which for a watcher means
            // the NULL it was pushed off the end to.
            usize::try_from(at).ok()
        }

        /// Remove `l[index]`.
        pub(super) fn remove(l: *mut List, index: usize) {
            // SAFETY: a list `counted` made, and an item of it.
            unsafe { tv_list_item_remove(l, at(l, index)) };
        }

        /// Remove `l[first..=last]`.
        pub(super) fn remove_run(l: *mut List, first: usize, last: usize) {
            // SAFETY: as above, and a run of items of `l`.
            unsafe { tv_list_remove_items(l, at(l, first), at(l, last)) };
        }

        /// Move `l[first..=last]` onto `tgt`'s tail.
        pub(super) fn move_run(l: *mut List, first: usize, last: usize, tgt: *mut List) {
            let cnt = index(last - first + 1);
            // SAFETY: as above, plus a second list of this module's own.
            unsafe { tv_list_move_items(l, at(l, first), at(l, last), tgt, cnt) };
        }

        /// Insert the number `n` in front of `l[index]`.
        pub(super) fn insert(l: *mut List, n: VarNumber, index: usize) {
            // SAFETY: as above, and a value the insert copies.
            unsafe { tv_list_insert_tv(l, &TypVal::Number(n), at(l, index)) };
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
