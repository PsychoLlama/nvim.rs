//! `describe('list')` from `test/unit/eval/typval_spec.lua`.
//!
//! The spec's 285 `alloc_log` hooks asserted an exact allocation sequence
//! through the `mem_*` function-pointer seam, and most cases here still do,
//! through [`crate::support::alloc::AllocLog`]. What they no longer say is
//! *per item*: a `List` owns its items in a `Vec<ListItem>`, whose growth
//! goes through Rust's global allocator, which the log deliberately does
//! not see. So the sequence a case asserts is the list's own `xcalloc` plus
//! the allocations of the *values* the items hold — and a case whose whole
//! point was the item allocation ("appending a number costs only the item")
//! now pins the contents, the reference counts and what gets freed instead.
//!
//! The porting rule that is unchanged: a size is always `size_of` or
//! `offset_of!`, never a literal, because the derivation *is* the assertion.
//!
//! The other thing the item store changed is *identity*. An item used to be
//! its own allocation, so a case could hold a `*mut ListItem` across an edit
//! and ask where it ended up; now the address is a borrow of the list's
//! array and any edit invalidates it. Every case that used an item as an
//! identity — which watcher stands where, which item a removal answered —
//! is written in indexes and values instead.
//!
//! Values are built and read back through [`crate::support::tv`], the twin
//! of `test/unit/eval/testutil.lua`.
//!
//! Every case needs a live editor, which Miri cannot start.

#![cfg(not(miri))]

use std::ffi::{CStr, c_char, c_int};
use std::ptr;

use neovim::eval::typval::{
    DictRef, ListRef, NumBuf, list_concat, list_copy, list_equal, list_extend, list_find,
    list_find_nr, list_find_str, list_first, list_free, list_free_contents, list_free_list,
    list_join, list_last, list_len, list_unref, tv_clear, tv_list_alloc,
};
use neovim::garray::ga_clear;
use neovim::mbyte::convert_setup;
use neovim::memory::xstrdup;
use neovim::types::{List, ListWatch, Refcount, TypVal, VAR_LIST, VarLock, VimConv};

use crate::support::alloc::{self, AllocLog};
use crate::support::tv::{self, Payload, Tv};
use crate::support::{check_emsg, cstr};

/// [`list_copy`] answering the pointer these cases are written against.
///
/// The copy comes back as an owning handle; the case takes the reference
/// over and gives it back with `list_free`/`list_unref`, which is what
/// upstream's `list_copy` + `tv_list_ref` pair left it holding.
///
/// # Safety
/// As [`list_copy`].
unsafe fn copied(conv: *const VimConv, orig: *mut List, deep: bool, copy_id: c_int) -> *mut List {
    let held = unsafe { ListRef::retained(orig) };
    unsafe { list_copy(conv, held, deep, copy_id) }.map_or(ptr::null_mut(), ListRef::into_raw)
}

/// The spec's bare Lua numbers, which `lua2typvalt` made floats.
fn f(n: f64) -> Tv {
    Tv::Float(n)
}

/// `1, 2, … n` as floats — the spec's `list(1, 2, 3, …)`.
fn floats(ns: impl IntoIterator<Item = i32>) -> Vec<Tv> {
    ns.into_iter().map(|n| f(f64::from(n))).collect()
}

/// The spec's `list_watch`: a watcher standing on `l[at]`, registered with
/// `l`.
///
/// The Lua allocated it with `ffi.new`, which the allocation log does not
/// see; a `Box` is the same statement here. It is handed out as a raw
/// pointer because the list stores the address, so moving the `Box`
/// afterwards would invalidate it — [`unwatch`] takes it back.
///
/// # Safety
/// `l` is a live list and `at` an index into it.
unsafe fn watch(l: *mut List, at: usize) -> *mut ListWatch {
    let lw = Box::into_raw(Box::new(ListWatch {
        lw_index: c_int::try_from(at).expect("a short list"),
        lw_next: ptr::null_mut(),
    }));
    // SAFETY: the caller's list, and a watcher `unwatch` outlives.
    unsafe { (*l).watch_add(lw) };
    lw
}

/// Unregister every watcher of `lws` from `l` and give its `Box` back.
///
/// # Safety
/// Every entry of `lws` was registered with `l` by [`watch`] and has not
/// been unregistered yet.
unsafe fn unwatch(l: *mut List, lws: &[*mut ListWatch]) {
    for &lw in lws {
        // SAFETY: the caller's promise, and the `Box` [`watch`] leaked.
        unsafe { (*l).watch_remove(lw) };
        drop(unsafe { Box::from_raw(lw) });
    }
}

/// Where each watcher of `lws` is standing, as an index into the list it
/// watches — `None` once it has been pushed off the end, which is what
/// upstream spelled as a NULL `lw_item`.
///
/// A cursor is an index and not an item address because an address is a
/// borrow of the list's array: it does not survive the very edits these
/// cases make.
///
/// # Safety
/// Every entry of `lws` points at a live watcher.
unsafe fn standing(lws: &[*mut ListWatch]) -> Vec<Option<usize>> {
    lws.iter()
        // SAFETY: the caller's promise: live watchers.
        .map(|&lw| usize::try_from(unsafe { (*lw).lw_index }).ok())
        .collect()
}

// ---------------------------------------------------------------- item

/// `describe('item') describe('remove()') itp('works')`, spec line 125.
///
/// The spec named the item to remove by address and asserted the *next
/// item's* address back; both are indexes now, and the answer is "the index
/// of what followed it", which is the index just vacated — or `None` when
/// the item removed was the last one. Nothing is allocated or freed: the
/// items hold floats, and the array they live in is Rust's.
#[test]
fn removing_an_item_answers_the_index_that_followed_it() {
    let log = AllocLog::start();
    // SAFETY: the list is this case's own and is freed at the end.
    unsafe {
        let l = tv::new_list(&floats(1..=7));
        log.check(&[alloc::list(l)]);

        // From the front, from the back, and from the middle.
        assert_eq!((*l).remove_at(0), Some(0));
        assert_eq!(tv::read_list(l), Tv::List(floats(2..=7)));
        assert_eq!((*l).remove_at(5), None, "there was nothing after it");
        assert_eq!(tv::read_list(l), Tv::List(floats(2..=6)));
        assert_eq!((*l).remove_at(2), Some(2));
        assert_eq!(
            tv::read_list(l),
            Tv::List(vec![f(2.0), f(3.0), f(5.0), f(6.0)])
        );
        log.check(&[]);

        list_free(l);
        log.check(&[alloc::freed(l)]);
    }
}

/// The same `describe`'s `itp('also frees the value')`, spec line 158: the
/// item is gone, and so is what it held.
#[test]
fn removing_an_item_frees_its_value() {
    let log = AllocLog::start();
    // SAFETY: as above.
    unsafe {
        let l = tv::new_list(&["a", "b", "c", "d"].map(Tv::s));
        let mut strings: Vec<*mut c_char> = tv::list_items(l)
            .iter()
            .map(|&li| (*li).li_tv.string())
            .collect();

        let mut expected = vec![alloc::list(l)];
        expected.extend(strings.iter().map(|&s| alloc::string(s, 1)));
        log.check(&expected);

        let mut left = vec!["a", "b", "c", "d"];
        for at in [0, 1, 1] {
            (*l).remove_at(at);
            log.check(&[alloc::freed(strings.remove(at))]);
            left.remove(at);
            assert_eq!(tv::read_list(l), Tv::List(left.iter().map(Tv::s).collect()));
        }

        list_free(l);
        log.check(&[alloc::freed(strings.remove(0)), alloc::freed(l)]);
    }
}

/// The same `describe`'s `itp('works and adjusts watchers correctly')`,
/// spec line 198 — `List::watch_add` and the shift that follows a removal
/// are tested only here.
///
/// The spec compared `lw_item` pointers, which named the item a `:for` loop
/// is standing on. The identity is the same, but it is spelled as an index
/// now, so the case says which *value* each watcher ends up on: removing an
/// item before a watcher moves the watcher down with it, removing the item
/// it stands on lands it on whatever followed, and removing the last item
/// pushes it off the end for good.
#[test]
fn removing_an_item_moves_the_watchers_standing_on_it() {
    let log = AllocLog::start();
    // SAFETY: as above; the watchers are unregistered before the list goes.
    unsafe {
        let l = tv::new_list(&floats(1..=7));
        // Three watchers: on the first, the middle and the last item.
        let lws = [watch(l, 0), watch(l, 3), watch(l, 6)];
        log.check(&[alloc::list(l)]);

        // The watched middle item goes: its watcher lands on what followed
        // it, the item holding 5, and the last one moves down with 7.
        assert_eq!((*l).remove_at(3), Some(3));
        assert_eq!(
            tv::read_list(l),
            Tv::List(floats(1..=3).into_iter().chain(floats(5..=7)).collect())
        );
        assert_eq!(standing(&lws), [Some(0), Some(3), Some(5)]);

        // Removing an item nobody watches still moves the watchers after
        // it, because the items they name moved: 5 and 7 are one place
        // nearer the front.
        assert_eq!((*l).remove_at(1), Some(1));
        assert_eq!(
            tv::read_list(l),
            Tv::List(vec![f(1.0), f(3.0), f(5.0), f(6.0), f(7.0)])
        );
        assert_eq!(standing(&lws), [Some(0), Some(2), Some(4)]);

        // A watcher on the last item is pushed off the end.
        assert_eq!((*l).remove_at(4), None);
        assert_eq!(
            tv::read_list(l),
            Tv::List(vec![f(1.0), f(3.0), f(5.0), f(6.0)])
        );
        assert_eq!(standing(&lws), [Some(0), Some(2), None]);

        // And the first: its watcher lands on the item that followed, 3.
        assert_eq!((*l).remove_at(0), Some(0));
        assert_eq!(tv::read_list(l), Tv::List(vec![f(3.0), f(5.0), f(6.0)]));
        assert_eq!(standing(&lws), [Some(0), Some(1), None]);

        unwatch(l, &lws);
        // Floats cost nothing, so the list header is all there is to free.
        list_free(l);
        log.check(&[alloc::freed(l)]);
    }
}

// ---------------------------------------------------------------- watch

/// `describe('watch') describe('remove()') itp('works')`, spec line 256:
/// the watch list is a stack, and removing from it frees nothing.
#[test]
fn removing_a_watch_unlinks_it_without_freeing() {
    let log = AllocLog::start();
    // SAFETY: the list is this case's own.
    unsafe {
        let l = tv::new_list(&floats(1..=7));
        assert!((*l).lv_watch.is_null());
        let lw = watch(l, 0);
        assert!(!(*l).lv_watch.is_null());
        log.clear();

        unwatch(l, &[lw]);
        assert!((*l).lv_watch.is_null());
        log.check(&[]);

        let lws = [watch(l, 0), watch(l, 0), watch(l, 0)];
        log.clear();

        // The newest is at the head, so removing the middle one leaves the
        // third watching and the first behind it.
        (*l).watch_remove(lws[1]);
        assert_eq!((*l).lv_watch, lws[2]);
        assert_eq!((*(*l).lv_watch).lw_next, lws[0]);
        (*l).watch_remove(lws[0]);
        assert_eq!((*l).lv_watch, lws[2]);
        assert!((*(*l).lv_watch).lw_next.is_null());
        (*l).watch_remove(lws[2]);
        assert!((*l).lv_watch.is_null());
        log.check(&[]);

        for lw in lws {
            drop(Box::from_raw(lw));
        }
        list_free(l);
    }
}

/// The same `describe`'s `itp('ignores not found watchers')`, spec line 281.
#[test]
fn removing_an_unregistered_watch_is_a_no_op() {
    let log = AllocLog::start();
    // SAFETY: `lw` was never registered, so nothing links to it.
    unsafe {
        let l = tv::new_list(&floats(1..=7));
        let mut lw = ListWatch {
            lw_index: 0,
            lw_next: ptr::null_mut(),
        };
        log.clear();
        (*l).watch_remove(&raw mut lw);
        log.check(&[]);
        list_free(l);
    }
}

// ------------------------------------------------------- free / unref

/// The three lists the free cases are stated over: a list of two scalars,
/// a list holding a dict, and a list holding a list.
///
/// Each comes with the allocations it is made of, in the order a full free
/// gives them back: the values the items hold, then the list header. The
/// items themselves are not on the list — they are slots in the list's own
/// array, which is Rust's to allocate and give back.
///
/// # Safety
/// The editor must be up. The caller owns all three.
unsafe fn three_lists(log: &AllocLog) -> [(*mut List, Vec<*mut std::ffi::c_void>); 3] {
    let mut out = Vec::new();
    // SAFETY: the lists are the caller's.
    unsafe {
        let l1 = tv::new_list(&[f(1.0), Tv::s("abc")]);
        let s1 = (*list_last(l1.as_mut())).li_tv.string();
        log.check(&[alloc::list(l1), alloc::string(s1, "abc".len())]);
        out.push((l1, vec![s1.cast(), l1.cast()]));

        let l2 = tv::new_list(&[Tv::Dict(vec![])]);
        let d2 = (*list_first(l2.as_mut())).li_tv.dict();
        log.check(&[alloc::list(l2), alloc::dict(d2)]);
        out.push((l2, vec![d2.cast(), l2.cast()]));

        let l3 = tv::new_list(&[Tv::List(vec![])]);
        let inner = (*list_first(l3.as_mut())).li_tv.list();
        log.check(&[alloc::list(l3), alloc::list(inner)]);
        out.push((l3, vec![inner.cast(), l3.cast()]));
    }
    out.try_into()
        .unwrap_or_else(|_| unreachable!("three lists"))
}

/// `describe('free()') itp('recursively frees list')`, spec line 290.
#[test]
fn freeing_a_list_frees_its_contents_then_itself() {
    let log = AllocLog::start();
    // SAFETY: the three lists are this case's own and are freed here.
    unsafe {
        for (l, allocated) in three_lists(&log) {
            list_free(l);
            log.check(
                &allocated
                    .iter()
                    .map(|&p| alloc::freed(p))
                    .collect::<Vec<_>>(),
            );
        }
    }
}

/// `describe('free_list()') itp('does not free list contents')`, spec line
/// 329 — and the one case whose answer the item store changed.
///
/// Upstream's `list_free_list` freed the header and left the items
/// linked off it, which the spec asserted as a deliberate leak. There is no
/// such half-state now: the items *are* the header's array, so giving the
/// header back gives them back too, and the values they hold are released
/// with them. What the pair still means is the split the garbage collector
/// uses — `list_free_contents` over every reachable list first, then
/// `list_free_list` over each — and the case pins that the second half
/// on its own is a complete free, so nothing is leaked and nothing is
/// freed twice.
#[test]
fn freeing_only_the_list_frees_the_items_with_it() {
    let log = AllocLog::start();
    // SAFETY: the lists are this case's own and go here.
    unsafe {
        for (l, allocated) in three_lists(&log) {
            list_free_list(l);
            log.check(
                &allocated
                    .iter()
                    .map(|&p| alloc::freed(p))
                    .collect::<Vec<_>>(),
            );
        }
    }
}

/// `describe('free_contents()')
/// itp('recursively frees list, except for the list structure itself')`,
/// spec line 361.
#[test]
fn freeing_only_the_contents_leaves_the_list() {
    let log = AllocLog::start();
    // SAFETY: the emptied lists are freed here.
    unsafe {
        for (l, allocated) in three_lists(&log) {
            list_free_contents(&mut *l);
            log.check(
                &allocated[..allocated.len() - 1]
                    .iter()
                    .map(|&p| alloc::freed(p))
                    .collect::<Vec<_>>(),
            );
            assert_eq!(
                list_len(l.as_ref()),
                0,
                "the list is still there, and empty"
            );
            list_free_list(l);
            log.check(&[alloc::freed(*allocated.last().expect("the list itself"))]);
        }
    }
}

/// `describe('unref()')
/// itp('recursively frees list when reference count goes to 0')`, spec line
/// 397.
#[test]
fn unref_frees_only_at_the_last_reference() {
    let log = AllocLog::start();
    // SAFETY: the list is this case's own and the second unref takes it.
    unsafe {
        let l = tv::new_list(&[Tv::List(vec![])]);
        let inner = (*list_first(l.as_mut())).li_tv.list();
        log.check(&[alloc::list(l), alloc::list(inner)]);

        (*l).lv_refcount = Refcount::new(2);
        list_unref(l);
        log.check(&[]);
        list_unref(l);
        log.check(&[alloc::freed(inner), alloc::freed(l)]);
    }
}

// ------------------------------------------------------- move / remove

/// `describe('drop_items()') itp('works')`, spec line 417, over its
/// successor.
///
/// `tv_list_drop_items` unlinked a run of items and handed the caller the
/// chain, which only made sense while an item was its own allocation. What
/// took its place is [`List::move_range_to`], which drains the run straight
/// onto another list's tail — `remove(l, first, last)` is the one caller
/// either ever had. So the case runs the spec's four-step walk against a
/// target list: the same runs leave the source, the same watchers move the
/// same way, and — the point `drop_items` was making — **nothing is freed**,
/// because the values moved rather than being released.
#[test]
fn moving_a_run_of_items_takes_them_without_freeing_and_moves_the_watchers() {
    let log = AllocLog::start();
    // SAFETY: both lists are this case's own and are released at the end.
    unsafe {
        let mut l_tv = Tv::List(floats(1..=13)).build();
        let l = l_tv.list();
        let tgt = tv::new_list(&[]);
        let lws = [watch(l, 0), watch(l, 6), watch(l, 12)];
        log.clear();

        // The run 1..3 off the front. The watcher standing inside it lands
        // on what followed, and the two after it move down three places.
        (*l).move_range_to(0, 2, &mut *tgt);
        assert_eq!(tv::read(&raw const l_tv), Tv::List(floats(4..=13)));
        assert_eq!(tv::read_list(tgt), Tv::List(floats(1..=3)));
        assert_eq!(standing(&lws), [Some(0), Some(3), Some(9)]);

        // The run 11..13 off the back: the watcher on 13 goes with it, and
        // a cursor past the last item is ended.
        (*l).move_range_to(7, 9, &mut *tgt);
        assert_eq!(tv::read(&raw const l_tv), Tv::List(floats(4..=10)));
        assert_eq!(
            tv::read_list(tgt),
            Tv::List(floats(1..=3).into_iter().chain(floats(11..=13)).collect())
        );
        assert_eq!(standing(&lws), [Some(0), Some(3), None]);

        // A run out of the middle, 6..8, which the second watcher is inside.
        (*l).move_range_to(2, 4, &mut *tgt);
        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(vec![f(4.0), f(5.0), f(9.0), f(10.0)])
        );
        assert_eq!(standing(&lws), [Some(0), Some(2), None]);

        // And the rest, which ends every cursor.
        (*l).move_range_to(0, 3, &mut *tgt);
        assert_eq!(tv::read(&raw const l_tv), Tv::List(vec![]));
        assert_eq!(standing(&lws), [None, None, None]);
        assert_eq!(
            tv::read_list(tgt),
            Tv::List(
                floats(1..=3)
                    .into_iter()
                    .chain(floats(11..=13))
                    .chain(floats(6..=8))
                    .chain([f(4.0), f(5.0), f(9.0), f(10.0)])
                    .collect()
            )
        );

        unwatch(l, &lws);
        // Thirteen items changed hands and not one allocation moved.
        log.check(&[]);

        list_free(tgt);
        log.check(&[alloc::freed(tgt)]);
        tv_clear(&mut l_tv);
    }
}

/// `describe('remove_items()') itp('works')`, spec line 462: the same walk,
/// but the items and their values are released.
///
/// The run is named by its two indexes rather than by the addresses of its
/// first and last items; the freed values are what is left of the spec's
/// expectations, and they still come back front to back.
#[test]
fn removing_a_run_of_items_frees_their_values() {
    let log = AllocLog::start();
    // SAFETY: the list is this case's own and is cleared at the end.
    unsafe {
        let strings: Vec<Tv> = (1..=13).map(|n| Tv::s(n.to_string())).collect();
        let mut l_tv = Tv::List(strings).build();
        let l = l_tv.list();
        let values: Vec<*mut c_char> = tv::list_items(l)
            .iter()
            .map(|&li| (*li).li_tv.string())
            .collect();
        let lws = [watch(l, 0), watch(l, 6), watch(l, 12)];
        log.clear();

        let text = |ns: &[i32]| Tv::List(ns.iter().map(|n| Tv::s(n.to_string())).collect());
        let freed = |which: &[usize]| -> Vec<_> {
            which.iter().map(|&i| alloc::freed(values[i])).collect()
        };

        (*l).remove_range(0, 2);
        assert_eq!(
            tv::read(&raw const l_tv),
            text(&[4, 5, 6, 7, 8, 9, 10, 11, 12, 13])
        );
        assert_eq!(standing(&lws), [Some(0), Some(3), Some(9)]);
        log.check(&freed(&[0, 1, 2]));

        (*l).remove_range(7, 9);
        assert_eq!(tv::read(&raw const l_tv), text(&[4, 5, 6, 7, 8, 9, 10]));
        assert_eq!(standing(&lws), [Some(0), Some(3), None]);
        log.check(&freed(&[10, 11, 12]));

        (*l).remove_range(2, 4);
        assert_eq!(tv::read(&raw const l_tv), text(&[4, 5, 9, 10]));
        assert_eq!(standing(&lws), [Some(0), Some(2), None]);
        log.check(&freed(&[5, 6, 7]));

        (*l).remove_range(0, 3);
        assert_eq!(tv::read(&raw const l_tv), Tv::List(vec![]));
        assert_eq!(standing(&lws), [None, None, None]);
        log.check(&freed(&[3, 4, 8, 9]));

        unwatch(l, &lws);
        log.check(&[]);
        tv_clear(&mut l_tv);
    }
}

// -------------------------------------------------------------- insert

/// `describe('insert') describe('()') itp('works')`, spec line 546.
///
/// `tv_list_insert` took an item the caller had allocated and spliced it in
/// front of another item it named by address; both halves are gone. What
/// the case is still about is *where the value lands*, so it inserts
/// through [`List::insert_copy`] and names the position by index: `None` is
/// the tail, `Some(0)` the front, and `Some(n)` in front of whatever is at
/// `n` **now** — which is the whole difference, because an index moves when
/// the items in front of it do.
#[test]
fn inserting_a_value_puts_it_in_front_of_the_index_named() {
    let log = AllocLog::start();
    // SAFETY: the list owns every value handed to it.
    unsafe {
        let mut l_tv = Tv::List(floats(1..=7)).build();
        let l = l_tv.list();
        log.clear();

        // A `None` "before" appends.
        (*l).insert_copy(&TypVal::Float(100500.0), None);
        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(floats(1..=7).into_iter().chain([f(100500.0)]).collect())
        );

        (*l).insert_copy(&TypVal::Float(0.0), Some(0));
        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(
                [f(0.0)]
                    .into_iter()
                    .chain(floats(1..=7))
                    .chain([f(100500.0)])
                    .collect()
            )
        );

        // The spec named the item holding 5, which the insert at the front
        // has moved to index 5.
        (*l).insert_copy(&TypVal::Float(4.5), Some(5));
        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(vec![
                f(0.0),
                f(1.0),
                f(2.0),
                f(3.0),
                f(4.0),
                f(4.5),
                f(5.0),
                f(6.0),
                f(7.0),
                f(100500.0),
            ])
        );

        // Ten items, and not one allocation: the slots are the list's own
        // array, which Rust grows.
        log.check(&[]);
        tv_clear(&mut l_tv);
        log.check(&[alloc::freed(l)]);
    }
}

/// The same `describe`'s `itp('works with an empty list')`, spec line 570.
#[test]
fn inserting_into_an_empty_list_makes_it_the_only_item() {
    let log = AllocLog::start();
    // SAFETY: as above.
    unsafe {
        let mut l_tv = Tv::List(vec![]).build();
        let l = l_tv.list();
        assert_eq!(list_len(l.as_ref()), 0);
        assert!(list_first(l.as_mut()).is_null());
        assert!(list_last(l.as_mut()).is_null());

        (*l).insert_copy(&TypVal::Float(100500.0), None);
        assert_eq!(
            list_first(l.as_mut()),
            list_last(l.as_mut()),
            "the only item"
        );
        assert_eq!(tv::read(&raw const l_tv), Tv::List(vec![f(100500.0)]));

        log.clear();
        tv_clear(&mut l_tv);
    }
}

/// `describe('insert') describe('tv()') itp('works')`, spec line 585: the
/// value is *copied* in, so a container gains a reference and a string is
/// duplicated.
///
/// With the item allocation gone, the copy is the only thing left in the
/// log — which is exactly what the case was about.
#[test]
fn inserting_a_value_copies_it() {
    let log = AllocLog::start();
    // SAFETY: each `TypVal` here is this case's own and is cleared.
    unsafe {
        let mut l_tv = Tv::List(vec![]).build();
        let l = l_tv.list();

        let mut inner_tv = Tv::List(vec![]).build();
        log.clear();
        let inner = inner_tv.list();
        assert_eq!((*inner).lv_refcount.get(), 1);
        (*l).insert_copy(&inner_tv, None);
        assert_eq!((*inner).lv_refcount.get(), 2, "the copy holds a reference");
        assert_eq!((*list_first(l.as_mut())).li_tv.list(), inner);
        log.check(&[]);

        let mut s_tv = Tv::s("test").build();
        log.check(&[alloc::string(s_tv.string(), "test".len())]);
        (*l).insert_copy(&s_tv, Some(0));
        log.check(&[alloc::string(
            (*list_first(l.as_mut())).li_tv.string(),
            "test".len(),
        )]);
        assert_ne!(
            (*list_first(l.as_mut())).li_tv.string(),
            s_tv.string(),
            "a copy, not the caller's string"
        );

        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(vec![Tv::s("test"), Tv::List(vec![])])
        );

        tv_clear(&mut l_tv);
        tv_clear(&mut inner_tv);
        tv_clear(&mut s_tv);
    }
}

// -------------------------------------------------------------- append

/// `describe('append') describe('list()') itp('works')`, spec line 616.
///
/// The spec's assertion was `a.li(l.lv_last)` — one item allocation per
/// append. There is none now, so what the case pins is the reference: the
/// appended list gains one, and a NULL list costs nothing at all.
#[test]
fn appending_a_list_takes_a_reference() {
    let log = AllocLog::start();
    // SAFETY: the outer list owns the items; the inner list is unref'd by
    // clearing the outer one.
    unsafe {
        let mut l_tv = Tv::List(vec![]).build();
        let l = l_tv.list();

        let inner = tv::new_list(&[f(1.0)]);
        log.clear();
        assert_eq!((*inner).lv_refcount.get(), 1);
        (*l).push_list(ListRef::retained(inner));
        assert_eq!((*inner).lv_refcount.get(), 2);
        assert_eq!((*list_first(l.as_mut())).li_tv.list(), inner);
        log.check(&[]);

        (*l).push_list(None);
        log.check(&[]);

        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(vec![Tv::List(vec![f(1.0)]), Tv::NullList])
        );

        tv_clear(&mut l_tv);
        assert_eq!((*inner).lv_refcount.get(), 1, "the list gave its back");
        list_unref(inner);
    }
}

/// The same `describe`'s `dict()`, spec line 639: as above, over a dict.
#[test]
fn appending_a_dict_takes_a_reference() {
    let log = AllocLog::start();
    // SAFETY: as above.
    unsafe {
        let mut l_tv = Tv::List(vec![]).build();
        let l = l_tv.list();

        let mut d_tv = Tv::dict([("test", f(1.0))]).build();
        let d = d_tv.dict();
        log.clear();
        assert_eq!((*d).dv_refcount.get(), 1);
        (*l).push_dict(DictRef::retained(d));
        assert_eq!((*d).dv_refcount.get(), 2);
        assert_eq!((*list_first(l.as_mut())).li_tv.dict(), d);
        log.check(&[]);

        (*l).push_dict(None);
        log.check(&[]);

        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(vec![Tv::dict([("test", f(1.0))]), Tv::NullDict])
        );

        tv_clear(&mut l_tv);
        assert_eq!((*d).dv_refcount.get(), 1, "the list gave its back");
        tv_clear(&mut d_tv);
    }
}

/// The same `describe`'s `string()`, spec line 663.
///
/// The spec's assertion was an *order*: the string is copied before the
/// item that will hold it is allocated. Only the copy is left, so the case
/// says what the copy is — the string's length plus its terminator, which
/// a negative length reads off the terminator itself — and that a NULL
/// string is copied by not copying anything.
#[test]
fn appending_a_string_copies_it() {
    let log = AllocLog::start();
    // SAFETY: the list owns everything appended to it.
    unsafe {
        let mut l_tv = Tv::List(vec![]).build();
        let l = l_tv.list();
        log.clear();

        let test = cstr("test");
        (*l).push_string(test.as_ptr(), 3);
        log.check(&[alloc::string((*list_last(l.as_mut())).li_tv.string(), 3)]);

        // A NULL string allocates nothing at all, at either length.
        (*l).push_string(ptr::null(), 0);
        log.check(&[]);
        (*l).push_string(ptr::null(), -1);
        log.check(&[]);

        (*l).push_string(test.as_ptr(), -1);
        log.check(&[alloc::string((*list_last(l.as_mut())).li_tv.string(), 4)]);

        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(vec![Tv::s("tes"), Tv::NullStr, Tv::NullStr, Tv::s("test")])
        );

        tv_clear(&mut l_tv);
    }
}

/// The same `describe`'s `allocated string()`, spec line 694: ownership is
/// transferred, so nothing is allocated at all.
///
/// The spec said that as "nothing but the item"; with no item to allocate,
/// the case says it as the identity of the pointer — the list holds the
/// caller's own allocation, and gives that same one back when it is
/// cleared.
#[test]
fn appending_an_allocated_string_takes_ownership() {
    let log = AllocLog::start();
    // SAFETY: `s` is handed to the list, which frees it.
    unsafe {
        let mut l_tv = Tv::List(vec![]).build();
        let l = l_tv.list();

        let s = xstrdup(cstr("test").as_ptr());
        log.clear();
        (*l).push_allocated_string(s);
        log.check(&[]);
        assert_eq!(
            (*list_last(l.as_mut())).li_tv.string(),
            s,
            "the caller's allocation itself, not a copy"
        );

        (*l).push_allocated_string(ptr::null_mut());
        log.check(&[]);
        (*l).push_allocated_string(ptr::null_mut());
        log.check(&[]);

        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(vec![Tv::s("test"), Tv::NullStr, Tv::NullStr])
        );

        tv_clear(&mut l_tv);
        // The two NULL strings are released too: `xfree(NULL)` is a call,
        // and the log records it.
        log.check(&[
            alloc::freed(s),
            alloc::freed(ptr::null::<c_char>()),
            alloc::freed(ptr::null::<c_char>()),
            alloc::freed(l),
        ]);
    }
}

/// The same `describe`'s `number()`, spec line 719.
///
/// The spec's point was that a number costs *only* the item. It costs
/// nothing now, so the case pins the two values instead — and that the
/// whole list is one allocation, given back in one.
#[test]
fn appending_a_number_allocates_nothing() {
    let log = AllocLog::start();
    // SAFETY: as above.
    unsafe {
        let mut l_tv = Tv::List(vec![]).build();
        let l = l_tv.list();
        log.clear();

        (*l).push_number(-100500);
        log.check(&[]);
        (*l).push_number(100500);
        log.check(&[]);

        assert_eq!((*list_first(l.as_mut())).li_tv.number(), -100500);
        assert_eq!((*list_last(l.as_mut())).li_tv.number(), 100500);
        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(vec![Tv::Int(-100500), Tv::Int(100500)])
        );

        tv_clear(&mut l_tv);
        log.check(&[alloc::freed(l)]);
    }
}

/// The same `describe`'s `tv()`, spec line 738: a copy, like `insert_tv`.
#[test]
fn appending_a_value_copies_it() {
    let log = AllocLog::start();
    // SAFETY: each value is this case's own and is cleared.
    unsafe {
        let mut l_tv = Tv::List(vec![]).build();
        let l = l_tv.list();

        let mut inner_tv = Tv::List(vec![]).build();
        log.clear();
        let inner = inner_tv.list();
        assert_eq!((*inner).lv_refcount.get(), 1);
        (*l).push_copy(&inner_tv);
        assert_eq!((*inner).lv_refcount.get(), 2);
        assert_eq!((*list_first(l.as_mut())).li_tv.list(), inner);
        log.check(&[]);

        let mut s_tv = Tv::s("test").build();
        log.check(&[alloc::string(s_tv.string(), "test".len())]);
        (*l).push_copy(&s_tv);
        log.check(&[alloc::string(
            (*list_last(l.as_mut())).li_tv.string(),
            "test".len(),
        )]);
        assert_ne!(
            (*list_last(l.as_mut())).li_tv.string(),
            s_tv.string(),
            "a copy, not the caller's string"
        );

        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(vec![Tv::List(vec![]), Tv::s("test")])
        );

        tv_clear(&mut l_tv);
        tv_clear(&mut inner_tv);
        tv_clear(&mut s_tv);
    }
}

/// The same `describe`'s `owned tv()`, spec line 767: the value is *moved*,
/// so the reference count does not rise and the string is not duplicated.
#[test]
fn appending_an_owned_value_moves_it() {
    let log = AllocLog::start();
    // SAFETY: the list takes both values; the caller must not clear them.
    unsafe {
        let mut l_tv = Tv::List(vec![]).build();
        let l = l_tv.list();

        let inner_tv = Tv::List(vec![]).build();
        log.clear();
        let inner = inner_tv.list();
        assert_eq!((*inner).lv_refcount.get(), 1);
        (*l).push(inner_tv);
        assert_eq!(
            (*inner).lv_refcount.get(),
            1,
            "the reference moved, not copied"
        );
        assert_eq!((*list_first(l.as_mut())).li_tv.list(), inner);
        log.check(&[]);

        let s_tv = Tv::s("test").build();
        let s_ptr = s_tv.string();
        log.check(&[alloc::string(s_ptr, "test".len())]);
        (*l).push(s_tv);
        assert_eq!(
            (*list_last(l.as_mut())).li_tv.string(),
            s_ptr,
            "the string itself moved in"
        );
        log.check(&[]);

        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(vec![Tv::List(vec![]), Tv::s("test")])
        );

        tv_clear(&mut l_tv);
    }
}

// ---------------------------------------------------------------- copy

/// `describe('copy()') itp('copies NULL correctly')`, spec line 802.
#[test]
fn copying_a_null_list_answers_null() {
    let _log = AllocLog::start();
    // SAFETY: no list is dereferenced.
    unsafe {
        for deep in [true, false] {
            for copy_id in [0, 1] {
                assert!(
                    copied(ptr::null_mut(), ptr::null_mut(), deep, copy_id).is_null(),
                    "deep {deep} copyID {copy_id}"
                );
            }
        }
    }
}

/// The corpus the two `copy()` cases walk: a dict, a list, a number, a
/// string and the three NULL containers.
fn copy_corpus() -> Tv {
    Tv::List(vec![
        Tv::dict([("«", Tv::s("»"))]),
        Tv::List(vec![Tv::s("„")]),
        f(1.0),
        Tv::s("“"),
        Tv::NullStr,
        Tv::NullList,
        Tv::NullDict,
    ])
}

/// `itp('copies list correctly without converting items')`, spec line 808:
/// a shallow copy shares its containers, a deep one rebuilds them.
///
/// The seven item allocations the spec interleaved through each sequence
/// are gone; what is left is the copy's own header and, in item order, the
/// values a deep copy had to rebuild — which is the statement the case was
/// making underneath the items.
#[test]
fn copying_a_list_shares_or_rebuilds_its_containers() {
    let log = AllocLog::start();
    // SAFETY: every list here is this case's own and is freed.
    unsafe {
        let mut l_tv = copy_corpus().build();
        let l = l_tv.list();
        let lis = tv::list_items(l);
        let inner_dict = (*lis[0]).li_tv.dict();
        let inner_list = (*lis[1]).li_tv.list();
        log.clear();

        assert_eq!((*inner_dict).dv_refcount.get(), 1);
        assert_eq!((*inner_list).lv_refcount.get(), 1);
        let shallow = copied(ptr::null_mut(), l, false, 0);
        assert_eq!((*inner_dict).dv_refcount.get(), 2);
        assert_eq!((*inner_list).lv_refcount.get(), 2);
        let copies = tv::list_items(shallow);
        assert_eq!((*copies[0]).li_tv.dict(), inner_dict);
        assert_eq!((*copies[1]).li_tv.list(), inner_list);
        assert_eq!(tv::read_list(shallow), copy_corpus());
        // A shallow copy rebuilds one thing: the string, which is not
        // reference counted and so cannot be shared.
        log.check(&[
            alloc::list(shallow),
            alloc::string((*copies[3]).li_tv.string(), "“".len()),
        ]);
        list_free(shallow);
        log.clear();

        assert_eq!((*inner_dict).dv_refcount.get(), 1);
        assert_eq!((*inner_list).lv_refcount.get(), 1);
        let deep = copied(ptr::null_mut(), l, true, 0);
        assert!(!deep.is_null());
        assert_eq!(
            (*inner_dict).dv_refcount.get(),
            1,
            "a deep copy shares nothing"
        );
        assert_eq!((*inner_list).lv_refcount.get(), 1);
        let copies = tv::list_items(deep);
        assert_ne!((*copies[0]).li_tv.dict(), inner_dict);
        assert_ne!((*copies[1]).li_tv.list(), inner_list);
        assert_eq!(tv::read_list(deep), copy_corpus());

        let copied_dict = (*copies[0]).li_tv.dict();
        let di = tv::first_di(copied_dict);
        let copied_list = (*copies[1]).li_tv.list();
        log.check(&[
            alloc::list(deep),
            alloc::dict(copied_dict),
            alloc::string((*di).di_tv.string(), "»".len()),
            alloc::list(copied_list),
            alloc::string(
                (*list_first(copied_list.as_mut())).li_tv.string(),
                "„".len(),
            ),
            alloc::string((*copies[3]).li_tv.string(), "“".len()),
        ]);

        list_free(deep);
        tv_clear(&mut l_tv);
    }
}

/// `itp('copies list correctly and converts items')`, spec line 870: the
/// same walk through a `VimConv`, which rewrites every string it copies.
///
/// The allocation for a converted string is the *source* length plus one,
/// not the answer's — which is why the sizes here look too big.
#[test]
fn a_converting_copy_rewrites_every_string() {
    let log = AllocLog::start();
    // SAFETY: the converter and every list are this case's own.
    unsafe {
        let mut vc: VimConv = std::mem::zeroed();
        assert_eq!(
            convert_setup(
                &raw mut vc,
                cstr("utf-8").as_ptr().cast_mut(),
                cstr("latin1").as_ptr().cast_mut(),
            ),
            Ok(())
        );

        let mut l_tv = copy_corpus().build();
        let l = l_tv.list();
        let lis = tv::list_items(l);
        let inner_dict = (*lis[0]).li_tv.dict();
        let inner_list = (*lis[1]).li_tv.list();
        log.clear();

        let deep = copied(&raw mut vc, l, true, 0);
        assert!(!deep.is_null());
        assert_eq!((*inner_dict).dv_refcount.get(), 1);
        assert_eq!((*inner_list).lv_refcount.get(), 1);
        let copies = tv::list_items(deep);
        assert_ne!((*copies[0]).li_tv.dict(), inner_dict);
        assert_ne!((*copies[1]).li_tv.list(), inner_list);
        assert_eq!(
            tv::read_list(deep),
            Tv::List(vec![
                Tv::Dict(vec![(vec![0xAB], Tv::Str(vec![0xBB]))]),
                Tv::List(vec![Tv::Str(vec![0xBF])]),
                f(1.0),
                Tv::Str(vec![0xBF]),
                Tv::NullStr,
                Tv::NullList,
                Tv::NullDict,
            ])
        );

        let copied_dict = (*copies[0]).li_tv.dict();
        let di = tv::first_di(copied_dict);
        let copied_list = (*copies[1]).li_tv.list();
        log.check_net(
            false,
            &[
                alloc::list(deep),
                alloc::dict(copied_dict),
                alloc::string((*di).di_tv.string(), "»".len()),
                alloc::list(copied_list),
                alloc::string(
                    (*list_first(copied_list.as_mut())).li_tv.string(),
                    "„".len(),
                ),
                alloc::string((*copies[3]).li_tv.string(), "“".len()),
            ],
        );

        list_free(deep);
        tv_clear(&mut l_tv);
        let _ = convert_setup(&raw mut vc, ptr::null_mut(), ptr::null_mut());
    }
}

/// `itp('returns different/same containers with(out) copyID')`, spec line
/// 914: a copyID makes a deep copy keep the sharing the original had.
#[test]
fn a_copy_id_preserves_sharing() {
    let _log = AllocLog::start();
    // SAFETY: every list here is this case's own.
    unsafe {
        let mut inner_tv = Tv::List(vec![]).build();
        let mut l_tv = Tv::List(vec![
            Tv::Copied(&raw const inner_tv),
            Tv::Copied(&raw const inner_tv),
        ])
        .build();
        let inner = inner_tv.list();
        assert_eq!((*inner).lv_refcount.get(), 3);
        let l = l_tv.list();
        assert_eq!(
            (*list_first(l.as_mut())).li_tv.list(),
            (*list_last(l.as_mut())).li_tv.list()
        );

        let without = copied(ptr::null_mut(), l, true, 0);
        assert_ne!(
            (*list_first(without.as_mut())).li_tv.list(),
            (*list_last(without.as_mut())).li_tv.list()
        );
        assert_eq!(
            tv::read_list(without),
            Tv::List(vec![Tv::List(vec![]), Tv::List(vec![])])
        );

        let with = copied(ptr::null_mut(), l, true, 2);
        assert_eq!(
            (*list_first(with.as_mut())).li_tv.list(),
            (*list_last(with.as_mut())).li_tv.list()
        );
        // The two items are the same container, which the read spells as a
        // cycle back to it.
        assert_eq!(
            tv::read_list(with),
            Tv::List(vec![Tv::List(vec![]), Tv::List(vec![])])
        );

        assert_eq!((*inner).lv_refcount.get(), 3);
        list_unref(without);
        list_unref(with);
        tv_clear(&mut l_tv);
        tv_clear(&mut inner_tv);
    }
}

/// `itp('works with self-referencing list with copyID')`, spec line 931.
#[test]
fn a_self_referencing_list_copies_into_a_self_referencing_copy() {
    let _log = AllocLog::start();
    // SAFETY: the cycle is broken before either list is released.
    unsafe {
        let mut l_tv = Tv::List(vec![]).build();
        let l = l_tv.list();
        assert_eq!((*l).lv_refcount.get(), 1);
        (*l).push_list(ListRef::retained(l));
        assert_eq!((*l).lv_refcount.get(), 2);

        let copy = copied(ptr::null_mut(), l, true, 2);
        assert_eq!((*copy).lv_refcount.get(), 2, "the copy holds itself");
        assert_eq!(tv::read_list(copy), Tv::List(vec![Tv::Cycle(0)]));

        // Break both cycles so the lists can go.
        (*l).remove_at(0);
        assert_eq!((*l).lv_refcount.get(), 1);
        (*copy).remove_at(0);
        assert_eq!((*copy).lv_refcount.get(), 1);

        list_unref(copy);
        tv_clear(&mut l_tv);
    }
}

// -------------------------------------------------------------- extend

/// `describe('extend()') itp('can extend list with itself')`, spec line 954.
///
/// `bef` is an index now rather than the address of the item to insert in
/// front of, and this is the one case where that is not a rename: the
/// insertion point walks along with the items already put in, and so does
/// the *source* index, because the list being read is the list being
/// written. The three rows are the tail, in front of the last item, and in
/// front of the first.
#[test]
fn a_list_can_be_extended_with_itself() {
    let log = AllocLog::start();
    // SAFETY: each list is this case's own and is freed.
    unsafe {
        for (bef, expected) in [
            (None, vec![f(1.0), DICT, f(1.0), DICT]),
            (Some(1), vec![f(1.0), f(1.0), DICT, DICT]),
            (Some(0), vec![f(1.0), DICT, f(1.0), DICT]),
        ] {
            let l = tv::new_list(&[f(1.0), Tv::Dict(vec![])]);
            log.clear();
            let d = (*list_last(l.as_mut())).li_tv.dict();
            assert_eq!((*l).lv_refcount.get(), 1);
            assert_eq!((*d).dv_refcount.get(), 1);

            list_extend(l, l, bef);

            // A float and a reference to a dict: neither allocates.
            log.check(&[]);
            assert_eq!((*l).lv_refcount.get(), 1);
            assert_eq!((*d).dv_refcount.get(), 2, "the dict gained one reference");
            assert_eq!(tv::read_list(l), Tv::List(expected), "bef {bef:?}");

            list_free(l);
            log.check(&[alloc::freed(d), alloc::freed(l)]);
        }
    }
}

/// A dict placeholder for the `extend` expectations, which never look
/// inside it.
const DICT: Tv = Tv::Dict(Vec::new());

/// The same `describe`'s `itp('can extend list with an empty list')`, spec
/// line 999: nothing is allocated and nothing changes.
#[test]
fn extending_with_an_empty_list_does_nothing() {
    let log = AllocLog::start();
    // SAFETY: both lists are this case's own.
    unsafe {
        let l = tv::new_list(&[f(1.0), Tv::Dict(vec![])]);
        let empty = tv::new_list(&[]);
        log.clear();
        let d = (*list_last(l.as_mut())).li_tv.dict();

        for bef in [None, Some(0), Some(1)] {
            list_extend(l, empty, bef);
            log.check(&[]);
            assert_eq!((*l).lv_refcount.get(), 1);
            assert_eq!((*d).dv_refcount.get(), 1);
            assert_eq!((*empty).lv_refcount.get(), 1);
            assert_eq!(tv::read_list(l), Tv::List(vec![f(1.0), DICT]));
        }

        list_free(l);
        list_free(empty);
    }
}

/// `extend(l, v:_null_list)`: the NULL list is the empty one, and extends
/// nothing.
///
/// Not a spec case — `null_spec.lua` is where the behaviour is written
/// down, and the reason it is here as well is that the pointer form read
/// the source through `tv_list_items`, which answers the empty slice for a
/// NULL list, where the borrowed form has to say so itself.
#[test]
fn extending_with_a_null_list_does_nothing() {
    let log = AllocLog::start();
    // SAFETY: the list is this case's own, and NULL is a list argument.
    unsafe {
        let l = tv::new_list(&[f(1.0), f(2.0)]);
        log.clear();

        for bef in [None, Some(0), Some(1)] {
            list_extend(l, ptr::null(), bef);
            log.check(&[]);
            assert_eq!((*l).lv_refcount.get(), 1);
            assert_eq!(tv::read_list(l), Tv::List(vec![f(1.0), f(2.0)]));
        }

        list_free(l);
    }
}

/// The same `describe`'s `itp('can extend list with another non-empty
/// list')`, spec line 1028.
///
/// The two copied items cost nothing to allocate — a float and a reference
/// — so what the case says is where they land and what the reference count
/// does: the source list itself is not held, the list *inside* it is, and
/// clearing the extended list gives that reference back.
#[test]
fn extending_with_another_list_copies_its_items() {
    let log = AllocLog::start();
    // SAFETY: as above.
    unsafe {
        let l2 = tv::new_list(&[f(42.0), Tv::List(vec![])]);
        let inner = (*list_last(l2.as_mut())).li_tv.list();
        assert_eq!((*l2).lv_refcount.get(), 1);
        assert_eq!((*inner).lv_refcount.get(), 1);

        for (bef, expected) in [
            (None, vec![f(1.0), DICT, f(42.0), LIST]),
            (Some(0), vec![f(42.0), LIST, f(1.0), DICT]),
            (Some(1), vec![f(1.0), f(42.0), LIST, DICT]),
        ] {
            let l = tv::new_list(&[f(1.0), Tv::Dict(vec![])]);
            log.clear();
            let d = (*list_last(l.as_mut())).li_tv.dict();
            assert_eq!((*l).lv_refcount.get(), 1);
            assert_eq!((*d).dv_refcount.get(), 1);

            list_extend(l, l2, bef);

            log.check(&[]);
            assert_eq!(
                (*l2).lv_refcount.get(),
                1,
                "the source list itself is not held"
            );
            assert_eq!((*inner).lv_refcount.get(), 2, "but its list item is");
            assert_eq!(tv::read_list(l), Tv::List(expected), "bef {bef:?}");

            list_free(l);
            assert_eq!((*inner).lv_refcount.get(), 1);
        }

        list_free(l2);
    }
}

/// A list placeholder, as [`DICT`].
const LIST: Tv = Tv::List(Vec::new());

// -------------------------------------------------------------- concat

/// `describe('concat()') itp('works with NULL lists')`, spec line 1084: a
/// NULL operand is the empty list, and two NULLs answer a NULL list.
///
/// The answer's own header is the only allocation a concatenation makes:
/// the items are slots in it, and the values are shared by reference.
#[test]
fn concatenating_with_a_null_list_copies_the_other_one() {
    let log = AllocLog::start();
    // SAFETY: every value here is this case's own.
    unsafe {
        let l = tv::new_list(&[f(1.0), Tv::Dict(vec![])]);
        log.clear();
        let d = (*list_last(l.as_mut())).li_tv.dict();
        assert_eq!((*l).lv_refcount.get(), 1);
        assert_eq!((*d).dv_refcount.get(), 1);

        let mut refs = 1;
        let mut results = Vec::new();
        for (l1, l2) in [(ptr::null_mut(), l), (l, ptr::null_mut())] {
            let mut rettv = Tv::Unknown.build();
            assert_eq!(list_concat(l1, l2, &mut rettv), Ok(()));
            assert_eq!((*l).lv_refcount.get(), 1);
            assert_eq!(rettv.v_type(), VAR_LIST);
            assert_eq!(tv::read(&raw const rettv), Tv::List(vec![f(1.0), DICT]));
            let out = rettv.list();
            assert_eq!((*out).lv_refcount.get(), 1);
            log.check(&[alloc::list(out)]);
            refs += 1;
            assert_eq!((*d).dv_refcount.get(), refs);
            results.push(rettv);
        }

        let mut rettv = Tv::Unknown.build();
        assert_eq!(
            list_concat(ptr::null_mut(), ptr::null_mut(), &mut rettv),
            Ok(())
        );
        assert_eq!(rettv.v_type(), VAR_LIST);
        assert_eq!(tv::read(&raw const rettv), Tv::NullList);
        log.check(&[]);

        for mut rettv in results {
            tv_clear(&mut rettv);
        }
        list_free(l);
    }
}

/// The same `describe`'s `itp('works with two different lists')`, spec line
/// 1122.
#[test]
fn concatenating_two_lists_copies_both() {
    let log = AllocLog::start();
    // SAFETY: as above.
    unsafe {
        let l1 = tv::new_list(&[f(1.0), Tv::Dict(vec![])]);
        let l2 = tv::new_list(&[f(3.0), Tv::List(vec![])]);
        let d = (*list_last(l1.as_mut())).li_tv.dict();
        let inner = (*list_last(l2.as_mut())).li_tv.list();
        assert_eq!(((*l1).lv_refcount.get(), (*d).dv_refcount.get()), (1, 1));
        assert_eq!(
            ((*l2).lv_refcount.get(), (*inner).lv_refcount.get()),
            (1, 1)
        );
        log.clear();

        let mut rettv = Tv::Unknown.build();
        assert_eq!(list_concat(l1, l2, &mut rettv), Ok(()));
        assert_eq!(((*l1).lv_refcount.get(), (*d).dv_refcount.get()), (1, 2));
        assert_eq!(
            ((*l2).lv_refcount.get(), (*inner).lv_refcount.get()),
            (1, 2)
        );
        let out = rettv.list();
        log.check(&[alloc::list(out)]);
        assert_eq!(list_len(out.as_ref()), 4);
        assert_eq!(
            tv::read(&raw const rettv),
            Tv::List(vec![f(1.0), DICT, f(3.0), LIST])
        );

        tv_clear(&mut rettv);
        list_free(l1);
        list_free(l2);
    }
}

/// The same `describe`'s `itp('can concatenate list with itself')`, spec
/// line 1146.
#[test]
fn concatenating_a_list_with_itself_copies_it_twice() {
    let log = AllocLog::start();
    // SAFETY: as above.
    unsafe {
        let l = tv::new_list(&[f(1.0), Tv::Dict(vec![])]);
        let d = (*list_last(l.as_mut())).li_tv.dict();
        assert_eq!(((*l).lv_refcount.get(), (*d).dv_refcount.get()), (1, 1));
        log.clear();

        let mut rettv = Tv::Unknown.build();
        assert_eq!(list_concat(l, l, &mut rettv), Ok(()));
        assert_eq!(((*l).lv_refcount.get(), (*d).dv_refcount.get()), (1, 3));
        let out = rettv.list();
        log.check(&[alloc::list(out)]);
        assert_eq!(list_len(out.as_ref()), 4);
        assert_eq!(
            tv::read(&raw const rettv),
            Tv::List(vec![f(1.0), DICT, f(1.0), DICT])
        );

        tv_clear(&mut rettv);
        list_free(l);
    }
}

/// The same `describe`'s `itp('can concatenate empty non-NULL lists')`,
/// spec line 1165: an empty operand costs only the answer's own header —
/// which is now what *every* concatenation costs, so the case is left
/// saying that an empty operand contributes no items and takes no
/// reference.
#[test]
fn concatenating_empty_lists_allocates_only_the_answer() {
    let log = AllocLog::start();
    // SAFETY: as above.
    unsafe {
        let l = tv::new_list(&[f(1.0), Tv::Dict(vec![])]);
        let le = tv::new_list(&[]);
        let le2 = tv::new_list(&[]);
        let d = (*list_last(l.as_mut())).li_tv.dict();
        log.clear();

        let mut kept = Vec::new();
        for (l1, l2, refs) in [(l, le, 2), (le, l, 3)] {
            let mut rettv = Tv::Unknown.build();
            assert_eq!(list_concat(l1, l2, &mut rettv), Ok(()));
            assert_eq!(((*l).lv_refcount.get(), (*d).dv_refcount.get()), (1, refs));
            assert_eq!(((*le).lv_refcount.get(), (*le2).lv_refcount.get()), (1, 1));
            let out = rettv.list();
            log.check(&[alloc::list(out)]);
            assert_eq!(list_len(out.as_ref()), 2);
            assert_eq!(tv::read(&raw const rettv), Tv::List(vec![f(1.0), DICT]));
            kept.push(rettv);
        }

        for (l1, l2) in [(le, le), (le, le2)] {
            let mut rettv = Tv::Unknown.build();
            assert_eq!(list_concat(l1, l2, &mut rettv), Ok(()));
            assert_eq!(((*l).lv_refcount.get(), (*d).dv_refcount.get()), (1, 3));
            log.check(&[alloc::list(rettv.list())]);
            assert_eq!(tv::read(&raw const rettv), Tv::List(vec![]));
            kept.push(rettv);
        }

        for mut rettv in kept {
            tv_clear(&mut rettv);
        }
        list_free(l);
        list_free(le);
        list_free(le2);
    }
}

// ---------------------------------------------------------------- join

/// `describe('join()') itp('works')`, spec line 1236.
#[test]
fn joining_a_list_renders_every_item() {
    let log = AllocLog::start();
    // SAFETY: each list and its growarray are this case's own.
    unsafe {
        let join = |l: *mut List, sep: &str| -> String {
            let mut ga = tv::ga_alloc(1, 80);
            assert_eq!(
                list_join(&raw mut ga, l.as_ref(), cstr(sep).as_ptr()),
                Ok(())
            );
            let out = if ga.ga_data.is_null() {
                String::new()
            } else {
                CStr::from_ptr(ga.ga_data.cast())
                    .to_string_lossy()
                    .into_owned()
            };
            ga_clear(&raw mut ga);
            out
        };

        let l = tv::new_list(&[Tv::s("boo"), Tv::s("far")]);
        assert_eq!(join(l, " "), "boo far");
        assert_eq!(join(l, ""), "boofar");
        list_free(l);

        let l = tv::new_list(&[Tv::s("boo")]);
        assert_eq!(join(l, " "), "boo");
        list_free(l);

        let l = tv::new_list(&[]);
        assert_eq!(join(l, " "), "");
        list_free(l);

        let l = tv::new_list(&[Tv::Dict(vec![]), Tv::s("far")]);
        assert_eq!(join(l, " "), "{} far");
        list_free(l);

        // A recursive list renders as the marker `string()` uses, not by
        // looping.
        let l = tv::new_list(&[Tv::List(vec![Tv::Cycle(1)]), Tv::s("far")]);
        assert_eq!(join(l, " "), "[[...@0]] far");
        let recursive = (*list_first(l.as_mut())).li_tv.list();
        (*recursive).remove_at(0);
        list_free(l);

        log.clear();
    }
}

// --------------------------------------------------------------- equal

/// The nine lists the two `list_equal` cases compare against the first.
///
/// # Safety
/// The editor must be up; the caller frees them.
unsafe fn equality_corpus() -> Vec<*mut List> {
    let inner = |items: Vec<Tv>| Tv::List(items);
    [
        vec![
            Tv::s("abc"),
            inner(vec![f(1.0), f(2.0), Tv::s("Abc")]),
            Tv::s("def"),
        ],
        vec![Tv::s("abc"), inner(vec![f(1.0), f(2.0), Tv::s("Abc")])],
        vec![
            Tv::s("abc"),
            inner(vec![f(1.0), f(2.0), Tv::s("Abc")]),
            Tv::s("Def"),
        ],
        vec![
            Tv::s("abc"),
            inner(vec![f(1.0), f(2.0), Tv::s("Abc"), f(4.0)]),
            Tv::s("def"),
        ],
        vec![
            Tv::s("Abc"),
            inner(vec![f(1.0), f(2.0), Tv::s("Abc")]),
            Tv::s("def"),
        ],
        vec![
            Tv::s("abc"),
            inner(vec![f(1.0), f(2.0), Tv::s("Abc")]),
            Tv::s("def"),
        ],
        vec![
            Tv::s("abc"),
            inner(vec![f(1.0), f(2.0), Tv::s("abc")]),
            Tv::s("def"),
        ],
        // The spec wrote these two `list('abc', nil, 'def')` and
        // `list('abc', {1, 2, nil}, 'def')`. Lua's `#` answers 3 for the
        // first constructor and 2 for the second, so the middle item is
        // `v:null` in one and the inner list is a pair in the other —
        // which is why both compare unequal for a different reason.
        vec![Tv::s("abc"), Tv::Nil, Tv::s("def")],
        vec![Tv::s("abc"), inner(vec![f(1.0), f(2.0)]), Tv::s("def")],
    ]
    .into_iter()
    // SAFETY: the caller's.
    .map(|items| unsafe { tv::new_list(&items) })
    .collect()
}

/// `describe('equal()') itp('compares empty and NULL lists correctly')`,
/// spec line 1263.
#[test]
fn a_null_list_equals_an_empty_one() {
    let _log = AllocLog::start();
    // SAFETY: both lists are this case's own.
    unsafe {
        let l = tv::new_list(&[]);
        let l2 = tv::new_list(&[]);
        let null: *mut List = ptr::null_mut();

        for ic in [true, false] {
            assert!(list_equal(l.as_ref(), null.as_ref(), ic));
            assert!(list_equal(null.as_ref(), l.as_ref(), ic));
            assert!(list_equal(null.as_ref(), null.as_ref(), ic));
            assert!(list_equal(l.as_ref(), l.as_ref(), ic));
            assert!(list_equal(l.as_ref(), l2.as_ref(), ic));
            assert!(list_equal(l2.as_ref(), l.as_ref(), ic));
        }

        list_free(l);
        list_free(l2);
    }
}

/// The same `describe`'s two `itp`s at spec lines 1281 and 1302, which run
/// the same nine comparisons with `ic` off and on.
#[test]
fn comparing_lists_folds_case_only_when_asked() {
    let _log = AllocLog::start();
    // SAFETY: every list is this case's own and is freed.
    unsafe {
        let ls = equality_corpus();
        // Index by index against `ls[0]`: exact first, case-insensitive
        // second. The rows that differ are the ones whose only difference
        // is a letter's case.
        let expected = [
            (true, true),
            (false, false),
            (false, true),
            (false, false),
            (false, true),
            (true, true),
            (false, true),
            (false, false),
            (false, false),
        ];
        for (i, (exact, folded)) in expected.into_iter().enumerate() {
            assert_eq!(
                list_equal(ls[0].as_ref(), ls[i].as_ref(), false),
                exact,
                "exact, list {i}"
            );
            assert_eq!(
                list_equal(ls[0].as_ref(), ls[i].as_ref(), true),
                folded,
                "folded, list {i}"
            );
        }
        for l in ls {
            list_free(l);
        }
    }
}

// ---------------------------------------------------------------- find

/// `describe('find') describe('()') itp('correctly indexes list')`, spec
/// line 1326.
///
/// The spec walked the same indexes twice, once with `lv_idx_item` warm and
/// once with it cleared, because `list_find` used to walk the links from
/// the nearest of three anchors and cached where it stopped. A list owns
/// its items in an array now: there is no cache and no walk, only the
/// bounds check and the arithmetic that turns a negative index into an
/// offset from the tail. So the case walks the spec's indexes once and
/// pins the arithmetic — including that repeating an index answers the
/// same item, which is all the second walk ever said.
#[test]
fn finding_an_item_by_index_works_from_either_end() {
    let log = AllocLog::start();
    // SAFETY: the list is this case's own and nothing edits it.
    unsafe {
        let l = tv::new_list(&floats(1..=5));
        let lis = tv::list_items(l);
        log.clear();

        for n in [-1, 0, 1] {
            assert!(list_find(None, n).is_null());
        }
        assert!(list_find(l.as_mut(), 5).is_null(), "past the end");
        assert!(list_find(l.as_mut(), -6).is_null(), "before the start");

        for (n, at) in [
            (-5, 0),
            (4, 4),
            (2, 2),
            (-3, 2),
            (2, 2),
            (2, 2),
            (-3, 2),
            (0, 0),
            (-1, 4),
        ] {
            assert_eq!(list_find(l.as_mut(), n), lis[at], "index {n}");
        }
        assert_eq!(list_first(l.as_mut()), lis[0]);
        assert_eq!(list_last(l.as_mut()), lis[4]);

        log.check(&[]);
        list_free(l);
    }
}

/// The `find > nr()` group, spec lines 1385–1454, in one case per shape.
#[test]
fn finding_a_number_by_index_reads_through_strings() {
    let log = AllocLog::start();
    // SAFETY: every list is this case's own.
    unsafe {
        let find_nr = |l: *mut List, n: c_int, msg: Option<&str>| -> (bool, i64) {
            let mut err = false;
            let ret = check_emsg(
                log.editor(),
                || list_find_nr(l.as_ref(), n, Some(&mut err)),
                msg,
            );
            (err, ret)
        };

        // Numbers, and strings that read as numbers, answer the same.
        for items in [
            (1..=5).map(|n| Tv::Int(i64::from(n))).collect::<Vec<_>>(),
            (1..=5).map(|n| Tv::s(n.to_string())).collect(),
        ] {
            let l = tv::new_list(&items);
            log.clear();
            for (n, want) in [(-5, 1), (4, 5), (2, 3), (-3, 3)] {
                assert_eq!(find_nr(l, n, None), (false, want));
            }
            log.check(&[]);
            list_free(l);
        }

        // A NULL string is zero, not an error.
        let l = tv::new_list(&[Tv::NullStr]);
        log.clear();
        assert_eq!(find_nr(l, 0, None), (false, 0));
        log.check(&[]);
        list_free(l);

        // A NULL list and an out-of-range index both set the error flag and
        // answer -1 without a message.
        for n in [-5, 4, 2, -3] {
            assert_eq!(find_nr(ptr::null_mut(), n, None), (true, -1));
        }
        let l = tv::new_list(&(1..=5).map(|n| Tv::Int(i64::from(n))).collect::<Vec<_>>());
        log.clear();
        for n in [-6, 5] {
            assert_eq!(find_nr(l, n, None), (true, -1));
        }
        log.check(&[]);
        list_free(l);

        // An item that is not a number reports, and answers 0.
        let l = tv::new_list(&[f(1.0), Tv::List(vec![]), Tv::Dict(vec![])]);
        for (n, msg) in [
            (0, "E805: Using a Float as a Number"),
            (1, "E745: Using a List as a Number"),
            (2, "E728: Using a Dictionary as a Number"),
            (-1, "E728: Using a Dictionary as a Number"),
            (-2, "E745: Using a List as a Number"),
            (-3, "E805: Using a Float as a Number"),
        ] {
            assert_eq!(find_nr(l, n, Some(msg)), (true, 0));
            log.clear();
        }
        list_free(l);
    }
}

/// The `find > str()` group, spec lines 1454–1502.
#[test]
fn finding_a_string_by_index_renders_scalars() {
    let log = AllocLog::start();
    // SAFETY: every list is this case's own; the answer is borrowed.
    unsafe {
        let find_str = |l: *mut List, n: c_int, msg: Option<&str>| -> Option<String> {
            let mut numbuf = NumBuf::new();
            let ret = check_emsg(
                log.editor(),
                || list_find_str(l.as_ref(), n, &mut numbuf),
                msg,
            );
            (!ret.is_null()).then(|| CStr::from_ptr(ret).to_string_lossy().into_owned())
        };

        // A float is rendered into the shared number buffer, which costs
        // two `free(NULL)`s in `vim_snprintf`.
        let l = tv::new_list(&[Tv::Int(1), f(2.5), Tv::Int(3), Tv::Int(4), Tv::Int(5)]);
        log.clear();
        assert_eq!(find_str(l, -5, None).as_deref(), Some("1"));
        assert_eq!(find_str(l, 1, None).as_deref(), Some("2.5"));
        assert_eq!(find_str(l, 4, None).as_deref(), Some("5"));
        assert_eq!(find_str(l, 2, None).as_deref(), Some("3"));
        assert_eq!(find_str(l, -3, None).as_deref(), Some("3"));
        log.check(&[
            alloc::freed(ptr::null::<u8>()),
            alloc::freed(ptr::null::<u8>()),
        ]);
        list_free(l);

        // A string item is answered in place.
        let l = tv::new_list(&(1..=5).map(|n| Tv::s(n.to_string())).collect::<Vec<_>>());
        log.clear();
        for (n, want) in [(-5, "1"), (4, "5"), (2, "3"), (-3, "3")] {
            assert_eq!(find_str(l, n, None).as_deref(), Some(want));
        }
        log.check(&[]);
        list_free(l);

        // A NULL string reads as empty.
        let l = tv::new_list(&[Tv::NullStr]);
        log.clear();
        assert_eq!(find_str(l, 0, None).as_deref(), Some(""));
        log.check(&[]);
        list_free(l);

        // Out of range answers NULL and reports the index.
        let l = tv::new_list(&(1..=5).map(|n| Tv::Int(i64::from(n))).collect::<Vec<_>>());
        assert_eq!(
            find_str(l, -6, Some("E684: List index out of range: -6")),
            None
        );
        log.clear();
        assert_eq!(
            find_str(l, 5, Some("E684: List index out of range: 5")),
            None
        );
        log.clear();
        list_free(l);

        // A container answers the empty string and reports.
        let l = tv::new_list(&[Tv::List(vec![]), Tv::Dict(vec![])]);
        for (n, msg) in [
            (0, "E730: Using a List as a String"),
            (1, "E731: Using a Dictionary as a String"),
            (-1, "E731: Using a Dictionary as a String"),
            (-2, "E730: Using a List as a String"),
        ] {
            assert_eq!(find_str(l, n, Some(msg)).as_deref(), Some(""));
            log.clear();
        }
        list_free(l);
    }
}

/// `describe('idx_of_item()') itp('works')`, spec line 1502 — the one case
/// whose subject is gone.
///
/// `tv_list_idx_of_item` searched a list for an item by address and
/// answered its index, or -1 when the item belonged to some other list.
/// There is nothing to search now: an item's identity **is** its index, and
/// its address is a borrow of one list's array that no other list can
/// hand out. So the case states the two halves of that instead — the index
/// round-trips through [`list_find`], and two lists never answer the
/// same address for any pair of indexes — plus the boundary answers the
/// spec's -1 stood for: no item, and so a NULL back.
#[test]
fn an_items_identity_is_its_index_into_one_list() {
    let _log = AllocLog::start();
    // SAFETY: both lists are this case's own.
    unsafe {
        let l = tv::new_list(&floats(1..=5));
        let l2 = tv::new_list(&[f(42.0), Tv::List(vec![])]);

        // Every index names the value that was built at it.
        for (i, want) in floats(1..=5).into_iter().enumerate() {
            let li = list_find(l.as_mut(), c_int::try_from(i).unwrap());
            assert!(!li.is_null(), "index {i}");
            assert_eq!(tv::read(&raw const (*li).li_tv), want, "index {i}");
        }

        // No index of one list names an item of the other.
        for i in 0..list_len(l.as_ref()) {
            for j in 0..list_len(l2.as_ref()) {
                assert_ne!(
                    list_find(l.as_mut(), i),
                    list_find(l2.as_mut(), j),
                    "{i} against {j}"
                );
            }
        }

        // What the spec's -1 stood for: there is no such item.
        assert!(list_find(l.as_mut(), 5).is_null());
        assert!(list_find(l.as_mut(), -6).is_null());
        assert!(list_find(None, 0).is_null());
        assert!(list_first(None).is_null());
        assert!(list_last(None).is_null());

        list_free(l);
        list_free(l2);
    }
}

/// The list allocator answers an empty header with one allocation and no
/// reference — the shape every case above starts from.
///
/// `len` is a capacity hint now rather than the ignored argument it was:
/// a caller that knows how many items are coming reserves them here. The
/// reservation goes through Rust's global allocator, which this log does
/// not see, so the assertion is that it does *not* show up — one `xcalloc`
/// however many items were asked for.
/// A fresh list is empty and owned by **one** reference: the handle
/// [`tv_list_alloc`] answers.
///
/// Upstream handed one back at a count of zero and left the first storer to
/// raise it, which is what made an unstored list something a caller had to
/// remember to free. The count is now exactly the number of live holders.
#[test]
fn a_fresh_list_is_empty_and_owned_by_its_handle() {
    let log = AllocLog::start();
    // SAFETY: the lists are this case's own, and each is released by the
    // handle going out of scope.
    unsafe {
        for len in [0, 10, -1] {
            let list = tv_list_alloc(len);
            let l = list.as_ptr();
            log.check(&[alloc::list(l)]);
            assert_eq!(list_len(l.as_ref()), 0, "len {len}");
            assert!(list_first(l.as_mut()).is_null());
            assert!(list_last(l.as_mut()).is_null());
            assert_eq!((*l).lv_refcount.get(), 1);
            assert_eq!((*l).lv_lock, VarLock::Unlocked);
            drop(list);
            log.check(&[alloc::freed(l)]);
        }
    }
}
