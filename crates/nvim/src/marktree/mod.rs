#![deny(unsafe_op_in_unsafe_fn)]

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
//! * **Ranges are two keys.** A `(ns, id)` pair with `MT_FLAG_END` set on the
//!   second. They are ordered like any other key, and the tree keeps them
//!   consistent across splices that would otherwise reverse them. Everything
//!   that maintains the second key lives in [`pair`].
//! * **Covering ranges are recorded on nodes, not smeared over keys.** A node
//!   carries the ids of the ranges that cover the whole of it; see
//!   [`intersect`]. That is what makes "which ranges cover this position" a
//!   walk down the tree.
//!
//! A tree is reached as `&mut MarkTree` and a node as [`Node`](node::Node), so
//! the entry points below are unsafe only in what a *reference* cannot say:
//! that an iterator handed alongside a tree is positioned in that tree.
//!
//! Derived from kbtree in klib, and from the marker tree of the Atom editor.
//! The layouts of `MarkTree`, `MarkTreeIter`, `MTNode`, `MTKey` and `MTPos` are
//! pinned by `test/unit/marktree_spec.lua`, which builds them with LuaJIT's FFI
//! and reads their fields directly.

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

use core::ffi::c_int;
use core::ptr;

use crate::global_cell::GlobalCell;
use crate::map::{map_del_uint64_t_ptr_t, map_put_ref_ptr_t_ptr_t, mh_get_ptr_t};
use crate::marktree::key::*;
use crate::marktree::meta::*;
use crate::marktree::node::*;
pub use crate::marktree::{check::*, inspect::*, iter::*, pair::*, rebalance::*, splice::*};
use crate::memory::xfree;
use crate::types::{
    MTKey, MTPos, Map_ptr_t_ptr_t, Map_uint64_t_MTDamagePair, Map_uint64_t_ptr_t, MapHash,
    MarkTree, MarkTreeIter, Set_uint64_t, ptr_t, uint16_t, uint32_t, uint64_t,
};

/// What a splice recorded about pairs whose halves crossed while it ran.
pub type MTDamageMap = Map_uint64_t_MTDamagePair;
pub const UINT32_MAX: ::core::ffi::c_uint = 4294967295 as ::core::ffi::c_uint;
pub const NULL: *mut ::core::ffi::c_void = ::core::ptr::null_mut::<::core::ffi::c_void>();
static value_init_ptr_t: GlobalCell<ptr_t> = GlobalCell::new(NULL);
pub const MAPHASH_INIT: MapHash = MapHash {
    n_buckets: 0 as uint32_t,
    size: 0 as uint32_t,
    n_occupied: 0 as uint32_t,
    upper_bound: 0 as uint32_t,
    n_keys: 0 as uint32_t,
    keys_capacity: 0 as uint32_t,
    hash: ::core::ptr::null_mut::<uint32_t>(),
};
pub const MH_TOMBSTONE: ::core::ffi::c_uint = UINT32_MAX;

/// Store `value` under `key`, replacing whatever was there.
///
/// # Safety
/// `map` must be a live map.
#[inline]
unsafe fn map_put_ptr_t_ptr_t(map: *mut Map_ptr_t_ptr_t, key: ptr_t, value: ptr_t) {
    let (init, fresh) = (ptr::null_mut(), ptr::null_mut());
    // SAFETY: `map` is live; the two nulls decline its optional "initial value"
    // and "was it new" out-parameters.
    let slot = unsafe { map_put_ref_ptr_t_ptr_t(map, key, init, fresh) };
    // SAFETY: `map_put_ref` answers a live slot of the map it was handed.
    unsafe { *slot = value };
}

/// The value stored under `key`, or null.
///
/// # Safety
/// `map` must be a live map.
#[inline]
unsafe fn map_get_ptr_t_ptr_t(map: *mut Map_ptr_t_ptr_t, key: ptr_t) -> ptr_t {
    // SAFETY: `map` is live, so its `set` is the map's own live key set.
    let k = unsafe { mh_get_ptr_t(&raw mut (*map).set, key) };
    if k == MH_TOMBSTONE as uint32_t {
        return value_init_ptr_t.get();
    }
    // SAFETY: a hash index the set answered is in bounds of `map.values`.
    unsafe { *(*map).values.add(k as usize) }
}

/// Insert `key`, and — where `end_row` is non-negative — the end key that makes
/// it a range.
///
/// # Safety
/// `b` must be a live tree.
pub unsafe fn marktree_put(
    b: &mut MarkTree,
    mut key: MTKey,
    end_row: c_int,
    end_col: c_int,
    end_right: bool,
) {
    assert!(
        key.flags as c_int & !(MT_FLAG_EXTERNAL_MASK | MT_FLAG_RIGHT_GRAVITY) == 0,
        "!(key.flags & ~(MT_FLAG_EXTERNAL_MASK | MT_FLAG_RIGHT_GRAVITY))"
    );
    if end_row >= 0 {
        key.flags |= MT_FLAG_PAIRED as uint16_t;
    }
    // SAFETY: `b` is a live tree, and `key` is the caller's to insert.
    unsafe { marktree_put_key(b, key) };
    if end_row < 0 {
        return;
    }

    let mut end_key = key;
    end_key.flags = (key.flags & !(MT_FLAG_RIGHT_GRAVITY as uint16_t))
        | MT_FLAG_END as uint16_t
        | if end_right {
            MT_FLAG_RIGHT_GRAVITY as uint16_t
        } else {
            0
        };
    end_key.pos = MTPos {
        row: end_row,
        col: end_col,
    };
    // SAFETY: as above.
    unsafe { marktree_put_key(b, end_key) };

    let mut itr = MarkTreeIter::default();
    let mut end_itr = MarkTreeIter::default();
    // SAFETY: `b` is live and the key was just inserted, so this finds it.
    unsafe { marktree_lookup(b, mt_lookup_key(key), Some(&mut itr)) };
    // SAFETY: as above, for the end key.
    unsafe { marktree_lookup(b, mt_lookup_key(end_key), Some(&mut end_itr)) };
    // SAFETY: `b` is live and both iterators are positioned in it.
    unsafe { marktree_intersect_pair(b, mt_lookup_key(key), &mut itr, &end_itr, false) };
}

/// Insert one already-built key, splitting a full node on the way down.
///
/// The first root is allocated at the internal node's size even though it
/// starts at level zero, so the tail it never uses is wasted for a tree that
/// stays under one node. Upstream does the same; nothing depends on it beyond
/// `marktree_free_node` not caring which size a node was.
///
/// # Safety
/// `b` must be a live tree.
pub unsafe fn marktree_put_key(b: &mut MarkTree, mut k: MTKey) {
    k.flags |= MT_FLAG_REAL as uint16_t; // let's be real.
    if b.root.is_null() {
        // SAFETY: `b` is a live tree, which is all `marktree_alloc_node` wants.
        b.root = unsafe { marktree_alloc_node(b, true) };
    }
    // SAFETY: a non-null root is a live node of `b`.
    let mut r = unsafe { Node::new(b.root) };
    if r.key_count() == MAX_KEYS {
        // The root is full: grow the tree by one level and split into the new
        // root, which is where every level the tree ever gains comes from.
        // SAFETY: `b` is a live tree.
        let s = unsafe { Node::new(marktree_alloc_node(b, true)) };
        b.root = s.as_ptr();
        s.set_level(r.level() + 1);
        s.set_key_count(0);
        s.set_child(0, r);
        s.set_child_meta(0, b.meta_root);
        r.set_parent(Some(s));
        r.set_parent_index(0);
        split_node(b, s, 0, k);
        r = s;
    }

    let meta_inc = meta_describe_key(k);
    marktree_putp_aux(b, r, k, &meta_inc);
    meta_add(&mut b.meta_root, &meta_inc);
    b.n_keys = b.n_keys.wrapping_add(1);
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
    fn bump(self, itr: &mut MarkTreeIter, by: c_int) {
        match self {
            Lasti::Cur => itr.i += by,
            Lasti::Level(lvl) => itr.s[lvl].i += by,
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
///
/// # Safety
/// `b` must be a live tree and `itr` positioned on one of its keys.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_del_itr(
    b: &mut MarkTree,
    itr: &mut MarkTreeIter,
    rev: bool,
) -> uint64_t {
    let mut adjustment = 0;
    // SAFETY: a positioned iterator names a live node of `b`.
    let cur = unsafe { Node::new(itr.x) };
    let curi = itr.i as usize;
    let id = mt_lookup_key(cur.key(curi));

    // 1. Orphan the other half of the pair, and retract the records of what
    //    the range covered. NB: the retraction must name exactly the pair the
    //    records were made for.
    let raw = cur.key(curi);
    let mut other = 0;
    if mt_paired(raw) && raw.flags as c_int & MT_FLAG_ORPHANED == 0 {
        other = mt_lookup_key_side(raw, !mt_end(raw));
        let mut other_itr = MarkTreeIter::default();
        // SAFETY: `b` is live; the lookup positions `other_itr` in it.
        unsafe { marktree_lookup(b, other, Some(&mut other_itr)) };
        // SAFETY: the lookup left `other_itr` on a live node of `b`.
        let onode = unsafe { Node::new(other_itr.x) };
        onode.update_key(other_itr.i as usize, |k| {
            k.flags |= MT_FLAG_ORPHANED as uint16_t
        });
        if mt_start(raw) {
            let mut this_itr = *itr; // a copy, because this one is mutated
            // SAFETY: `b` is live and both iterators are positioned in it.
            unsafe { marktree_intersect_pair(b, id, &mut this_itr, &other_itr, true) };
        } else {
            // SAFETY: as above.
            unsafe { marktree_intersect_pair(b, other, &mut other_itr, itr, true) };
        }
    }

    // 2. An internal key cannot be removed in place; steal the previous key,
    //    which is in a leaf, and put it here instead.
    if !cur.is_leaf() {
        if rev {
            ::std::process::abort();
        }
        // SAFETY: `b` is live and `itr` is positioned in it.
        unsafe { marktree_itr_prev(b, itr) };
        adjustment = -1;
    }

    // 3. Delete the leaf key.
    // SAFETY: `itr` still names a live node of `b` after the step above.
    let mut x = unsafe { Node::new(itr.x) };
    debug_assert!(x.is_leaf(), "x->level == 0");
    let mut intkey = x.key(itr.i as usize);
    let mut meta_inc = meta_describe_key(intkey);
    let hole = itr.i as usize;
    if x.key_count() > hole + 1 {
        x.copy_keys_within(hole + 1..x.key_count(), hole);
    }
    x.set_key_count(x.key_count() - 1);
    b.n_keys = b.n_keys.wrapping_sub(1);
    let map: *mut Map_uint64_t_ptr_t = (&raw mut b.id2node).cast();
    // SAFETY: `map` is `b`'s own live map; the null declines the out-parameter
    // that would report the value removed.
    unsafe { map_del_uint64_t_ptr_t(map, id, ptr::null_mut()) };

    // 4. Write the stolen key over the one the caller wanted gone, rebasing it
    //    onto every node between the two, and repair the covering records the
    //    move disturbed.
    if adjustment == -1 {
        let mut ilvl = itr.lvl - 1;
        let mut lnode = x;
        let mut start_id = 0;
        let mut did_bubble = false;
        if mt_end(intkey) {
            start_id = mt_lookup_key_side(intkey, false);
        }
        loop {
            if ilvl < 0 {
                ::std::process::abort();
            }
            let p = lnode.parent().expect("a node below `cur` has a parent");
            let i = itr.s[ilvl as usize].i as usize;
            debug_assert!(p.child(i) == lnode, "p->ptr[i] == lnode");
            if i > 0 {
                unrelative(p.key(i - 1).pos, &mut intkey.pos);
            }
            if p != cur && start_id != 0 && p.child(0).intersection().contains(start_id) {
                // After the first time round, this also undoes the addition the
                // previous round made just below.
                let last = usize::from(lnode != x);
                // One less than the children, because `ptr[n]` is the last.
                for k in 0..p.key_count() + last {
                    unintersect_node(p.child(k), start_id, true);
                }
                intersect_node(p, start_id);
                did_bubble = true;
            }
            p.update_child_meta(lnode.parent_index(), |m| meta_sub(m, &meta_inc));
            lnode = p;
            ilvl -= 1;
            if lnode == cur {
                break;
            }
        }

        let mut deleted = cur.key(curi);
        meta_inc = meta_describe_key(deleted);
        cur.set_key(curi, intkey);
        // SAFETY: `b` is live and `cur` one of its nodes, holding a key at
        // `curi` — the one just written.
        unsafe { refkey(b, cur.as_ptr(), curi as c_int) };
        // Where we bubbled `start_id` up to a parent, its record is already
        // there; otherwise the leaf may need one of its own.
        if mt_end(cur.key(curi)) && !did_bubble {
            let pi = x.pseudo_index(0); // note: sloppy pseudo-index
            // SAFETY: `b` is a live tree.
            let pi_start = unsafe { pseudo_index_for_id(b, start_id, true) };
            if pi_start > 0 && pi_start < pi {
                intersect_node(x, start_id);
            }
        }

        // The subtree that hung off the replaced key is now based on a
        // different key, so rebase its leftmost spine onto the difference.
        relative(intkey.pos, &mut deleted.pos);
        if deleted.pos.row != 0 || deleted.pos.col != 0 {
            let mut y = Some(cur.child(curi + 1));
            while let Some(node) = y {
                for k in 0..node.key_count() {
                    node.update_key(k, |key| unrelative(deleted.pos, &mut key.pos));
                }
                y = if node.is_leaf() {
                    None
                } else {
                    Some(node.child(0))
                };
            }
        }
        itr.i -= 1;
    }

    let mut lnode = cur;
    while let Some(p) = lnode.parent() {
        p.update_child_meta(lnode.parent_index(), |m| meta_sub(m, &meta_inc));
        lnode = p;
    }
    for m in 0..META_COUNT {
        debug_assert!(
            b.meta_root[m] >= meta_inc[m],
            "b->meta_root[m] >= meta_inc[m]"
        );
    }
    meta_sub(&mut b.meta_root, &meta_inc);

    // 5. Repair upward from the leaf.
    let mut itr_dirty = false;
    let mut rlvl = itr.lvl - 1;
    let mut lasti = Lasti::Cur;
    let mut ppos = itr.pos;
    let min_keys = MT_BRANCH_FACTOR as usize - 1;
    while x.as_ptr() != b.root {
        debug_assert!(rlvl >= 0, "rlvl >= 0");
        let p = x.parent().expect("a node below the root has a parent");
        if x.key_count() >= min_keys {
            // This node is fine, so the rest of the tree is.
            break;
        }
        let pi = itr.s[rlvl as usize].i;
        debug_assert!(p.child(pi as usize) == x, "p->ptr[pi] == x");
        if pi > 0 {
            ppos.row -= p.key(pi as usize - 1).pos.row;
            ppos.col = itr.s[rlvl as usize].oldcol;
        }
        // `ppos` is now the position of `p`.

        if pi > 0 && p.child(pi as usize - 1).key_count() > min_keys {
            lasti.bump(itr, 1);
            itr_dirty = true;
            // Steal one key from the left neighbour.
            pivot_right(b, ppos, p, pi as usize - 1);
            break;
        } else if pi < p.key_count() as c_int && p.child(pi as usize + 1).key_count() > min_keys {
            // Steal one key from the right neighbour.
            pivot_left(b, ppos, p, pi as usize);
            break;
        } else if pi > 0 {
            assert!(
                p.child(pi as usize - 1).key_count() == min_keys,
                "p->ptr[pi - 1]->n == T - 1"
            );
            // Merge with the left neighbour.
            lasti.bump(itr, MT_BRANCH_FACTOR as c_int);
            x = merge_node(b, p, pi as usize - 1);
            if lasti == Lasti::Cur {
                // TRICKY: we merged the node the iterator was on.
                itr.x = x.as_ptr();
            }
            itr.s[rlvl as usize].i -= 1;
            itr_dirty = true;
        } else {
            assert!(
                pi < p.key_count() as c_int && p.child(pi as usize + 1).key_count() == min_keys,
                "pi < p->n && p->ptr[pi + 1]->n == T - 1"
            );
            // No iterator adjustment is needed.
            merge_node(b, p, pi as usize);
        }
        lasti = Lasti::Level(rlvl as usize);
        rlvl -= 1;
        x = p;
    }

    // 6. The root may have run out of keys; drop it and promote its child.
    // SAFETY: `b` is live, so its root is one of its live nodes.
    let root = unsafe { Node::new(b.root) };
    if root.key_count() == 0 {
        if itr.lvl > 0 {
            itr.s.copy_within(1..itr.lvl as usize, 0);
            itr.lvl -= 1;
        }
        if !root.is_leaf() {
            let promoted = root.child(0);
            debug_assert!(
                b.meta_root == root.child_meta(0),
                "b->meta_root[m] == oldroot->meta[0][m]"
            );
            b.root = promoted.as_ptr();
            promoted.set_parent(None);
            // SAFETY: `b` is live and `root` is now detached from it.
            unsafe { marktree_free_node(b, root.as_ptr()) };
        } else {
            // No items left; nothing for the iterator to point at. Not strictly
            // needed — deleting the right-most mark would be handled anyway.
            itr.x = ptr::null_mut();
        }
    }

    if !itr.x.is_null() && itr_dirty {
        // SAFETY: `b` is live and `itr` names a live node of it.
        unsafe { marktree_itr_fix_pos(b, itr) };
    }

    // BONUS STEP: leave the iterator on the key after the deleted one.
    if adjustment == -1 {
        // Tricky: we stand at the hole in the previous leaf, and the internal
        // key is now the one we stole, so skip that one as well.
        // SAFETY: `b` is live and `itr` is positioned in it.
        unsafe { marktree_itr_next(b, itr) };
        // SAFETY: as above.
        unsafe { marktree_itr_next(b, itr) };
    } else if !itr.x.is_null() {
        // SAFETY: a non-null `itr.x` names a live node of `b`.
        let node = unsafe { Node::new(itr.x) };
        if itr.i as usize >= node.key_count() {
            // We deleted the last key of a leaf node; go to the internal key
            // after it.
            debug_assert!(node.is_leaf(), "itr->x->level == 0");
            // SAFETY: as above.
            unsafe { marktree_itr_next(b, itr) };
        }
    }

    other
}

/// Re-count the meta kinds of the key under the iterator, up to the root, after
/// a consumer edited its flags in place.
///
/// # Safety
/// `b` must be a live tree and `itr` positioned on one of its keys.
pub unsafe fn marktree_revise_meta(b: &mut MarkTree, itr: &mut MarkTreeIter, old_key: MTKey) {
    // SAFETY: a positioned iterator names a live node of `b`.
    let x = unsafe { Node::new(itr.x) };
    let meta_old = meta_describe_key(old_key);
    let meta_new = meta_describe_key(x.key(itr.i as usize));
    if meta_new == meta_old {
        return;
    }

    let mut lnode = x;
    while let Some(p) = lnode.parent() {
        p.update_child_meta(lnode.parent_index(), |m| {
            meta_apply_delta(m, &meta_new, &meta_old)
        });
        lnode = p;
    }
    meta_apply_delta(&mut b.meta_root, &meta_new, &meta_old);
}

/// Drop every mark and every node, leaving the tree as it was born.
///
/// # Safety
/// `b` must be a live tree, and nothing may name its nodes afterwards.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_clear(b: &mut MarkTree) {
    if !b.root.is_null() {
        // SAFETY: a non-null root is a live node of `b`.
        let root = unsafe { Node::new(b.root) };
        // SAFETY: `b` is live and `root` is the whole of its tree.
        unsafe { marktree_free_subtree(b, root) };
        b.root = ptr::null_mut();
    }

    // `map_destroy(uint64_t, b->id2node)`.
    let map = &mut b.id2node[0];
    // SAFETY: the key array is the map's own, and nothing names it after this.
    unsafe { xfree(map.set.keys.cast()) };
    // SAFETY: as above, for the hash array.
    unsafe { xfree(map.set.h.hash.cast()) };
    map.set = Set_uint64_t {
        h: MAPHASH_INIT,
        keys: ptr::null_mut(),
    };
    // SAFETY: as above, for the value array.
    unsafe { xfree(map.values.cast()) };
    map.values = ptr::null_mut();

    b.n_keys = 0;
    b.meta_root = [0; META_COUNT];
    debug_assert!(b.n_nodes == 0, "b->n_nodes == 0");
}

/// Free `x` and everything below it.
///
/// # Safety
/// `b` must be a live tree and `x` one of its nodes, detached from anything
/// that outlives the call.
pub unsafe fn marktree_free_subtree(b: &mut MarkTree, x: Node) {
    if !x.is_leaf() {
        for i in 0..=x.key_count() {
            // SAFETY: a child of a live internal node is a live node of `b`.
            unsafe { marktree_free_subtree(b, x.child(i)) };
        }
    }
    // SAFETY: `x`'s children are gone and nothing else names it.
    unsafe { marktree_free_node(b, x.as_ptr()) };
}

/// Try to move `key` to `newpos` without it leaving the leaf `x` it sits in.
///
/// Answers false when the new position is outside the span `x` covers, which is
/// when the caller has to delete and re-insert the mark instead. Nothing is
/// written in that case.
fn move_within_leaf(x: Node, itr: &MarkTreeIter, mut key: MTKey, mut newpos: MTPos) -> bool {
    if x.parent().is_some() {
        // Strictly *after* the key before `x` — not optimal when `x` is the
        // very first leaf of the entire tree, but that is fine.
        if !pos_less(itr.pos, newpos) {
            return false;
        }
        relative(itr.pos, &mut newpos);
        // Strictly before the end of `x`. (This could be made sharper by
        // finding the internal key just after `x`, but meh.)
        if !pos_less(newpos, x.key(x.key_count() - 1).pos) {
            return false;
        }
    }
    // Otherwise the tree is one node, so `newpos` is already relative to
    // `itr.pos`.
    if key.pos.row == newpos.row && key.pos.col == newpos.col {
        return true;
    }
    key.pos = newpos;

    // Tricky: movement could be minimised better in either direction.
    let (mut new_i, matched) = find_key(x.keys(), key);
    if !matched {
        new_i += 1;
    }
    let (i, new_i) = (itr.i as usize, new_i as usize);
    if new_i == i {
        x.update_key(i, |k| k.pos = newpos);
    } else if new_i < i {
        x.copy_keys_within(new_i..i, new_i + 1);
        x.set_key(new_i, key);
    } else {
        x.copy_keys_within(i + 1..new_i, i);
        x.set_key(new_i - 1, key);
    }
    true
}

/// Move the mark the iterator names to `(row, col)`.
///
/// The iterator is invalid after the call unless the mark stayed put within its
/// own leaf.
///
/// # Safety
/// `b` must be a live tree and `itr` positioned on one of its keys.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_move(
    b: &mut MarkTree,
    itr: &mut MarkTreeIter,
    row: c_int,
    col: c_int,
) {
    // SAFETY: a positioned iterator names a live node of `b`.
    let x = unsafe { Node::new(itr.x) };
    let mut key = x.key(itr.i as usize);
    let newpos = MTPos { row, col };
    if x.is_leaf() && move_within_leaf(x, itr, key, newpos) {
        return;
    }

    // SAFETY: `b` is live and `itr` is positioned in it.
    let other = unsafe { marktree_del_itr(b, itr, false) };
    key.pos = newpos;
    // SAFETY: `b` is a live tree.
    unsafe { marktree_put_key(b, key) };
    if other != 0 {
        // SAFETY: `b` is live and `key` was just re-inserted into it.
        unsafe { marktree_restore_pair(b, key) };
    }
    itr.x = ptr::null_mut(); // the put may have invalidated it
}

/// The key with this `(ns, id, end)`, or `MT_INVALID_KEY`.
///
/// # Safety
/// `b` must be a live tree.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_lookup_ns(
    b: &mut MarkTree,
    ns: uint32_t,
    id: uint32_t,
    end: bool,
    itr: Option<&mut MarkTreeIter>,
) -> MTKey {
    // SAFETY: `b` is live; the iterator is optional and is written, not read.
    unsafe { marktree_lookup(b, mt_lookup_id(ns, id, end), itr) }
}

/// The key with this lookup handle, or `MT_INVALID_KEY`. `itr`, where given, is
/// left on the key — or emptied when there is none.
///
/// # Safety
/// `b` must be a live tree.
pub unsafe fn marktree_lookup(
    b: &mut MarkTree,
    id: uint64_t,
    itr: Option<&mut MarkTreeIter>,
) -> MTKey {
    // SAFETY: `b` is live, so `id2node` answers null or one of its live nodes.
    let Some(n) = (unsafe { Node::from_ptr(id2node(b, id)) }) else {
        if let Some(itr) = itr {
            itr.x = ptr::null_mut();
        }
        return MT_INVALID_KEY;
    };
    for i in 0..n.key_count() {
        if mt_lookup_key(n.key(i)) == id {
            // SAFETY: `b` is live, `n` is one of its nodes and holds a key at
            // `i`; `itr` is null or the caller's iterator.
            return unsafe { marktree_itr_set_node(b, itr, n, i as c_int) };
        }
    }
    // The id map named a node that does not hold the key, so the tree is
    // corrupt and there is nothing sensible to answer.
    ::std::process::abort()
}
