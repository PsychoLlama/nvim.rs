#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! Whole-tree invariant checks, and the entry points the unit suite drives.
//!
//! `crates/nvim/tests/unit/marktree.rs` builds trees of thousands of marks and
//! calls [`marktree_check`] after every batch, so these are not dead debug
//! code: they are the oracle. What they assert, per node:
//!
//! * the fill bounds, and that the root is the only node allowed to be short;
//! * that the keys are in order once their relative positions are resolved,
//!   including the gravity tie-break;
//! * that `id2node` names the node each key actually lives in;
//! * that every child's `parent` and `p_idx` point back correctly;
//! * that the meta counts equal what a fresh walk of the subtree computes.
//!
//! [`marktree_check_intersections`] is the expensive one: it empties every
//! node's set, rebuilds all of them from the pairs themselves, and compares.
//!
//! Which assertions are `assert!` and which are `debug_assert!` is deliberate
//! and matches the C: nothing here is tightened or relaxed by the port.

use core::mem;

use crate::decoration::DECOR_HIGHLIGHT_INLINE_INIT;
use crate::marktree::iter::{marktree_itr_current, marktree_itr_first, marktree_itr_next};
use crate::marktree::key::{
    MT_BRANCH_FACTOR, MtFlags, mt_flags, mt_lookup_id, mt_lookup_key, mt_right, mt_start, pos_leq,
    unrelative,
};
use crate::marktree::meta::MetaCount;
use crate::marktree::node::{MAX_KEYS, NodeId};
use crate::marktree::pair::marktree_intersect_pair;
use crate::registry::{IdMap, id_map};
use crate::types::{DecorInline, MTKey, MTPos, MarkTree, MarkTreeIter, size_t, uint32_t, uint64_t};

use super::{marktree_del_itr, marktree_lookup, marktree_lookup_ns, marktree_put};

/// Check every invariant of the whole tree, aborting on the first violation.
pub fn marktree_check(tree: &mut MarkTree) {
    let Some(root) = tree.root else {
        debug_assert!(tree.n_keys == 0 as size_t, "b->n_keys == 0");
        debug_assert!(tree.node_count() == 0, "b->n_nodes == 0");
        // The C's `b->id2node == NULL` arm is dead — `id2node` was a
        // one-element array, so its address was never null — and stays dead
        // here, where it is a table by value.
        assert!(
            tree.id2node.is_empty(),
            "b->id2node == NULL || map_size(b->id2node) == 0"
        );
        return;
    };
    let mut last = MTPos::default();
    let mut last_right = false;
    let meta_root = tree.meta_root;
    let nkeys = check_node(tree, root, &mut last, &mut last_right, meta_root);
    debug_assert!(tree.n_keys == nkeys, "b->n_keys == nkeys");
    debug_assert!(
        tree.n_keys == tree.id2node.len(),
        "b->n_keys == map_size(b->id2node)"
    );
}

/// Each node's intersection set as [`recurse_nodes`] found it, filed under the
/// node. A node whose set was empty has no entry.
type Records = IdMap<NodeId, Vec<uint64_t>>;

/// Check `x`'s own invariants and its whole subtree's, and answer how many
/// keys the subtree holds.
///
/// `last` carries the previous key's absolute position across the walk and
/// `last_right` its gravity, which is what makes the ordering check able to
/// span a node boundary. `meta_node` is what the *parent* recorded for this
/// subtree, and the last assertion is that a fresh walk agrees with it.
fn check_node(
    tree: &MarkTree,
    x: NodeId,
    last: &mut MTPos,
    last_right: &mut bool,
    meta_node: MetaCount,
) -> size_t {
    let node = &tree[x];
    let n = node.key_count();
    debug_assert!(n <= MAX_KEYS, "x->n <= 2 * T - 1");
    assert!(
        n >= if Some(x) != tree.root {
            MT_BRANCH_FACTOR as usize - 1
        } else {
            0
        },
        "x->n >= (x != b->root ? T - 1 : 0)"
    );
    let mut n_keys = n;
    for i in 0..n {
        if !node.is_leaf() {
            n_keys += check_node(tree, node.child(i), last, last_right, *node.child_meta(i));
        } else {
            *last = MTPos::default();
        }
        if i > 0 {
            unrelative(node.key(i - 1).pos, last);
        }
        let key = node.key(i);
        debug_assert!(pos_leq(*last, key.pos), "pos_leq(*last, x->key[i].pos)");
        if last.row == key.pos.row && last.col == key.pos.col {
            debug_assert!(
                !*last_right || mt_right(key),
                "!*last_right || mt_right(x->key[i])"
            );
        }
        *last_right = mt_right(key);
        debug_assert!(key.pos.col >= 0, "x->key[i].pos.col >= 0");
        debug_assert!(
            tree.id2node(mt_lookup_key(key)) == Some(x),
            "pmap_get(uint64_t)(b->id2node, mt_lookup_key(x->key[i])) == x"
        );
    }
    if !node.is_leaf() {
        n_keys += check_node(tree, node.child(n), last, last_right, *node.child_meta(n));
        unrelative(node.key(n - 1).pos, last);
        for i in 0..=n {
            let child = node.child(i);
            debug_assert!(tree[child].parent() == Some(x), "x->ptr[i]->parent == x");
            debug_assert!(tree[child].parent_index() == i, "x->ptr[i]->p_idx == i");
            assert!(
                tree[child].level() == node.level() - 1,
                "x->ptr[i]->level == x->level - 1"
            );
            for j in 0..i {
                debug_assert!(child != node.child(j), "x->ptr[i] != x->ptr[j]");
            }
        }
    } else if n > 0 {
        *last = node.key(n - 1).pos;
    }
    debug_assert!(meta_node == node.meta(), "meta_node_ref[m] == meta_node[m]");
    n_keys
}

/// Rebuild every intersection set from the pairs themselves and check it
/// against what was there.
///
/// Three steps: move each node's set aside and empty it; walk every mark and,
/// for each start of a pair, intersect the nodes between the two halves as if
/// the pair had just been inserted; then compare each node's rebuilt set
/// against the one that was moved aside.
pub fn marktree_check_intersections(tree: &mut MarkTree) -> bool {
    let Some(root) = tree.root else {
        return true;
    };
    let mut checked: Records = id_map();
    recurse_nodes(tree, root, &mut checked);
    let mut iter = MarkTreeIter::default();
    marktree_itr_first(tree, &mut iter);
    loop {
        let mark = marktree_itr_current(tree, &iter);
        if mark.pos.row < 0 {
            break;
        }
        if mt_start(mark) {
            let mut end_iter = MarkTreeIter::default();
            let end_id = mt_lookup_id(mark.ns, mark.id, true);
            let k = marktree_lookup(tree, end_id, Some(&mut end_iter));
            if k.pos.row >= 0 {
                // A copy, because intersecting walks the start iterator.
                let mut start_iter = iter;
                let id = mt_lookup_key(mark);
                marktree_intersect_pair(tree, id, &mut start_iter, &end_iter, false);
            }
        }
        marktree_itr_next(tree, &mut iter);
    }
    recurse_nodes_compare(tree, root, &checked)
}

/// Record `x`'s intersection set in `checked` and empty it, for the whole
/// subtree, so the walk that rebuilds the sets can be compared against the
/// record.
fn recurse_nodes(tree: &mut MarkTree, x: NodeId, checked: &mut Records) {
    let set = mem::take(tree[x].intersection_mut());
    if !set.is_empty() {
        checked.insert(x, set.as_slice().to_vec());
    }
    if !tree[x].is_leaf() {
        for i in 0..=tree[x].key_count() {
            let child = tree[x].child(i);
            recurse_nodes(tree, child, checked);
        }
    }
}

/// Does `x`'s rebuilt intersection set match what [`recurse_nodes`] recorded
/// for it? Recurses over the whole subtree.
fn recurse_nodes_compare(tree: &MarkTree, x: NodeId, checked: &Records) -> bool {
    let rebuilt = tree[x].intersection().as_slice();
    let recorded = checked.get(&x).map_or(&[][..], Vec::as_slice);
    if recorded != rebuilt {
        return false;
    }
    if !tree[x].is_leaf() {
        for i in 0..=tree[x].key_count() {
            if !recurse_nodes_compare(tree, tree[x].child(i), checked) {
                return false;
            }
        }
    }
    true
}

/// One end of a mark, as the unit suite spells it.
#[derive(Copy, Clone)]
pub struct MarkEnd {
    pub row: ::core::ffi::c_int,
    pub col: ::core::ffi::c_int,
    /// Which side of an insertion at this exact position the mark stays on.
    pub right_gravity: bool,
}

/// Put a mark, spelling out every flag — the unit suite's way in.
///
/// `end` is `None` for a point mark; the sentinel row the tree itself uses for
/// one does not escape into the callers.
pub fn marktree_put_test(
    tree: &mut MarkTree,
    ns: uint32_t,
    id: uint32_t,
    start: MarkEnd,
    end: Option<MarkEnd>,
    meta_inline: bool,
) {
    let flags = mt_flags(start.right_gravity, false, false, false)
        | MtFlags::DECOR_VIRT_TEXT_INLINE.when(meta_inline);
    let key = MTKey {
        pos: MTPos {
            row: start.row,
            col: start.col,
        },
        ns,
        id,
        flags,
        decor_data: DecorInline::Hl(DECOR_HIGHLIGHT_INLINE_INIT),
    };
    let end = end.unwrap_or(MarkEnd {
        row: -1,
        col: -1,
        right_gravity: false,
    });
    marktree_put(tree, key, end.row, end.col, end.right_gravity);
}

/// `mt_right` where the unit suite can reach it.
pub fn mt_right_test(key: MTKey) -> bool {
    mt_right(key)
}

/// Delete both halves of the pair `(ns, id)`.
pub fn marktree_del_pair_test(tree: &mut MarkTree, ns: uint32_t, id: uint32_t) {
    let mut iter = MarkTreeIter::default();
    marktree_lookup_ns(tree, ns, id, false, Some(&mut iter));
    let other = marktree_del_itr(tree, &mut iter, false);
    debug_assert!(other != 0, "other");
    marktree_lookup(tree, other, Some(&mut iter));
    marktree_del_itr(tree, &mut iter, false);
}
