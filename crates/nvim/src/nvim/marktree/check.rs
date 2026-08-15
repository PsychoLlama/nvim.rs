#![deny(unsafe_op_in_unsafe_fn)]

//! Whole-tree invariant checks, and the entry points the unit spec drives.
//!
//! `test/unit/marktree_spec.lua` builds trees of thousands of random marks and
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

use crate::src::nvim::marktree::iter::{
    marktree_itr_current, marktree_itr_first, marktree_itr_next,
};
use crate::src::nvim::marktree::key::{
    DECOR_HIGHLIGHT_INLINE_INIT, MT_BRANCH_FACTOR, MT_FLAG_DECOR_VIRT_TEXT_INLINE, mt_flags,
    mt_lookup_id, mt_lookup_key, mt_right, mt_start, pos_leq, unrelative,
};
use crate::src::nvim::marktree::meta::MetaCount;
use crate::src::nvim::marktree::node::{MAX_KEYS, Node, id2node};
use crate::src::nvim::marktree::pair::marktree_intersect_pair;
use crate::src::nvim::memory::{xfree, xmemdup};
use crate::src::nvim::types::{
    DecorInlineData, MTKey, MTPos, Map_ptr_t_ptr_t, Map_uint64_t_ptr_t, MarkTree, MarkTreeIter,
    Set_ptr_t, size_t, uint16_t, uint32_t, uint64_t,
};

use super::{
    MAPHASH_INIT, NULL, map_get_ptr_t_ptr_t, map_put_ptr_t_ptr_t, marktree_del_itr,
    marktree_lookup, marktree_lookup_ns, marktree_put,
};

/// Check every invariant of the whole tree, aborting on the first violation.
///
/// # Safety
/// `b` must be a live tree.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_check(b: &mut MarkTree) {
    // SAFETY: `b` is a live tree, so its root is null or one of its live nodes.
    let Some(root) = (unsafe { Node::from_ptr(b.root) }) else {
        debug_assert!(b.n_keys == 0 as size_t, "b->n_keys == 0");
        debug_assert!(b.n_nodes == 0 as size_t, "b->n_nodes == 0");
        // The C's `b->id2node == NULL` arm is dead — `id2node` is a
        // one-element array, so its address is never null — and stays dead
        // here for the same reason.
        // SAFETY: `b` is live, so its `id2node` array is a live map.
        assert!(
            unsafe { id2node_size(b) } == 0 as uint32_t,
            "b->id2node == NULL || map_size(b->id2node) == 0"
        );
        return;
    };
    let mut last = MTPos::default();
    let mut last_right = false;
    let meta_root = b.meta_root;
    let nkeys = check_node(b, root, &mut last, &mut last_right, meta_root);
    debug_assert!(b.n_keys == nkeys, "b->n_keys == nkeys");
    // SAFETY: `b` is live, so its `id2node` array is a live map.
    let mapped = unsafe { id2node_size(b) };
    debug_assert!(
        b.n_keys == mapped as size_t,
        "b->n_keys == map_size(b->id2node)"
    );
}

/// How many keys the tree's id map holds.
///
/// # Safety
/// `b` must be a live tree.
unsafe fn id2node_size(b: &MarkTree) -> uint32_t {
    let map: *const Map_uint64_t_ptr_t = (&raw const b.id2node).cast();
    // SAFETY: `b` is live, so its one-element `id2node` array is a live map.
    unsafe { (*map).set.h.size }
}

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
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_check_intersections(b: &mut MarkTree) -> bool {
    // SAFETY: `b` is a live tree, so its root is null or one of its live nodes.
    let Some(root) = (unsafe { Node::from_ptr(b.root) }) else {
        return true;
    };
    // klib's `MAP_INIT`: an empty map owning nothing.
    let mut checked = Map_ptr_t_ptr_t {
        set: Set_ptr_t {
            h: MAPHASH_INIT,
            keys: ptr::null_mut(),
        },
        values: ptr::null_mut(),
    };
    // SAFETY: `checked` is a live, empty map this function owns.
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
    let status = unsafe { recurse_nodes_compare(root, &mut checked) };
    // SAFETY: `checked` is the live map built above, and its values are the
    // buffers `recurse_nodes` allocated. Nothing names it afterwards.
    unsafe { destroy_checked(&mut checked) };
    status
}

/// Record `x`'s intersection set in `checked` and empty it, for the whole
/// subtree, so the walk that rebuilds the sets can be compared against the
/// record.
///
/// # Safety
/// `checked` must be a live map.
unsafe fn recurse_nodes(x: Node, checked: &mut Map_ptr_t_ptr_t) {
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
        // SAFETY: `checked` is a live map per the caller.
        unsafe { map_put_ptr_t_ptr_t(checked, x.as_ptr().cast(), owned) };
    }
    if !x.is_leaf() {
        for i in 0..=x.key_count() {
            // SAFETY: `checked` is a live map per the caller.
            unsafe { recurse_nodes(x.child(i), checked) };
        }
    }
}

/// Does `x`'s rebuilt intersection set match what [`recurse_nodes`] recorded
/// for it? Recurses over the whole subtree.
///
/// # Safety
/// `checked` must be the live map [`recurse_nodes`] filled.
unsafe fn recurse_nodes_compare(x: Node, checked: &mut Map_ptr_t_ptr_t) -> bool {
    // SAFETY: `checked` is a live map per the caller.
    let recorded = unsafe { map_get_ptr_t_ptr_t(checked, x.as_ptr().cast()) } as *mut uint64_t;
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
            // SAFETY: `checked` is a live map per the caller.
            if !unsafe { recurse_nodes_compare(x.child(i), checked) } {
                return false;
            }
        }
    }
    true
}

/// Free the record map and every buffer in it — klib's `map_destroy`, plus
/// the values, which this file allocated itself.
///
/// # Safety
/// `checked` must be a live map whose values are `xfree`-able buffers, and
/// nothing may name it afterwards.
unsafe fn destroy_checked(checked: &mut Map_ptr_t_ptr_t) {
    for i in 0..checked.set.h.n_keys as usize {
        // SAFETY: the map holds `n_keys` values, each its own buffer.
        unsafe { xfree(*checked.values.add(i)) };
    }
    // SAFETY: the key array is the map's own.
    unsafe { xfree(checked.set.keys.cast()) };
    // SAFETY: the hash table is the map's own.
    unsafe { xfree(checked.set.h.hash.cast()) };
    // SAFETY: the value array is the map's own.
    unsafe { xfree(checked.values.cast()) };
}

/// Put a mark, spelling out every flag — `marktree_spec.lua`'s way in.
///
/// # Safety
/// `b` must be a live tree.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_put_test(
    b: &mut MarkTree,
    ns: uint32_t,
    id: uint32_t,
    row: ::core::ffi::c_int,
    col: ::core::ffi::c_int,
    right_gravity: bool,
    end_row: ::core::ffi::c_int,
    end_col: ::core::ffi::c_int,
    end_right: bool,
    meta_inline: bool,
) {
    let inline = if meta_inline {
        MT_FLAG_DECOR_VIRT_TEXT_INLINE
    } else {
        0
    };
    let flags = mt_flags(right_gravity, false, false, false) as ::core::ffi::c_int | inline;
    let key = MTKey {
        pos: MTPos { row, col },
        ns,
        id,
        flags: flags as uint16_t,
        decor_data: DecorInlineData {
            hl: DECOR_HIGHLIGHT_INLINE_INIT,
        },
    };
    // SAFETY: `b` is a live tree.
    unsafe { marktree_put(b, key, end_row, end_col, end_right) };
}

/// `mt_right` where `marktree_spec.lua` can reach it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn mt_right_test(key: MTKey) -> bool {
    mt_right(key)
}

/// Delete both halves of the pair `(ns, id)`.
///
/// # Safety
/// `b` must be a live tree holding that pair.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_del_pair_test(b: &mut MarkTree, ns: uint32_t, id: uint32_t) {
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
