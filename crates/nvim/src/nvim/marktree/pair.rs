#![deny(unsafe_op_in_unsafe_fn)]

//! Paired marks: the second key, and the records of what a range covers.
//!
//! A range is two keys sharing a `(ns, id)`, the later one carrying
//! `MT_FLAG_END`. Everything that keeps the two halves consistent with each
//! other, and with the covering records the nodes between them carry, lives
//! here; the parent module owns the tree's per-key operations.
//!
//! The covering records are the reason this is not just bookkeeping. A node
//! whose whole span a range covers records that range's id in its
//! [intersection set](super::intersect), and a node whose *parent* already
//! records it does not — so a range spanning a million lines is written into a
//! handful of nodes rather than a million. [`marktree_intersect_pair`] is the
//! walk that establishes (or, with `delete`, retracts) exactly that set of
//! records, and [`pseudo_index_for_id`] is how the rebalancer decides which
//! side of a moving boundary a half sits on without walking to it.

use core::ffi::c_int;
use core::{ptr, slice};

use crate::src::nvim::marktree::intersect::{IdSet, intersect_mov};
use crate::src::nvim::marktree::iter::marktree_itr_next_skip;
use crate::src::nvim::marktree::key::{
    MARKTREE_END_FLAG, MT_FLAG_ORPHANED, mt_end, mt_lookup_key, mt_lookup_key_side, mt_paired,
};
use crate::src::nvim::marktree::node::{Node, id2node};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::types::{
    Intersection, MTKey, MTPos, MarkTree, MarkTreeIter, size_t, uint16_t, uint64_t,
};

use super::{marktree_lookup, marktree_lookup_ns};

/// Record that the range `id` covers the whole of `x`.
pub fn intersect_node(x: Node, id: uint64_t) {
    debug_assert!(id & MARKTREE_END_FLAG == 0, "!(id & MARKTREE_END_FLAG)");
    x.intersection().insert_sorted(id);
}

/// Drop that record. `strict` asserts the id was there to drop.
pub fn unintersect_node(x: Node, id: uint64_t, strict: bool) {
    debug_assert!(id & MARKTREE_END_FLAG == 0, "!(id & MARKTREE_END_FLAG)");
    x.intersection().remove(id, strict);
}

/// The index `itr` holds at level `lvl`, plus `q` where that is the level the
/// iterator is actually on — the C's `iat(itr, l, q)` macro.
///
/// The two iterators of a pair are compared level by level, and only the one
/// standing *on* a level has a meaningful `i` there; for every level above it
/// the answer is the step it took on the way down.
fn iat(itr: &MarkTreeIter, lvl: c_int, q: c_int) -> c_int {
    if lvl == itr.lvl {
        itr.i + q
    } else {
        itr.s[lvl as usize].i
    }
}

/// Record (or, with `delete`, unrecord) that the range `id` covers the nodes
/// between its two halves.
///
/// Walks up from the start half and down to the end half, marking every node
/// that the range covers *entirely*: a node whose parent the range also covers
/// is left alone, because the parent's record already implies it. That is what
/// keeps a range spanning a million lines out of a million nodes' sets.
///
/// `itr` is left wherever the walk ended; `end_itr` is only read.
///
/// # Safety
/// `b` must be a live tree and both iterators positioned in it.
pub unsafe fn marktree_intersect_pair(
    b: &mut MarkTree,
    id: uint64_t,
    itr: &mut MarkTreeIter,
    end_itr: &MarkTreeIter,
    delete: bool,
) {
    let mut lvl = 0;
    let maxlvl = itr.lvl.min(end_itr.lvl);
    while lvl < maxlvl {
        if itr.s[lvl as usize].i > end_itr.s[lvl as usize].i {
            return; // empty range
        } else if itr.s[lvl as usize].i < end_itr.s[lvl as usize].i {
            break; // work to do
        }
        lvl += 1;
    }
    if lvl == maxlvl && iat(itr, lvl, 1) > iat(end_itr, lvl, 0) {
        return; // empty range
    }

    while !itr.x.is_null() {
        // SAFETY: a positioned iterator names a live node of `b`.
        let x = unsafe { Node::new(itr.x) };
        let skip = if itr.x == end_itr.x {
            if x.is_leaf() || itr.i >= end_itr.i {
                break;
            }
            true
        } else if itr.lvl > lvl {
            true
        } else if iat(itr, lvl, 1) < iat(end_itr, lvl, 1) {
            true
        } else {
            lvl += 1;
            false
        };
        // Stepping over a subtree is exactly the case where the range covers
        // the whole of it, so that is where the record goes.
        if skip && !x.is_leaf() {
            let covered = x.child((itr.i + 1) as usize);
            if delete {
                unintersect_node(covered, id, true);
            } else {
                intersect_node(covered, id);
            }
        }
        // SAFETY: `b` is live and `itr` is positioned in it; neither optional
        // out-parameter is wanted.
        unsafe { marktree_itr_next_skip(b, itr, skip, true, ptr::null_mut(), ptr::null()) };
    }
}

/// Copy `set` into the caller's `out` buffer and record its length.
///
/// # Safety
/// `out` must have room for `set.len()` ids and `n_out` must be live.
unsafe fn answer(set: &IdSet, out: *mut uint64_t, n_out: *mut size_t) {
    // SAFETY: the caller promises `out` has room for the whole set.
    unsafe { ptr::copy_nonoverlapping(set.as_slice().as_ptr(), out, set.len()) };
    // SAFETY: `n_out` is a live out-parameter per the caller.
    unsafe { *n_out = set.len() };
}

/// `intersect_mov` over four caller-owned arrays, for `marktree_spec.lua`.
///
/// Answers false, writing nothing, when either result is longer than the
/// buffer offered for it.
///
/// # Safety
/// `x`, `y` and `win` must name `nx`, `ny` and `nwin` ids; `wout` and `dout`
/// must have room for the counts `nwout` and `ndout` arrive holding, and all
/// four out-parameters must be live.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn intersect_mov_test(
    x: *const uint64_t,
    nx: size_t,
    y: *const uint64_t,
    ny: size_t,
    win: *const uint64_t,
    nwin: size_t,
    wout: *mut uint64_t,
    nwout: *mut size_t,
    dout: *mut uint64_t,
    ndout: *mut size_t,
) -> bool {
    // x is immutable as far as intersect_mov is concerned, and y may shrink —
    // whatever it loses shows up in d. Neither is ever grown, so borrowing the
    // caller's arrays as sets is enough.
    let mut xs = borrowed(x, nx);
    let mut ys = borrowed(y, ny);
    let mut ws = borrowed(ptr::null(), 0);
    let mut ds = borrowed(ptr::null(), 0);
    // SAFETY: each of the four names a live `Intersection` local of this frame,
    // and no other view of any of them exists.
    let (xs, ys) = unsafe { (IdSet::new(&raw mut xs), IdSet::new(&raw mut ys)) };
    // SAFETY: as above.
    let (ws, ds) = unsafe { (IdSet::new(&raw mut ws), IdSet::new(&raw mut ds)) };
    ws.init();
    ds.init();
    // SAFETY: `win` names `nwin` ids per the caller.
    ws.extend_from_slice(unsafe { slice::from_raw_parts(win, nwin) });

    intersect_mov(&xs, &ys, &ws, &ds);

    // SAFETY: both counts are live out-parameters, arriving as capacities.
    let (wcap, dcap) = unsafe { (*nwout, *ndout) };
    let fits = ws.len() <= wcap && ds.len() <= dcap;
    if fits {
        // SAFETY: `fits` is exactly the promise `answer` needs about `wout`.
        unsafe { answer(&ws, wout, nwout) };
        // SAFETY: and about `dout`.
        unsafe { answer(&ds, dout, ndout) };
    }
    // SAFETY: `take_heap` answers the set's own buffer, or null if it never
    // left its inline array; either is `xfree`-able exactly once.
    unsafe { xfree(ws.take_heap()) };
    // SAFETY: as above.
    unsafe { xfree(ds.take_heap()) };
    fits
}

/// An `Intersection` header over a caller's array, with no capacity of its own
/// so that nothing ever tries to grow or free it.
fn borrowed(items: *const uint64_t, size: size_t) -> Intersection {
    Intersection {
        size,
        capacity: 0,
        items: items.cast_mut(),
        init_array: [0; 4],
    }
}

/// Re-record the intersections for the pair `key` belongs to, after one of its
/// halves has been re-inserted.
///
/// # Safety
/// `b` must be a live tree.
pub unsafe fn marktree_restore_pair(b: &mut MarkTree, key: MTKey) {
    let mut itr = MarkTreeIter::default();
    let mut end_itr = MarkTreeIter::default();
    // SAFETY: `b` is live; a lookup only writes the iterator it is handed.
    unsafe { marktree_lookup(b, mt_lookup_key_side(key, false), Some(&mut itr)) };
    // SAFETY: as above.
    unsafe { marktree_lookup(b, mt_lookup_key_side(key, true), Some(&mut end_itr)) };
    if itr.x.is_null() || end_itr.x.is_null() {
        // The other end is waiting to be restored later; this runs again for it.
        return;
    }
    // SAFETY: a lookup that found its key left the iterator on a live node.
    let (start, end) = unsafe { (Node::new(itr.x), Node::new(end_itr.x)) };
    start.update_key(itr.i as usize, |k| {
        k.flags &= !(MT_FLAG_ORPHANED as uint16_t)
    });
    end.update_key(end_itr.i as usize, |k| {
        k.flags &= !(MT_FLAG_ORPHANED as uint16_t)
    });

    let id = mt_lookup_key_side(key, false);
    // SAFETY: `b` is live and both iterators are positioned in it.
    unsafe { marktree_intersect_pair(b, id, &mut itr, &end_itr, false) };
}

/// An ordering key for where the mark `id` sits in the tree, or zero if there
/// is no such mark — a valid pseudo-index is never zero.
///
/// With `sloppy`, two keys in the same *leaf* share an index; the callers that
/// pass it only need to know which side of a node boundary the mark is on.
///
/// # Safety
/// `b` must be a live tree.
pub unsafe fn pseudo_index_for_id(b: &mut MarkTree, id: uint64_t, sloppy: bool) -> uint64_t {
    // SAFETY: `b` is live, so `id2node` answers null or one of its live nodes.
    let Some(n) = (unsafe { Node::from_ptr(id2node(b, id)) }) else {
        return 0;
    };
    let mut i = 0;
    if !n.is_leaf() || !sloppy {
        while i < n.key_count() {
            if mt_lookup_key(n.key(i)) == id {
                break;
            }
            i += 1;
        }
        debug_assert!(i < n.key_count(), "i < n->n");
        if !n.is_leaf() {
            i += 1; // an internal key `i` comes after child `i`
        }
    }
    n.pseudo_index(i as c_int)
}

/// Where the other half of `mark`'s pair sits, or `mark`'s own position if it
/// is unpaired.
///
/// # Safety
/// As [`marktree_get_alt`].
pub unsafe fn marktree_get_altpos(
    b: &mut MarkTree,
    mark: MTKey,
    itr: Option<&mut MarkTreeIter>,
) -> MTPos {
    // SAFETY: the caller's promise, passed straight on.
    unsafe { marktree_get_alt(b, mark, itr) }.pos
}

/// The other half of `mark`'s pair, or `mark` itself if it is unpaired.
///
/// # Safety
/// `b` must be a live tree and `mark` a key read out of it.
pub unsafe fn marktree_get_alt(
    b: &mut MarkTree,
    mark: MTKey,
    itr: Option<&mut MarkTreeIter>,
) -> MTKey {
    if mt_paired(mark) {
        // SAFETY: `b` is live; the iterator is optional and is written, not read.
        unsafe { marktree_lookup_ns(b, mark.ns, mark.id, !mt_end(mark), itr) }
    } else {
        mark
    }
}
