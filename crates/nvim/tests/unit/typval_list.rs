//! `describe('list')` from `test/unit/eval/typval_spec.lua`.
//!
//! Every case here asserts an exact allocation sequence through
//! [`crate::support::alloc::AllocLog`], which is what the spec's 285
//! `alloc_log` hooks did through the `mem_*` function-pointer seam. The
//! porting rule is in `support::alloc`: a size is always `size_of` or
//! `offset_of!`, never a literal, because the derivation *is* the assertion.
//!
//! Values are built and read back through [`crate::support::tv`], the twin
//! of `test/unit/eval/testutil.lua`.
//!
//! Every case needs a live editor, which Miri cannot start.

#![cfg(not(miri))]

use std::ffi::{CStr, c_char, c_int};
use std::ptr;

use neovim::eval::typval::{
    NumBuf, tv_clear, tv_list_alloc, tv_list_append_allocated_string, tv_list_append_dict,
    tv_list_append_list, tv_list_append_number, tv_list_append_owned_tv, tv_list_append_string,
    tv_list_append_tv, tv_list_concat, tv_list_copy, tv_list_drop_items, tv_list_equal,
    tv_list_extend, tv_list_find, tv_list_find_nr, tv_list_find_str, tv_list_free,
    tv_list_free_contents, tv_list_free_list, tv_list_idx_of_item, tv_list_insert,
    tv_list_insert_tv, tv_list_item_remove, tv_list_join, tv_list_remove_items, tv_list_unref,
    tv_list_watch_add, tv_list_watch_remove,
};
use neovim::garray::ga_clear;
use neovim::mbyte::convert_setup;
use neovim::memory::{xfree, xstrdup};
use neovim::types::{
    OK, Refcount, VAR_FLOAT, VAR_LIST, VarLock, list_T, listitem_T, listwatch_T, typval_T,
    typval_vval_union, vimconv_T,
};

use crate::support::alloc::{self, AllocLog};
use crate::support::tv::{self, Tv};
use crate::support::{check_emsg, cstr};

/// The spec's bare Lua numbers, which `lua2typvalt` made floats.
fn f(n: f64) -> Tv {
    Tv::Float(n)
}

/// `1, 2, … n` as floats — the spec's `list(1, 2, 3, …)`.
fn floats(ns: impl IntoIterator<Item = i32>) -> Vec<Tv> {
    ns.into_iter().map(|n| f(f64::from(n))).collect()
}

/// The spec's `list_watch`: a watcher standing on `li`, registered with `l`.
///
/// The Lua allocated it with `ffi.new`, which the allocation log does not
/// see; a `Box` is the same statement here.
///
/// # Safety
/// `l` is a live list and `li` one of its items (or NULL).
unsafe fn watch(l: *mut list_T, li: *mut listitem_T) -> Box<listwatch_T> {
    let mut lw = Box::new(listwatch_T {
        lw_item: li,
        lw_next: ptr::null_mut(),
    });
    unsafe { tv_list_watch_add(l, &raw mut *lw) };
    lw
}

/// Where each watcher of `lws` is standing.
fn standing(lws: &[Box<listwatch_T>]) -> Vec<*mut listitem_T> {
    lws.iter().map(|lw| lw.lw_item).collect()
}

// ---------------------------------------------------------------- item

/// `describe('item') describe('remove()') itp('works')`, spec line 125.
#[test]
fn removing_an_item_answers_the_next_and_frees_it() {
    let log = AllocLog::start();
    // SAFETY: the list is this case's own and is freed at the end.
    unsafe {
        let l = tv::new_list(&floats(1..=7));
        let mut lis = tv::list_items(l);
        let mut expected = vec![alloc::list(l)];
        expected.extend(lis.iter().map(|&li| alloc::li(li)));
        log.check(&expected);

        // From the front, from the back, and from the middle.
        for at in [0, 5, 2] {
            let after = lis.get(at + 1).copied().unwrap_or(ptr::null_mut());
            assert_eq!(tv_list_item_remove(l, lis[at]), after);
            log.check(&[alloc::freed(lis.remove(at))]);
            assert_eq!(tv::list_items(l), lis);
        }

        tv_list_free(l);
    }
}

/// The same `describe`'s `itp('also frees the value')`, spec line 158.
#[test]
fn removing_an_item_frees_its_value_first() {
    let log = AllocLog::start();
    // SAFETY: as above.
    unsafe {
        let l = tv::new_list(&["a", "b", "c", "d"].map(Tv::s));
        let mut lis = tv::list_items(l);
        let mut strings: Vec<*mut c_char> =
            lis.iter().map(|&li| (*li).li_tv.vval.v_string).collect();

        let mut expected = vec![alloc::list(l)];
        for (&li, &s) in lis.iter().zip(&strings) {
            expected.push(alloc::string(s, 1));
            expected.push(alloc::li(li));
        }
        log.check(&expected);

        for at in [0, 1, 1] {
            let after = lis.get(at + 1).copied().unwrap_or(ptr::null_mut());
            assert_eq!(tv_list_item_remove(l, lis[at]), after);
            log.check(&[
                alloc::freed(strings.remove(at)),
                alloc::freed(lis.remove(at)),
            ]);
            assert_eq!(tv::list_items(l), lis);
        }

        tv_list_free(l);
    }
}

/// The same `describe`'s `itp('works and adjusts watchers correctly')`,
/// spec line 198 — `tv_list_watch_add` and `tv_list_watch_fix` are tested
/// only here.
#[test]
fn removing_an_item_moves_the_watchers_standing_on_it() {
    let log = AllocLog::start();
    // SAFETY: as above; the watchers are unregistered before the list goes.
    unsafe {
        let l = tv::new_list(&floats(1..=7));
        let lis = tv::list_items(l);
        // Three watchers: on the first, the middle and the last item.
        let lws = [watch(l, lis[0]), watch(l, lis[3]), watch(l, lis[6])];

        let mut expected = vec![alloc::list(l)];
        expected.extend(lis.iter().map(|&li| alloc::li(li)));
        log.check(&expected);

        assert_eq!(tv_list_item_remove(l, lis[3]), lis[4]);
        log.check(&[alloc::freed(lis[3])]);
        assert_eq!(standing(&lws), [lis[0], lis[4], lis[6]]);

        // Removing an item nobody watches moves nobody.
        assert_eq!(tv_list_item_remove(l, lis[1]), lis[2]);
        log.check(&[alloc::freed(lis[1])]);
        assert_eq!(standing(&lws), [lis[0], lis[4], lis[6]]);

        // A watcher on the last item is pushed off the end.
        assert_eq!(tv_list_item_remove(l, lis[6]), ptr::null_mut());
        log.check(&[alloc::freed(lis[6])]);
        assert_eq!(standing(&lws), [lis[0], lis[4], ptr::null_mut()]);

        assert_eq!(tv_list_item_remove(l, lis[0]), lis[2]);
        log.check(&[alloc::freed(lis[0])]);
        assert_eq!(standing(&lws), [lis[2], lis[4], ptr::null_mut()]);

        for lw in &lws {
            tv_list_watch_remove(l, (&raw const **lw).cast_mut());
        }
        tv_list_free(l);
        log.check(&[
            alloc::freed(lis[2]),
            alloc::freed(lis[4]),
            alloc::freed(lis[5]),
            alloc::freed(l),
        ]);
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
        let lw = watch(l, (*l).lv_first);
        assert!(!(*l).lv_watch.is_null());
        log.clear();

        tv_list_watch_remove(l, (&raw const *lw).cast_mut());
        assert!((*l).lv_watch.is_null());
        log.check(&[]);

        let lws = [
            watch(l, (*l).lv_first),
            watch(l, (*l).lv_first),
            watch(l, (*l).lv_first),
        ];
        let at = |lw: &listwatch_T| (&raw const *lw).cast_mut();
        log.clear();

        // The newest is at the head, so removing the middle one leaves the
        // third watching and the first behind it.
        tv_list_watch_remove(l, at(&lws[1]));
        assert_eq!((*l).lv_watch, at(&lws[2]));
        assert_eq!((*(*l).lv_watch).lw_next, at(&lws[0]));
        tv_list_watch_remove(l, at(&lws[0]));
        assert_eq!((*l).lv_watch, at(&lws[2]));
        assert!((*(*l).lv_watch).lw_next.is_null());
        tv_list_watch_remove(l, at(&lws[2]));
        assert!((*l).lv_watch.is_null());
        log.check(&[]);

        tv_list_free(l);
    }
}

/// The same `describe`'s `itp('ignores not found watchers')`, spec line 281.
#[test]
fn removing_an_unregistered_watch_is_a_no_op() {
    let log = AllocLog::start();
    // SAFETY: `lw` was never registered, so nothing links to it.
    unsafe {
        let l = tv::new_list(&floats(1..=7));
        let mut lw = listwatch_T {
            lw_item: ptr::null_mut(),
            lw_next: ptr::null_mut(),
        };
        log.clear();
        tv_list_watch_remove(l, &raw mut lw);
        log.check(&[]);
        tv_list_free(l);
    }
}

// ------------------------------------------------------- free / unref

/// The three lists the free cases are stated over: a list of two scalars,
/// a list holding a dict, and a list holding a list.
///
/// # Safety
/// The editor must be up. The caller owns all three.
unsafe fn three_lists(log: &AllocLog) -> [(*mut list_T, Vec<*mut std::ffi::c_void>); 3] {
    let mut out = Vec::new();
    // SAFETY: the lists are the caller's.
    unsafe {
        let l1 = tv::new_list(&[f(1.0), Tv::s("abc")]);
        let s1 = (*(*l1).lv_last).li_tv.vval.v_string;
        log.check(&[
            alloc::list(l1),
            alloc::li((*l1).lv_first),
            alloc::string(s1, "abc".len()),
            alloc::li((*l1).lv_last),
        ]);
        out.push((
            l1,
            vec![
                (*l1).lv_first.cast(),
                s1.cast(),
                (*l1).lv_last.cast(),
                l1.cast(),
            ],
        ));

        let l2 = tv::new_list(&[Tv::Dict(vec![])]);
        let d2 = (*(*l2).lv_first).li_tv.vval.v_dict;
        log.check(&[alloc::list(l2), alloc::dict(d2), alloc::li((*l2).lv_first)]);
        out.push((l2, vec![d2.cast(), (*l2).lv_first.cast(), l2.cast()]));

        let l3 = tv::new_list(&[Tv::List(vec![])]);
        let inner = (*(*l3).lv_first).li_tv.vval.v_list;
        log.check(&[
            alloc::list(l3),
            alloc::list(inner),
            alloc::li((*l3).lv_first),
        ]);
        out.push((l3, vec![inner.cast(), (*l3).lv_first.cast(), l3.cast()]));
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
            tv_list_free(l);
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
/// 329: the list header alone, which is what a caller who has taken the
/// items over wants.
#[test]
fn freeing_only_the_list_leaves_its_contents() {
    let log = AllocLog::start();
    // SAFETY: the items are deliberately leaked, as the spec left them.
    unsafe {
        for (l, allocated) in three_lists(&log) {
            tv_list_free_list(l);
            log.check(&[alloc::freed(*allocated.last().expect("the list itself"))]);
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
            tv_list_free_contents(l);
            log.check(
                &allocated[..allocated.len() - 1]
                    .iter()
                    .map(|&p| alloc::freed(p))
                    .collect::<Vec<_>>(),
            );
            tv_list_free_list(l);
            log.clear();
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
        let inner = (*(*l).lv_first).li_tv.vval.v_list;
        let li = (*l).lv_first;
        log.check(&[alloc::list(l), alloc::list(inner), alloc::li(li)]);

        (*l).lv_refcount = Refcount::new(2);
        tv_list_unref(l);
        log.check(&[]);
        tv_list_unref(l);
        log.check(&[alloc::freed(inner), alloc::freed(li), alloc::freed(l)]);
    }
}

// ------------------------------------------------------- drop / remove

/// `describe('drop_items()') itp('works')`, spec line 417: unlink a run of
/// items without freeing them, moving any watcher that stood inside it.
#[test]
fn dropping_a_run_of_items_unlinks_them_and_moves_the_watchers() {
    let log = AllocLog::start();
    // SAFETY: the list is this case's own; the dropped items are the
    // caller's afterwards and are freed at the end.
    unsafe {
        let mut l_tv = Tv::List(floats(1..=13)).build();
        let l = l_tv.vval.v_list;
        let lis = tv::list_items(l);
        let lws = [watch(l, lis[0]), watch(l, lis[6]), watch(l, lis[12])];
        log.clear();

        tv_list_drop_items(l, lis[0], lis[2]);
        assert_eq!(tv::read(&raw const l_tv), Tv::List(floats(4..=13)));
        assert_eq!(standing(&lws), [lis[3], lis[6], lis[12]]);

        tv_list_drop_items(l, lis[10], lis[12]);
        assert_eq!(tv::read(&raw const l_tv), Tv::List(floats(4..=10)));
        assert_eq!(standing(&lws), [lis[3], lis[6], ptr::null_mut()]);

        tv_list_drop_items(l, lis[5], lis[7]);
        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(vec![f(4.0), f(5.0), f(9.0), f(10.0)])
        );
        assert_eq!(standing(&lws), [lis[3], lis[8], ptr::null_mut()]);

        tv_list_drop_items(l, lis[3], lis[9]);
        assert_eq!(tv::read(&raw const l_tv), Tv::List(vec![]));
        assert_eq!(standing(&lws), [ptr::null_mut(); 3]);

        for lw in &lws {
            tv_list_watch_remove(l, (&raw const **lw).cast_mut());
        }
        log.check(&[]);

        // The unlinked items hold numbers, so they are ours to release.
        for li in lis {
            xfree(li.cast());
        }
        tv_clear(&raw mut l_tv);
    }
}

/// `describe('remove_items()') itp('works')`, spec line 462: the same walk,
/// but the items and their values are released.
#[test]
fn removing_a_run_of_items_frees_them_with_their_values() {
    let log = AllocLog::start();
    // SAFETY: the list is this case's own and is cleared at the end.
    unsafe {
        let strings: Vec<Tv> = (1..=13).map(|n| Tv::s(n.to_string())).collect();
        let mut l_tv = Tv::List(strings).build();
        let l = l_tv.vval.v_list;
        let lis = tv::list_items(l);
        let values: Vec<*mut c_char> = lis.iter().map(|&li| (*li).li_tv.vval.v_string).collect();
        let lws = [watch(l, lis[0]), watch(l, lis[6]), watch(l, lis[12])];
        log.clear();

        let text = |ns: &[i32]| Tv::List(ns.iter().map(|n| Tv::s(n.to_string())).collect());
        let freed = |range: &[usize]| -> Vec<_> {
            range
                .iter()
                .flat_map(|&i| [alloc::freed(values[i]), alloc::freed(lis[i])])
                .collect()
        };

        tv_list_remove_items(l, lis[0], lis[2]);
        assert_eq!(
            tv::read(&raw const l_tv),
            text(&[4, 5, 6, 7, 8, 9, 10, 11, 12, 13])
        );
        assert_eq!(standing(&lws), [lis[3], lis[6], lis[12]]);
        log.check(&freed(&[0, 1, 2]));

        tv_list_remove_items(l, lis[10], lis[12]);
        assert_eq!(tv::read(&raw const l_tv), text(&[4, 5, 6, 7, 8, 9, 10]));
        assert_eq!(standing(&lws), [lis[3], lis[6], ptr::null_mut()]);
        log.check(&freed(&[10, 11, 12]));

        tv_list_remove_items(l, lis[5], lis[7]);
        assert_eq!(tv::read(&raw const l_tv), text(&[4, 5, 9, 10]));
        assert_eq!(standing(&lws), [lis[3], lis[8], ptr::null_mut()]);
        log.check(&freed(&[5, 6, 7]));

        tv_list_remove_items(l, lis[3], lis[9]);
        assert_eq!(tv::read(&raw const l_tv), Tv::List(vec![]));
        assert_eq!(standing(&lws), [ptr::null_mut(); 3]);
        log.check(&freed(&[3, 4, 8, 9]));

        for lw in &lws {
            tv_list_watch_remove(l, (&raw const **lw).cast_mut());
        }
        log.check(&[]);
        tv_clear(&raw mut l_tv);
    }
}

// -------------------------------------------------------------- insert

/// `describe('insert') describe('()') itp('works')`, spec line 546.
#[test]
fn inserting_an_item_puts_it_before_the_one_named() {
    let log = AllocLog::start();
    // SAFETY: the list owns every item handed to it.
    unsafe {
        let mut l_tv = Tv::List(floats(1..=7)).build();
        let l = l_tv.vval.v_list;
        let lis = tv::list_items(l);

        let float_item = |n: f64| {
            let li = tv::li_alloc();
            (*li).li_tv = typval_T {
                v_type: VAR_FLOAT,
                v_lock: VarLock::Unlocked,
                vval: typval_vval_union { v_float: n },
            };
            li
        };

        // A NULL "before" appends.
        let li = float_item(100500.0);
        tv_list_insert(l, li, ptr::null_mut());
        assert_eq!((*l).lv_last, li);
        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(floats(1..=7).into_iter().chain([f(100500.0)]).collect())
        );

        let li = float_item(0.0);
        tv_list_insert(l, li, lis[0]);
        assert_eq!((*l).lv_first, li);

        let li = float_item(4.5);
        tv_list_insert(l, li, lis[4]);
        assert_eq!(tv::list_items(l)[5], li);
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

        log.clear();
        tv_clear(&raw mut l_tv);
    }
}

/// The same `describe`'s `itp('works with an empty list')`, spec line 570.
#[test]
fn inserting_into_an_empty_list_makes_it_the_only_item() {
    let log = AllocLog::start();
    // SAFETY: as above.
    unsafe {
        let mut l_tv = Tv::List(vec![]).build();
        let l = l_tv.vval.v_list;
        assert!((*l).lv_first.is_null());
        assert!((*l).lv_last.is_null());

        let li = tv::li_alloc();
        (*li).li_tv = typval_T {
            v_type: VAR_FLOAT,
            v_lock: VarLock::Unlocked,
            vval: typval_vval_union { v_float: 100500.0 },
        };
        tv_list_insert(l, li, ptr::null_mut());
        assert_eq!((*l).lv_last, li);
        assert_eq!(tv::read(&raw const l_tv), Tv::List(vec![f(100500.0)]));

        log.clear();
        tv_clear(&raw mut l_tv);
    }
}

/// `describe('insert') describe('tv()') itp('works')`, spec line 585: the
/// value is *copied* in, so a container gains a reference and a string is
/// duplicated.
#[test]
fn inserting_a_value_copies_it() {
    let log = AllocLog::start();
    // SAFETY: each `typval_T` here is this case's own and is cleared.
    unsafe {
        let mut l_tv = Tv::List(vec![]).build();
        let l = l_tv.vval.v_list;

        let mut inner_tv = Tv::List(vec![]).build();
        log.clear();
        let inner = inner_tv.vval.v_list;
        assert_eq!((*inner).lv_refcount.get(), 1);
        tv_list_insert_tv(l, &raw mut inner_tv, ptr::null_mut());
        assert_eq!((*inner).lv_refcount.get(), 2, "the copy holds a reference");
        assert_eq!((*(*l).lv_first).li_tv.vval.v_list, inner);
        log.check(&[alloc::li((*l).lv_first)]);

        let mut s_tv = Tv::s("test").build();
        log.check(&[alloc::string(s_tv.vval.v_string, "test".len())]);
        tv_list_insert_tv(l, &raw mut s_tv, (*l).lv_first);
        log.check(&[
            alloc::li((*l).lv_first),
            alloc::string((*(*l).lv_first).li_tv.vval.v_string, "test".len()),
        ]);

        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(vec![Tv::s("test"), Tv::List(vec![])])
        );

        tv_clear(&raw mut l_tv);
        tv_clear(&raw mut inner_tv);
        tv_clear(&raw mut s_tv);
    }
}

// -------------------------------------------------------------- append

/// `describe('append') describe('list()') itp('works')`, spec line 616.
#[test]
fn appending_a_list_takes_a_reference() {
    let log = AllocLog::start();
    // SAFETY: the outer list owns the items; the inner list is unref'd by
    // clearing the outer one.
    unsafe {
        let mut l_tv = Tv::List(vec![]).build();
        let l = l_tv.vval.v_list;

        let inner = tv::new_list(&[f(1.0)]);
        log.clear();
        assert_eq!((*inner).lv_refcount.get(), 1);
        tv_list_append_list(l, inner);
        assert_eq!((*inner).lv_refcount.get(), 2);
        assert_eq!((*(*l).lv_first).li_tv.vval.v_list, inner);
        log.check(&[alloc::li((*l).lv_last)]);

        // A NULL list still costs an item.
        tv_list_append_list(l, ptr::null_mut());
        log.check(&[alloc::li((*l).lv_last)]);

        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(vec![Tv::List(vec![f(1.0)]), Tv::NullList])
        );

        tv_clear(&raw mut l_tv);
        tv_list_unref(inner);
    }
}

/// The same `describe`'s `dict()`, spec line 639.
#[test]
fn appending_a_dict_takes_a_reference() {
    let log = AllocLog::start();
    // SAFETY: as above.
    unsafe {
        let mut l_tv = Tv::List(vec![]).build();
        let l = l_tv.vval.v_list;

        let mut d_tv = Tv::dict([("test", f(1.0))]).build();
        let d = d_tv.vval.v_dict;
        log.clear();
        assert_eq!((*d).dv_refcount.get(), 1);
        tv_list_append_dict(l, d);
        assert_eq!((*d).dv_refcount.get(), 2);
        assert_eq!((*(*l).lv_first).li_tv.vval.v_dict, d);
        log.check(&[alloc::li((*l).lv_last)]);

        tv_list_append_dict(l, ptr::null_mut());
        log.check(&[alloc::li((*l).lv_last)]);

        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(vec![Tv::dict([("test", f(1.0))]), Tv::NullDict])
        );

        tv_clear(&raw mut l_tv);
        tv_clear(&raw mut d_tv);
    }
}

/// The same `describe`'s `string()`, spec line 663.
///
/// The assertion is the *order*: the string is copied before the item that
/// will hold it is allocated. A negative length means "to the terminator".
#[test]
fn appending_a_string_copies_it_then_appends() {
    let log = AllocLog::start();
    // SAFETY: the list owns everything appended to it.
    unsafe {
        let mut l_tv = Tv::List(vec![]).build();
        let l = l_tv.vval.v_list;
        log.clear();

        let test = cstr("test");
        tv_list_append_string(l, test.as_ptr(), 3);
        log.check(&[
            alloc::string((*(*l).lv_last).li_tv.vval.v_string, 3),
            alloc::li((*l).lv_last),
        ]);

        // A NULL string allocates nothing but the item, at either length.
        tv_list_append_string(l, ptr::null(), 0);
        log.check(&[alloc::li((*l).lv_last)]);
        tv_list_append_string(l, ptr::null(), -1);
        log.check(&[alloc::li((*l).lv_last)]);

        tv_list_append_string(l, test.as_ptr(), -1);
        log.check(&[
            alloc::string((*(*l).lv_last).li_tv.vval.v_string, 4),
            alloc::li((*l).lv_last),
        ]);

        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(vec![Tv::s("tes"), Tv::NullStr, Tv::NullStr, Tv::s("test")])
        );

        tv_clear(&raw mut l_tv);
    }
}

/// The same `describe`'s `allocated string()`, spec line 694: ownership is
/// transferred, so nothing but the item is allocated.
#[test]
fn appending_an_allocated_string_takes_ownership() {
    let log = AllocLog::start();
    // SAFETY: `s` is handed to the list, which frees it.
    unsafe {
        let mut l_tv = Tv::List(vec![]).build();
        let l = l_tv.vval.v_list;

        let s = xstrdup(cstr("test").as_ptr());
        log.clear();
        tv_list_append_allocated_string(l, s);
        log.check(&[alloc::li((*l).lv_last)]);

        tv_list_append_allocated_string(l, ptr::null_mut());
        log.check(&[alloc::li((*l).lv_last)]);
        tv_list_append_allocated_string(l, ptr::null_mut());
        log.check(&[alloc::li((*l).lv_last)]);

        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(vec![Tv::s("test"), Tv::NullStr, Tv::NullStr])
        );

        tv_clear(&raw mut l_tv);
    }
}

/// The same `describe`'s `number()`, spec line 719.
#[test]
fn appending_a_number_costs_only_the_item() {
    let log = AllocLog::start();
    // SAFETY: as above.
    unsafe {
        let mut l_tv = Tv::List(vec![]).build();
        let l = l_tv.vval.v_list;
        log.clear();

        tv_list_append_number(l, -100500);
        log.check(&[alloc::li((*l).lv_last)]);
        tv_list_append_number(l, 100500);
        log.check(&[alloc::li((*l).lv_last)]);

        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(vec![Tv::Int(-100500), Tv::Int(100500)])
        );

        tv_clear(&raw mut l_tv);
    }
}

/// The same `describe`'s `tv()`, spec line 738: a copy, like `insert_tv`.
#[test]
fn appending_a_value_copies_it() {
    let log = AllocLog::start();
    // SAFETY: each value is this case's own and is cleared.
    unsafe {
        let mut l_tv = Tv::List(vec![]).build();
        let l = l_tv.vval.v_list;

        let mut inner_tv = Tv::List(vec![]).build();
        log.clear();
        let inner = inner_tv.vval.v_list;
        assert_eq!((*inner).lv_refcount.get(), 1);
        tv_list_append_tv(l, &raw mut inner_tv);
        assert_eq!((*inner).lv_refcount.get(), 2);
        assert_eq!((*(*l).lv_first).li_tv.vval.v_list, inner);
        log.check(&[alloc::li((*l).lv_first)]);

        let mut s_tv = Tv::s("test").build();
        log.check(&[alloc::string(s_tv.vval.v_string, "test".len())]);
        tv_list_append_tv(l, &raw mut s_tv);
        log.check(&[
            alloc::li((*l).lv_last),
            alloc::string((*(*l).lv_last).li_tv.vval.v_string, "test".len()),
        ]);

        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(vec![Tv::List(vec![]), Tv::s("test")])
        );

        tv_clear(&raw mut l_tv);
        tv_clear(&raw mut inner_tv);
        tv_clear(&raw mut s_tv);
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
        let l = l_tv.vval.v_list;

        let inner_tv = Tv::List(vec![]).build();
        log.clear();
        let inner = inner_tv.vval.v_list;
        assert_eq!((*inner).lv_refcount.get(), 1);
        tv_list_append_owned_tv(l, inner_tv);
        assert_eq!(
            (*inner).lv_refcount.get(),
            1,
            "the reference moved, not copied"
        );
        assert_eq!((*(*l).lv_first).li_tv.vval.v_list, inner);
        log.check(&[alloc::li((*l).lv_first)]);

        let s_tv = Tv::s("test").build();
        log.check(&[alloc::string(s_tv.vval.v_string, "test".len())]);
        tv_list_append_owned_tv(l, s_tv);
        assert_eq!(
            (*(*l).lv_last).li_tv.vval.v_string,
            s_tv.vval.v_string,
            "the string itself moved in"
        );
        log.check(&[alloc::li((*l).lv_last)]);

        assert_eq!(
            tv::read(&raw const l_tv),
            Tv::List(vec![Tv::List(vec![]), Tv::s("test")])
        );

        tv_clear(&raw mut l_tv);
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
                    tv_list_copy(ptr::null_mut(), ptr::null_mut(), deep, copy_id).is_null(),
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
#[test]
fn copying_a_list_shares_or_rebuilds_its_containers() {
    let log = AllocLog::start();
    // SAFETY: every list here is this case's own and is freed.
    unsafe {
        let mut l_tv = copy_corpus().build();
        let l = l_tv.vval.v_list;
        let lis = tv::list_items(l);
        let inner_dict = (*lis[0]).li_tv.vval.v_dict;
        let inner_list = (*lis[1]).li_tv.vval.v_list;
        log.clear();

        assert_eq!((*inner_dict).dv_refcount.get(), 1);
        assert_eq!((*inner_list).lv_refcount.get(), 1);
        let shallow = tv_list_copy(ptr::null_mut(), l, false, 0);
        assert_eq!((*inner_dict).dv_refcount.get(), 2);
        assert_eq!((*inner_list).lv_refcount.get(), 2);
        let copies = tv::list_items(shallow);
        assert_eq!((*copies[0]).li_tv.vval.v_dict, inner_dict);
        assert_eq!((*copies[1]).li_tv.vval.v_list, inner_list);
        assert_eq!(tv::read_list(shallow), copy_corpus());
        let mut expected = vec![alloc::list(shallow)];
        for (i, &li) in copies.iter().enumerate() {
            expected.push(alloc::li(li));
            if i == 3 {
                expected.push(alloc::string((*li).li_tv.vval.v_string, "“".len()));
            }
        }
        log.check(&expected);
        tv_list_free(shallow);
        log.clear();

        assert_eq!((*inner_dict).dv_refcount.get(), 1);
        assert_eq!((*inner_list).lv_refcount.get(), 1);
        let deep = tv_list_copy(ptr::null_mut(), l, true, 0);
        assert!(!deep.is_null());
        assert_eq!(
            (*inner_dict).dv_refcount.get(),
            1,
            "a deep copy shares nothing"
        );
        assert_eq!((*inner_list).lv_refcount.get(), 1);
        let copies = tv::list_items(deep);
        assert_ne!((*copies[0]).li_tv.vval.v_dict, inner_dict);
        assert_ne!((*copies[1]).li_tv.vval.v_list, inner_list);
        assert_eq!(tv::read_list(deep), copy_corpus());

        let copied_dict = (*copies[0]).li_tv.vval.v_dict;
        let di = tv::first_di(copied_dict);
        let copied_list = (*copies[1]).li_tv.vval.v_list;
        log.check(&[
            alloc::list(deep),
            alloc::li(copies[0]),
            alloc::dict(copied_dict),
            alloc::di(di, "«".len()),
            alloc::string((*di).di_tv.vval.v_string, "»".len()),
            alloc::li(copies[1]),
            alloc::list(copied_list),
            alloc::li((*copied_list).lv_first),
            alloc::string((*(*copied_list).lv_first).li_tv.vval.v_string, "„".len()),
            alloc::li(copies[2]),
            alloc::li(copies[3]),
            alloc::string((*copies[3]).li_tv.vval.v_string, "“".len()),
            alloc::li(copies[4]),
            alloc::li(copies[5]),
            alloc::li(copies[6]),
        ]);

        tv_list_free(deep);
        tv_clear(&raw mut l_tv);
    }
}

/// `itp('copies list correctly and converts items')`, spec line 870: the
/// same walk through a `vimconv_T`, which rewrites every string it copies.
///
/// The allocation for a converted string is the *source* length plus one,
/// not the answer's — which is why the sizes here look too big.
#[test]
fn a_converting_copy_rewrites_every_string() {
    let log = AllocLog::start();
    // SAFETY: the converter and every list are this case's own.
    unsafe {
        let mut vc: vimconv_T = std::mem::zeroed();
        assert_eq!(
            convert_setup(
                &raw mut vc,
                cstr("utf-8").as_ptr().cast_mut(),
                cstr("latin1").as_ptr().cast_mut(),
            ),
            OK
        );

        let mut l_tv = copy_corpus().build();
        let l = l_tv.vval.v_list;
        let lis = tv::list_items(l);
        let inner_dict = (*lis[0]).li_tv.vval.v_dict;
        let inner_list = (*lis[1]).li_tv.vval.v_list;
        log.clear();

        let deep = tv_list_copy(&raw mut vc, l, true, 0);
        assert!(!deep.is_null());
        assert_eq!((*inner_dict).dv_refcount.get(), 1);
        assert_eq!((*inner_list).lv_refcount.get(), 1);
        let copies = tv::list_items(deep);
        assert_ne!((*copies[0]).li_tv.vval.v_dict, inner_dict);
        assert_ne!((*copies[1]).li_tv.vval.v_list, inner_list);
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

        let copied_dict = (*copies[0]).li_tv.vval.v_dict;
        let di = tv::first_di(copied_dict);
        let copied_list = (*copies[1]).li_tv.vval.v_list;
        log.check_net(
            false,
            &[
                alloc::list(deep),
                alloc::li(copies[0]),
                alloc::dict(copied_dict),
                alloc::di(di, 1),
                alloc::string((*di).di_tv.vval.v_string, "»".len()),
                alloc::li(copies[1]),
                alloc::list(copied_list),
                alloc::li((*copied_list).lv_first),
                alloc::string((*(*copied_list).lv_first).li_tv.vval.v_string, "„".len()),
                alloc::li(copies[2]),
                alloc::li(copies[3]),
                alloc::string((*copies[3]).li_tv.vval.v_string, "“".len()),
                alloc::li(copies[4]),
                alloc::li(copies[5]),
                alloc::li(copies[6]),
            ],
        );

        tv_list_free(deep);
        tv_clear(&raw mut l_tv);
        convert_setup(&raw mut vc, ptr::null_mut(), ptr::null_mut());
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
        let inner = inner_tv.vval.v_list;
        assert_eq!((*inner).lv_refcount.get(), 3);
        let l = l_tv.vval.v_list;
        assert_eq!(
            (*(*l).lv_first).li_tv.vval.v_list,
            (*(*l).lv_last).li_tv.vval.v_list
        );

        let without = tv_list_copy(ptr::null_mut(), l, true, 0);
        assert_ne!(
            (*(*without).lv_first).li_tv.vval.v_list,
            (*(*without).lv_last).li_tv.vval.v_list
        );
        assert_eq!(
            tv::read_list(without),
            Tv::List(vec![Tv::List(vec![]), Tv::List(vec![])])
        );

        let with = tv_list_copy(ptr::null_mut(), l, true, 2);
        assert_eq!(
            (*(*with).lv_first).li_tv.vval.v_list,
            (*(*with).lv_last).li_tv.vval.v_list
        );
        // The two items are the same container, which the read spells as a
        // cycle back to it.
        assert_eq!(
            tv::read_list(with),
            Tv::List(vec![Tv::List(vec![]), Tv::List(vec![])])
        );

        assert_eq!((*inner).lv_refcount.get(), 3);
        tv_list_unref(without);
        tv_list_unref(with);
        tv_clear(&raw mut l_tv);
        tv_clear(&raw mut inner_tv);
    }
}

/// `itp('works with self-referencing list with copyID')`, spec line 931.
#[test]
fn a_self_referencing_list_copies_into_a_self_referencing_copy() {
    let _log = AllocLog::start();
    // SAFETY: the cycle is broken before either list is released.
    unsafe {
        let mut l_tv = Tv::List(vec![]).build();
        let l = l_tv.vval.v_list;
        assert_eq!((*l).lv_refcount.get(), 1);
        tv_list_append_list(l, l);
        assert_eq!((*l).lv_refcount.get(), 2);

        let copy = tv_list_copy(ptr::null_mut(), l, true, 2);
        assert_eq!((*copy).lv_refcount.get(), 2, "the copy holds itself");
        assert_eq!(tv::read_list(copy), Tv::List(vec![Tv::Cycle(0)]));

        // Break both cycles so the lists can go.
        tv_list_item_remove(l, tv::list_items(l)[0]);
        assert_eq!((*l).lv_refcount.get(), 1);
        tv_list_item_remove(copy, tv::list_items(copy)[0]);
        assert_eq!((*copy).lv_refcount.get(), 1);

        tv_list_unref(copy);
        tv_clear(&raw mut l_tv);
    }
}

// -------------------------------------------------------------- extend

/// `describe('extend()') itp('can extend list with itself')`, spec line 954.
#[test]
fn a_list_can_be_extended_with_itself() {
    let log = AllocLog::start();
    // SAFETY: each list is this case's own and is freed.
    unsafe {
        // `bef` picks where the copied run lands: the end, before the last
        // item, or before the first.
        for (before, expected) in [
            (Before::End, vec![f(1.0), DICT, f(1.0), DICT]),
            (Before::Last, vec![f(1.0), f(1.0), DICT, DICT]),
            (Before::First, vec![f(1.0), DICT, f(1.0), DICT]),
        ] {
            let l = tv::new_list(&[f(1.0), Tv::Dict(vec![])]);
            log.clear();
            let d = (*(*l).lv_last).li_tv.vval.v_dict;
            assert_eq!((*l).lv_refcount.get(), 1);
            assert_eq!((*d).dv_refcount.get(), 1);

            let bef = match before {
                Before::End => ptr::null_mut(),
                Before::Last => (*l).lv_last,
                Before::First => (*l).lv_first,
            };
            tv_list_extend(l, l, bef);

            let items = tv::list_items(l);
            let (a, b) = match before {
                Before::End => (items[2], items[3]),
                Before::Last => (items[1], items[2]),
                Before::First => (items[0], items[1]),
            };
            log.check(&[alloc::li(a), alloc::li(b)]);
            assert_eq!((*l).lv_refcount.get(), 1);
            assert_eq!((*d).dv_refcount.get(), 2, "the dict gained one reference");
            assert_eq!(tv::read_list(l), Tv::List(expected));

            tv_list_free(l);
        }
    }
}

/// A dict placeholder for the `extend` expectations, which never look
/// inside it.
const DICT: Tv = Tv::Dict(Vec::new());

/// Where `tv_list_extend` puts the copied run.
enum Before {
    End,
    Last,
    First,
}

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
        let d = (*(*l).lv_last).li_tv.vval.v_dict;

        for bef in [ptr::null_mut(), (*l).lv_first, (*l).lv_last] {
            tv_list_extend(l, empty, bef);
            log.check(&[]);
            assert_eq!((*l).lv_refcount.get(), 1);
            assert_eq!((*d).dv_refcount.get(), 1);
            assert_eq!((*empty).lv_refcount.get(), 1);
            assert_eq!(tv::read_list(l), Tv::List(vec![f(1.0), DICT]));
        }

        tv_list_free(l);
        tv_list_free(empty);
    }
}

/// The same `describe`'s `itp('can extend list with another non-empty
/// list')`, spec line 1028.
#[test]
fn extending_with_another_list_copies_its_items() {
    let log = AllocLog::start();
    // SAFETY: as above.
    unsafe {
        let l2 = tv::new_list(&[f(42.0), Tv::List(vec![])]);
        let inner = (*(*l2).lv_last).li_tv.vval.v_list;
        assert_eq!((*l2).lv_refcount.get(), 1);
        assert_eq!((*inner).lv_refcount.get(), 1);

        for (before, expected, at) in [
            (Before::End, vec![f(1.0), DICT, f(42.0), LIST], [2, 3]),
            (Before::First, vec![f(42.0), LIST, f(1.0), DICT], [0, 1]),
            (Before::Last, vec![f(1.0), f(42.0), LIST, DICT], [1, 2]),
        ] {
            let l = tv::new_list(&[f(1.0), Tv::Dict(vec![])]);
            log.clear();
            let d = (*(*l).lv_last).li_tv.vval.v_dict;
            assert_eq!((*l).lv_refcount.get(), 1);
            assert_eq!((*d).dv_refcount.get(), 1);

            let bef = match before {
                Before::End => ptr::null_mut(),
                Before::Last => (*l).lv_last,
                Before::First => (*l).lv_first,
            };
            tv_list_extend(l, l2, bef);

            let items = tv::list_items(l);
            log.check(&[alloc::li(items[at[0]]), alloc::li(items[at[1]])]);
            assert_eq!(
                (*l2).lv_refcount.get(),
                1,
                "the source list itself is not held"
            );
            assert_eq!((*inner).lv_refcount.get(), 2, "but its list item is");
            assert_eq!(tv::read_list(l), Tv::List(expected));

            tv_list_free(l);
            assert_eq!((*inner).lv_refcount.get(), 1);
        }

        tv_list_free(l2);
    }
}

/// A list placeholder, as [`DICT`].
const LIST: Tv = Tv::List(Vec::new());

// -------------------------------------------------------------- concat

/// `describe('concat()') itp('works with NULL lists')`, spec line 1084: a
/// NULL operand is the empty list, and two NULLs answer a NULL list.
#[test]
fn concatenating_with_a_null_list_copies_the_other_one() {
    let log = AllocLog::start();
    // SAFETY: every value here is this case's own.
    unsafe {
        let l = tv::new_list(&[f(1.0), Tv::Dict(vec![])]);
        log.clear();
        let d = (*(*l).lv_last).li_tv.vval.v_dict;
        assert_eq!((*l).lv_refcount.get(), 1);
        assert_eq!((*d).dv_refcount.get(), 1);

        let mut refs = 1;
        let mut results = Vec::new();
        for (l1, l2) in [(ptr::null_mut(), l), (l, ptr::null_mut())] {
            let mut rettv = Tv::Unknown.build();
            assert_eq!(tv_list_concat(l1, l2, &raw mut rettv), Ok(()));
            assert_eq!((*l).lv_refcount.get(), 1);
            assert_eq!(rettv.v_type, VAR_LIST);
            assert_eq!(tv::read(&raw const rettv), Tv::List(vec![f(1.0), DICT]));
            let out = rettv.vval.v_list;
            assert_eq!((*out).lv_refcount.get(), 1);
            log.check(&[
                alloc::list(out),
                alloc::li((*out).lv_first),
                alloc::li((*out).lv_last),
            ]);
            refs += 1;
            assert_eq!((*d).dv_refcount.get(), refs);
            results.push(rettv);
        }

        let mut rettv = Tv::Unknown.build();
        assert_eq!(
            tv_list_concat(ptr::null_mut(), ptr::null_mut(), &raw mut rettv),
            Ok(())
        );
        assert_eq!(rettv.v_type, VAR_LIST);
        assert_eq!(tv::read(&raw const rettv), Tv::NullList);
        log.check(&[]);

        for mut rettv in results {
            tv_clear(&raw mut rettv);
        }
        tv_list_free(l);
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
        let d = (*(*l1).lv_last).li_tv.vval.v_dict;
        let inner = (*(*l2).lv_last).li_tv.vval.v_list;
        assert_eq!(((*l1).lv_refcount.get(), (*d).dv_refcount.get()), (1, 1));
        assert_eq!(
            ((*l2).lv_refcount.get(), (*inner).lv_refcount.get()),
            (1, 1)
        );
        log.clear();

        let mut rettv = Tv::Unknown.build();
        assert_eq!(tv_list_concat(l1, l2, &raw mut rettv), Ok(()));
        assert_eq!(((*l1).lv_refcount.get(), (*d).dv_refcount.get()), (1, 2));
        assert_eq!(
            ((*l2).lv_refcount.get(), (*inner).lv_refcount.get()),
            (1, 2)
        );
        let out = rettv.vval.v_list;
        let items = tv::list_items(out);
        log.check(&[
            alloc::list(out),
            alloc::li(items[0]),
            alloc::li(items[1]),
            alloc::li(items[2]),
            alloc::li(items[3]),
        ]);
        assert_eq!(
            tv::read(&raw const rettv),
            Tv::List(vec![f(1.0), DICT, f(3.0), LIST])
        );

        tv_clear(&raw mut rettv);
        tv_list_free(l1);
        tv_list_free(l2);
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
        let d = (*(*l).lv_last).li_tv.vval.v_dict;
        assert_eq!(((*l).lv_refcount.get(), (*d).dv_refcount.get()), (1, 1));
        log.clear();

        let mut rettv = Tv::Unknown.build();
        assert_eq!(tv_list_concat(l, l, &raw mut rettv), Ok(()));
        assert_eq!(((*l).lv_refcount.get(), (*d).dv_refcount.get()), (1, 3));
        let out = rettv.vval.v_list;
        let items = tv::list_items(out);
        log.check(&[
            alloc::list(out),
            alloc::li(items[0]),
            alloc::li(items[1]),
            alloc::li(items[2]),
            alloc::li(items[3]),
        ]);
        assert_eq!(
            tv::read(&raw const rettv),
            Tv::List(vec![f(1.0), DICT, f(1.0), DICT])
        );

        tv_clear(&raw mut rettv);
        tv_list_free(l);
    }
}

/// The same `describe`'s `itp('can concatenate empty non-NULL lists')`,
/// spec line 1165: an empty operand costs only the answer's own header.
#[test]
fn concatenating_empty_lists_allocates_only_the_answer() {
    let log = AllocLog::start();
    // SAFETY: as above.
    unsafe {
        let l = tv::new_list(&[f(1.0), Tv::Dict(vec![])]);
        let le = tv::new_list(&[]);
        let le2 = tv::new_list(&[]);
        let d = (*(*l).lv_last).li_tv.vval.v_dict;
        log.clear();

        let mut kept = Vec::new();
        for (l1, l2, refs) in [(l, le, 2), (le, l, 3)] {
            let mut rettv = Tv::Unknown.build();
            assert_eq!(tv_list_concat(l1, l2, &raw mut rettv), Ok(()));
            assert_eq!(((*l).lv_refcount.get(), (*d).dv_refcount.get()), (1, refs));
            assert_eq!(((*le).lv_refcount.get(), (*le2).lv_refcount.get()), (1, 1));
            let out = rettv.vval.v_list;
            log.check(&[
                alloc::list(out),
                alloc::li((*out).lv_first),
                alloc::li((*out).lv_last),
            ]);
            assert_eq!(tv::read(&raw const rettv), Tv::List(vec![f(1.0), DICT]));
            kept.push(rettv);
        }

        for (l1, l2) in [(le, le), (le, le2)] {
            let mut rettv = Tv::Unknown.build();
            assert_eq!(tv_list_concat(l1, l2, &raw mut rettv), Ok(()));
            assert_eq!(((*l).lv_refcount.get(), (*d).dv_refcount.get()), (1, 3));
            log.check(&[alloc::list(rettv.vval.v_list)]);
            assert_eq!(tv::read(&raw const rettv), Tv::List(vec![]));
            kept.push(rettv);
        }

        for mut rettv in kept {
            tv_clear(&raw mut rettv);
        }
        tv_list_free(l);
        tv_list_free(le);
        tv_list_free(le2);
    }
}

// ---------------------------------------------------------------- join

/// `describe('join()') itp('works')`, spec line 1236.
#[test]
fn joining_a_list_renders_every_item() {
    let log = AllocLog::start();
    // SAFETY: each list and its growarray are this case's own.
    unsafe {
        let join = |l: *mut list_T, sep: &str| -> String {
            let mut ga = tv::ga_alloc(1, 80);
            assert_eq!(tv_list_join(&raw mut ga, l, cstr(sep).as_ptr()), Ok(()));
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
        tv_list_free(l);

        let l = tv::new_list(&[Tv::s("boo")]);
        assert_eq!(join(l, " "), "boo");
        tv_list_free(l);

        let l = tv::new_list(&[]);
        assert_eq!(join(l, " "), "");
        tv_list_free(l);

        let l = tv::new_list(&[Tv::Dict(vec![]), Tv::s("far")]);
        assert_eq!(join(l, " "), "{} far");
        tv_list_free(l);

        // A recursive list renders as the marker `string()` uses, not by
        // looping.
        let l = tv::new_list(&[Tv::List(vec![Tv::Cycle(1)]), Tv::s("far")]);
        assert_eq!(join(l, " "), "[[...@0]] far");
        let recursive = (*(*l).lv_first).li_tv.vval.v_list;
        tv_list_item_remove(recursive, (*recursive).lv_first);
        tv_list_free(l);

        log.clear();
    }
}

// --------------------------------------------------------------- equal

/// The nine lists the two `tv_list_equal` cases compare against the first.
///
/// # Safety
/// The editor must be up; the caller frees them.
unsafe fn equality_corpus() -> Vec<*mut list_T> {
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
        let null = ptr::null_mut();

        for ic in [true, false] {
            assert!(tv_list_equal(l, null, ic));
            assert!(tv_list_equal(null, l, ic));
            assert!(tv_list_equal(null, null, ic));
            assert!(tv_list_equal(l, l, ic));
            assert!(tv_list_equal(l, l2, ic));
            assert!(tv_list_equal(l2, l, ic));
        }

        tv_list_free(l);
        tv_list_free(l2);
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
            assert_eq!(tv_list_equal(ls[0], ls[i], false), exact, "exact, list {i}");
            assert_eq!(
                tv_list_equal(ls[0], ls[i], true),
                folded,
                "folded, list {i}"
            );
        }
        for l in ls {
            tv_list_free(l);
        }
    }
}

// ---------------------------------------------------------------- find

/// `describe('find') describe('()') itp('correctly indexes list')`, spec
/// line 1326.
///
/// `tv_list_find` caches the last index it answered on the list, so the
/// case walks the same indexes with the cache warm and with it cleared.
#[test]
fn finding_an_item_by_index_works_from_either_end() {
    let log = AllocLog::start();
    // SAFETY: the list is this case's own.
    unsafe {
        let l = tv::new_list(&floats(1..=5));
        let lis = tv::list_items(l);
        log.clear();

        for n in [-1, 0, 1] {
            assert!(tv_list_find(ptr::null_mut(), n).is_null());
        }
        assert!(tv_list_find(l, 5).is_null(), "past the end");
        assert!(tv_list_find(l, -6).is_null(), "before the start");

        // Warm cache: every lookup after the first walks from `lv_idx_item`.
        let walk = [(-5, 0), (4, 4), (2, 2), (-3, 2), (2, 2), (2, 2), (-3, 2)];
        for (n, at) in walk {
            assert_eq!(tv_list_find(l, n), lis[at], "index {n}");
        }
        // Cold cache: the same answers with the cache cleared each time.
        for (n, at) in walk {
            (*l).lv_idx_item = ptr::null_mut();
            assert_eq!(tv_list_find(l, n), lis[at], "index {n}, cold");
        }

        (*l).lv_idx_item = ptr::null_mut();
        for (n, at) in [
            (2, 2),
            (-5, 0),
            (2, 2),
            (4, 4),
            (2, 2),
            (2, 2),
            (2, 2),
            (-3, 2),
            (2, 2),
            (2, 2),
            (2, 2),
            (-3, 2),
        ] {
            assert_eq!(tv_list_find(l, n), lis[at], "index {n}");
        }

        log.check(&[]);
        tv_list_free(l);
    }
}

/// The `find > nr()` group, spec lines 1385–1454, in one case per shape.
#[test]
fn finding_a_number_by_index_reads_through_strings() {
    let log = AllocLog::start();
    // SAFETY: every list is this case's own.
    unsafe {
        let find_nr = |l: *mut list_T, n: c_int, msg: Option<&str>| -> (bool, i64) {
            let mut err = false;
            let ret = check_emsg(log.editor(), || tv_list_find_nr(l, n, &raw mut err), msg);
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
            tv_list_free(l);
        }

        // A NULL string is zero, not an error.
        let l = tv::new_list(&[Tv::NullStr]);
        log.clear();
        assert_eq!(find_nr(l, 0, None), (false, 0));
        log.check(&[]);
        tv_list_free(l);

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
        tv_list_free(l);

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
        tv_list_free(l);
    }
}

/// The `find > str()` group, spec lines 1454–1502.
#[test]
fn finding_a_string_by_index_renders_scalars() {
    let log = AllocLog::start();
    // SAFETY: every list is this case's own; the answer is borrowed.
    unsafe {
        let find_str = |l: *mut list_T, n: c_int, msg: Option<&str>| -> Option<String> {
            let mut numbuf = NumBuf::new();
            let ret = check_emsg(log.editor(), || tv_list_find_str(l, n, &mut numbuf), msg);
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
        tv_list_free(l);

        // A string item is answered in place.
        let l = tv::new_list(&(1..=5).map(|n| Tv::s(n.to_string())).collect::<Vec<_>>());
        log.clear();
        for (n, want) in [(-5, "1"), (4, "5"), (2, "3"), (-3, "3")] {
            assert_eq!(find_str(l, n, None).as_deref(), Some(want));
        }
        log.check(&[]);
        tv_list_free(l);

        // A NULL string reads as empty.
        let l = tv::new_list(&[Tv::NullStr]);
        log.clear();
        assert_eq!(find_str(l, 0, None).as_deref(), Some(""));
        log.check(&[]);
        tv_list_free(l);

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
        tv_list_free(l);

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
        tv_list_free(l);
    }
}

/// `describe('idx_of_item()') itp('works')`, spec line 1502.
#[test]
fn an_items_index_is_minus_one_when_it_is_not_in_the_list() {
    let _log = AllocLog::start();
    // SAFETY: both lists are this case's own.
    unsafe {
        let l = tv::new_list(&floats(1..=5));
        let l2 = tv::new_list(&[f(42.0), Tv::List(vec![])]);
        let lis = tv::list_items(l);
        let lis2 = tv::list_items(l2);

        for (i, &li) in lis.iter().enumerate() {
            assert_eq!(tv_list_idx_of_item(l, li), c_int::try_from(i).unwrap());
        }
        assert_eq!(tv_list_idx_of_item(l, lis2[0]), -1);
        assert_eq!(tv_list_idx_of_item(l, ptr::null()), -1);
        assert_eq!(tv_list_idx_of_item(ptr::null(), ptr::null()), -1);
        assert_eq!(tv_list_idx_of_item(ptr::null(), lis[0]), -1);

        tv_list_free(l);
        tv_list_free(l2);
    }
}

/// The list allocator answers a zeroed header with one reference and no
/// items — the shape every case above starts from.
#[test]
fn a_fresh_list_is_empty_and_unreferenced() {
    let log = AllocLog::start();
    // SAFETY: the list is this case's own.
    unsafe {
        let l = tv_list_alloc(0);
        log.check(&[alloc::list(l)]);
        assert!((*l).lv_first.is_null());
        assert!((*l).lv_last.is_null());
        assert_eq!((*l).lv_refcount.get(), 0);
        assert_eq!((*l).lv_lock, VarLock::Unlocked);
        tv_list_free(l);
        log.check(&[alloc::freed(l)]);
    }
}
