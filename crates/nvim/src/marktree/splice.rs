#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]
#![forbid(unsafe_code)]

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

use crate::narrow::as_index;
use crate::narrow::count_as_int;
use core::ffi::c_int;

use crate::map::DenseMap;
use crate::marktree::iter::{
    MT_MAX_DEPTH, marktree_itr_current, marktree_itr_get_ext, marktree_itr_next,
    marktree_itr_next_skip, marktree_itr_pos, marktree_itr_prev, marktree_itr_set_node, raw_key,
};
use crate::marktree::key::{
    MARKTREE_END_FLAG, mt_end, mt_lookup_key_side, mt_paired, mt_right, pos_leq, relative,
    unrelative,
};
use crate::marktree::meta::{meta_apply_delta, meta_describe_key};
use crate::marktree::node::NodeId;
use crate::marktree::pair::{marktree_intersect_pair, marktree_restore_pair};
use crate::os::libc::{int32_t, uint64_t};
use crate::types::{ColNr, MTDamage, MTDamagePair, MTKey, MTPos, MarkTree, MarkTreeIter};

use super::{marktree_del_itr, marktree_lookup, marktree_put_key};

/// What a splice recorded about pairs whose halves crossed while it ran: per
/// start id, where each half was and where it went. Walked in insertion
/// order, as upstream's `map_foreach` walks it.
type DamageMap = DenseMap<uint64_t, MTDamagePair>;

/// Neither half has moved.
const NO_DAMAGE: MTDamage = MTDamage {
    old: None,
    new: None,
    old_i: 0,
    new_i: 0,
};

// -- the iterator ------------------------------------------------------------

/// The two iterators are parked on the very same key.
///
/// This is `iter.rs`'s `itr_eq`, stated over the node and the index rather
/// than over the key address the two compute: the addresses are equal exactly
/// when both parts are.
fn same_key(a: &MarkTreeIter, b: &MarkTreeIter) -> bool {
    a.node == b.node && a.index == b.index
}

/// [`marktree_itr_get_ext`] with the two arguments this file never varies:
/// right gravity, and no metadata filter.
fn iter_get_ext(
    tree: &mut MarkTree,
    p: MTPos,
    iter: &mut MarkTreeIter,
    last: bool,
    oldbase: Option<&mut [MTPos; MT_MAX_DEPTH]>,
) {
    marktree_itr_get_ext(tree, p, iter, last, true, oldbase, None);
}

/// Step to the next key, optionally skipping over whole subtrees and recording
/// where each one used to be.
fn iter_next_skip(
    tree: &mut MarkTree,
    iter: &mut MarkTreeIter,
    skip: bool,
    oldbase: Option<&mut [MTPos; MT_MAX_DEPTH]>,
) {
    marktree_itr_next_skip(tree, iter, skip, false, oldbase, None);
}

/// Park `iter` on key `i` of node `n`, which the damage map recorded earlier in
/// this splice.
fn iter_set_node(tree: &mut MarkTree, iter: &mut MarkTreeIter, n: Option<NodeId>, i: c_int) {
    let n = n.expect("a recorded half of a damaged pair");
    marktree_itr_set_node(tree, Some(iter), n, i);
}

// -- the damage map ----------------------------------------------------------

/// Record that the key `iter1` is on has swapped places with the one `iter2`
/// is on, so that the pair it belongs to can be re-intersected once the walk is
/// done. `key` is the key `iter1` is parked on.
fn check_damage(damage: &mut DamageMap, key: MTKey, iter1: &MarkTreeIter, iter2: &MarkTreeIter) {
    let start_id = mt_lookup_key_side(key, false);
    let untouched = MTDamagePair {
        start: NO_DAMAGE,
        end: NO_DAMAGE,
    };
    let p = damage.get_or_insert(start_id, untouched);
    let me = if mt_end(key) {
        &mut p.end
    } else {
        &mut p.start
    };
    debug_assert!(me.new.is_none(), "me->new == NULL");
    *me = MTDamage {
        old: iter1.node,
        new: iter2.node,
        old_i: iter1.index,
        new_i: iter2.index,
    };
}

// -- the splice itself -------------------------------------------------------

/// Swap the keys two iterators are parked on, each keeping its own position.
///
/// The meta counts move with the keys, up to the two nodes' common ancestor,
/// and any pair whose halves crossed is recorded in `damage`.
fn swap_keys(
    tree: &mut MarkTree,
    iter1: &MarkTreeIter,
    iter2: &MarkTreeIter,
    damage: &mut DamageMap,
) {
    let x1 = iter1.node.expect("a positioned iterator");
    let x2 = iter2.node.expect("a positioned iterator");
    let (i1, i2) = (as_index(iter1.index), as_index(iter2.index));
    let (key1, key2) = (tree[x1].key(i1), tree[x2].key(i2));

    if !tree[x1].is_leaf() || x1 != x2 {
        if mt_paired(key1) {
            check_damage(damage, key1, iter1, iter2);
        }
        if mt_paired(key2) {
            check_damage(damage, key2, iter2, iter1);
        }
    }

    if x1 != x2 {
        let meta_inc_1 = meta_describe_key(key1);
        let meta_inc_2 = meta_describe_key(key2);
        if meta_inc_1 != meta_inc_2 {
            let (mut a, mut c) = (x1, x2);
            while a != c {
                if tree[a].level() <= tree[c].level() {
                    // As the root uniquely has the highest level, `a` is not it.
                    let p = tree[a]
                        .parent()
                        .expect("a node below the root has a parent");
                    let i = tree[a].parent_index();
                    meta_apply_delta(tree[p].child_meta_mut(i), &meta_inc_2, &meta_inc_1);
                    a = p;
                }
                if tree[c].level() < tree[a].level() {
                    let p = tree[c]
                        .parent()
                        .expect("a node below the root has a parent");
                    let i = tree[c].parent_index();
                    meta_apply_delta(tree[p].child_meta_mut(i), &meta_inc_1, &meta_inc_2);
                    c = p;
                }
            }
        }
    }

    tree[x1].set_key(
        i1,
        MTKey {
            pos: key1.pos,
            ..key2
        },
    );
    tree[x2].set_key(
        i2,
        MTKey {
            pos: key2.pos,
            ..key1
        },
    );
    tree.refkey(x1, i1);
    tree.refkey(x2, i2);
}

/// Apply a text change to every mark at or after `start_line`, `start_col`,
/// and answer whether any of them moved.
pub fn marktree_splice(
    tree: &mut MarkTree,
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
    let mut iter = MarkTreeIter::default();
    let mut end_iter = MarkTreeIter::default();
    let mut oldbase = [MTPos::default(); MT_MAX_DEPTH];

    iter_get_ext(tree, start, &mut iter, false, Some(&mut oldbase));
    if !iter.is_positioned() {
        // den e FÄRDIG
        return false;
    }
    let delta = MTPos {
        row: new_extent.row - old_extent.row,
        col: new_extent.col - old_extent.col,
    };

    if may_delete {
        let (ipos, key) = (marktree_itr_pos(tree, &iter), raw_key(tree, &iter));
        if !pos_leq(old_extent, ipos)
            || (old_extent.row == ipos.row && old_extent.col == ipos.col && !mt_right(key))
        {
            iter_get_ext(tree, old_extent, &mut end_iter, true, None);
            debug_assert!(end_iter.is_positioned(), "enditr->x");
            // "assert" (itr <= enditr)
        } else {
            may_delete = false;
        }
    }

    let mut past_right = false;
    let mut moved = false;
    let mut damage = DamageMap::new();

    // Follow the general strategy of messing things up and fixing them later.
    // `oldbase` carries what is needed to work out a child's old position.
    if may_delete {
        'collapse: while iter.is_positioned() && !past_right {
            let mut loc_start = start;
            let mut loc_old = old_extent;
            relative(iter.pos, &mut loc_start);
            relative(oldbase[as_index(iter.depth)], &mut loc_old);

            while let Some(x) = iter.node {
                let i = as_index(iter.index);
                // NB: strictly should be less than the right gravity of
                // loc_old, but the iterator comparison below will already
                // break on that.
                if !pos_leq(tree[x].key(i).pos, loc_old) {
                    break 'collapse;
                }

                if mt_right(tree[x].key(i)) {
                    while !same_key(&iter, &end_iter) && mt_right(raw_key(tree, &end_iter)) {
                        marktree_itr_prev(tree, &mut end_iter);
                    }
                    if !mt_right(raw_key(tree, &end_iter)) {
                        swap_keys(tree, &iter, &end_iter, &mut damage);
                    } else {
                        // Past the right edge: `break 'collapse` leaves the
                        // walk, so the flag the loop guard reads is not set.
                        break 'collapse;
                    }
                }

                if same_key(&iter, &end_iter) {
                    // Actually, will be past_right after this key.
                    past_right = true;
                }

                moved = true;
                if !tree[x].is_leaf() {
                    let depth = as_index(iter.depth);
                    oldbase[depth + 1] = tree[x].key(i).pos;
                    let base = oldbase[depth];
                    unrelative(base, &mut oldbase[depth + 1]);
                    tree[x].key_mut(i).pos = loc_start;
                    iter_next_skip(tree, &mut iter, false, Some(&mut oldbase));
                    break;
                }
                tree[x].key_mut(i).pos = loc_start;
                if i + 1 < tree[x].key_count() {
                    iter.index += 1;
                    if past_right {
                        break;
                    }
                } else {
                    marktree_itr_next(tree, &mut iter);
                    break;
                }
            }
        }

        'shift: while iter.is_positioned() {
            let mut loc_new = new_extent;
            relative(iter.pos, &mut loc_new);
            let mut limit = old_extent;
            relative(oldbase[as_index(iter.depth)], &mut limit);

            while let Some(x) = iter.node {
                let i = as_index(iter.index);
                if pos_leq(limit, tree[x].key(i).pos) {
                    break 'shift;
                }

                let oldpos = tree[x].key(i).pos;
                tree[x].key_mut(i).pos = loc_new;
                moved = true;
                if !tree[x].is_leaf() {
                    let depth = as_index(iter.depth);
                    oldbase[depth + 1] = oldpos;
                    let base = oldbase[depth];
                    unrelative(base, &mut oldbase[depth + 1]);
                    iter_next_skip(tree, &mut iter, false, Some(&mut oldbase));
                    break;
                } else if i + 1 < tree[x].key_count() {
                    iter.index += 1;
                } else {
                    marktree_itr_next(tree, &mut iter);
                    break;
                }
            }
        }
    }

    // Every key from here on shifts. Within one node only the index moves, so
    // the node is borrowed once and its keys walked in place; the iterator
    // steps (skipping the subtrees, whose positions are relative to the keys
    // just moved) only when the node is done.
    let mut done = false;
    while let Some(x) = iter.node
        && !done
    {
        let base = oldbase[as_index(iter.depth)];
        let node = &mut tree[x];
        loop {
            let key = node.key_mut(as_index(iter.index));
            unrelative(base, &mut key.pos);
            let realrow = key.pos.row;
            debug_assert!(realrow >= old_extent.row, "realrow >= old_extent.row");
            if realrow == old_extent.row {
                if delta.col != 0 {
                    key.pos.col += delta.col;
                }
            } else if same_line {
                // Optimization: a column-only adjustment can skip the rest of
                // the rows.
                done = true;
            }
            if delta.row != 0 {
                key.pos.row += delta.row;
                moved = true;
            }
            relative(iter.pos, &mut key.pos);
            if done || iter.index + 1 >= count_as_int(node.key_count()) {
                break;
            }
            iter.index += 1;
        }
        if !done {
            iter_next_skip(tree, &mut iter, true, None);
        }
    }

    for (&start_id, &d) in damage.iter() {
        if d.start.old.is_some() && d.end.old.is_some() {
            // Both ends of the pair moved.
            iter_set_node(tree, &mut iter, d.start.old, d.start.old_i);
            iter_set_node(tree, &mut end_iter, d.end.old, d.end.old_i);
            marktree_intersect_pair(tree, start_id, &mut iter, &end_iter, true);
            iter_set_node(tree, &mut iter, d.start.new, d.start.new_i);
            iter_set_node(tree, &mut end_iter, d.end.new, d.end.new_i);
            marktree_intersect_pair(tree, start_id, &mut iter, &end_iter, false);
        } else if d.start.old.is_some() {
            // Only the start moved.
            let mut endpos = MarkTreeIter::default();
            marktree_lookup(tree, start_id | MARKTREE_END_FLAG, Some(&mut endpos));
            if endpos.is_positioned() {
                iter_set_node(tree, &mut iter, d.start.old, d.start.old_i);
                end_iter = endpos;
                marktree_intersect_pair(tree, start_id, &mut iter, &end_iter, true);
                iter_set_node(tree, &mut iter, d.start.new, d.start.new_i);
                end_iter = endpos;
                marktree_intersect_pair(tree, start_id, &mut iter, &end_iter, false);
            }
        } else if d.end.old.is_some() {
            // Only the end moved.
            let mut startpos = MarkTreeIter::default();
            marktree_lookup(tree, start_id, Some(&mut startpos));
            if startpos.is_positioned() {
                iter = startpos;
                iter_set_node(tree, &mut end_iter, d.end.old, d.end.old_i);
                marktree_intersect_pair(tree, start_id, &mut iter, &end_iter, true);
                iter = startpos;
                iter_set_node(tree, &mut end_iter, d.end.new, d.end.new_i);
                marktree_intersect_pair(tree, start_id, &mut iter, &end_iter, false);
            }
        }
    }

    moved
}

/// Move the marks in a region elsewhere, as `:move` does.
///
/// The marks inside the region are lifted out, the two splices shift
/// everything else, and then they go back in at the destination.
pub fn marktree_move_region(
    tree: &mut MarkTree,
    start_row: c_int,
    start_col: ColNr,
    extent_row: c_int,
    extent_col: ColNr,
    new_row: c_int,
    new_col: ColNr,
) {
    let start = MTPos {
        row: start_row,
        col: start_col.get(),
    };
    let size = MTPos {
        row: extent_row,
        col: extent_col.get(),
    };
    let mut end = size;
    unrelative(start, &mut end);
    let mut iter = MarkTreeIter::default();
    iter_get_ext(tree, start, &mut iter, false, None);

    let mut saved: Vec<MTKey> = Vec::new();
    while iter.is_positioned() {
        let mut k = marktree_itr_current(tree, &iter);
        if !pos_leq(k.pos, end) || (k.pos.row == end.row && k.pos.col == end.col && mt_right(k)) {
            break;
        }
        relative(start, &mut k.pos);
        saved.push(k);
        marktree_del_itr(tree, &mut iter, false);
    }

    marktree_splice(tree, start.row, start.col, size.row, size.col, 0, 0);
    let new = MTPos {
        row: new_row,
        col: new_col.get(),
    };
    marktree_splice(tree, new.row, new.col, 0, 0, size.row, size.col);

    for mut item in saved {
        unrelative(new, &mut item.pos);
        marktree_put_key(tree, item);
        if mt_paired(item) {
            // The other end might be later in `saved`; this bails out safely
            // then, and runs again for it.
            marktree_restore_pair(tree, item);
        }
    }
}
