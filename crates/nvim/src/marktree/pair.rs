#![forbid(unsafe_code)]

//! Paired marks: the second key, and the records of what a range covers.
//!
//! A range is two keys sharing a `(ns, id)`, the later one carrying
//! [`MtFlags::END`]. Everything that keeps the two halves consistent with each
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

use crate::marktree::iter::marktree_itr_next_skip;
use crate::marktree::key::{
    MARKTREE_END_FLAG, MtFlags, mt_end, mt_lookup_key, mt_lookup_key_side, mt_paired,
};
use crate::marktree::node::NodeId;
use crate::types::{MTKey, MTPos, MarkTree, MarkTreeIter, uint64_t};

use super::{marktree_lookup, marktree_lookup_ns};

/// Record that the range `id` covers the whole of `x`.
pub fn intersect_node(tree: &mut MarkTree, x: NodeId, id: uint64_t) {
    debug_assert!(id & MARKTREE_END_FLAG == 0, "!(id & MARKTREE_END_FLAG)");
    tree[x].intersection_mut().insert_sorted(id);
}

/// Drop that record. `strict` asserts the id was there to drop.
pub fn unintersect_node(tree: &mut MarkTree, x: NodeId, id: uint64_t, strict: bool) {
    debug_assert!(id & MARKTREE_END_FLAG == 0, "!(id & MARKTREE_END_FLAG)");
    tree[x].intersection_mut().remove(id, strict);
}

/// The index `itr` holds at level `lvl`, plus `q` where that is the level the
/// iterator is actually on — the C's `iat(itr, l, q)` macro.
///
/// The two iterators of a pair are compared level by level, and only the one
/// standing *on* a level has a meaningful `i` there; for every level above it
/// the answer is the step it took on the way down.
fn iat(iter: &MarkTreeIter, depth: c_int, q: c_int) -> c_int {
    if depth == iter.depth {
        iter.index + q
    } else {
        iter.path[depth as usize].index
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
pub fn marktree_intersect_pair(
    tree: &mut MarkTree,
    id: uint64_t,
    iter: &mut MarkTreeIter,
    end_iter: &MarkTreeIter,
    delete: bool,
) {
    let mut depth = 0;
    let max_depth = iter.depth.min(end_iter.depth);
    while depth < max_depth {
        let (here, there) = (
            iter.path[depth as usize].index,
            end_iter.path[depth as usize].index,
        );
        if here > there {
            return; // empty range
        } else if here < there {
            break; // work to do
        }
        depth += 1;
    }
    if depth == max_depth && iat(iter, depth, 1) > iat(end_iter, depth, 0) {
        return; // empty range
    }

    while let Some(x) = iter.node {
        let skip = if iter.node == end_iter.node {
            if tree[x].is_leaf() || iter.index >= end_iter.index {
                break;
            }
            true
        } else if iter.depth > depth || iat(iter, depth, 1) < iat(end_iter, depth, 1) {
            true
        } else {
            depth += 1;
            false
        };
        // Stepping over a subtree is exactly the case where the range covers
        // the whole of it, so that is where the record goes.
        if skip && !tree[x].is_leaf() {
            let covered = tree[x].child((iter.index + 1) as usize);
            if delete {
                unintersect_node(tree, covered, id, true);
            } else {
                intersect_node(tree, covered, id);
            }
        }
        marktree_itr_next_skip(tree, iter, skip, true, None, None);
    }
}

/// Re-record the intersections for the pair `key` belongs to, after one of its
/// halves has been re-inserted.
pub fn marktree_restore_pair(tree: &mut MarkTree, key: MTKey) {
    let mut iter = MarkTreeIter::default();
    let mut end_iter = MarkTreeIter::default();
    marktree_lookup(tree, mt_lookup_key_side(key, false), Some(&mut iter));
    marktree_lookup(tree, mt_lookup_key_side(key, true), Some(&mut end_iter));
    if !iter.is_positioned() || !end_iter.is_positioned() {
        // The other end is waiting to be restored later; this runs again for it.
        return;
    }
    tree.key_mut(&iter).flags.clear(MtFlags::ORPHANED);
    tree.key_mut(&end_iter).flags.clear(MtFlags::ORPHANED);

    let id = mt_lookup_key_side(key, false);
    marktree_intersect_pair(tree, id, &mut iter, &end_iter, false);
}

/// An ordering key for where the mark `id` sits in the tree, or zero if there
/// is no such mark — a valid pseudo-index is never zero.
///
/// With `sloppy`, two keys in the same *leaf* share an index; the callers that
/// pass it only need to know which side of a node boundary the mark is on.
pub fn pseudo_index_for_id(tree: &MarkTree, id: uint64_t, sloppy: bool) -> uint64_t {
    let Some(n) = tree.id2node(id) else {
        return 0;
    };
    let node = &tree[n];
    let mut i = 0;
    if !node.is_leaf() || !sloppy {
        while i < node.key_count() {
            if mt_lookup_key(node.key(i)) == id {
                break;
            }
            i += 1;
        }
        debug_assert!(i < node.key_count(), "i < n->n");
        if !node.is_leaf() {
            i += 1; // an internal key `i` comes after child `i`
        }
    }
    tree.pseudo_index(n, i)
}

/// Where the other half of `mark`'s pair sits, or `mark`'s own position if it
/// is unpaired.
pub fn marktree_get_altpos(
    tree: &mut MarkTree,
    mark: MTKey,
    iter: Option<&mut MarkTreeIter>,
) -> MTPos {
    marktree_get_alt(tree, mark, iter).pos
}

/// The other half of `mark`'s pair, or `mark` itself if it is unpaired.
pub fn marktree_get_alt(
    tree: &mut MarkTree,
    mark: MTKey,
    iter: Option<&mut MarkTreeIter>,
) -> MTKey {
    if mt_paired(mark) {
        marktree_lookup_ns(tree, mark.ns, mark.id, !mt_end(mark), iter)
    } else {
        mark
    }
}
