#![deny(unsafe_op_in_unsafe_fn)]

//! Adjusting every mark for a buffer edit.
//!
//! `marktree_splice` is given the start of the edited region, the extent that
//! was removed and the extent that replaced it, and has to move every mark at
//! or after the start. Three regions behave differently:
//!
//! * before the start: untouched;
//! * inside the deleted region: collapses to one end of it, chosen by the
//!   mark's gravity — right-gravity marks ride to the new end, left-gravity
//!   ones stay at the start;
//! * after: shifted by the difference between the old and new extents, and
//!   only the first line of the tail also moves by a column delta.
//!
//! Because a node's keys are stored relative to each other, a splice that lands
//! in the middle of a node can be applied to the node's *own* position and stop
//! there — the keys inside it never move. That is the whole reason the tree is
//! shaped this way, and it is why the code walks down looking for the deepest
//! node whose span contains the whole edit before it starts rewriting anything.
//!
//! Collapsing a range can leave two marks in the wrong order, since the
//! relative encoding cannot express a negative offset. [`swap_keys`] restores
//! the order and [`check_damage`] records the pairs whose ends crossed, so
//! [`marktree_restore_pair`] can put them back once the walk is done.
//!
//! # How this file reaches the tree
//!
//! The walk is driven by a `MarkTreeIter`. `iter.rs` takes that by reference
//! now, but its entry points are still `unsafe` — a reference cannot say that
//! an iterator is positioned in the tree handed alongside it — so the calls
//! are wrapped once each in the small shims below rather than at every site.
//! Two facts they all rest on: the iterators here are locals this file
//! positioned itself, and a positioned iterator (`x` non-null — the loops all
//! test that) names a live node of the tree it was positioned in. The shims
//! also fix the arguments this file never varies, which is most of them.

use core::ffi::c_int;
use core::ptr;

use crate::map::map_put_ref_uint64_t_mt_damage_pair;
use crate::marktree::iter::{
    MT_MAX_DEPTH, marktree_itr_current, marktree_itr_get_ext, marktree_itr_next,
    marktree_itr_next_skip, marktree_itr_pos, marktree_itr_prev, marktree_itr_set_node,
};
use crate::marktree::key::{
    MARKTREE_END_FLAG, mt_end, mt_lookup_key_side, mt_paired, mt_right, pos_leq, relative,
    unrelative,
};
use crate::marktree::meta::{meta_apply_delta, meta_describe_key};
use crate::marktree::node::{Node, refkey};
use crate::marktree::pair::{marktree_intersect_pair, marktree_restore_pair};
use crate::memory::xfree;
use crate::types::{
    MTDamage, MTKey, MTNode, MTPos, MarkTree, MarkTreeIter, Set_uint64_t, colnr_T, int32_t,
    uint64_t,
};

use super::{MAPHASH_INIT, MTDamageMap, marktree_del_itr, marktree_lookup, marktree_put_key};

/// An empty damage map, owning nothing — klib's `MAP_INIT`.
const DAMAGE_INIT: MTDamageMap = MTDamageMap {
    set: Set_uint64_t {
        h: MAPHASH_INIT,
        keys: ptr::null_mut(),
    },
    values: ptr::null_mut(),
};

// -- the iterator, which iter.rs still addresses by raw pointer --------------

/// The key an iterator is parked on, still relative to its node.
///
/// # Safety
/// `itr` must be positioned — `x` non-null — in a live tree.
#[inline]
unsafe fn rawkey(itr: &MarkTreeIter) -> MTKey {
    // SAFETY: the caller promises `itr` is positioned in a live tree.
    unsafe { Node::new(itr.x) }.key(itr.i as usize)
}

/// The two iterators are parked on the very same key.
///
/// This is `iter.rs`'s `itr_eq`, stated over the node and the index rather
/// than over the key address the two compute: the addresses are equal exactly
/// when both parts are.
fn same_key(a: &MarkTreeIter, b: &MarkTreeIter) -> bool {
    a.x == b.x && a.i == b.i
}

/// [`marktree_itr_get_ext`] with the two arguments this file never varies:
/// right gravity, and no metadata filter.
fn itr_get_ext(
    b: &mut MarkTree,
    p: MTPos,
    itr: &mut MarkTreeIter,
    last: bool,
    oldbase: Option<&mut [MTPos; MT_MAX_DEPTH]>,
) {
    // SAFETY: `b` is a live tree and this is what positions `itr` in it.
    unsafe { marktree_itr_get_ext(b, p, itr, last, true, oldbase, None) };
}

/// Step to the next key, optionally skipping over whole subtrees and recording
/// where each one used to be.
fn itr_next_skip(
    b: &mut MarkTree,
    itr: &mut MarkTreeIter,
    skip: bool,
    oldbase: Option<&mut [MTPos; MT_MAX_DEPTH]>,
) {
    // SAFETY: `b` is a live tree and `itr` is positioned in it.
    unsafe { marktree_itr_next_skip(b, itr, skip, false, oldbase, None) };
}

/// Step to the next key.
fn itr_next(b: &mut MarkTree, itr: &mut MarkTreeIter) {
    // SAFETY: `b` is a live tree and `itr` is positioned in it.
    unsafe { marktree_itr_next(b, itr) };
}

/// Park `itr` on key `i` of node `n`, which the damage map recorded earlier in
/// this splice.
fn itr_set_node(b: &mut MarkTree, itr: &mut MarkTreeIter, n: *mut MTNode, i: c_int) {
    // SAFETY: `b` is a live tree and `n` one of its nodes: the damage map only
    // ever holds nodes this splice found through `b`, and nothing between the
    // two points frees a node.
    unsafe { marktree_itr_set_node(b, Some(itr), Node::new(n), i) };
}

/// Find the mark `id` and park `itr` on it, or leave `itr.x` null.
fn lookup(b: &mut MarkTree, id: uint64_t, itr: &mut MarkTreeIter) {
    // SAFETY: `b` is a live tree; a lookup only writes the iterator it is
    // handed, and answers a null `x` when there is no such mark.
    unsafe { marktree_lookup(b, id, Some(itr)) };
}

/// Record — or, with `delete`, retract — the nodes the range `id` covers.
fn intersect_pair(
    b: &mut MarkTree,
    id: uint64_t,
    itr: &mut MarkTreeIter,
    end_itr: &MarkTreeIter,
    delete: bool,
) {
    // SAFETY: `b` is a live tree and both iterators are positioned in it.
    unsafe { marktree_intersect_pair(b, id, itr, end_itr, delete) };
}

// -- the damage map ----------------------------------------------------------

/// Record that the key `itr1` is on has swapped places with the one `itr2` is
/// on, so that the pair it belongs to can be re-intersected once the walk is
/// done. `key` is the key `itr1` is parked on.
fn check_damage(damage: &mut MTDamageMap, key: MTKey, itr1: &MarkTreeIter, itr2: &MarkTreeIter) {
    let start_id = mt_lookup_key_side(key, false);
    let (init, fresh) = (ptr::null_mut(), ptr::null_mut());
    // SAFETY: `damage` is a live map; the two nulls decline its optional
    // "initial value" and "was it new" out-parameters, and `map_put_ref`
    // answers a live slot of the map it was handed.
    let p = unsafe { &mut *map_put_ref_uint64_t_mt_damage_pair(damage, start_id, init, fresh) };
    let me = if mt_end(key) {
        &mut p.end
    } else {
        &mut p.start
    };
    debug_assert!(me.new.is_null(), "me->new == NULL");
    *me = MTDamage {
        old: itr1.x,
        new: itr2.x,
        old_i: itr1.i,
        new_i: itr2.i,
    };
}

/// `map_destroy`: give back the map's three buffers and leave it empty.
fn destroy_damage(damage: &mut MTDamageMap) {
    let (keys, hash) = (damage.set.keys.cast(), damage.set.h.hash.cast());
    let values = damage.values.cast();
    // SAFETY: the key array is this map's own, and is `xfree`-able once.
    unsafe { xfree(keys) };
    // SAFETY: as above, for the hash table behind it.
    unsafe { xfree(hash) };
    // SAFETY: as above, for the values.
    unsafe { xfree(values) };
    *damage = DAMAGE_INIT;
}

// -- the splice itself -------------------------------------------------------

/// Swap the keys two iterators are parked on, each keeping its own position.
///
/// The meta counts move with the keys, up to the two nodes' common ancestor,
/// and any pair whose halves crossed is recorded in `damage`.
fn swap_keys(b: &mut MarkTree, itr1: &MarkTreeIter, itr2: &MarkTreeIter, damage: &mut MTDamageMap) {
    // SAFETY: both iterators are positioned in `b`.
    let (x1, x2) = unsafe { (Node::new(itr1.x), Node::new(itr2.x)) };
    let (i1, i2) = (itr1.i as usize, itr2.i as usize);
    let (key1, key2) = (x1.key(i1), x2.key(i2));

    if !x1.is_leaf() || x1 != x2 {
        if mt_paired(key1) {
            check_damage(damage, key1, itr1, itr2);
        }
        if mt_paired(key2) {
            check_damage(damage, key2, itr2, itr1);
        }
    }

    if x1 != x2 {
        let meta_inc_1 = meta_describe_key(key1);
        let meta_inc_2 = meta_describe_key(key2);
        if meta_inc_1 != meta_inc_2 {
            let (mut a, mut c) = (x1, x2);
            while a != c {
                if a.level() <= c.level() {
                    // As the root uniquely has the highest level, `a` is not it.
                    let p = a.parent().expect("a node below the root has a parent");
                    let i = a.parent_index();
                    p.update_child_meta(i, |m| meta_apply_delta(m, &meta_inc_2, &meta_inc_1));
                    a = p;
                }
                if c.level() < a.level() {
                    let p = c.parent().expect("a node below the root has a parent");
                    let i = c.parent_index();
                    p.update_child_meta(i, |m| meta_apply_delta(m, &meta_inc_1, &meta_inc_2));
                    c = p;
                }
            }
        }
    }

    x1.set_key(
        i1,
        MTKey {
            pos: key1.pos,
            ..key2
        },
    );
    x2.set_key(
        i2,
        MTKey {
            pos: key2.pos,
            ..key1
        },
    );
    // SAFETY: `b` is a live tree and `x1` one of its nodes, now holding the
    // key at `i1`.
    unsafe { refkey(b, x1.as_ptr(), i1 as c_int) };
    // SAFETY: as above, for `x2`.
    unsafe { refkey(b, x2.as_ptr(), i2 as c_int) };
}

/// Apply a text change to every mark at or after `start_line`, `start_col`,
/// and answer whether any of them moved.
///
/// # Safety
/// `b` must be a live tree.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_splice(
    b: &mut MarkTree,
    start_line: int32_t,
    start_col: c_int,
    old_extent_line: c_int,
    old_extent_col: c_int,
    new_extent_line: c_int,
    new_extent_col: c_int,
) -> bool {
    let start = MTPos {
        row: start_line,
        col: start_col,
    };
    let mut old_extent = MTPos {
        row: old_extent_line,
        col: old_extent_col,
    };
    let mut new_extent = MTPos {
        row: new_extent_line,
        col: new_extent_col,
    };

    let mut may_delete = old_extent.row != 0 || old_extent.col != 0;
    let same_line = old_extent.row == 0 && new_extent.row == 0;
    unrelative(start, &mut old_extent);
    unrelative(start, &mut new_extent);
    let mut itr = MarkTreeIter::default();
    let mut enditr = MarkTreeIter::default();
    let mut oldbase = [MTPos::default(); MT_MAX_DEPTH];

    itr_get_ext(b, start, &mut itr, false, Some(&mut oldbase));
    if itr.x.is_null() {
        // den e FÄRDIG
        return false;
    }
    let delta = MTPos {
        row: new_extent.row - old_extent.row,
        col: new_extent.col - old_extent.col,
    };

    if may_delete {
        // SAFETY: `itr` was just positioned in `b`, and is not past the end.
        let (ipos, key) = unsafe { (marktree_itr_pos(&itr), rawkey(&itr)) };
        if !pos_leq(old_extent, ipos)
            || (old_extent.row == ipos.row && old_extent.col == ipos.col && !mt_right(key))
        {
            itr_get_ext(b, old_extent, &mut enditr, true, None);
            debug_assert!(!enditr.x.is_null(), "enditr->x");
            // "assert" (itr <= enditr)
        } else {
            may_delete = false;
        }
    }

    let mut past_right = false;
    let mut moved = false;
    let mut damage = DAMAGE_INIT;

    // Follow the general strategy of messing things up and fixing them later.
    // `oldbase` carries what is needed to work out a child's old position.
    if may_delete {
        'collapse: while !itr.x.is_null() && !past_right {
            let mut loc_start = start;
            let mut loc_old = old_extent;
            relative(itr.pos, &mut loc_start);
            relative(oldbase[itr.lvl as usize], &mut loc_old);

            loop {
                // SAFETY: the loop guard checked `itr` is still positioned.
                let x = unsafe { Node::new(itr.x) };
                let i = itr.i as usize;
                // NB: strictly should be less than the right gravity of
                // loc_old, but the iterator comparison below will already
                // break on that.
                if !pos_leq(x.key(i).pos, loc_old) {
                    break 'collapse;
                }

                if mt_right(x.key(i)) {
                    // SAFETY: `may_delete` means the lookup above positioned
                    // `enditr` in `b`; stepping back leaves it positioned.
                    while !same_key(&itr, &enditr) && mt_right(unsafe { rawkey(&enditr) }) {
                        // SAFETY: as above.
                        unsafe { marktree_itr_prev(b, &mut enditr) };
                    }
                    // SAFETY: as above.
                    if !mt_right(unsafe { rawkey(&enditr) }) {
                        swap_keys(b, &itr, &enditr, &mut damage);
                    } else {
                        past_right = true;
                        break 'collapse;
                    }
                }

                if same_key(&itr, &enditr) {
                    // Actually, will be past_right after this key.
                    past_right = true;
                }

                moved = true;
                if !x.is_leaf() {
                    let lvl = itr.lvl as usize;
                    oldbase[lvl + 1] = x.key(i).pos;
                    let base = oldbase[lvl];
                    unrelative(base, &mut oldbase[lvl + 1]);
                    x.update_key(i, |k| k.pos = loc_start);
                    itr_next_skip(b, &mut itr, false, Some(&mut oldbase));
                    break;
                }
                x.update_key(i, |k| k.pos = loc_start);
                if i + 1 < x.key_count() {
                    itr.i += 1;
                    if past_right {
                        break;
                    }
                } else {
                    itr_next(b, &mut itr);
                    break;
                }
            }
        }

        'shift: while !itr.x.is_null() {
            let mut loc_new = new_extent;
            relative(itr.pos, &mut loc_new);
            let mut limit = old_extent;
            relative(oldbase[itr.lvl as usize], &mut limit);

            loop {
                // SAFETY: the loop guard checked `itr` is still positioned.
                let x = unsafe { Node::new(itr.x) };
                let i = itr.i as usize;
                if pos_leq(limit, x.key(i).pos) {
                    break 'shift;
                }

                let oldpos = x.key(i).pos;
                x.update_key(i, |k| k.pos = loc_new);
                moved = true;
                if !x.is_leaf() {
                    let lvl = itr.lvl as usize;
                    oldbase[lvl + 1] = oldpos;
                    let base = oldbase[lvl];
                    unrelative(base, &mut oldbase[lvl + 1]);
                    itr_next_skip(b, &mut itr, false, Some(&mut oldbase));
                    break;
                } else if i + 1 < x.key_count() {
                    itr.i += 1;
                } else {
                    itr_next(b, &mut itr);
                    break;
                }
            }
        }
    }

    while !itr.x.is_null() {
        // SAFETY: the loop guard checked `itr` is still positioned.
        let x = unsafe { Node::new(itr.x) };
        let i = itr.i as usize;
        let base = oldbase[itr.lvl as usize];
        x.update_key(i, |k| unrelative(base, &mut k.pos));
        let realrow = x.key(i).pos.row;
        debug_assert!(realrow >= old_extent.row, "realrow >= old_extent.row");
        let mut done = false;
        if realrow == old_extent.row {
            if delta.col != 0 {
                x.update_key(i, |k| k.pos.col += delta.col);
            }
        } else if same_line {
            // Optimization: a column-only adjustment can skip the rest of the
            // rows.
            done = true;
        }
        if delta.row != 0 {
            x.update_key(i, |k| k.pos.row += delta.row);
            moved = true;
        }
        let base = itr.pos;
        x.update_key(i, |k| relative(base, &mut k.pos));
        if done {
            break;
        }
        itr_next_skip(b, &mut itr, true, None);
    }

    let (keys, values) = (damage.set.keys, damage.values);
    for idx in 0..damage.set.h.n_keys as usize {
        // SAFETY: klib's `map_foreach` — the map's `n_keys` leading entries of
        // `keys` and `values` are live and parallel, and nothing in the body
        // touches the map.
        let (start_id, d) = unsafe { (*keys.add(idx), *values.add(idx)) };
        if !d.start.old.is_null() && !d.end.old.is_null() {
            // Both ends of the pair moved.
            itr_set_node(b, &mut itr, d.start.old, d.start.old_i);
            itr_set_node(b, &mut enditr, d.end.old, d.end.old_i);
            intersect_pair(b, start_id, &mut itr, &enditr, true);
            itr_set_node(b, &mut itr, d.start.new, d.start.new_i);
            itr_set_node(b, &mut enditr, d.end.new, d.end.new_i);
            intersect_pair(b, start_id, &mut itr, &enditr, false);
        } else if !d.start.old.is_null() {
            // Only the start moved.
            let mut endpos = MarkTreeIter::default();
            lookup(b, start_id | MARKTREE_END_FLAG, &mut endpos);
            if !endpos.x.is_null() {
                itr_set_node(b, &mut itr, d.start.old, d.start.old_i);
                enditr = endpos;
                intersect_pair(b, start_id, &mut itr, &enditr, true);
                itr_set_node(b, &mut itr, d.start.new, d.start.new_i);
                enditr = endpos;
                intersect_pair(b, start_id, &mut itr, &enditr, false);
            }
        } else if !d.end.old.is_null() {
            // Only the end moved.
            let mut startpos = MarkTreeIter::default();
            lookup(b, start_id, &mut startpos);
            if !startpos.x.is_null() {
                itr = startpos;
                itr_set_node(b, &mut enditr, d.end.old, d.end.old_i);
                intersect_pair(b, start_id, &mut itr, &enditr, true);
                itr = startpos;
                itr_set_node(b, &mut enditr, d.end.new, d.end.new_i);
                intersect_pair(b, start_id, &mut itr, &enditr, false);
            }
        }
    }
    destroy_damage(&mut damage);

    moved
}

/// Move the marks in a region elsewhere, as `:move` does.
///
/// The marks inside the region are lifted out, the two splices shift
/// everything else, and then they go back in at the destination.
pub fn marktree_move_region(
    b: &mut MarkTree,
    start_row: c_int,
    start_col: colnr_T,
    extent_row: c_int,
    extent_col: colnr_T,
    new_row: c_int,
    new_col: colnr_T,
) {
    let start = MTPos {
        row: start_row,
        col: start_col,
    };
    let size = MTPos {
        row: extent_row,
        col: extent_col,
    };
    let mut end = size;
    unrelative(start, &mut end);
    let mut itr = MarkTreeIter::default();
    itr_get_ext(b, start, &mut itr, false, None);

    let mut saved: Vec<MTKey> = Vec::new();
    while !itr.x.is_null() {
        // SAFETY: the loop guard checked `itr` is still positioned in `b`.
        let mut k = unsafe { marktree_itr_current(&mut itr) };
        if !pos_leq(k.pos, end) || (k.pos.row == end.row && k.pos.col == end.col && mt_right(k)) {
            break;
        }
        relative(start, &mut k.pos);
        saved.push(k);
        // SAFETY: `b` is live and `itr` is positioned on one of its keys;
        // deleting leaves it on the next one.
        unsafe { marktree_del_itr(b, &mut itr, false) };
    }

    // SAFETY: `b` is a live tree; the extents are plain numbers.
    unsafe { marktree_splice(b, start.row, start.col, size.row, size.col, 0, 0) };
    let new = MTPos {
        row: new_row,
        col: new_col,
    };
    // SAFETY: as above.
    unsafe { marktree_splice(b, new.row, new.col, 0, 0, size.row, size.col) };

    for mut item in saved {
        unrelative(new, &mut item.pos);
        // SAFETY: `b` is a live tree.
        unsafe { marktree_put_key(b, item) };
        if mt_paired(item) {
            // The other end might be later in `saved`; this bails out safely
            // then, and runs again for it.
            // SAFETY: as above.
            unsafe { marktree_restore_pair(b, item) };
        }
    }
}
