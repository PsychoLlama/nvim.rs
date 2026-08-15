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

use crate::src::nvim::marktree::intersect::{IdSet, intersect_mov};
use crate::src::nvim::marktree::iter::marktree_itr_next_skip;
use crate::src::nvim::marktree::key::{
    MARKTREE_END_FLAG, MT_FLAG_ORPHANED, mt_end, mt_lookup_key, mt_lookup_key_side, mt_paired,
};
use crate::src::nvim::marktree::node::{id2node, inner, pseudo_index};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::types::{
    Intersection, MTKey, MTNode, MTPos, MarkTree, MarkTreeIter, MarkTreeIter_s as C2Rust_Unnamed_2,
    int32_t, size_t, uint16_t, uint32_t, uint64_t,
};

use super::{ix, marktree_lookup, marktree_lookup_ns};

/// Record that the range `id` covers the whole of `x`.
pub(crate) fn intersect_node(x: *mut MTNode, id: uint64_t) {
    debug_assert!(id & MARKTREE_END_FLAG == 0, "!(id & MARKTREE_END_FLAG)");
    unsafe { ix(x) }.insert_sorted(id);
}

/// Drop that record. `strict` asserts the id was there to drop.
pub(crate) fn unintersect_node(x: *mut MTNode, id: uint64_t, strict: bool) {
    debug_assert!(id & MARKTREE_END_FLAG == 0, "!(id & MARKTREE_END_FLAG)");
    unsafe { ix(x) }.remove(id, strict);
}

/// Record (or, with `delete`, unrecord) that the range `id` covers the nodes
/// between its two halves.
///
/// Walks up from the start half and down to the end half, marking every node
/// that the range covers *entirely*: a node whose parent the range also covers
/// is left alone, because the parent's record already implies it. That is what
/// keeps a range spanning a million lines out of a million nodes' sets.
///
/// `itr` is mutated; `end_itr` is not.
pub unsafe extern "C" fn marktree_intersect_pair(
    mut b: *mut MarkTree,
    mut id: uint64_t,
    mut itr: *mut MarkTreeIter,
    mut end_itr: *mut MarkTreeIter,
    mut delete: bool,
) {
    let mut lvl: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    let mut maxlvl: ::core::ffi::c_int = if (*itr).lvl < (*end_itr).lvl {
        (*itr).lvl
    } else {
        (*end_itr).lvl
    };
    while lvl < maxlvl {
        if (*itr).s[lvl as usize].i > (*end_itr).s[lvl as usize].i {
            return;
        } else {
            if (*itr).s[lvl as usize].i < (*end_itr).s[lvl as usize].i {
                break;
            }
            lvl += 1;
        }
    }
    if lvl == maxlvl
        && (if lvl == (*itr).lvl {
            (*itr).i + 1 as ::core::ffi::c_int
        } else {
            (*itr).s[lvl as usize].i
        }) > (if lvl == (*end_itr).lvl {
            (*end_itr).i + 0 as ::core::ffi::c_int
        } else {
            (*end_itr).s[lvl as usize].i
        })
    {
        return;
    }
    while !(*itr).x.is_null() {
        let mut skip: bool = false;
        if (*itr).x == (*end_itr).x {
            if (*(*itr).x).level as ::core::ffi::c_int == 0 as ::core::ffi::c_int
                || (*itr).i >= (*end_itr).i
            {
                break;
            }
            skip = true;
        } else if (*itr).lvl > lvl {
            skip = true;
        } else if (if lvl == (*itr).lvl {
            (*itr).i + 1 as ::core::ffi::c_int
        } else {
            (*itr).s[lvl as usize].i
        }) < (if lvl == (*end_itr).lvl {
            (*end_itr).i + 1 as ::core::ffi::c_int
        } else {
            (*end_itr).s[lvl as usize].i
        }) {
            skip = true;
        } else {
            lvl += 1;
        }
        if skip {
            if (*(*itr).x).level != 0 {
                let mut x: *mut MTNode =
                    (*inner((*itr).x)).i_ptr[((*itr).i + 1 as ::core::ffi::c_int) as usize];
                if delete {
                    unintersect_node(x, id, true);
                } else {
                    intersect_node(x, id);
                }
            }
        }
        marktree_itr_next_skip(
            b,
            itr,
            skip,
            true,
            ::core::ptr::null_mut::<MTPos>(),
            ::core::ptr::null::<uint32_t>(),
        );
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn intersect_mov_test(
    mut x: *const uint64_t,
    mut nx: size_t,
    mut y: *const uint64_t,
    mut ny: size_t,
    mut win: *const uint64_t,
    mut nwin: size_t,
    mut wout: *mut uint64_t,
    mut nwout: *mut size_t,
    mut dout: *mut uint64_t,
    mut ndout: *mut size_t,
) -> bool {
    // x is immutable as far as intersect_mov is concerned, and y may shrink —
    // whatever it loses shows up in d. Neither is ever grown, so borrowing the
    // caller's arrays as sets is enough.
    let mut xs = Intersection {
        size: nx,
        capacity: 0,
        items: x as *mut uint64_t,
        init_array: [0; 4],
    };
    let mut ys = Intersection {
        size: ny,
        capacity: 0,
        items: y as *mut uint64_t,
        init_array: [0; 4],
    };
    let mut ws = Intersection {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut(),
        init_array: [0; 4],
    };
    let mut ds = Intersection {
        size: 0,
        capacity: 0,
        items: ::core::ptr::null_mut(),
        init_array: [0; 4],
    };
    let (xs, ys) = (IdSet::new(&raw mut xs), IdSet::new(&raw mut ys));
    let (ws, ds) = (IdSet::new(&raw mut ws), IdSet::new(&raw mut ds));
    ws.init();
    ds.init();
    ws.extend_from_slice(::core::slice::from_raw_parts(win, nwin));

    intersect_mov(&xs, &ys, &ws, &ds);

    let fits = ws.len() <= *nwout && ds.len() <= *ndout;
    if fits {
        ::core::ptr::copy_nonoverlapping(ws.as_slice().as_ptr(), wout, ws.len());
        *nwout = ws.len();
        ::core::ptr::copy_nonoverlapping(ds.as_slice().as_ptr(), dout, ds.len());
        *ndout = ds.len();
    }
    xfree(ws.take_heap());
    xfree(ds.take_heap());
    return fits;
}

/// Re-record the intersections for the pair `key` belongs to, after one of its
/// halves has been re-inserted.
pub unsafe extern "C" fn marktree_restore_pair(mut b: *mut MarkTree, mut key: MTKey) {
    let mut itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_2 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }; 1];
    let mut end_itr: [MarkTreeIter; 1] = [MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        lvl: 0,
        x: ::core::ptr::null_mut::<MTNode>(),
        i: 0,
        s: [C2Rust_Unnamed_2 { oldcol: 0, i: 0 }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    }; 1];
    marktree_lookup(
        b,
        mt_lookup_key_side(key, false),
        &raw mut itr as *mut MarkTreeIter,
    );
    marktree_lookup(
        b,
        mt_lookup_key_side(key, true),
        &raw mut end_itr as *mut MarkTreeIter,
    );
    if (*(&raw mut itr as *mut MarkTreeIter)).x.is_null()
        || (*(&raw mut end_itr as *mut MarkTreeIter)).x.is_null()
    {
        return;
    }
    (*(*(&raw mut itr as *mut MarkTreeIter)).x).key
        [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
        .flags = ((*(*(&raw mut itr as *mut MarkTreeIter)).x).key
        [(*(&raw mut itr as *mut MarkTreeIter)).i as usize]
        .flags as ::core::ffi::c_int
        & !MT_FLAG_ORPHANED as uint16_t as ::core::ffi::c_int) as uint16_t;
    (*(*(&raw mut end_itr as *mut MarkTreeIter)).x).key
        [(*(&raw mut end_itr as *mut MarkTreeIter)).i as usize]
        .flags = ((*(*(&raw mut end_itr as *mut MarkTreeIter)).x).key
        [(*(&raw mut end_itr as *mut MarkTreeIter)).i as usize]
        .flags as ::core::ffi::c_int
        & !MT_FLAG_ORPHANED as uint16_t as ::core::ffi::c_int) as uint16_t;
    marktree_intersect_pair(
        b,
        mt_lookup_key_side(key, false),
        &raw mut itr as *mut MarkTreeIter,
        &raw mut end_itr as *mut MarkTreeIter,
        false,
    );
}

pub unsafe extern "C" fn pseudo_index_for_id(
    mut b: *mut MarkTree,
    mut id: uint64_t,
    mut sloppy: bool,
) -> uint64_t {
    let mut n: *mut MTNode = id2node(b, id);
    if n.is_null() {
        return 0 as uint64_t;
    }
    let mut i: ::core::ffi::c_int = 0 as ::core::ffi::c_int;
    if (*n).level as ::core::ffi::c_int != 0 || !sloppy {
        i = 0 as ::core::ffi::c_int;
        while (i as int32_t) < (*n).n {
            if mt_lookup_key((*n).key[i as usize]) == id {
                break;
            }
            i += 1;
        }
        debug_assert!((i as int32_t) < (*n).n, "i < n->n");
        if (*n).level != 0 {
            i += 1 as ::core::ffi::c_int;
        }
    }
    return pseudo_index(n, i);
}

pub unsafe extern "C" fn marktree_get_altpos(
    mut b: *mut MarkTree,
    mut mark: MTKey,
    mut itr: *mut MarkTreeIter,
) -> MTPos {
    return marktree_get_alt(b, mark, itr).pos;
}
pub unsafe extern "C" fn marktree_get_alt(
    mut b: *mut MarkTree,
    mut mark: MTKey,
    mut itr: *mut MarkTreeIter,
) -> MTKey {
    return if mt_paired(mark) as ::core::ffi::c_int != 0 {
        marktree_lookup_ns(b, mark.ns, mark.id, !mt_end(mark), itr)
    } else {
        mark
    };
}
