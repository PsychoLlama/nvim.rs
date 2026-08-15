#![deny(unsafe_op_in_unsafe_fn)]

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
//! Every entry point takes a [`Node`] and a `&mut MarkTree`, which is what
//! makes them safe functions: the promises the raw pointers used to carry are
//! discharged by [`Node::new`] at the tree's edge, so everything here is
//! ordinary checked code over the accessors. What is left unchecked is a
//! handful of one-line calls into the allocator, the id map and the scratch
//! sets — the operations a reference genuinely cannot describe.
//!
//! One deviation from the C is worth stating, because it is deliberate and
//! repeated. The C sets `x->n` at the *end* of an operation, after it has
//! finished shuffling children whose indices already run past the old count.
//! [`Node::child`] checks its index against the count, so this file writes the
//! new count *before* those loops instead, keeping the count in step with the
//! children rather than with the keys. Nothing reads a node's count between the
//! two points — no callback runs and no other node refers to it — so the tree
//! every caller sees afterwards is byte-for-byte the one the C leaves.

use core::ffi::c_int;
use core::ptr;

use crate::src::nvim::marktree::intersect::{
    IdSet, intersect_add, intersect_common, intersect_merge, intersect_mov, intersect_sub,
};
use crate::src::nvim::marktree::key::{
    MARKTREE_END_FLAG, MT_BRANCH_FACTOR, key_cmp, mt_end, mt_lookup_id, mt_lookup_key,
    mt_lookup_key_side, mt_start, relative, unrelative,
};
use crate::src::nvim::marktree::meta::{MetaCount, meta_add, meta_describe_key, meta_sub};
use crate::src::nvim::marktree::node::{
    INTERSECT_INLINE, MAX_CHILDREN, MAX_KEYS, Node, find_key, marktree_alloc_node,
    marktree_free_node, refkey,
};
use crate::src::nvim::marktree::pair::{intersect_node, pseudo_index_for_id, unintersect_node};
use crate::src::nvim::memory::xfree;
use crate::src::nvim::types::{Intersection, MTKey, MTPos, MarkTree, uint64_t};

/// Nested to keep the name out of the flat cdef namespace `ffigen` builds,
/// the same reason node.rs nests its own sizes.
mod sizes {
    use super::MT_BRANCH_FACTOR;

    /// The branch factor, as the index type the rest of this file counts in. A
    /// node splits at [`MAX_KEYS`](super::MAX_KEYS) keys and each half keeps
    /// `T - 1` of them.
    pub const T: usize = MT_BRANCH_FACTOR as usize;
}
use sizes::T;

/// Record that `x` now holds the key at `i`, so a lookup by id finds it.
fn rekey(b: &mut MarkTree, x: Node, i: usize) {
    // SAFETY: `b` is a live tree and `x` one of its live nodes; every caller
    // has just written the key at `i`.
    unsafe { refkey(b, x.as_ptr(), i as c_int) };
}

/// Where the mark `id` sits, to within a leaf — see [`pseudo_index_for_id`].
/// Every caller here only needs to know which side of a moving node boundary
/// the mark is on.
fn sloppy_index(b: &mut MarkTree, id: uint64_t) -> uint64_t {
    // SAFETY: `b` is a live tree.
    unsafe { pseudo_index_for_id(b, id, true) }
}

/// Storage for a set that lives no longer than one rebalancing step. Keep it
/// in a local of the operation that needs it and view it with [`scratch`].
fn scratch_storage() -> Intersection {
    Intersection {
        size: 0,
        capacity: 0,
        items: ptr::null_mut(),
        init_array: [0; INTERSECT_INLINE],
    }
}

/// An empty view of `storage`, which must not move, and must not be touched
/// other than through the view, for as long as the view is used.
fn scratch(storage: &mut Intersection) -> IdSet {
    // SAFETY: `storage` is a live `Intersection` owned by the caller's frame,
    // and the borrow proves this is the only view of it.
    let set = unsafe { IdSet::new(storage) };
    set.init();
    set
}

/// Give back whatever a set grew onto the heap, leaving it empty and inline.
fn free_heap(set: &IdSet) {
    // SAFETY: `take_heap` answers the set's own heap buffer, or null if it
    // never left its inline array; either is `xfree`-able exactly once.
    unsafe { xfree(set.take_heap()) };
}

/// Split the full child `i` of `x` in two, moving its middle key up into `x`.
///
/// `next` is the key on its way in, and is needed only for the case where a
/// range's start half is already in the tree and its end half is not: that id
/// must not be recorded as covering the new half yet, since
/// [`marktree_intersect_pair`](super::marktree_intersect_pair) records it once
/// both halves are in.
pub fn split_node(b: &mut MarkTree, x: Node, i: usize, next: MTKey) {
    let y = x.child(i);
    // SAFETY: `b` is a live tree, so a node freshly allocated into it is live.
    let z = unsafe { Node::new(marktree_alloc_node(b, !y.is_leaf())) };
    z.set_level(y.level());
    z.set_key_count(T - 1);
    let last_start = if mt_end(next) {
        mt_lookup_id(next.ns, next.id, false)
    } else {
        MARKTREE_END_FLAG
    };

    // z inherits everything y intersected: the split does not change which
    // ranges cover either half.
    z.intersection().clear();
    z.intersection()
        .extend_from_slice(y.intersection().as_slice());

    if y.is_leaf() {
        let pi = y.pseudo_index(0); // note: sloppy pseudo-index
        for j in 0..T {
            let k = y.key(j);
            let pi_end = sloppy_index(b, mt_lookup_id(k.ns, k.id, true));
            if mt_start(k) && pi_end > pi && mt_lookup_key(k) != last_start {
                intersect_node(z, mt_lookup_id(k.ns, k.id, false));
            }
        }
        // note: y's key at `T - 1` moves up, and so is checked for both halves
        for j in T - 1..MAX_KEYS {
            let k = y.key(j);
            let pi_start = sloppy_index(b, mt_lookup_id(k.ns, k.id, false));
            if mt_end(k) && pi_start > 0 && pi_start < pi {
                intersect_node(y, mt_lookup_id(k.ns, k.id, false));
            }
        }
    }

    z.copy_keys_from(0, y, T..MAX_KEYS);
    for j in 0..T - 1 {
        rekey(b, z, j);
    }
    if !y.is_leaf() {
        z.copy_children_from(0, y, T..MAX_CHILDREN);
        z.reparent_children(0..T);
    }
    y.set_key_count(T - 1);

    let n = x.key_count();
    x.copy_children_within(i + 1..n + 1, i + 2);
    x.set_child(i + 1, z);
    x.set_child_meta(i + 1, z.meta());
    z.set_parent(Some(x)); // == y's parent
    // The separating key is not in place yet, but every child is: see the
    // module docs on why the count moves here rather than after the keys.
    x.set_key_count(n + 1);
    x.reparent_children(i + 1..n + 2);
    x.copy_keys_within(i..n, i + 1);

    // Move y's middle key up to the internal layer.
    x.set_key(i, y.key(T - 1));
    rekey(b, x, i);

    // y used to contain all of z and the key just moved up; discount those.
    let meta_inc = meta_describe_key(x.key(i));
    let moved = x.child_meta(i + 1);
    x.update_child_meta(i, |m| meta_sub(m, &moved));
    x.update_child_meta(i, |m| meta_sub(m, &meta_inc));

    let base = x.key(i).pos;
    for j in 0..T - 1 {
        z.update_key(j, |k| relative(base, &mut k.pos));
    }
    if i > 0 {
        let base = x.key(i - 1).pos;
        x.update_key(i, |k| unrelative(base, &mut k.pos));
    }

    if !y.is_leaf() {
        bubble_up(y);
        bubble_up(z);
    }
}

/// Merge child `i + 1` of `p` into child `i`, with the key that separated them
/// between the two halves, and answer the node that is left.
pub fn merge_node(b: &mut MarkTree, p: Node, i: usize) -> Node {
    let x = p.child(i);
    let y = p.child(i + 1);
    let (xn, yn) = (x.key_count(), y.key_count());

    // What x and y both intersected becomes the merged node's own set; what
    // only one of them did stays on that half's keys.
    let mut storage = scratch_storage();
    let merged = scratch(&mut storage);
    intersect_merge(&merged, &x.intersection(), &y.intersection());

    x.set_key(xn, p.key(i));
    rekey(b, x, xn);
    if i > 0 {
        let base = p.key(i - 1).pos;
        x.update_key(xn, |k| relative(base, &mut k.pos));
    }
    let meta_inc = meta_describe_key(x.key(xn));

    x.copy_keys_from(xn + 1, y, 0..yn);
    let base = x.key(xn).pos;
    for k in 0..yn {
        rekey(b, x, xn + 1 + k);
        x.update_key(xn + 1 + k, |key| unrelative(base, &mut key.pos));
    }

    // x now holds everything of y, plus the key that used to separate them.
    x.set_key_count(xn + yn + 1);
    if !x.is_leaf() {
        // Bubble down: ranges that intersected old-x but not old-y, or the
        // other way round, have to move to their respective children.
        x.copy_children_from(xn + 1, y, 0..yn + 1);
        for k in 0..=xn {
            for &id in x.intersection().as_slice() {
                intersect_node(x.child(k), id);
            }
        }
        for ky in 0..=yn {
            // The nodes that used to be y's, now the second half of x.
            let k = xn + ky + 1;
            let child = x.child(k);
            child.set_parent(Some(x));
            child.set_parent_index(k);
            for &id in y.intersection().as_slice() {
                intersect_node(child, id);
            }
        }
    }

    let absorbed = p.child_meta(i + 1);
    p.update_child_meta(i, |m| meta_add(m, &absorbed));
    p.update_child_meta(i, |m| meta_add(m, &meta_inc));

    let pn = p.key_count();
    p.copy_keys_within(i + 1..pn, i);
    p.copy_children_within(i + 2..pn + 1, i + 1);
    // One child has gone; the survivors past it have all shifted down one.
    p.set_key_count(pn - 1);
    p.reparent_children(i + 1..pn);

    // SAFETY: `b` is a live tree and `y` one of its nodes, now detached from
    // it — nothing above still names it.
    unsafe { marktree_free_node(b, y.as_ptr()) };

    free_heap(&x.intersection());
    // The scratch set's storage moves into x wholesale, heap buffer included,
    // which is why it is not freed here.
    x.intersection().move_from(&merged);
    x
}

/// Move one key from child `i` of `p` to child `i + 1`, through `p`.
pub fn pivot_right(b: &mut MarkTree, _p_pos: MTPos, p: Node, i: usize) {
    let x = p.child(i);
    let y = p.child(i + 1);
    let (xn, yn) = (x.key_count(), y.key_count());

    y.copy_keys_within(0..yn, 1);
    // y takes p's separating key at the front: see the module docs on why the
    // count moves here rather than after the children.
    y.set_key_count(yn + 1);
    if !y.is_leaf() {
        y.copy_children_within(0..yn + 1, 1);
        y.reparent_children(1..yn + 2);
    }

    y.set_key(0, p.key(i));
    rekey(b, y, 0);
    p.set_key(i, x.key(xn - 1));
    rekey(b, p, i);

    let meta_inc_y = meta_describe_key(y.key(0));
    let meta_inc_x = meta_describe_key(p.key(i));
    p.update_child_meta(i + 1, |m| meta_add(m, &meta_inc_y));
    p.update_child_meta(i, |m| meta_sub(m, &meta_inc_x));

    if !x.is_leaf() {
        // x's last child follows the key, and takes its counts with it.
        y.copy_children_from(0, x, xn..xn + 1);
        let moved = y.child_meta(0);
        p.update_child_meta(i + 1, |m| meta_add(m, &moved));
        p.update_child_meta(i, |m| meta_sub(m, &moved));
        y.child(0).set_parent(Some(y));
        y.child(0).set_parent_index(0);
    }
    x.set_key_count(xn - 1);

    if i > 0 {
        let base = p.key(i - 1).pos;
        p.update_key(i, |k| unrelative(base, &mut k.pos));
    }
    let base = p.key(i).pos;
    y.update_key(0, |k| relative(base, &mut k.pos));
    let base = y.key(0).pos;
    for k in 1..yn + 1 {
        y.update_key(k, |key| unrelative(base, &mut key.pos));
    }

    // Repair x's intersections.
    if !x.is_leaf() {
        // y's new first child moved across from x, so it has to take on the
        // difference between its two parents' sets; that in turn may push some
        // of old-y's ids down onto old-y's other children.
        let mut storage = scratch_storage();
        let demoted = scratch(&mut storage);
        let moved = y.child(0).intersection();
        intersect_mov(&x.intersection(), &y.intersection(), &moved, &demoted);
        if !demoted.is_empty() {
            for yi in 1..y.key_count() + 1 {
                intersect_add(&y.child(yi).intersection(), &demoted);
            }
        }
        free_heap(&demoted);

        bubble_up(x);
    } else {
        // If x's last key used to be an end key, see whether it now covers all
        // of x.
        if mt_end(p.key(i)) {
            let pi = x.pseudo_index(0); // note: sloppy pseudo-index
            let start_id = mt_lookup_key_side(p.key(i), false);
            let pi_start = sloppy_index(b, start_id);
            if pi_start > 0 && pi_start < pi {
                intersect_node(x, start_id);
            }
        }
        if mt_start(y.key(0)) {
            // No check needed: just delete it if it was there.
            unintersect_node(y, mt_lookup_key(y.key(0)), false);
        }
    }
}

/// Move one key from child `i + 1` of `p` to child `i`, through `p`.
pub fn pivot_left(b: &mut MarkTree, _p_pos: MTPos, p: Node, i: usize) {
    let x = p.child(i);
    let y = p.child(i + 1);
    let (xn, yn) = (x.key_count(), y.key_count());

    // Reverse of how we "always" do it — but pivot_left is just the inverse of
    // pivot_right, so reverse it literally.
    let base = y.key(0).pos;
    for k in 1..yn {
        y.update_key(k, |key| relative(base, &mut key.pos));
    }
    let base = p.key(i).pos;
    y.update_key(0, |k| unrelative(base, &mut k.pos));
    if i > 0 {
        let base = p.key(i - 1).pos;
        p.update_key(i, |k| relative(base, &mut k.pos));
    }

    x.set_key(xn, p.key(i));
    rekey(b, x, xn);
    p.set_key(i, y.key(0));
    rekey(b, p, i);

    let meta_inc_x = meta_describe_key(x.key(xn));
    let meta_inc_y = meta_describe_key(p.key(i));
    p.update_child_meta(i, |m| meta_add(m, &meta_inc_x));
    p.update_child_meta(i + 1, |m| meta_sub(m, &meta_inc_y));

    if !x.is_leaf() {
        // y's first child follows the key, and takes its counts with it.
        let moved_child = y.child(0);
        x.copy_children_from(xn + 1, y, 0..1);
        let moved = y.child_meta(0);
        p.update_child_meta(i + 1, |m| meta_sub(m, &moved));
        p.update_child_meta(i, |m| meta_add(m, &moved));
        moved_child.set_parent(Some(x));
        moved_child.set_parent_index(xn + 1);
    }

    y.copy_keys_within(1..yn, 0);
    // y gave its first key away: see the module docs on why the count moves
    // here rather than after the children.
    y.set_key_count(yn - 1);
    if !y.is_leaf() {
        y.copy_children_within(1..yn + 1, 0);
        y.reparent_children(0..yn);
    }
    x.set_key_count(xn + 1);

    // Repair x's and y's intersections.
    if !x.is_leaf() {
        // x's new last child moved across from y, so it has to take on the
        // difference between its two parents' sets; that in turn may push some
        // of old-x's ids down onto old-x's other children.
        let mut storage = scratch_storage();
        let demoted = scratch(&mut storage);
        let n = x.key_count();
        let moved = x.child(n).intersection();
        intersect_mov(&y.intersection(), &x.intersection(), &moved, &demoted);
        if !demoted.is_empty() {
            // The child at `n` is deliberately skipped.
            for xi in 0..n {
                intersect_add(&x.child(xi).intersection(), &demoted);
            }
        }
        free_heap(&demoted);

        bubble_up(y);
    } else {
        // If y's first key used to be a start key, see whether it now covers
        // all of y.
        if mt_start(p.key(i)) {
            let pi = y.pseudo_index(0); // note: sloppy pseudo-index
            let end_id = mt_lookup_key_side(p.key(i), true);
            let pi_end = sloppy_index(b, end_id);
            if pi_end > pi {
                intersect_node(y, mt_lookup_key(p.key(i)));
            }
        }
        let last = x.key(x.key_count() - 1);
        if mt_end(last) {
            // No check needed: just delete it if it was there.
            unintersect_node(x, mt_lookup_key_side(last, false), false);
        }
    }
}

/// `x` shrank, or is one half of a split. Ranges that used to cover every one
/// of its children now cover `x` itself, so hoist them one level.
fn bubble_up(x: Node) {
    let mut storage = scratch_storage();
    let common = scratch(&mut storage);
    let n = x.key_count();
    // By the tree's invariants the largest subset of *all* the children is the
    // intersection of the first with the last.
    let (first, last) = (x.child(0).intersection(), x.child(n).intersection());
    intersect_common(&common, &first, &last);
    if !common.is_empty() {
        for i in 0..=n {
            intersect_sub(&x.child(i).intersection(), &common);
        }
        intersect_add(&x.intersection(), &common);
    }
    free_heap(&common);
}

/// Insert `k` into the subtree rooted at `x`, which must not be a full node
/// (even if there might be internal space).
#[inline]
pub fn marktree_putp_aux(b: &mut MarkTree, x: Node, mut k: MTKey, meta_inc: &MetaCount) {
    // TODO(bfredl): ugh, make sure this is the _last_ valid (pos, gravity)
    // position, to minimize movement
    let mut i = (find_key(x.keys(), k).0 + 1) as usize;
    if x.is_leaf() {
        let n = x.key_count();
        if i != n {
            x.copy_keys_within(i..n, i + 1);
        }
        x.set_key(i, k);
        rekey(b, x, i);
        x.set_key_count(n + 1);
    } else {
        if x.child(i).key_count() == MAX_KEYS {
            split_node(b, x, i, k);
            if key_cmp(k, x.key(i)) > 0 {
                i += 1;
            }
        }
        if i > 0 {
            relative(x.key(i - 1).pos, &mut k.pos);
        }
        marktree_putp_aux(b, x.child(i), k, meta_inc);
        x.update_child_meta(i, |m| meta_add(m, meta_inc));
    }
}
