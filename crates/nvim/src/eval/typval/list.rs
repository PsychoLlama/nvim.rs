//! Allocating, freeing and editing a `List` and the items it owns.
//!
//! [`tv_list_alloc`] and [`list_free`] are the reference-counted pair,
//! [`list_unref`] the one every caller actually uses.  The `ListWatch`
//! half ([`List::watch_add`], [`watch_shift`]) is how a `:for`
//! loop survives having the item it is standing on removed underneath it,
//! and [`ListRef::remove_range`] / [`List::move_range_to`] are the two ways
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
//! # Releasing items
//!
//! A value can name the list it is in, and releasing it reads that list
//! again. So nothing here releases a value while a borrow of the list is
//! live: the items are taken out first ([`List::take_range`]), the borrow
//! ends, and *then* they are dropped -- which is why the entry points that
//! release take a [`ListRef`].

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use super::*;

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

    /// Register a cursor standing on the first item, and answer the id it
    /// is known by from here on.
    pub fn watch_add(&mut self) -> u32 {
        let id = self
            .lv_watch
            .iter()
            .map(|watch| watch.id)
            .max()
            .map_or(1, |id| id + 1);
        self.lv_watch.push(ListWatch { id, index: 0 });
        id
    }

    /// Where the cursor `id` stands: an index of the list, or
    /// [`ListWatch::ENDED`].
    pub fn watch_index(&self, id: u32) -> ::core::ffi::c_int {
        self.lv_watch
            .iter()
            .find(|watch| watch.id == id)
            .map_or(ListWatch::ENDED, |watch| watch.index)
    }

    /// Move the cursor `id` to `index`.
    pub fn set_watch_index(&mut self, id: u32, index: ::core::ffi::c_int) {
        if let Some(watch) = self.lv_watch.iter_mut().find(|watch| watch.id == id) {
            watch.index = index;
        }
    }

    /// Unregister the cursor `id`.
    pub fn watch_remove(&mut self, id: u32) {
        self.lv_watch.retain(|watch| watch.id != id);
    }

    /// Whether any `:for` loop is walking this list.
    #[inline(always)]
    pub(crate) fn is_watched(&self) -> bool {
        !self.lv_watch.is_empty()
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
    if l.lv_watch.is_empty() {
        return;
    }
    let len = index_of(l.len());
    for watch in &mut l.lv_watch {
        if watch.index >= at {
            // Clamped at `at`: a cursor inside a removed run lands on
            // whatever followed the run.
            let moved = (watch.index + count).max(at);
            watch.index = if moved >= len {
                ListWatch::ENDED
            } else {
                moved
            };
        }
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
    for watch in &mut l.lv_watch {
        let to = usize::try_from(watch.index)
            .ok()
            .and_then(|at| moved.get(at));
        if let Some(&to) = to {
            watch.index = to;
        }
    }
}

/// A length or index of a list, as the `int` the family counts in.
///
/// Lists are `int`-indexed the whole way down (`list_len`, `E684`, the
/// `[n1:n2]` arithmetic), so this is where the width changes, once.  A list
/// longer than `INT_MAX` cannot be built: every path that adds an item goes
/// through a length this saturates.  A subscript the user wrote goes through
/// it too, which is why it takes a signed width as well: upstream truncated
/// the `varnumber_T` and let the bounds check reject whatever came out, and
/// saturating is the same answer for every index a list can hold.
#[inline(always)]
pub(crate) fn index_of<N>(n: N) -> ::core::ffi::c_int
where
    N: Copy + Default + PartialOrd + TryInto<::core::ffi::c_int>,
{
    let negative = n < N::default();
    n.try_into().unwrap_or(if negative {
        ::core::ffi::c_int::MIN
    } else {
        ::core::ffi::c_int::MAX
    })
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

    /// The handle this value holds, borrowed -- what to keep across a call
    /// that can run user code, `.edit()`ing it one statement at a time.
    #[inline(always)]
    pub(crate) fn list_shared(&self) -> Option<&ListRef> {
        match self {
            TypVal::List(list) => (**list).as_ref(),
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

    /// Another reference to the list this value holds -- `None` for every
    /// other kind and for `v:_null_list`.
    #[inline(always)]
    pub(crate) fn list_handle(&self) -> Option<ListRef> {
        match self {
            TypVal::List(list) => (**list).clone(),
            _ => None,
        }
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

impl List {
    /// Take the items `self[first..=last]` out without releasing what they
    /// hold; the caller owns them now, and drops them once its borrow of
    /// the list has ended.  `first..=last` must be a run of the list's
    /// items.
    pub(crate) fn take_range(&mut self, first: usize, last: usize) -> Vec<ListItem> {
        let taken: Vec<ListItem> = self.lv_items.drain(first..=last).collect();
        // The items are gone, so the cursors move now.
        watch_shift(self, index_of(first), -index_of(taken.len()));
        taken
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
    /// arguments, so each item takes a real copy -- written over the
    /// borrowed bits, which were never this list's to release.
    pub(crate) fn own_items(&mut self) {
        for item in &mut self.lv_items {
            let copy = item.li_tv.clone();
            item.li_tv.overwrite(copy);
        }
    }
}

/// The removals that release what they take: on the handle, because the
/// release happens after the borrow of the list has ended.
impl ListRef {
    /// Remove the items `self[first..=last]`, releasing what they hold.
    pub fn remove_range(&self, first: usize, last: usize) {
        // Dropped after the cursors have moved and the borrow has ended:
        // releasing a value can re-enter the evaluator, which must not see
        // a list whose watchers still name items that are gone -- and the
        // value may name this very list.
        let taken = self.edit().take_range(first, last);
        drop(taken);
    }

    /// Remove `self[at]`, releasing the value it held.
    ///
    /// Answers the index of the item that followed it, which is `at` again
    /// -- or `None` when the removed item was the last one.
    pub fn remove_at(&self, at: usize) -> Option<usize> {
        self.remove_range(at, at);
        (at < self.len()).then_some(at)
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

/// Free every item in `list`, leaving the list itself allocated and empty.
///
/// The items are taken out before anything is released, and released once
/// the borrow of the list has ended: releasing a value can re-enter the
/// evaluator, which must not find a list half way through being emptied --
/// and a value naming this list reads it again.
pub fn list_free_contents(list: &ListRef) {
    let items = ::core::mem::take(&mut list.edit().lv_items);
    debug_assert!(!list.is_watched());
    // Dropping the array clears each value in turn, front to back.
    drop(items);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::collect::var_item_copy_with;
    use crate::global_cell::editor_state_lock;
    use ::core::mem::ManuallyDrop;

    /// The safe layer these cases are written against, so that a case reads
    /// as ordinary code and says only what it is about.
    mod l {
        use super::*;

        /// A count the case wrote, as the `int` this family indexes by.
        fn index(n: usize) -> ::core::ffi::c_int {
            ::core::ffi::c_int::try_from(n).expect("a short list")
        }

        /// `[0, 1, ..., len - 1]`, the list every case below edits.
        pub(super) fn counted(len: usize) -> ListRef {
            let mut list = tv_list_alloc(ptrdiff_t::try_from(len).expect("a short list"));
            for n in 0..len {
                list.push_number(VarNumber::try_from(n).expect("a small number"));
            }
            list
        }

        /// The numbers `l` holds, so a case can say which items survived
        /// rather than how many.
        pub(super) fn numbers(l: &List) -> Vec<VarNumber> {
            l.items()
                .iter()
                .map(|li| li.li_tv.number_or_zero())
                .collect()
        }

        /// A watcher standing on `l[at]`, registered with `l`.
        pub(super) fn watch(l: &mut List, at: usize) -> u32 {
            assert!(at < l.len(), "no item at {at}");
            let id = l.watch_add();
            l.set_watch_index(id, index(at));
            id
        }

        /// Where a watcher is standing, as an index into `l` -- `None` once
        /// it has been pushed off the end.
        ///
        /// The identity a `:for` loop holds on to is *the item*, and what
        /// the watcher bookkeeping owes is that the item does not change
        /// under an edit somewhere else in the list.
        pub(super) fn watching(l: &List, id: u32) -> Option<usize> {
            // `ENDED` is negative, which is the NULL upstream pushed a
            // cursor off the end to.
            usize::try_from(l.watch_index(id)).ok()
        }

        /// Insert the number `n` in front of `l[at]`.
        pub(super) fn insert(l: &mut List, n: VarNumber, at: usize) {
            l.insert_copy(&TypVal::Number(n), Some(at));
        }

        /// One step of a `:for` loop's cursor: the number it stands on,
        /// with the cursor moved past it -- what `next_for_item` does
        /// before the body runs.  `None` once the walk has ended.
        pub(super) fn step(l: &mut List, id: u32) -> Option<VarNumber> {
            let at = watching(l, id)?;
            let now = numbers(l);
            let item = *now.get(at)?;
            let next = if at + 1 >= now.len() {
                ListWatch::ENDED
            } else {
                index(at + 1)
            };
            l.set_watch_index(id, next);
            Some(item)
        }

        /// Unregister every watcher and let `l` go; the pair every case
        /// ends with.
        pub(super) fn done(mut l: ListRef, ids: &[u32]) {
            for &id in ids {
                l.watch_remove(id);
            }
            drop(l);
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
        let _serial = editor_state_lock();
        let mut list = l::counted(5);
        let lw = l::watch(&mut list, 1);
        list.remove_at(3);
        assert_eq!(l::numbers(&list), [0, 1, 2, 4]);
        assert_eq!(l::watching(&list, lw), Some(1));
        l::done(list, &[lw]);
    }

    #[test]
    fn removing_an_item_before_the_watcher_keeps_it_on_the_same_item() {
        let _serial = editor_state_lock();
        let mut list = l::counted(5);
        let lw = l::watch(&mut list, 3);
        list.remove_at(1);
        assert_eq!(l::numbers(&list), [0, 2, 3, 4]);
        // The item it stands on is still the one holding 3 -- which has
        // moved down one place.
        assert_eq!(l::watching(&list, lw), Some(2));
        l::done(list, &[lw]);
    }

    #[test]
    fn removing_the_watched_item_lands_the_watcher_on_the_next_one() {
        let _serial = editor_state_lock();
        let mut list = l::counted(5);
        let lw = l::watch(&mut list, 2);
        list.remove_at(2);
        assert_eq!(l::numbers(&list), [0, 1, 3, 4]);
        // Index 2 again, but the item that *followed* the removed one.
        assert_eq!(l::watching(&list, lw), Some(2));
        l::done(list, &[lw]);
    }

    #[test]
    fn removing_the_watched_last_item_pushes_the_watcher_off_the_end() {
        let _serial = editor_state_lock();
        let mut list = l::counted(3);
        let lw = l::watch(&mut list, 2);
        list.remove_at(2);
        assert_eq!(l::numbers(&list), [0, 1]);
        assert_eq!(l::watching(&list, lw), None);
        l::done(list, &[lw]);
    }

    #[test]
    fn a_watcher_off_the_end_stays_off_it_when_the_list_grows_again() {
        // What ends a `:for` loop whose body appends to the list it is
        // walking: the cursor is already past the end, and nothing puts it
        // back.
        let _serial = editor_state_lock();
        let mut list = l::counted(2);
        let lw = l::watch(&mut list, 1);
        list.remove_at(1);
        assert_eq!(l::watching(&list, lw), None);
        list.push_number(9);
        assert_eq!(l::watching(&list, lw), None);
        l::done(list, &[lw]);
    }

    #[test]
    fn removing_a_run_around_the_watcher_lands_it_after_the_run() {
        let _serial = editor_state_lock();
        let mut list = l::counted(7);
        let lws = [
            l::watch(&mut list, 0),
            l::watch(&mut list, 3),
            l::watch(&mut list, 6),
        ];
        list.remove_range(2, 4);
        assert_eq!(l::numbers(&list), [0, 1, 5, 6]);
        assert_eq!(l::watching(&list, lws[0]), Some(0));
        // Was on 3, inside the run: now on what followed the run, 5.
        assert_eq!(l::watching(&list, lws[1]), Some(2));
        // Was on 6, after the run: still on 6.
        assert_eq!(l::watching(&list, lws[2]), Some(3));
        l::done(list, &lws);
    }

    #[test]
    fn inserting_before_the_watcher_keeps_it_on_the_same_item() {
        let _serial = editor_state_lock();
        let mut list = l::counted(4);
        let lw = l::watch(&mut list, 2);
        l::insert(&mut list, 90, 1);
        assert_eq!(l::numbers(&list), [0, 90, 1, 2, 3]);
        assert_eq!(l::watching(&list, lw), Some(3));
        l::done(list, &[lw]);
    }

    #[test]
    fn inserting_after_the_watcher_leaves_it_where_it_was() {
        let _serial = editor_state_lock();
        let mut list = l::counted(4);
        let lw = l::watch(&mut list, 1);
        l::insert(&mut list, 90, 3);
        assert_eq!(l::numbers(&list), [0, 1, 2, 90, 3]);
        assert_eq!(l::watching(&list, lw), Some(1));
        l::done(list, &[lw]);
    }

    #[test]
    fn inserting_at_the_watched_item_pushes_the_watcher_up() {
        let _serial = editor_state_lock();
        let mut list = l::counted(4);
        let lw = l::watch(&mut list, 1);
        l::insert(&mut list, 90, 1);
        assert_eq!(l::numbers(&list), [0, 90, 1, 2, 3]);
        // Still on the item holding 1, now one place further along.
        assert_eq!(l::watching(&list, lw), Some(2));
        l::done(list, &[lw]);
    }

    #[test]
    fn moving_the_watched_run_to_another_list_lands_the_watcher_after_it() {
        let _serial = editor_state_lock();
        let mut list = l::counted(6);
        let mut tgt = l::counted(0);
        let lws = [l::watch(&mut list, 1), l::watch(&mut list, 4)];
        list.move_range_to(1, 2, &mut tgt);
        assert_eq!(l::numbers(&list), [0, 3, 4, 5]);
        assert_eq!(l::numbers(&tgt), [1, 2]);
        // A watcher follows the list it is registered with, not the items
        // that left it.
        assert_eq!(l::watching(&list, lws[0]), Some(1));
        assert_eq!(l::watching(&list, lws[1]), Some(2));
        l::done(list, &lws);
        l::done(tgt, &[]);
    }

    /// A walk whose body removes the item *after* the one it stands on:
    /// the cursor already names that item, so it lands on the one after.
    #[test]
    fn a_walk_that_removes_the_next_item_skips_it() {
        let _serial = editor_state_lock();
        let mut list = l::counted(5);
        let lw = l::watch(&mut list, 0);
        let mut seen = Vec::new();
        while let Some(n) = l::step(&mut list, lw) {
            seen.push(n);
            if n == 1 {
                // The cursor stands on 2; take it away.
                assert_eq!(l::watching(&list, lw), Some(2));
                list.remove_at(2);
                assert_eq!(l::watching(&list, lw), Some(2));
            }
        }
        assert_eq!(seen, [0, 1, 3, 4]);
        assert_eq!(l::numbers(&list), [0, 1, 3, 4]);
        l::done(list, &[lw]);
    }

    /// A walk whose body removes the item it was just handed: the cursor is
    /// past it already, so it shifts down with what follows and the walk
    /// misses nothing.
    #[test]
    fn a_walk_that_removes_its_current_item_misses_nothing() {
        let _serial = editor_state_lock();
        let mut list = l::counted(5);
        let lw = l::watch(&mut list, 0);
        let mut seen = Vec::new();
        while let Some(n) = l::step(&mut list, lw) {
            seen.push(n);
            if n == 1 || n == 2 {
                let at = l::numbers(&list)
                    .iter()
                    .position(|&v| v == n)
                    .expect("the current item");
                let before = l::watching(&list, lw).expect("not at the end");
                list.remove_at(at);
                assert_eq!(l::watching(&list, lw), Some(before - 1));
            }
        }
        assert_eq!(seen, [0, 1, 2, 3, 4]);
        assert_eq!(l::numbers(&list), [0, 3, 4]);
        l::done(list, &[lw]);
    }

    /// A body that removes a run containing the cursor's item, and then one
    /// reaching the end: the first lands the cursor after the run, the
    /// second ends the walk.
    #[test]
    fn a_walk_that_removes_a_run_around_its_cursor_resumes_after_it() {
        let _serial = editor_state_lock();
        let mut list = l::counted(8);
        let lw = l::watch(&mut list, 0);
        let mut seen = Vec::new();
        while let Some(n) = l::step(&mut list, lw) {
            seen.push(n);
            match n {
                1 => {
                    // The cursor is on 2; take 1..=3 away.
                    list.remove_range(1, 3);
                    assert_eq!(l::numbers(&list), [0, 4, 5, 6, 7]);
                    assert_eq!(l::watching(&list, lw), Some(1));
                }
                5 => {
                    // The cursor is on 6; take it and everything after.
                    list.remove_range(3, 4);
                    assert_eq!(l::watching(&list, lw), None);
                }
                _ => {}
            }
        }
        assert_eq!(seen, [0, 1, 4, 5]);
        assert_eq!(l::numbers(&list), [0, 4, 5]);
        l::done(list, &[lw]);
    }

    /// A body that inserts in front of its current item shifts the cursor
    /// with the items, so the walk neither repeats nor visits the new one.
    #[test]
    fn a_walk_that_inserts_before_its_item_does_not_visit_the_insert() {
        let _serial = editor_state_lock();
        let mut list = l::counted(4);
        let lw = l::watch(&mut list, 0);
        let mut seen = Vec::new();
        while let Some(n) = l::step(&mut list, lw) {
            seen.push(n);
            if n == 2 {
                l::insert(&mut list, 90, 2);
                assert_eq!(l::watching(&list, lw), Some(4));
            }
        }
        assert_eq!(seen, [0, 1, 2, 3]);
        assert_eq!(l::numbers(&list), [0, 1, 90, 2, 3]);
        l::done(list, &[lw]);
    }

    /// Two cursors on one list -- a nested `:for` -- each follow their own
    /// item through the same edit.
    #[test]
    fn two_watchers_on_one_list_each_follow_their_item() {
        let _serial = editor_state_lock();
        let mut list = l::counted(6);
        let outer = l::watch(&mut list, 2);
        let inner = l::watch(&mut list, 4);
        assert_ne!(outer, inner);
        list.remove_at(3);
        assert_eq!(l::watching(&list, outer), Some(2));
        assert_eq!(l::watching(&list, inner), Some(3));
        // Removing the outer cursor's item leaves the inner one alone.
        list.remove_at(2);
        assert_eq!(l::numbers(&list), [0, 1, 4, 5]);
        assert_eq!(l::watching(&list, outer), Some(2));
        assert_eq!(l::watching(&list, inner), Some(2));
        // Both land on the same item, and both end together.
        list.remove_range(2, 3);
        assert_eq!(l::watching(&list, outer), None);
        assert_eq!(l::watching(&list, inner), None);
        // Unregistering one leaves the other registered.
        list.watch_remove(outer);
        let ids: Vec<u32> = list.lv_watch.iter().map(|watch| watch.id).collect();
        assert_eq!(ids, [inner]);
        l::done(list, &[inner]);
    }

    /// `watch_permute` moves each cursor to where its item went.
    #[test]
    fn a_permutation_carries_every_watcher_with_its_item() {
        let _serial = editor_state_lock();
        let mut list = l::counted(4);
        let lws = [l::watch(&mut list, 0), l::watch(&mut list, 3)];
        // Reverse: index i goes to 3 - i.
        list.lv_items.reverse();
        watch_permute(&mut list, &[3, 2, 1, 0]);
        assert_eq!(l::numbers(&list), [3, 2, 1, 0]);
        assert_eq!(l::watching(&list, lws[0]), Some(3));
        assert_eq!(l::watching(&list, lws[1]), Some(0));
        l::done(list, &lws);
    }

    /// The reference count of `l`.
    fn refs(l: &List) -> i32 {
        l.lv_refcount.get()
    }

    /// A handle over `list` that owns no reference -- what the collector
    /// holds while it frees a list nobody references.
    fn view(list: &ListRef) -> ManuallyDrop<ListRef> {
        let view = ManuallyDrop::new(list.clone());
        view.edit().lv_refcount.release();
        view
    }

    /// Give `view` a reference of its own again and let it go: the release
    /// that frees whatever is left of the list.
    fn free(view: ManuallyDrop<ListRef>) {
        view.edit().lv_refcount.retain();
        drop(ManuallyDrop::into_inner(view));
    }

    /// Hold `tv_in_free_unref_items` up for a scope, and put it down even
    /// when an assertion unwinds through it -- a flag left up would turn
    /// every later case's frees into leaks.
    struct Collecting;
    impl Collecting {
        fn start() -> Collecting {
            tv_in_free_unref_items.set(true);
            Collecting
        }
    }
    impl Drop for Collecting {
        fn drop(&mut self) {
            tv_in_free_unref_items.set(false);
        }
    }

    /// While the collector is freeing, the last reference going does not
    /// free the list: the collector frees the whole graph itself, and a
    /// free from inside it would free something twice.
    #[test]
    fn a_list_released_mid_collection_waits_for_the_collector() {
        let _serial = editor_state_lock();
        let list = l::counted(3);
        let held = view(&list);
        assert_eq!(refs(&held), 1);
        {
            let _collecting = Collecting::start();
            drop(list);
            // Still allocated, still holding its items, at zero.
            assert_eq!(refs(&held), 0);
            assert_eq!(l::numbers(&held), [0, 1, 2]);
        }
        // What `free_unref_items` does with it: contents, then the list.
        list_free_contents(&held);
        free(held);
    }

    /// A list that holds itself never reaches zero by itself; the
    /// collector's two passes are what free it.
    ///
    /// Pass 1 is `list_free_contents`, exactly as `free_unref_items` calls
    /// it, and dropping the self-reference walks the list again -- which is
    /// why the items are taken out under a borrow that has ended by then.
    #[test]
    fn a_self_cycle_is_freed_by_the_collectors_two_passes() {
        let _serial = editor_state_lock();
        let mut list = l::counted(1);
        let itself = list.clone();
        list.push_list(Some(itself));
        assert_eq!(refs(&list), 2);
        let held = view(&list);
        drop(list);
        assert_eq!(refs(&held), 1, "the cycle keeps it alive");

        let collecting = Collecting::start();
        // Pass 1: the contents.  The self-reference goes to zero, and the
        // flag keeps that from freeing the list under the walk.
        list_free_contents(&held);
        assert_eq!(refs(&held), 0);
        drop(collecting);
        // Pass 2: the structure.
        free(held);
    }

    /// A deep copy of a list that holds itself: the copy's item names the
    /// *copy*, and the original's count is what it was.
    #[test]
    fn a_deep_copy_of_a_self_cycle_points_at_the_copy() {
        let _serial = editor_state_lock();
        let mut list = l::counted(1);
        let itself = list.clone();
        list.push_list(Some(itself));
        let from = TypVal::list(Some(list.clone()));
        let before = refs(&list);

        let mut to = TypVal::Unknown;
        let copy_id = crate::eval::get_copy_id();
        let copied = var_item_copy_with(None, &from, &mut to, true, copy_id);
        assert_eq!(copied, Ok(()));
        let copy = to.list_shared().expect("a list").clone();
        assert!(!copy.ptr_eq(&list));
        assert_eq!(l::numbers(&copy)[0], 0);
        let inner = copy.items()[1].li_tv.list_shared().expect("a list");
        assert!(inner.ptr_eq(&copy), "the cycle followed the original");
        assert_eq!(refs(&list), before);
        // The value `to` holds, the copy's item, and `copy` here.
        assert_eq!(refs(&copy), 3);

        // Break both cycles, then let the values go.
        for l in [&copy, &list] {
            l.remove_at(1);
        }
        drop(copy);
        let mut from = from;
        tv_clear(&mut from);
        tv_clear(&mut to);
        l::done(list, &[]);
    }

    /// `remove(l, i)` where `l[i]` is `l` itself: releasing a list value
    /// walks that list (`tv_clear` -> `encode_vim_to_nothing` reads its
    /// length), so the removal takes the item out under a borrow that has
    /// ended before it is released.  The same shape is
    /// `let l = [1] | call add(l, l) | call remove(l, 1)`.
    #[test]
    fn removing_an_item_that_names_its_own_list() {
        let _serial = editor_state_lock();
        let mut list = l::counted(1);
        let itself = list.clone();
        list.push_list(Some(itself));
        assert_eq!(refs(&list), 2);
        list.remove_at(1);
        assert_eq!(refs(&list), 1);
        assert_eq!(l::numbers(&list), [0]);
        l::done(list, &[]);
    }

    /// A shared but acyclic item stays shared in a deep copy -- one copy,
    /// referenced twice -- and a copy without an id (`deepcopy(x, 1)`'s
    /// `noref`) copies it twice.
    #[test]
    fn a_deep_copy_keeps_sharing_only_under_a_copy_id() {
        let _serial = editor_state_lock();
        let shared = l::counted(2);
        let mut outer = l::counted(0);
        for _ in 0..2 {
            outer.push_list(Some(shared.clone()));
        }
        let from = TypVal::list(Some(outer.clone()));
        let item = |l: &List, at: usize| l.items()[at].li_tv.list_or_null();

        for (copy_id, shares) in [(crate::eval::get_copy_id(), true), (0, false)] {
            let mut to = TypVal::Unknown;
            let copied = var_item_copy_with(None, &from, &mut to, true, copy_id);
            assert_eq!(copied, Ok(()));
            let copy = to.list_ref().expect("a list");
            assert_ne!(item(copy, 0), shared.as_ptr());
            assert_eq!(item(copy, 0) == item(copy, 1), shares);
            let second = copy.items()[1].li_tv.list_ref().expect("a list");
            assert_eq!(l::numbers(second), [0, 1]);
            tv_clear(&mut to);
        }
        assert_eq!(refs(&shared), 3);

        let mut from = from;
        tv_clear(&mut from);
        l::done(outer, &[]);
        l::done(shared, &[]);
    }
}
