#![deny(unsafe_op_in_unsafe_fn)]

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

use core::ptr;

use crate::marktree::iter::{marktree_itr_current, marktree_itr_first, marktree_itr_next};
use crate::marktree::key::{
    DECOR_HIGHLIGHT_INLINE_INIT, MT_BRANCH_FACTOR, MtFlags, mt_flags, mt_lookup_id, mt_lookup_key,
    mt_right, mt_start, pos_leq, unrelative,
};
use crate::marktree::meta::MetaCount;
use crate::marktree::node::{MAX_KEYS, Node, id2node};
use crate::marktree::pair::marktree_intersect_pair;
use crate::memory::{xfree, xmemdup};
use crate::registry::{IdMap, id_map};
use crate::types::{
    DecorInlineData, MTKey, MTNode, MTPos, MarkTree, MarkTreeIter, size_t, uint32_t, uint64_t,
};

use super::{NULL, marktree_del_itr, marktree_lookup, marktree_lookup_ns, marktree_put};

/// Check every invariant of the whole tree, aborting on the first violation.
///
/// # Safety
/// `b` must be a live tree.
pub unsafe fn marktree_check(b: &mut MarkTree) {
    // SAFETY: `b` is a live tree, so its root is null or one of its live nodes.
    let Some(root) = (unsafe { Node::from_ptr(b.root) }) else {
        debug_assert!(b.n_keys == 0 as size_t, "b->n_keys == 0");
        debug_assert!(b.n_nodes == 0 as size_t, "b->n_nodes == 0");
        // The C's `b->id2node == NULL` arm is dead — `id2node` was a
        // one-element array, so its address was never null — and stays dead
        // here, where it is a table by value.
        assert!(
            b.id2node.is_empty(),
            "b->id2node == NULL || map_size(b->id2node) == 0"
        );
        return;
    };
    let mut last = MTPos::default();
    let mut last_right = false;
    let meta_root = b.meta_root;
    let nkeys = check_node(b, root, &mut last, &mut last_right, meta_root);
    debug_assert!(b.n_keys == nkeys, "b->n_keys == nkeys");
    debug_assert!(
        b.n_keys == b.id2node.len(),
        "b->n_keys == map_size(b->id2node)"
    );
}

/// Each node's intersection set as [`recurse_nodes`] found it: a
/// sentinel-terminated buffer this file owns, filed under the node's
/// address.
type Records = IdMap<*const MTNode, *mut uint64_t>;

/// Check `x`'s own invariants and its whole subtree's, and answer how many
/// keys the subtree holds.
///
/// `last` carries the previous key's absolute position across the walk and
/// `last_right` its gravity, which is what makes the ordering check able to
/// span a node boundary. `meta_node` is what the *parent* recorded for this
/// subtree, and the last assertion is that a fresh walk agrees with it.
fn check_node(
    b: &mut MarkTree,
    x: Node,
    last: &mut MTPos,
    last_right: &mut bool,
    meta_node: MetaCount,
) -> size_t {
    let n = x.key_count();
    debug_assert!(n <= MAX_KEYS, "x->n <= 2 * T - 1");
    assert!(
        n >= if x.as_ptr() != b.root {
            MT_BRANCH_FACTOR as usize - 1
        } else {
            0
        },
        "x->n >= (x != b->root ? T - 1 : 0)"
    );
    let mut n_keys = n;
    for i in 0..n {
        if !x.is_leaf() {
            n_keys += check_node(b, x.child(i), last, last_right, x.child_meta(i));
        } else {
            *last = MTPos::default();
        }
        if i > 0 {
            unrelative(x.key(i - 1).pos, last);
        }
        let key = x.key(i);
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
            // SAFETY: `b` is a live tree.
            unsafe { id2node(b, mt_lookup_key(key)) } == x.as_ptr(),
            "pmap_get(uint64_t)(b->id2node, mt_lookup_key(x->key[i])) == x"
        );
    }
    if !x.is_leaf() {
        n_keys += check_node(b, x.child(n), last, last_right, x.child_meta(n));
        unrelative(x.key(n - 1).pos, last);
        for i in 0..=n {
            let child = x.child(i);
            debug_assert!(child.parent() == Some(x), "x->ptr[i]->parent == x");
            debug_assert!(child.parent_index() == i, "x->ptr[i]->p_idx == i");
            assert!(
                child.level() == x.level() - 1,
                "x->ptr[i]->level == x->level - 1"
            );
            for j in 0..i {
                debug_assert!(child != x.child(j), "x->ptr[i] != x->ptr[j]");
            }
        }
    } else if n > 0 {
        *last = x.key(n - 1).pos;
    }
    debug_assert!(meta_node == x.meta(), "meta_node_ref[m] == meta_node[m]");
    n_keys
}

/// Rebuild every intersection set from the pairs themselves and check it
/// against what was there.
///
/// Three steps: move each node's set aside and empty it; walk every mark and,
/// for each start of a pair, intersect the nodes between the two halves as if
/// the pair had just been inserted; then compare each node's rebuilt set
/// against the one that was moved aside.
///
/// # Safety
/// `b` must be a live tree.
pub unsafe fn marktree_check_intersections(b: &mut MarkTree) -> bool {
    // SAFETY: `b` is a live tree, so its root is null or one of its live nodes.
    let Some(root) = (unsafe { Node::from_ptr(b.root) }) else {
        return true;
    };
    let mut checked: Records = id_map();
    // SAFETY: `root` is a live node of `b`.
    unsafe { recurse_nodes(root, &mut checked) };
    let mut itr = MarkTreeIter::default();
    // SAFETY: `b` is a live tree and this is what positions `itr` in it.
    unsafe { marktree_itr_first(b, &mut itr) };
    loop {
        // SAFETY: `itr` is positioned in `b`, or empty.
        let mark = unsafe { marktree_itr_current(&mut itr) };
        if mark.pos.row < 0 {
            break;
        }
        if mt_start(mark) {
            let mut end_itr = MarkTreeIter::default();
            let end_id = mt_lookup_id(mark.ns, mark.id, true);
            // SAFETY: `b` is live; a lookup only writes the iterator it is
            // handed, and answers a negative row when there is no such mark.
            let k = unsafe { marktree_lookup(b, end_id, Some(&mut end_itr)) };
            if k.pos.row >= 0 {
                // A copy, because intersecting walks the start iterator.
                let mut start_itr = itr;
                let id = mt_lookup_key(mark);
                // SAFETY: `b` is live and both iterators are positioned in it.
                unsafe { marktree_intersect_pair(b, id, &mut start_itr, &end_itr, false) };
            }
        }
        // SAFETY: `b` is live and `itr` is positioned in it.
        unsafe { marktree_itr_next(b, &mut itr) };
    }
    // SAFETY: `root` is live and `checked` holds what its subtree intersected.
    let status = unsafe { recurse_nodes_compare(root, &checked) };
    // SAFETY: the values are the buffers `recurse_nodes` allocated, and
    // nothing names them afterwards; the table itself drops here.
    for &record in checked.values() {
        unsafe { xfree(record.cast()) };
    }
    status
}

/// Record `x`'s intersection set in `checked` and empty it, for the whole
/// subtree, so the walk that rebuilds the sets can be compared against the
/// record.
///
/// # Safety
/// `x` must be a live node.
unsafe fn recurse_nodes(x: Node, checked: &mut Records) {
    let set = x.intersection();
    if !set.is_empty() {
        // The recorded copy is terminated with a sentinel no id can equal.
        set.push(uint64_t::MAX);
        let bytes = set.len() * size_of::<uint64_t>();
        let copy = if set.is_inline() {
            // SAFETY: the set's slice names `bytes` initialised bytes.
            unsafe { xmemdup(set.as_slice().as_ptr().cast(), bytes) }
        } else {
            NULL
        };
        // An inline set was just copied out; one that had gone to the heap
        // hands over the buffer it already owns.
        let heap = set.take_heap();
        let owned = if heap.is_null() { copy } else { heap };
        checked.insert(x.as_ptr().cast_const(), owned.cast::<uint64_t>());
    }
    if !x.is_leaf() {
        for i in 0..=x.key_count() {
            // SAFETY: a live node's children are live.
            unsafe { recurse_nodes(x.child(i), checked) };
        }
    }
}

/// Does `x`'s rebuilt intersection set match what [`recurse_nodes`] recorded
/// for it? Recurses over the whole subtree.
///
/// # Safety
/// `checked` must be the table [`recurse_nodes`] filled, `x` a live node.
unsafe fn recurse_nodes_compare(x: Node, checked: &Records) -> bool {
    let recorded = checked
        .get(&x.as_ptr().cast_const())
        .copied()
        .unwrap_or(ptr::null_mut());
    let rebuilt = x.intersection();
    if recorded.is_null() {
        if !rebuilt.is_empty() {
            return false;
        }
    } else {
        // The record is sentinel-terminated; a node with an empty set was
        // never recorded at all.
        let mut i = 0;
        loop {
            // SAFETY: the record ends in the sentinel, which stops the walk
            // before it can read past the buffer.
            let id = unsafe { *recorded.add(i) };
            if id == uint64_t::MAX {
                if i != rebuilt.len() {
                    return false;
                }
                break;
            }
            if rebuilt.len() <= i || id != rebuilt.as_slice()[i] {
                return false;
            }
            i += 1;
        }
    }
    if !x.is_leaf() {
        for i in 0..=x.key_count() {
            // SAFETY: a live node's children are live.
            if !unsafe { recurse_nodes_compare(x.child(i), checked) } {
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
///
/// # Safety
/// `b` must be a live tree.
pub unsafe fn marktree_put_test(
    b: &mut MarkTree,
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
        decor_data: DecorInlineData {
            hl: DECOR_HIGHLIGHT_INLINE_INIT,
        },
    };
    let end = end.unwrap_or(MarkEnd {
        row: -1,
        col: -1,
        right_gravity: false,
    });
    // SAFETY: `b` is a live tree.
    unsafe { marktree_put(b, key, end.row, end.col, end.right_gravity) };
}

/// `mt_right` where the unit suite can reach it.
pub unsafe fn mt_right_test(key: MTKey) -> bool {
    mt_right(key)
}

/// Delete both halves of the pair `(ns, id)`.
///
/// # Safety
/// `b` must be a live tree holding that pair.
pub unsafe fn marktree_del_pair_test(b: &mut MarkTree, ns: uint32_t, id: uint32_t) {
    let mut itr = MarkTreeIter::default();
    // SAFETY: `b` is live; a lookup only writes the iterator it is handed.
    unsafe { marktree_lookup_ns(b, ns, id, false, Some(&mut itr)) };
    // SAFETY: `b` is live and `itr` is positioned in it.
    let other = unsafe { marktree_del_itr(b, &mut itr, false) };
    debug_assert!(other != 0, "other");
    // SAFETY: `b` is live; a lookup only writes the iterator it is handed.
    unsafe { marktree_lookup(b, other, Some(&mut itr)) };
    // SAFETY: `b` is live and `itr` is positioned in it.
    unsafe { marktree_del_itr(b, &mut itr, false) };
}
