#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

//! The extmark store: a wide B-tree keyed by (row, col).
//!
//! Marks go in with [`marktree_put`]; a text change is applied to all of them
//! at once with [`marktree_splice`](splice::marktree_splice); everything else —
//! reading, deleting — goes through `MarkTreeIter`. Position an iterator with
//! [`marktree_itr_get`](iter::marktree_itr_get), or find a mark by its id with
//! [`marktree_lookup`], then read with
//! [`marktree_itr_current`](iter::marktree_itr_current) and step with
//! [`marktree_itr_next`](iter::marktree_itr_next). [`marktree_del_itr`] deletes
//! the mark under the iterator and leaves it on the next one.
//!
//! Three design decisions account for most of the code:
//!
//! * **Positions are relative.** A key's position is stored relative to the key
//!   before it in the same node, and a node's to its parent's. So a change
//!   affecting a whole subtree can be applied to one node instead of to every
//!   mark in it, which is what makes a splice near the top of a large buffer
//!   cheap. Everything that moves a key between nodes has to rebase it.
//! * **Ranges are two keys.** A `(ns, id)` pair with [`MtFlags::END`] set on
//!   the second. They are ordered like any other key, and the tree keeps them
//!   consistent across splices that would otherwise reverse them. Everything
//!   that maintains the second key lives in [`pair`].
//! * **Covering ranges are recorded on nodes, not smeared over keys.** A node
//!   carries the ids of the ranges that cover the whole of it; see
//!   [`intersect`]. That is what makes "which ranges cover this position" a
//!   walk down the tree.
//!
//! A tree is reached as `&mut MarkTree`, and its nodes live in the tree's own
//! slab, named by [`NodeId`](node::NodeId) — see [`node`]. An iterator names
//! its node by id too, so one handed alongside the wrong tree, or one a change
//! left stale, reads a wrong key or panics; it never reaches freed memory.
//!
//! Derived from kbtree in klib, and from the marker tree of the Atom editor.

pub mod check;
pub mod cursor;
pub mod inspect;
pub mod intersect;
pub mod iter;
pub mod key;
pub mod meta;
pub mod node;
pub mod pair;
pub mod rebalance;
pub mod splice;

use crate::decoration::defs::DecorInline;
use crate::marktree::node::{NodeId, NodeSlab};
use crate::narrow::as_index;
use crate::narrow::count_as_int;
use crate::os::libc::{int32_t, size_t, uint32_t, uint64_t};
use crate::registry::{IdMap, id_map};
use core::ffi::c_int;

pub use crate::marktree::check::*;
pub(crate) use crate::marktree::inspect::*;
pub use crate::marktree::iter::*;
use crate::marktree::key::*;
use crate::marktree::meta::*;
use crate::marktree::node::*;
pub use crate::marktree::pair::*;
pub use crate::marktree::rebalance::*;
pub use crate::marktree::splice::*;

/// Insert `key`, and — where `end_row` is non-negative — the end key that makes
/// it a range.
pub fn marktree_put(
    tree: &mut MarkTree,
    mut key: MTKey,
    end_row: c_int,
    end_col: c_int,
    end_right: bool,
) {
    assert!(
        key.flags
            .without(MtFlags::EXTERNAL_MASK | MtFlags::RIGHT_GRAVITY)
            .is_empty(),
        "key.flags names only externally settable bits"
    );
    if end_row >= 0 {
        key.flags |= MtFlags::PAIRED;
    }
    marktree_put_key(tree, key);
    if end_row < 0 {
        return;
    }

    let mut end_key = key;
    end_key.flags = key.flags.without(MtFlags::RIGHT_GRAVITY)
        | MtFlags::END
        | MtFlags::RIGHT_GRAVITY.when(end_right);
    end_key.pos = MTPos {
        row: end_row,
        col: end_col,
    };
    marktree_put_key(tree, end_key);

    let mut iter = MarkTreeIter::default();
    let mut end_iter = MarkTreeIter::default();
    marktree_lookup(tree, mt_lookup_key(key), Some(&mut iter));
    marktree_lookup(tree, mt_lookup_key(end_key), Some(&mut end_iter));
    marktree_intersect_pair(tree, mt_lookup_key(key), &mut iter, &end_iter, false);
}

/// Insert one already-built key, splitting a full node on the way down.
///
/// Upstream allocated the first root at the internal node's size even though it
/// starts at level zero; here a level-zero node is a leaf, and the tree's
/// first internal node is the root a split creates above it.
pub fn marktree_put_key(tree: &mut MarkTree, mut k: MTKey) {
    k.flags |= MtFlags::REAL; // let's be real.
    let mut r = match tree.root {
        Some(root) => root,
        None => {
            let root = tree.alloc_node(0);
            tree.root = Some(root);
            root
        }
    };
    if tree[r].key_count() == MAX_KEYS {
        // The root is full: grow the tree by one level and split into the new
        // root, which is where every level the tree ever gains comes from.
        let s = tree.alloc_node(tree[r].level() + 1);
        tree.root = Some(s);
        let meta_root = tree.meta_root;
        let top = &mut tree[s];
        top.set_key_count(0);
        top.set_child(0, r);
        *top.child_meta_mut(0) = meta_root;
        tree[r].set_parent(Some(s));
        tree[r].set_parent_index(0);
        split_node(tree, s, 0, k);
        r = s;
    }

    let meta_inc = meta_describe_key(k);
    marktree_putp_aux(tree, r, k, &meta_inc);
    meta_add(&mut tree.meta_root, &meta_inc);
    tree.n_keys = tree.n_keys.wrapping_add(1);
}

/// Which iterator index the repair walk is currently adjusting.
///
/// The C holds an `int *` that alternates between `&itr->i` and
/// `&itr->s[rlvl].i`, and compares it *by address* to decide whether the node
/// it just merged is the one the iterator stands on. Naming the two cases says
/// the same thing without the pointer.
#[derive(Copy, Clone, PartialEq, Eq)]
enum Lasti {
    /// The key index within the node the iterator is on.
    Cur,
    /// The descent index the iterator took at this level.
    Level(usize),
}

impl Lasti {
    fn bump(self, iter: &mut MarkTreeIter, by: c_int) {
        match self {
            Lasti::Cur => iter.index += by,
            Lasti::Level(depth) => iter.path[depth].index += by,
        }
    }
}

/// Delete the mark the iterator names, and answer the id of the other half of
/// its pair (zero for an unpaired mark).
///
/// The protocol, which is why this is as long as it is:
///
/// 1. The caller hands us a valid iterator.
/// 2. If it names a key in an internal node, step one place left or right to
///    reach a leaf key — the *auxiliary* key.
/// 3. Delete that leaf key. The leaf may now be undersized.
/// 4. If step 2 happened, write the auxiliary key over the one the caller
///    actually wanted gone, rebasing its position.
/// 5. Repair upward from the leaf: if the node is big enough, stop; else steal
///    from the left sibling, else from the right; else merge with a sibling,
///    which may leave the parent undersized, so repeat for the parent.
/// 6. If step 5 reached the root and left it with no keys, drop it and promote
///    its only child.
///
/// The iterator stays valid and points at the key *after* the deleted one.
///
/// `rev` says the caller intends to keep iterating backwards and deleting keys
/// before this one. Iterating forward is the recommended strategy and passes
/// false.
pub fn marktree_del_itr(tree: &mut MarkTree, iter: &mut MarkTreeIter, rev: bool) -> uint64_t {
    let mut adjustment = 0;
    let cur = iter.node.expect("a positioned iterator");
    let curi = as_index(iter.index);
    let raw = tree[cur].key(curi);
    let id = mt_lookup_key(raw);

    // 1. Orphan the other half of the pair, and retract the records of what
    //    the range covered. NB: the retraction must name exactly the pair the
    //    records were made for.
    let mut other = 0;
    if mt_paired(raw) && !raw.flags.has(MtFlags::ORPHANED) {
        other = mt_lookup_key_side(raw, !mt_end(raw));
        let mut other_iter = MarkTreeIter::default();
        marktree_lookup(tree, other, Some(&mut other_iter));
        tree.key_mut(&other_iter).flags |= MtFlags::ORPHANED;
        if mt_start(raw) {
            let mut this_iter = *iter; // a copy, because this one is mutated
            marktree_intersect_pair(tree, id, &mut this_iter, &other_iter, true);
        } else {
            marktree_intersect_pair(tree, other, &mut other_iter, iter, true);
        }
    }

    // 2. An internal key cannot be removed in place; steal the previous key,
    //    which is in a leaf, and put it here instead.
    if !tree[cur].is_leaf() {
        if rev {
            ::std::process::abort();
        }
        marktree_itr_prev(tree, iter);
        adjustment = -1;
    }

    // 3. Delete the leaf key.
    let mut x = iter.node.expect("a leaf key before an internal one");
    debug_assert!(tree[x].is_leaf(), "x->level == 0");
    let mut intkey = tree[x].key(as_index(iter.index));
    let mut meta_inc = meta_describe_key(intkey);
    let hole = as_index(iter.index);
    let leaf = &mut tree[x];
    if leaf.key_count() > hole + 1 {
        leaf.copy_keys_within(hole + 1..leaf.key_count(), hole);
    }
    leaf.set_key_count(leaf.key_count() - 1);
    tree.n_keys = tree.n_keys.wrapping_sub(1);
    tree.id2node.remove(&id);

    // 4. Write the stolen key over the one the caller wanted gone, rebasing it
    //    onto every node between the two, and repair the covering records the
    //    move disturbed.
    if adjustment == -1 {
        let mut depth = iter.depth - 1;
        let mut lnode = x;
        let mut start_id = 0;
        let mut did_bubble = false;
        if mt_end(intkey) {
            start_id = mt_lookup_key_side(intkey, false);
        }
        loop {
            if depth < 0 {
                ::std::process::abort();
            }
            let p = tree[lnode]
                .parent()
                .expect("a node below `cur` has a parent");
            let i = as_index(iter.path[as_index(depth)].index);
            debug_assert!(tree[p].child(i) == lnode, "p->ptr[i] == lnode");
            if i > 0 {
                unrelative(tree[p].key(i - 1).pos, &mut intkey.pos);
            }
            if p != cur && start_id != 0 && tree[tree[p].child(0)].intersection().contains(start_id)
            {
                // After the first time round, this also undoes the addition the
                // previous round made just below.
                let last = usize::from(lnode != x);
                // One less than the children, because `ptr[n]` is the last.
                for k in 0..tree[p].key_count() + last {
                    let child = tree[p].child(k);
                    unintersect_node(tree, child, start_id, true);
                }
                intersect_node(tree, p, start_id);
                did_bubble = true;
            }
            let at = tree[lnode].parent_index();
            meta_sub(tree[p].child_meta_mut(at), &meta_inc);
            lnode = p;
            depth -= 1;
            if lnode == cur {
                break;
            }
        }

        let mut deleted = tree[cur].key(curi);
        meta_inc = meta_describe_key(deleted);
        tree[cur].set_key(curi, intkey);
        tree.refkey(cur, curi);
        // Where we bubbled `start_id` up to a parent, its record is already
        // there; otherwise the leaf may need one of its own.
        if mt_end(tree[cur].key(curi)) && !did_bubble {
            let pi = tree.pseudo_index(x, 0); // note: sloppy pseudo-index
            let pi_start = pseudo_index_for_id(tree, start_id, true);
            if pi_start > 0 && pi_start < pi {
                intersect_node(tree, x, start_id);
            }
        }

        // The subtree that hung off the replaced key is now based on a
        // different key, so rebase its leftmost spine onto the difference.
        relative(intkey.pos, &mut deleted.pos);
        if deleted.pos.row != 0 || deleted.pos.col != 0 {
            let mut y = Some(tree[cur].child(curi + 1));
            while let Some(node) = y {
                let node = &mut tree[node];
                for k in 0..node.key_count() {
                    unrelative(deleted.pos, &mut node.key_mut(k).pos);
                }
                y = if node.is_leaf() {
                    None
                } else {
                    Some(node.child(0))
                };
            }
        }
        iter.index -= 1;
    }

    let mut lnode = cur;
    while let Some(p) = tree[lnode].parent() {
        let at = tree[lnode].parent_index();
        meta_sub(tree[p].child_meta_mut(at), &meta_inc);
        lnode = p;
    }
    for (m, &inc) in meta_inc.iter().enumerate() {
        debug_assert!(tree.meta_root[m] >= inc, "b->meta_root[m] >= meta_inc[m]");
    }
    meta_sub(&mut tree.meta_root, &meta_inc);

    // 5. Repair upward from the leaf.
    let mut iter_dirty = false;
    let mut rdepth = iter.depth - 1;
    let mut lasti = Lasti::Cur;
    let mut ppos = iter.pos;
    let min_keys = MT_BRANCH_FACTOR as usize - 1;
    while Some(x) != tree.root {
        debug_assert!(rdepth >= 0, "rlvl >= 0");
        let p = tree[x]
            .parent()
            .expect("a node below the root has a parent");
        if tree[x].key_count() >= min_keys {
            // This node is fine, so the rest of the tree is.
            break;
        }
        let pi = iter.path[as_index(rdepth)].index;
        debug_assert!(tree[p].child(as_index(pi)) == x, "p->ptr[pi] == x");
        if pi > 0 {
            ppos.row -= tree[p].key(as_index(pi) - 1).pos.row;
            ppos.col = iter.path[as_index(rdepth)].old_col;
        }
        // `ppos` is now the position of `p`.

        if pi > 0 && tree[tree[p].child(as_index(pi) - 1)].key_count() > min_keys {
            lasti.bump(iter, 1);
            iter_dirty = true;
            // Steal one key from the left neighbour.
            pivot_right(tree, ppos, p, as_index(pi) - 1);
            break;
        } else if pi < count_as_int(tree[p].key_count())
            && tree[tree[p].child(as_index(pi) + 1)].key_count() > min_keys
        {
            // Steal one key from the right neighbour.
            pivot_left(tree, ppos, p, as_index(pi));
            break;
        } else if pi > 0 {
            assert!(
                tree[tree[p].child(as_index(pi) - 1)].key_count() == min_keys,
                "p->ptr[pi - 1]->n == T - 1"
            );
            // Merge with the left neighbour.
            lasti.bump(iter, MT_BRANCH_FACTOR.cast_signed());
            x = merge_node(tree, p, as_index(pi) - 1);
            if lasti == Lasti::Cur {
                // TRICKY: we merged the node the iterator was on.
                iter.node = Some(x);
            }
            iter.path[as_index(rdepth)].index -= 1;
            iter_dirty = true;
        } else {
            assert!(
                pi < count_as_int(tree[p].key_count())
                    && tree[tree[p].child(as_index(pi) + 1)].key_count() == min_keys,
                "pi < p->n && p->ptr[pi + 1]->n == T - 1"
            );
            // No iterator adjustment is needed.
            merge_node(tree, p, as_index(pi));
        }
        lasti = Lasti::Level(as_index(rdepth));
        rdepth -= 1;
        x = p;
    }

    // 6. The root may have run out of keys; drop it and promote its child.
    let root = tree
        .root
        .expect("a tree that held the deleted key has a root");
    if tree[root].key_count() == 0 {
        if iter.depth > 0 {
            iter.path.copy_within(1..as_index(iter.depth), 0);
            iter.depth -= 1;
        }
        if !tree[root].is_leaf() {
            let promoted = tree[root].child(0);
            debug_assert!(
                tree.meta_root == *tree[root].child_meta(0),
                "b->meta_root[m] == oldroot->meta[0][m]"
            );
            tree.root = Some(promoted);
            tree[promoted].set_parent(None);
            // The old root is detached from the tree now.
            tree.free_node(root);
        } else {
            // No items left; nothing for the iterator to point at. Not strictly
            // needed — deleting the right-most mark would be handled anyway.
            iter.node = None;
        }
    }

    if iter.is_positioned() && iter_dirty {
        marktree_itr_fix_pos(tree, iter);
    }

    // BONUS STEP: leave the iterator on the key after the deleted one.
    if adjustment == -1 {
        // Tricky: we stand at the hole in the previous leaf, and the internal
        // key is now the one we stole, so skip that one as well.
        marktree_itr_next(tree, iter);
        marktree_itr_next(tree, iter);
    } else if let Some(node) = iter.node
        && as_index(iter.index) >= tree[node].key_count()
    {
        // We deleted the last key of a leaf node; go to the internal key
        // after it.
        debug_assert!(tree[node].is_leaf(), "itr->x->level == 0");
        marktree_itr_next(tree, iter);
    }

    other
}

/// Re-count the meta kinds of the key under the iterator, up to the root, after
/// a consumer edited its flags in place.
pub fn marktree_revise_meta(tree: &mut MarkTree, iter: &mut MarkTreeIter, old_key: MTKey) {
    let x = iter.node.expect("a positioned iterator");
    let meta_old = meta_describe_key(old_key);
    let meta_new = meta_describe_key(tree[x].key(as_index(iter.index)));
    if meta_new == meta_old {
        return;
    }

    let mut lnode = x;
    while let Some(p) = tree[lnode].parent() {
        let at = tree[lnode].parent_index();
        meta_apply_delta(tree[p].child_meta_mut(at), &meta_new, &meta_old);
        lnode = p;
    }
    meta_apply_delta(&mut tree.meta_root, &meta_new, &meta_old);
}

/// Drop every mark and every node, leaving the tree as it was born.
///
/// An iterator taken out of the tree before is stale afterwards: stepping it
/// panics on its node id.
pub fn marktree_clear(tree: &mut MarkTree) {
    tree.root = None;
    tree.free_all_nodes();

    // Upstream's `map_destroy(uint64_t, b->id2node)`. The table keeps its
    // allocation, which is what a cleared tree wants: it is about to be
    // filled again, or it is a buffer's field on its way out.
    tree.id2node.clear();

    tree.n_keys = 0;
    tree.meta_root = [0; META_COUNT];
    debug_assert!(tree.node_count() == 0, "b->n_nodes == 0");
}

/// Try to move `key` to `newpos` without it leaving the leaf `x` it sits in.
///
/// Answers false when the new position is outside the span `x` covers, which is
/// when the caller has to delete and re-insert the mark instead. Nothing is
/// written in that case.
fn move_within_leaf(
    tree: &mut MarkTree,
    x: NodeId,
    iter: &MarkTreeIter,
    mut key: MTKey,
    mut newpos: MTPos,
) -> bool {
    let leaf = &mut tree[x];
    if leaf.parent().is_some() {
        // Strictly *after* the key before `x` — not optimal when `x` is the
        // very first leaf of the entire tree, but that is fine.
        if !pos_less(iter.pos, newpos) {
            return false;
        }
        relative(iter.pos, &mut newpos);
        // Strictly before the end of `x`. (This could be made sharper by
        // finding the internal key just after `x`, but meh.)
        if !pos_less(newpos, leaf.key(leaf.key_count() - 1).pos) {
            return false;
        }
    }
    // Otherwise the tree is one node, so `newpos` is already relative to
    // `iter.pos`.
    if key.pos.row == newpos.row && key.pos.col == newpos.col {
        return true;
    }
    key.pos = newpos;

    // Tricky: movement could be minimised better in either direction.
    let (mut new_i, matched) = find_key(leaf.keys(), key);
    if !matched {
        new_i += 1;
    }
    let (i, new_i) = (as_index(iter.index), as_index(new_i));
    if new_i == i {
        leaf.key_mut(i).pos = newpos;
    } else if new_i < i {
        leaf.copy_keys_within(new_i..i, new_i + 1);
        leaf.set_key(new_i, key);
    } else {
        leaf.copy_keys_within(i + 1..new_i, i);
        leaf.set_key(new_i - 1, key);
    }
    true
}

/// Move the mark the iterator names to `(row, col)`.
///
/// The iterator is invalid after the call unless the mark stayed put within its
/// own leaf.
pub fn marktree_move(tree: &mut MarkTree, iter: &mut MarkTreeIter, row: c_int, col: c_int) {
    let x = iter.node.expect("a positioned iterator");
    let mut key = tree[x].key(as_index(iter.index));
    let newpos = MTPos { row, col };
    if tree[x].is_leaf() && move_within_leaf(tree, x, iter, key, newpos) {
        return;
    }

    let other = marktree_del_itr(tree, iter, false);
    key.pos = newpos;
    marktree_put_key(tree, key);
    if other != 0 {
        marktree_restore_pair(tree, key);
    }
    iter.node = None; // the put may have invalidated it
}

/// The key with this `(ns, id, end)`, or `MT_INVALID_KEY`.
pub fn marktree_lookup_ns(
    tree: &mut MarkTree,
    ns: uint32_t,
    id: uint32_t,
    end: bool,
    iter: Option<&mut MarkTreeIter>,
) -> MTKey {
    marktree_lookup(tree, mt_lookup_id(ns, id, end), iter)
}

/// The key with this lookup handle, or `MT_INVALID_KEY`. `iter`, where given,
/// is left on the key — or emptied when there is none.
pub fn marktree_lookup(
    tree: &mut MarkTree,
    id: uint64_t,
    iter: Option<&mut MarkTreeIter>,
) -> MTKey {
    let Some(n) = tree.id2node(id) else {
        if let Some(iter) = iter {
            iter.node = None;
        }
        return MT_INVALID_KEY;
    };
    let Some(i) = tree[n].keys().iter().position(|&k| mt_lookup_key(k) == id) else {
        // The id map named a node that does not hold the key, so the tree is
        // corrupt and there is nothing sensible to answer.
        ::std::process::abort()
    };
    marktree_itr_set_node(tree, iter, n, count_as_int(i))
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct MTKey {
    pub pos: MTPos,
    pub ns: uint32_t,
    pub id: uint32_t,
    pub flags: MtFlags,
    pub decor_data: DecorInline,
}
#[derive(Copy, Clone)]
pub struct MTPair {
    pub start: MTKey,
    pub end_pos: MTPos,
    pub end_right_gravity: bool,
}
#[derive(Copy, Clone, Default)]
#[repr(C)]
pub struct MTPos {
    pub row: int32_t,
    pub col: int32_t,
}
/// The extmark B-tree of one buffer.
///
/// Its nodes live in [`NodeSlab`], addressed by [`NodeId`]: a node names its
/// parent and children by id, and so does an iterator, so nothing outside the
/// slab holds a node's address.
pub struct MarkTree {
    /// The top node, or `None` for a tree that has never held a key or has
    /// been emptied by `marktree_clear`.
    pub(crate) root: Option<NodeId>,
    pub meta_root: [uint32_t; 5],
    pub n_keys: size_t,
    pub(crate) nodes: NodeSlab,
    /// Every key's lookup handle to the node it lives in, so a mark can be
    /// found without a walk. Asked, never iterated.
    pub(crate) id2node: IdMap<uint64_t, NodeId>,
}

impl MarkTree {
    /// A tree with nothing in it.
    pub const EMPTY: MarkTree = MarkTree {
        root: None,
        meta_root: [0; 5],
        n_keys: 0,
        nodes: NodeSlab::new(),
        id2node: id_map(),
    };
}

impl Default for MarkTree {
    fn default() -> Self {
        MarkTree::EMPTY
    }
}
/// A cursor into a [`MarkTree`].
///
/// `Copy`, and deliberately: forking an iterator to look ahead is what the
/// marktree code does with it. It names its node by [`NodeId`] and the path
/// down to it by child index, so it holds nothing a tree change can leave
/// dangling: a stale iterator reads a wrong key, never freed memory.
#[derive(Copy, Clone, Default)]
pub struct MarkTreeIter {
    /// The absolute position the keys of `node` are relative to.
    pub pos: MTPos,
    /// How far below the root `node` is.
    pub depth: ::core::ffi::c_int,
    /// The node the iterator stands in, or `None` once a walk has run off the
    /// end (or before one has started).
    pub node: Option<NodeId>,
    /// The key within `node`.
    pub index: ::core::ffi::c_int,
    /// The child index taken at each level above `node`.
    pub path: [PathStep; 20],
    pub intersect_idx: size_t,
    pub intersect_pos: MTPos,
    pub intersect_pos_x: MTPos,
}

impl MarkTreeIter {
    /// An iterator positioned nowhere — what a walk starts from.
    pub const EMPTY: MarkTreeIter = MarkTreeIter {
        pos: MTPos { row: 0, col: 0 },
        depth: 0,
        node: None,
        index: 0,
        path: [PathStep {
            old_col: 0,
            index: 0,
        }; 20],
        intersect_idx: 0,
        intersect_pos: MTPos { row: 0, col: 0 },
        intersect_pos_x: MTPos { row: 0, col: 0 },
    };

    /// Whether the iterator names a key at all.
    pub fn is_positioned(&self) -> bool {
        self.node.is_some()
    }
}

/// One level of a [`MarkTreeIter`]'s path: which child it descended into, and
/// the column its position had before it did.
#[derive(Copy, Clone, Default)]
pub struct PathStep {
    pub old_col: ::core::ffi::c_int,
    pub index: ::core::ffi::c_int,
}
pub type MetaIndex = ::core::ffi::c_uint;
