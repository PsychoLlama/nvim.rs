#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! Keeping the tree balanced across an insertion or a deletion.
//!
//! A node holds between `MT_BRANCH_FACTOR - 1` and `2 * MT_BRANCH_FACTOR - 1`
//! keys. Insertion splits a full child on the way down; deletion borrows from a
//! sibling ([`pivot_left`]/[`pivot_right`]) or merges with one
//! ([`merge_node`]) on the way back up.
//!
//! What makes this more than a textbook B-tree is that every key position is
//! stored relative to the key before it, and every node carries meta counts and
//! a set of covering ranges. So each of these operations has three jobs at
//! once: move the keys, rebase the positions of everything on either side of
//! the boundary that moved, and re-home the meta counts and the intersection
//! sets. Getting any one of the three wrong shows up as a corrupt tree several
//! operations later, which is what `marktree_check` exists to catch.
//!
//! # How this file reaches the tree
//!
//! Every entry point takes a `&mut MarkTree` and the [`NodeId`]s of the nodes
//! it works on, and reaches each node as `tree[x]`. A step that combines two
//! nodes' intersection sets takes them out of the nodes, works on the owned
//! sets, and puts the results back.
//!
//! One deviation from the C is worth stating, because it is deliberate and
//! repeated. The C sets `x->n` at the *end* of an operation, after it has
//! finished shuffling children whose indices already run past the old count.
//! `Node::child` checks its index against the count, so this file writes the
//! new count *before* those loops instead, keeping the count in step with the
//! children rather than with the keys. Nothing reads a node's count between the
//! two points — no callback runs and no other node refers to it — so the tree
//! every caller sees afterwards is byte-for-byte the one the C leaves.

use core::mem;

use crate::marktree::intersect::{
    IdSet, intersect_add, intersect_common, intersect_merge, intersect_mov, intersect_sub,
};
use crate::marktree::key::{
    MARKTREE_END_FLAG, MT_BRANCH_FACTOR, key_cmp, mt_end, mt_lookup_id, mt_lookup_key,
    mt_lookup_key_side, mt_start, relative, unrelative,
};
use crate::marktree::meta::{MetaCount, meta_add, meta_describe_key, meta_sub};
use crate::marktree::node::{MAX_CHILDREN, MAX_KEYS, NodeId, find_key};
use crate::marktree::pair::{intersect_node, pseudo_index_for_id, unintersect_node};
use crate::marktree::{MTKey, MTPos, MarkTree};
use crate::os::libc::uint64_t;

/// Nested to keep the name out of the flat cdef namespace `ffigen` builds,
/// the same reason node.rs nests its own sizes.
mod sizes {
    use super::MT_BRANCH_FACTOR;

    /// The branch factor, as the index type the rest of this file counts in. A
    /// node splits at [`MAX_KEYS`](super::MAX_KEYS) keys and each half keeps
    /// `T - 1` of them.
    pub(super) const T: usize = MT_BRANCH_FACTOR as usize;
}
use sizes::T;

/// Where the mark `id` sits, to within a leaf — see [`pseudo_index_for_id`].
/// Every caller here only needs to know which side of a moving node boundary
/// the mark is on.
fn sloppy_index(tree: &MarkTree, id: uint64_t) -> uint64_t {
    pseudo_index_for_id(tree, id, true)
}

/// Split the full child `i` of `x` in two, moving its middle key up into `x`.
///
/// `next` is the key on its way in, and is needed only for the case where a
/// range's start half is already in the tree and its end half is not: that id
/// must not be recorded as covering the new half yet, since
/// [`marktree_intersect_pair`](super::marktree_intersect_pair) records it once
/// both halves are in.
pub fn split_node(tree: &mut MarkTree, x: NodeId, i: usize, next: MTKey) {
    let y = tree[x].child(i);
    let z = tree.alloc_node(tree[y].level());
    tree[z].set_key_count(T - 1);
    let last_start = if mt_end(next) {
        mt_lookup_id(next.ns, next.id, false)
    } else {
        MARKTREE_END_FLAG
    };

    // z inherits everything y intersected: the split does not change which
    // ranges cover either half.
    let inherited = tree[y].intersection().clone();
    *tree[z].intersection_mut() = inherited;

    if tree[y].is_leaf() {
        let pi = tree.pseudo_index(y, 0); // note: sloppy pseudo-index
        for j in 0..T {
            let k = tree[y].key(j);
            let pi_end = sloppy_index(tree, mt_lookup_id(k.ns, k.id, true));
            if mt_start(k) && pi_end > pi && mt_lookup_key(k) != last_start {
                intersect_node(tree, z, mt_lookup_id(k.ns, k.id, false));
            }
        }
        // note: y's key at `T - 1` moves up, and so is checked for both halves
        for j in T - 1..MAX_KEYS {
            let k = tree[y].key(j);
            let pi_start = sloppy_index(tree, mt_lookup_id(k.ns, k.id, false));
            if mt_end(k) && pi_start > 0 && pi_start < pi {
                intersect_node(tree, y, mt_lookup_id(k.ns, k.id, false));
            }
        }
    }

    tree.copy_keys_between(z, 0, y, T..MAX_KEYS);
    for j in 0..T - 1 {
        tree.refkey(z, j);
    }
    if !tree[y].is_leaf() {
        tree.copy_children_between(z, 0, y, T..MAX_CHILDREN);
        tree.reparent_children(z, 0..T);
    }
    tree[y].set_key_count(T - 1);

    let n = tree[x].key_count();
    let z_meta = tree[z].meta();
    let parent = &mut tree[x];
    parent.copy_children_within(i + 1..n + 1, i + 2);
    parent.set_child(i + 1, z);
    *parent.child_meta_mut(i + 1) = z_meta;
    // The separating key is not in place yet, but every child is: see the
    // module docs on why the count moves here rather than after the keys.
    parent.set_key_count(n + 1);
    tree[z].set_parent(Some(x)); // == y's parent
    tree.reparent_children(x, i + 1..n + 2);
    tree[x].copy_keys_within(i..n, i + 1);

    // Move y's middle key up to the internal layer.
    let middle = tree[y].key(T - 1);
    tree[x].set_key(i, middle);
    tree.refkey(x, i);

    // y used to contain all of z and the key just moved up; discount those.
    let parent = &mut tree[x];
    let meta_inc = meta_describe_key(parent.key(i));
    let moved = *parent.child_meta(i + 1);
    meta_sub(parent.child_meta_mut(i), &moved);
    meta_sub(parent.child_meta_mut(i), &meta_inc);

    let base = parent.key(i).pos;
    if i > 0 {
        let before = parent.key(i - 1).pos;
        unrelative(before, &mut parent.key_mut(i).pos);
    }
    for j in 0..T - 1 {
        relative(base, &mut tree[z].key_mut(j).pos);
    }

    if !tree[y].is_leaf() {
        bubble_up(tree, y);
        bubble_up(tree, z);
    }
}

/// Merge child `i + 1` of `p` into child `i`, with the key that separated them
/// between the two halves, and answer the node that is left.
pub fn merge_node(tree: &mut MarkTree, p: NodeId, i: usize) -> NodeId {
    let x = tree[p].child(i);
    let y = tree[p].child(i + 1);
    let (xn, yn) = (tree[x].key_count(), tree[y].key_count());

    // What x and y both intersected becomes the merged node's own set; what
    // only one of them did stays on that half's keys.
    let mut merged = IdSet::EMPTY;
    let mut x_only = mem::take(tree[x].intersection_mut());
    let mut y_only = mem::take(tree[y].intersection_mut());
    intersect_merge(&mut merged, &mut x_only, &mut y_only);

    let separator = tree[p].key(i);
    tree[x].set_key(xn, separator);
    tree.refkey(x, xn);
    if i > 0 {
        let base = tree[p].key(i - 1).pos;
        relative(base, &mut tree[x].key_mut(xn).pos);
    }
    let meta_inc = meta_describe_key(tree[x].key(xn));

    tree.copy_keys_between(x, xn + 1, y, 0..yn);
    let base = tree[x].key(xn).pos;
    for k in 0..yn {
        tree.refkey(x, xn + 1 + k);
        unrelative(base, &mut tree[x].key_mut(xn + 1 + k).pos);
    }

    // x now holds everything of y, plus the key that used to separate them.
    tree[x].set_key_count(xn + yn + 1);
    if !tree[x].is_leaf() {
        // Bubble down: ranges that intersected old-x but not old-y, or the
        // other way round, have to move to their respective children.
        tree.copy_children_between(x, xn + 1, y, 0..yn + 1);
        for k in 0..=xn {
            let child = tree[x].child(k);
            for &id in x_only.as_slice() {
                intersect_node(tree, child, id);
            }
        }
        for ky in 0..=yn {
            // The nodes that used to be y's, now the second half of x.
            let k = xn + ky + 1;
            let child = tree[x].child(k);
            tree[child].set_parent(Some(x));
            tree[child].set_parent_index(k);
            for &id in y_only.as_slice() {
                intersect_node(tree, child, id);
            }
        }
    }

    let parent = &mut tree[p];
    let absorbed = *parent.child_meta(i + 1);
    meta_add(parent.child_meta_mut(i), &absorbed);
    meta_add(parent.child_meta_mut(i), &meta_inc);

    let pn = parent.key_count();
    parent.copy_keys_within(i + 1..pn, i);
    parent.copy_children_within(i + 2..pn + 1, i + 1);
    // One child has gone; the survivors past it have all shifted down one.
    parent.set_key_count(pn - 1);
    tree.reparent_children(p, i + 1..pn);

    // y is detached from the tree now: nothing above still names it.
    tree.free_node(y);

    *tree[x].intersection_mut() = merged;
    x
}

/// Move one key from child `i` of `p` to child `i + 1`, through `p`.
pub fn pivot_right(tree: &mut MarkTree, _p_pos: MTPos, p: NodeId, i: usize) {
    let x = tree[p].child(i);
    let y = tree[p].child(i + 1);
    let (xn, yn) = (tree[x].key_count(), tree[y].key_count());

    tree[y].copy_keys_within(0..yn, 1);
    // y takes p's separating key at the front: see the module docs on why the
    // count moves here rather than after the children.
    tree[y].set_key_count(yn + 1);
    if !tree[y].is_leaf() {
        tree[y].copy_children_within(0..yn + 1, 1);
        tree.reparent_children(y, 1..yn + 2);
    }

    let separator = tree[p].key(i);
    tree[y].set_key(0, separator);
    tree.refkey(y, 0);
    let last = tree[x].key(xn - 1);
    tree[p].set_key(i, last);
    tree.refkey(p, i);

    let meta_inc_y = meta_describe_key(tree[y].key(0));
    let meta_inc_x = meta_describe_key(tree[p].key(i));
    meta_add(tree[p].child_meta_mut(i + 1), &meta_inc_y);
    meta_sub(tree[p].child_meta_mut(i), &meta_inc_x);

    if !tree[x].is_leaf() {
        // x's last child follows the key, and takes its counts with it.
        tree.copy_children_between(y, 0, x, xn..xn + 1);
        let moved = *tree[y].child_meta(0);
        meta_add(tree[p].child_meta_mut(i + 1), &moved);
        meta_sub(tree[p].child_meta_mut(i), &moved);
        let child = tree[y].child(0);
        tree[child].set_parent(Some(y));
        tree[child].set_parent_index(0);
    }
    tree[x].set_key_count(xn - 1);

    if i > 0 {
        let base = tree[p].key(i - 1).pos;
        unrelative(base, &mut tree[p].key_mut(i).pos);
    }
    let base = tree[p].key(i).pos;
    relative(base, &mut tree[y].key_mut(0).pos);
    let base = tree[y].key(0).pos;
    for k in 1..yn + 1 {
        unrelative(base, &mut tree[y].key_mut(k).pos);
    }

    // Repair x's intersections.
    if !tree[x].is_leaf() {
        // y's new first child moved across from x, so it has to take on the
        // difference between its two parents' sets; that in turn may push some
        // of old-y's ids down onto old-y's other children.
        let mut demoted = IdSet::EMPTY;
        let moved = tree[y].child(0);
        let mut y_set = mem::take(tree[y].intersection_mut());
        let mut moved_set = mem::take(tree[moved].intersection_mut());
        intersect_mov(
            tree[x].intersection(),
            &mut y_set,
            &mut moved_set,
            &mut demoted,
        );
        *tree[y].intersection_mut() = y_set;
        *tree[moved].intersection_mut() = moved_set;
        if !demoted.is_empty() {
            for yi in 1..tree[y].key_count() + 1 {
                let child = tree[y].child(yi);
                intersect_add(tree[child].intersection_mut(), &demoted);
            }
        }

        bubble_up(tree, x);
    } else {
        // If x's last key used to be an end key, see whether it now covers all
        // of x.
        let moved_up = tree[p].key(i);
        if mt_end(moved_up) {
            let pi = tree.pseudo_index(x, 0); // note: sloppy pseudo-index
            let start_id = mt_lookup_key_side(moved_up, false);
            let pi_start = sloppy_index(tree, start_id);
            if pi_start > 0 && pi_start < pi {
                intersect_node(tree, x, start_id);
            }
        }
        let first = tree[y].key(0);
        if mt_start(first) {
            // No check needed: just delete it if it was there.
            unintersect_node(tree, y, mt_lookup_key(first), false);
        }
    }
}

/// Move one key from child `i + 1` of `p` to child `i`, through `p`.
pub fn pivot_left(tree: &mut MarkTree, _p_pos: MTPos, p: NodeId, i: usize) {
    let x = tree[p].child(i);
    let y = tree[p].child(i + 1);
    let (xn, yn) = (tree[x].key_count(), tree[y].key_count());

    // Reverse of how we "always" do it — but pivot_left is just the inverse of
    // pivot_right, so reverse it literally.
    let base = tree[y].key(0).pos;
    for k in 1..yn {
        relative(base, &mut tree[y].key_mut(k).pos);
    }
    let base = tree[p].key(i).pos;
    unrelative(base, &mut tree[y].key_mut(0).pos);
    if i > 0 {
        let base = tree[p].key(i - 1).pos;
        relative(base, &mut tree[p].key_mut(i).pos);
    }

    let separator = tree[p].key(i);
    tree[x].set_key(xn, separator);
    tree.refkey(x, xn);
    let first = tree[y].key(0);
    tree[p].set_key(i, first);
    tree.refkey(p, i);

    let meta_inc_x = meta_describe_key(tree[x].key(xn));
    let meta_inc_y = meta_describe_key(tree[p].key(i));
    meta_add(tree[p].child_meta_mut(i), &meta_inc_x);
    meta_sub(tree[p].child_meta_mut(i + 1), &meta_inc_y);

    if !tree[x].is_leaf() {
        // y's first child follows the key, and takes its counts with it.
        let moved_child = tree[y].child(0);
        tree.copy_children_between(x, xn + 1, y, 0..1);
        let moved = *tree[y].child_meta(0);
        meta_sub(tree[p].child_meta_mut(i + 1), &moved);
        meta_add(tree[p].child_meta_mut(i), &moved);
        tree[moved_child].set_parent(Some(x));
        tree[moved_child].set_parent_index(xn + 1);
    }

    tree[y].copy_keys_within(1..yn, 0);
    // y gave its first key away: see the module docs on why the count moves
    // here rather than after the children.
    tree[y].set_key_count(yn - 1);
    if !tree[y].is_leaf() {
        tree[y].copy_children_within(1..yn + 1, 0);
        tree.reparent_children(y, 0..yn);
    }
    tree[x].set_key_count(xn + 1);

    // Repair x's and y's intersections.
    if !tree[x].is_leaf() {
        // x's new last child moved across from y, so it has to take on the
        // difference between its two parents' sets; that in turn may push some
        // of old-x's ids down onto old-x's other children.
        let mut demoted = IdSet::EMPTY;
        let n = tree[x].key_count();
        let moved = tree[x].child(n);
        let mut x_set = mem::take(tree[x].intersection_mut());
        let mut moved_set = mem::take(tree[moved].intersection_mut());
        intersect_mov(
            tree[y].intersection(),
            &mut x_set,
            &mut moved_set,
            &mut demoted,
        );
        *tree[x].intersection_mut() = x_set;
        *tree[moved].intersection_mut() = moved_set;
        if !demoted.is_empty() {
            // The child at `n` is deliberately skipped.
            for xi in 0..n {
                let child = tree[x].child(xi);
                intersect_add(tree[child].intersection_mut(), &demoted);
            }
        }

        bubble_up(tree, y);
    } else {
        // If y's first key used to be a start key, see whether it now covers
        // all of y.
        let moved_up = tree[p].key(i);
        if mt_start(moved_up) {
            let pi = tree.pseudo_index(y, 0); // note: sloppy pseudo-index
            let end_id = mt_lookup_key_side(moved_up, true);
            let pi_end = sloppy_index(tree, end_id);
            if pi_end > pi {
                intersect_node(tree, y, mt_lookup_key(moved_up));
            }
        }
        let last = tree[x].key(tree[x].key_count() - 1);
        if mt_end(last) {
            // No check needed: just delete it if it was there.
            unintersect_node(tree, x, mt_lookup_key_side(last, false), false);
        }
    }
}

/// `x` shrank, or is one half of a split. Ranges that used to cover every one
/// of its children now cover `x` itself, so hoist them one level.
fn bubble_up(tree: &mut MarkTree, x: NodeId) {
    let mut common = IdSet::EMPTY;
    let n = tree[x].key_count();
    // By the tree's invariants the largest subset of *all* the children is the
    // intersection of the first with the last.
    let (first, last) = (tree[x].child(0), tree[x].child(n));
    intersect_common(
        &mut common,
        tree[first].intersection(),
        tree[last].intersection(),
    );
    if !common.is_empty() {
        for i in 0..=n {
            let child = tree[x].child(i);
            intersect_sub(tree[child].intersection_mut(), &common);
        }
        intersect_add(tree[x].intersection_mut(), &common);
    }
}

/// Insert `k` into the subtree rooted at `x`, which must not be a full node
/// (even if there might be internal space).
#[inline]
pub fn marktree_putp_aux(tree: &mut MarkTree, x: NodeId, mut k: MTKey, meta_inc: &MetaCount) {
    // TODO(bfredl): ugh, make sure this is the _last_ valid (pos, gravity)
    // position, to minimize movement
    let node = &mut tree[x];
    let mut i =
        usize::try_from(find_key(node.keys(), k).0 + 1).expect("find_key answers -1 or more");
    if node.is_leaf() {
        let n = node.key_count();
        if i != n {
            node.copy_keys_within(i..n, i + 1);
        }
        node.set_key(i, k);
        node.set_key_count(n + 1);
        tree.refkey(x, i);
    } else {
        let child = node.child(i);
        if tree[child].key_count() == MAX_KEYS {
            split_node(tree, x, i, k);
            if key_cmp(k, tree[x].key(i)) > 0 {
                i += 1;
            }
        }
        let node = &tree[x];
        if i > 0 {
            relative(node.key(i - 1).pos, &mut k.pos);
        }
        let child = node.child(i);
        marktree_putp_aux(tree, child, k, meta_inc);
        meta_add(tree[x].child_meta_mut(i), meta_inc);
    }
}
