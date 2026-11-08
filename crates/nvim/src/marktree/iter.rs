#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

//! Walking the tree.
//!
//! `MarkTreeIter` is a cursor: the node and index it currently names, plus the
//! path it took to get there (`path[depth]`, one entry per level) and the
//! absolute position of that node, since positions in the tree are stored
//! relative to the enclosing key. Callers all over the editor hold one across
//! an edit; it names nodes by [`NodeId`], so a stale one reads a wrong key at
//! worst.
//!
//! Three walks share the machinery:
//!
//! * the plain one, [`marktree_itr_next`]/[`marktree_itr_prev`];
//! * the filtered one, which reads a node's meta counts and skips any subtree
//!   that holds none of the decoration kinds the caller asked for;
//! * the overlap walk, which answers "which ranges cover this position" by
//!   reading the intersection set on each node of the path down.

use crate::narrow::as_index;
use crate::narrow::count_as_int;
use core::ffi::c_int;

use crate::decoration::defs::{DecorHighlightInline, DecorInline};
use crate::marktree::key::{
    MARKTREE_END_FLAG, MT_INVALID_KEY, MtFlags, compose, mt_end, mt_lookup_id, mt_start,
    mtpair_from, pos_leq, pos_less, relative, unrelative,
};
use crate::marktree::meta::{MetaCount, filtered_key_flags, meta_has};
use crate::marktree::node::{NodeId, find_key};
use crate::marktree::{MTKey, MTPair, MTPos, MarkTree, MarkTreeIter};
use crate::os::libc::int32_t;

use super::marktree_lookup;

/// Nested to keep the name out of the flat cdef namespace `ffigen` builds,
/// the same reason node.rs nests its own sizes.
mod sizes {
    /// The deepest the tree can get: the C's `MT_MAX_DEPTH`, which is also the
    /// length of `MarkTreeIter::path` and so the range `MarkTreeIter::depth`
    /// indexes. It bounds an `oldbase` array for the same reason.
    pub const MT_MAX_DEPTH: usize = 20;
}
pub use sizes::MT_MAX_DEPTH;

/// A bare key at `pos`, for [`find_key`] to compare against.
///
/// Only the position and the flags take part in the ordering, so the
/// decoration is left zeroed — deliberately *not*
/// `DECOR_HIGHLIGHT_INLINE_INIT`, whose priority is not zero.
fn search_key(pos: MTPos, flags: MtFlags) -> MTKey {
    MTKey {
        pos,
        ns: 0,
        id: 0,
        flags,
        decor_data: DecorInline::Hl(DecorHighlightInline {
            flags: 0,
            priority: 0,
            hl_id: 0,
            conceal_char: 0,
        }),
    }
}

/// The iterator's path entry at its own depth.
#[inline(always)]
fn here(iter: &mut MarkTreeIter) -> &mut crate::marktree::PathStep {
    &mut iter.path[as_index(iter.depth)]
}

/// Position `iter` at the first key at or after (row, col).
pub fn marktree_itr_get(
    tree: &mut MarkTree,
    row: int32_t,
    col: c_int,
    iter: &mut MarkTreeIter,
) -> bool {
    let p = MTPos { row, col };
    marktree_itr_get_ext(tree, p, iter, false, false, None, None)
}

/// Position `iter` at `p`, with every knob the family has.
///
/// `last` asks for the key *before* `p` instead of the one at or after it,
/// `gravity` breaks a tie at `p` on the right-gravity side, `oldbase` records
/// the absolute position of every node on the way down, and `filter` stops the
/// descent at the first subtree holding none of the wanted decoration kinds.
pub fn marktree_itr_get_ext(
    tree: &mut MarkTree,
    p: MTPos,
    iter: &mut MarkTreeIter,
    last: bool,
    gravity: bool,
    mut oldbase: Option<&mut [MTPos; MT_MAX_DEPTH]>,
    filter: Option<&MetaCount>,
) -> bool {
    let Some(root) = tree.root.filter(|_| tree.n_keys != 0) else {
        iter.node = None;
        return false;
    };
    let mut k = search_key(p, MtFlags::RIGHT_GRAVITY.when(gravity));
    if last && !gravity {
        k.flags = MtFlags::LAST;
    }
    iter.pos = MTPos::default();
    let mut x = root;
    iter.node = Some(x);
    iter.depth = 0;
    if let Some(oldbase) = oldbase.as_deref_mut() {
        oldbase[as_index(iter.depth)] = iter.pos;
    }
    loop {
        let node = &tree[x];
        iter.index = find_key(node.keys(), k).0 + 1;
        if node.is_leaf() {
            break;
        }
        if let Some(filter) = filter
            && !meta_has(node.child_meta(as_index(iter.index)), filter)
        {
            break;
        }
        let old_col = iter.pos.col;
        let index = iter.index;
        let step = here(iter);
        step.index = index;
        step.old_col = old_col;
        if iter.index > 0 {
            let sep = node.key(as_index(iter.index) - 1).pos;
            compose(&mut iter.pos, sep);
            relative(sep, &mut k.pos);
        }
        x = node.child(as_index(iter.index));
        iter.node = Some(x);
        iter.depth += 1;
        if let Some(oldbase) = oldbase.as_deref_mut() {
            oldbase[as_index(iter.depth)] = iter.pos;
        }
    }
    if last {
        marktree_itr_prev(tree, iter)
    } else if iter.index >= count_as_int(tree[x].key_count()) {
        marktree_itr_next_skip(tree, iter, true, false, None, None)
    } else {
        true
    }
}

/// Position `iter` at the tree's first key.
pub fn marktree_itr_first(tree: &mut MarkTree, iter: &mut MarkTreeIter) -> bool {
    let Some(root) = tree.root.filter(|_| tree.n_keys != 0) else {
        iter.node = None;
        return false;
    };
    let mut x = root;
    iter.node = Some(x);
    iter.index = 0;
    iter.depth = 0;
    iter.pos = MTPos::default();
    while !tree[x].is_leaf() {
        let step = here(iter);
        step.index = 0;
        step.old_col = 0;
        iter.depth += 1;
        x = tree[x].child(0);
        iter.node = Some(x);
    }
    true
}

/// Step to the next key.
pub fn marktree_itr_next(tree: &mut MarkTree, iter: &mut MarkTreeIter) -> bool {
    marktree_itr_next_skip(tree, iter, false, false, None, None)
}

/// Step to the next key, optionally skipping the rest of the current subtree.
///
/// `skip` leaves the node the iterator is on instead of descending into it,
/// and `filter` sets `skip` by itself for a subtree holding none of the wanted
/// decoration kinds. `preload` stops one level short of a leaf, leaving
/// `iter.index` at -1, which is what the pair walk wants. `oldbase` records
/// where each level started.
#[inline]
pub fn marktree_itr_next_skip(
    tree: &MarkTree,
    iter: &mut MarkTreeIter,
    mut skip: bool,
    preload: bool,
    mut oldbase: Option<&mut [MTPos; MT_MAX_DEPTH]>,
    filter: Option<&MetaCount>,
) -> bool {
    let Some(mut x) = iter.node else {
        return false;
    };
    let mut node = &tree[x];
    iter.index += 1;
    if let Some(filter) = filter
        && !node.is_leaf()
        && !meta_has(node.child_meta(as_index(iter.index)), filter)
    {
        skip = true;
    }
    if node.is_leaf() || skip {
        if preload && node.is_leaf() && skip {
            iter.index = count_as_int(node.key_count());
        } else if iter.index < count_as_int(node.key_count()) {
            return true;
        }
        // Walk up until a node still has a key left after the one we came out
        // of, undoing that level's contribution to the absolute position.
        while iter.index >= count_as_int(node.key_count()) {
            let Some(parent) = node.parent() else {
                iter.node = None;
                return false;
            };
            x = parent;
            node = &tree[x];
            iter.node = Some(x);
            iter.depth -= 1;
            let step = *here(iter);
            iter.index = step.index;
            if iter.index > 0 {
                iter.pos.row -= node.key(as_index(iter.index) - 1).pos.row;
                iter.pos.col = step.old_col;
            }
        }
    } else {
        // The key we stepped onto is an internal one, so the next key in order
        // is the leftmost of the subtree between it and the previous key.
        while !node.is_leaf() {
            if iter.index > 0 {
                here(iter).old_col = iter.pos.col;
                let sep = node.key(as_index(iter.index) - 1).pos;
                compose(&mut iter.pos, sep);
            }
            if iter.index == 0
                && let Some(oldbase) = oldbase.as_deref_mut()
            {
                oldbase[as_index(iter.depth) + 1] = oldbase[as_index(iter.depth)];
            }
            here(iter).index = iter.index;
            let child = node.child(as_index(iter.index));
            debug_assert!(
                tree[child].parent() == Some(x),
                "itr->x->ptr[itr->i]->parent == itr->x"
            );
            iter.depth += 1;
            x = child;
            node = &tree[x];
            iter.node = Some(x);
            if preload && !node.is_leaf() {
                iter.index = -1;
                break;
            }
            iter.index = 0;
            if let Some(filter) = filter
                && !node.is_leaf()
                && !meta_has(node.child_meta(0), filter)
            {
                break;
            }
        }
    }
    true
}

/// Position `iter` at the first key at or after (row, col) that the filter
/// wants, giving up at (stop_row, stop_col).
pub fn marktree_itr_get_filter(
    tree: &mut MarkTree,
    row: int32_t,
    col: c_int,
    stop_row: c_int,
    stop_col: c_int,
    filter: &MetaCount,
    iter: &mut MarkTreeIter,
) -> bool {
    if !meta_has(&tree.meta_root, filter) {
        return false;
    }
    let p = MTPos { row, col };
    if !marktree_itr_get_ext(tree, p, iter, false, false, None, Some(filter)) {
        return false;
    }
    marktree_itr_check_filter(tree, iter, stop_row, stop_col, filter)
}

/// Leave the subtrees the filter has nothing in, stepping out of each one
/// whose parent says so, and answer whether the iterator still names a node.
pub fn marktree_itr_step_out_filter(
    tree: &mut MarkTree,
    iter: &mut MarkTreeIter,
    filter: &MetaCount,
) -> bool {
    if !meta_has(&tree.meta_root, filter) {
        iter.node = None;
        return false;
    }
    while let Some(x) = iter.node {
        let Some(parent) = tree[x].parent() else {
            break;
        };
        if meta_has(tree[parent].child_meta(tree[x].parent_index()), filter) {
            return true;
        }
        iter.index = count_as_int(tree[x].key_count());
        marktree_itr_next_skip(tree, iter, true, false, None, None);
    }
    iter.is_positioned()
}

/// Step to the next key the filter wants, giving up at (stop_row, stop_col).
pub fn marktree_itr_next_filter(
    tree: &mut MarkTree,
    iter: &mut MarkTreeIter,
    stop_row: c_int,
    stop_col: c_int,
    filter: &MetaCount,
) -> bool {
    if !marktree_itr_next_skip(tree, iter, false, false, None, Some(filter)) {
        return false;
    }
    marktree_itr_check_filter(tree, iter, stop_row, stop_col, filter)
}

/// Advance until the iterator is on a key the filter actually wants — the meta
/// counts only promise that the *subtree* holds one — or past the stop
/// position, which empties the iterator.
fn marktree_itr_check_filter(
    tree: &mut MarkTree,
    iter: &mut MarkTreeIter,
    stop_row: c_int,
    stop_col: c_int,
    filter: &MetaCount,
) -> bool {
    let stop_pos = MTPos {
        row: stop_row,
        col: stop_col,
    };
    let key_filter = filtered_key_flags(filter);
    loop {
        let pos = marktree_itr_pos(tree, iter);
        if pos_leq(stop_pos, pos) {
            iter.node = None;
            return false;
        }
        let k = raw_key(tree, iter);
        if !mt_end(k) && k.flags.has(key_filter) {
            return true;
        }
        if !marktree_itr_next_skip(tree, iter, false, false, None, Some(filter)) {
            return false;
        }
    }
}

/// Step to the previous key.
pub fn marktree_itr_prev(tree: &MarkTree, iter: &mut MarkTreeIter) -> bool {
    let Some(mut x) = iter.node else {
        return false;
    };
    let mut node = &tree[x];
    if node.is_leaf() {
        iter.index -= 1;
        if iter.index >= 0 {
            return true;
        }
        // Walk up until a node still has a key before the one we came out of.
        while iter.index < 0 {
            let Some(parent) = node.parent() else {
                iter.node = None;
                return false;
            };
            x = parent;
            node = &tree[x];
            iter.node = Some(x);
            iter.depth -= 1;
            let step = *here(iter);
            iter.index = step.index - 1;
            if iter.index >= 0 {
                iter.pos.row -= node.key(as_index(iter.index)).pos.row;
                iter.pos.col = step.old_col;
            }
        }
    } else {
        // The previous key is the rightmost of the subtree before this one.
        while !node.is_leaf() {
            if iter.index > 0 {
                here(iter).old_col = iter.pos.col;
                let sep = node.key(as_index(iter.index) - 1).pos;
                compose(&mut iter.pos, sep);
            }
            here(iter).index = iter.index;
            let child = node.child(as_index(iter.index));
            debug_assert!(
                tree[child].parent() == Some(x),
                "itr->x->ptr[itr->i]->parent == itr->x"
            );
            x = child;
            node = &tree[x];
            iter.node = Some(x);
            iter.index = count_as_int(node.key_count());
            iter.depth += 1;
        }
        iter.index -= 1;
    }
    true
}

/// The key a positioned `iter` stands on, still relative to its node.
///
/// # Panics
/// If the iterator is not positioned.
#[inline]
pub(crate) fn raw_key(tree: &MarkTree, iter: &MarkTreeIter) -> MTKey {
    let node = iter.node.expect("a positioned iterator");
    tree[node].key(as_index(iter.index))
}

/// The absolute position of the key `iter` is on.
pub fn marktree_itr_pos(tree: &MarkTree, iter: &MarkTreeIter) -> MTPos {
    let mut pos = raw_key(tree, iter).pos;
    unrelative(iter.pos, &mut pos);
    pos
}

/// The key `iter` is on, at its absolute position, or [`MT_INVALID_KEY`] once
/// the walk has run off the end.
pub fn marktree_itr_current(tree: &MarkTree, iter: &MarkTreeIter) -> MTKey {
    if !iter.is_positioned() {
        return MT_INVALID_KEY;
    }
    let mut key = raw_key(tree, iter);
    unrelative(iter.pos, &mut key.pos);
    key
}

/// Position the iterator to enumerate the ranges overlapping (row, col).
///
/// Follow it with [`marktree_itr_step_overlap`] until that returns false.
///
/// To get everything overlapping a *region* rather than a point: run this loop
/// for the region's start, then keep calling [`marktree_itr_next`] until the
/// iterator passes the region's end, taking the start halves (and unpaired
/// marks) and skipping the end halves.
///
/// Answers false when no mark can possibly be found. True is not a promise:
/// the first `step_overlap` may still answer false.
pub fn marktree_itr_get_overlap(
    tree: &mut MarkTree,
    row: c_int,
    col: c_int,
    iter: &mut MarkTreeIter,
) -> bool {
    if tree.n_keys == 0 {
        iter.node = None;
        return false;
    }
    iter.node = tree.root;
    iter.index = -1;
    iter.depth = 0;
    iter.pos = MTPos::default();
    iter.intersect_pos = MTPos { row, col };
    iter.intersect_pos_x = MTPos { row, col };
    iter.intersect_idx = 0;
    true
}

/// Yield one more range overlapping the position
/// [`marktree_itr_get_overlap`] was given.
///
/// Two phases. First, walk from the root towards the node holding the sought
/// position, and at each node on the way hand back every id in its intersection
/// set — those are exactly the ranges covering the whole of that subtree, so
/// they cover the position without being stored anywhere near it. Second, once
/// the leaf is reached, scan it for the ends of ranges that started before the
/// position, which the sets do not record because such a range does not cover
/// its own leaf entirely.
///
/// Answers false once every overlapping pair has been handed back, at which
/// point the iterator is an ordinary one positioned at (row, col).
pub fn marktree_itr_step_overlap(
    tree: &mut MarkTree,
    iter: &mut MarkTreeIter,
    pair: &mut MTPair,
) -> bool {
    let Some(mut x) = iter.node else {
        return false;
    };
    // Phase one: down the path, handing back each node's covering ranges.
    while iter.index == -1 {
        let set = tree[x].intersection();
        if iter.intersect_idx < set.len() {
            let id = set.as_slice()[iter.intersect_idx];
            iter.intersect_idx += 1;
            let halves = (
                marktree_lookup(tree, id, None),
                marktree_lookup(tree, id | MARKTREE_END_FLAG, None),
            );
            *pair = mtpair_from(halves.0, halves.1);
            return true;
        }
        if tree[x].is_leaf() {
            iter.index = 0;
            here(iter).index = iter.index;
            break;
        }
        let k = search_key(iter.intersect_pos_x, MtFlags::NONE);
        iter.index = find_key(tree[x].keys(), k).0 + 1;
        let (index, old_col) = (iter.index, iter.pos.col);
        let step = here(iter);
        step.index = index;
        step.old_col = old_col;
        if iter.index > 0 {
            let sep = tree[x].key(as_index(iter.index) - 1).pos;
            compose(&mut iter.pos, sep);
            relative(sep, &mut iter.intersect_pos_x);
        }
        x = tree[x].child(as_index(iter.index));
        iter.node = Some(x);
        iter.depth += 1;
        iter.index = -1;
        iter.intersect_idx = 0;
    }
    // Phase two, first half: starts in this leaf that are before the sought
    // position and whose end is not.
    while iter.index < count_as_int(tree[x].key_count())
        && pos_less(tree[x].key(as_index(iter.index)).pos, iter.intersect_pos_x)
    {
        let mut k = tree[x].key(as_index(iter.index));
        iter.index += 1;
        here(iter).index = iter.index;
        if !mt_start(k) {
            continue;
        }
        let end = marktree_lookup(tree, mt_lookup_id(k.ns, k.id, true), None);
        if pos_less(end.pos, iter.intersect_pos) {
            continue;
        }
        unrelative(iter.pos, &mut k.pos);
        *pair = mtpair_from(k, end);
        return true;
    }
    // Second half: ends in this leaf whose start is in another node, which is
    // the case the intersection sets cannot record.
    while iter.index < count_as_int(tree[x].key_count()) {
        let mut k = tree[x].key(as_index(iter.index));
        iter.index += 1;
        if !mt_end(k) {
            continue;
        }
        let id = mt_lookup_id(k.ns, k.id, false);
        if tree.id2node(id) == Some(x) {
            continue;
        }
        unrelative(iter.pos, &mut k.pos);
        let start = marktree_lookup(tree, id, None);
        if pos_leq(iter.intersect_pos, start.pos) {
            continue;
        }
        *pair = mtpair_from(start, k);
        return true;
    }
    iter.index = here(iter).index;
    debug_assert!(iter.index >= 0, "itr->i >= 0");
    if iter.index >= count_as_int(tree[x].key_count()) {
        marktree_itr_next(tree, iter);
    }
    false
}

/// Park `iter`, where given, on key `i` of node `n`, and answer that key at its
/// absolute position.
///
/// The position comes from walking `n`'s ancestors, which is also where the
/// iterator's path is rebuilt, so this works for a node reached any way at all
/// — an id-map lookup, or a node a splice recorded earlier.
pub fn marktree_itr_set_node(
    tree: &MarkTree,
    iter: Option<&mut MarkTreeIter>,
    mut n: NodeId,
    i: c_int,
) -> MTKey {
    let mut key = tree[n].key(as_index(i));
    let root_level = tree.root_level().expect("a tree holding `n` has a root");
    let mut iter = iter;
    if let Some(iter) = iter.as_deref_mut() {
        iter.index = i;
        iter.node = Some(n);
        iter.depth = count_as_int(root_level - tree[n].level());
    }
    let mut node = &tree[n];
    while let Some(p) = node.parent() {
        let i = node.parent_index();
        let parent = &tree[p];
        debug_assert!(parent.child(i) == n, "p->ptr[i] == n");
        if let Some(iter) = iter.as_deref_mut() {
            iter.path[root_level - parent.level()].index = count_as_int(i);
        }
        if i > 0 {
            unrelative(parent.key(i - 1).pos, &mut key.pos);
        }
        (n, node) = (p, parent);
    }
    if let Some(iter) = iter {
        marktree_itr_fix_pos(tree, iter);
    }
    key
}

/// Recompute `iter.pos` — and the `old_col` of every level — by walking the
/// path the iterator recorded, after something moved the keys it was rebased
/// on.
#[inline]
pub fn marktree_itr_fix_pos(tree: &MarkTree, iter: &mut MarkTreeIter) {
    let mut pos = MTPos::default();
    let mut x = tree.root.expect("a positioned iterator's tree has a root");
    for step in &mut iter.path[..as_index(iter.depth)] {
        step.old_col = pos.col;
        let node = &tree[x];
        let i = as_index(step.index);
        if i > 0 {
            compose(&mut pos, node.key(i - 1).pos);
        }
        debug_assert!(!node.is_leaf(), "x->level");
        x = node.child(i);
    }
    iter.pos = pos;
    debug_assert!(Some(x) == iter.node, "x == itr->x");
}
