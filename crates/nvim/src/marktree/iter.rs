#![deny(unsafe_op_in_unsafe_fn)]

//! Walking the tree.
//!
//! `MarkTreeIter` is a cursor: the node and index it currently names, plus the
//! path it took to get there (`s[lvl]`, one entry per level) and the absolute
//! position of that node, since positions in the tree are stored relative to
//! the enclosing key. Callers all over the editor hold one across an edit, so
//! the layout is fixed and the entry points keep their C signatures.
//!
//! Three walks share the machinery:
//!
//! * the plain one, [`marktree_itr_next`]/[`marktree_itr_prev`];
//! * the filtered one, which reads a node's meta counts and skips any subtree
//!   that holds none of the decoration kinds the caller asked for;
//! * the overlap walk, which answers "which ranges cover this position" by
//!   reading the intersection set on each node of the path down.
//!
//! # How this file reaches the tree
//!
//! Every walk keeps the node it is on in a [`Node`] local *alongside* `itr.x`,
//! and moves it with [`Node::parent`] / [`Node::child`]. So a descent of any
//! depth costs exactly one promise — the one that turns the tree's root, or
//! the node an iterator arrives holding, into a `Node` — and the body of the
//! walk is ordinary checked code. That is why the entry points are still
//! `unsafe` although they take `&mut MarkTree`: a reference cannot say that an
//! iterator handed alongside a tree is positioned in *that* tree, nor that a
//! tree's `root` names a live node.

use core::ffi::c_int;
use core::ptr;

use crate::marktree::key::{
    MARKTREE_END_FLAG, MT_FLAG_LAST, MT_FLAG_RIGHT_GRAVITY, MT_INVALID_KEY, compose, mt_end,
    mt_lookup_id, mt_start, mtpair_from, pos_leq, pos_less, relative, unrelative,
};
use crate::marktree::meta::{MetaCount, filtered_key_flags, meta_has};
use crate::marktree::node::{Node, find_key, id2node};
use crate::types::{
    DecorHighlightInline, DecorInlineData, MTKey, MTPair, MTPos, MarkTree, MarkTreeIter,
    MetaFilter, int32_t, uint16_t, uint32_t,
};

use super::marktree_lookup;

/// Nested to keep the name out of the flat cdef namespace `ffigen` builds,
/// the same reason node.rs nests its own sizes.
mod sizes {
    /// The deepest the tree can get: the C's `MT_MAX_DEPTH`, which is also the
    /// length of `MarkTreeIter::s` and so the range `MarkTreeIter::lvl`
    /// indexes. It bounds an `oldbase` array for the same reason.
    pub const MT_MAX_DEPTH: usize = 20;
}
pub use sizes::MT_MAX_DEPTH;

/// A bare key at `pos`, for [`find_key`] to compare against.
///
/// Only the position and the flags take part in the ordering, so the
/// decoration is left zeroed — deliberately *not*
/// `DECOR_HIGHLIGHT_INLINE_INIT`, whose priority is not zero.
fn search_key(pos: MTPos, flags: c_int) -> MTKey {
    MTKey {
        pos,
        ns: 0,
        id: 0,
        flags: flags as uint16_t,
        decor_data: DecorInlineData {
            hl: DecorHighlightInline {
                flags: 0,
                priority: 0,
                hl_id: 0,
                conceal_char: 0,
            },
        },
    }
}

/// The filter a C caller handed in, which every one of them supplies.
///
/// # Safety
/// `filter`, where non-null, must name a live `MetaCount`.
unsafe fn as_filter<'a>(filter: MetaFilter) -> &'a MetaCount {
    // SAFETY: the caller promises a live `MetaCount`. The C dereferences it
    // unconditionally too — there is no "no filter" case on this path.
    unsafe { &*filter.cast() }
}

/// Position `itr` at the first key at or after (row, col).
///
/// # Safety
/// `b` must be a live tree.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_itr_get(
    b: &mut MarkTree,
    row: int32_t,
    col: c_int,
    itr: &mut MarkTreeIter,
) -> bool {
    let p = MTPos { row, col };
    // SAFETY: `b` is a live tree and this is what positions `itr` in it.
    unsafe { marktree_itr_get_ext(b, p, itr, false, false, None, None) }
}

/// Position `itr` at `p`, with every knob the family has.
///
/// `last` asks for the key *before* `p` instead of the one at or after it,
/// `gravity` breaks a tie at `p` on the right-gravity side, `oldbase` records
/// the absolute position of every node on the way down, and `filter` stops the
/// descent at the first subtree holding none of the wanted decoration kinds.
///
/// # Safety
/// `b` must be a live tree.
pub unsafe fn marktree_itr_get_ext(
    b: &mut MarkTree,
    p: MTPos,
    itr: &mut MarkTreeIter,
    last: bool,
    gravity: bool,
    mut oldbase: Option<&mut [MTPos; MT_MAX_DEPTH]>,
    filter: Option<&MetaCount>,
) -> bool {
    if b.n_keys == 0 {
        itr.x = ptr::null_mut();
        return false;
    }
    let mut k = search_key(p, if gravity { MT_FLAG_RIGHT_GRAVITY } else { 0 });
    if last && !gravity {
        k.flags = MT_FLAG_LAST as uint16_t;
    }
    itr.pos = MTPos::default();
    // SAFETY: `b` is live and holds keys, so its root is one of its live nodes.
    let mut x = unsafe { Node::new(b.root) };
    itr.x = x.as_ptr();
    itr.lvl = 0;
    if let Some(oldbase) = oldbase.as_deref_mut() {
        oldbase[itr.lvl as usize] = itr.pos;
    }
    loop {
        itr.i = find_key(x.keys(), k).0 + 1;
        if x.is_leaf() {
            break;
        }
        if let Some(filter) = filter
            && !meta_has(&x.child_meta(itr.i as usize), filter)
        {
            break;
        }
        itr.s[itr.lvl as usize].i = itr.i;
        itr.s[itr.lvl as usize].oldcol = itr.pos.col;
        if itr.i > 0 {
            let sep = x.key(itr.i as usize - 1).pos;
            compose(&mut itr.pos, sep);
            relative(sep, &mut k.pos);
        }
        x = x.child(itr.i as usize);
        itr.x = x.as_ptr();
        itr.lvl += 1;
        if let Some(oldbase) = oldbase.as_deref_mut() {
            oldbase[itr.lvl as usize] = itr.pos;
        }
    }
    if last {
        // SAFETY: `b` is live and `itr` is now positioned in it.
        unsafe { marktree_itr_prev(b, itr) }
    } else if itr.i >= x.key_count() as c_int {
        // SAFETY: as above. The descent stopped past the node's last key, so
        // the next key is the one in an ancestor.
        unsafe { marktree_itr_next_skip(b, itr, true, false, None, None) }
    } else {
        true
    }
}

/// Position `itr` at the tree's first key.
///
/// # Safety
/// `b` must be a live tree.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_itr_first(b: &mut MarkTree, itr: &mut MarkTreeIter) -> bool {
    if b.n_keys == 0 {
        itr.x = ptr::null_mut();
        return false;
    }
    // SAFETY: `b` is live and holds keys, so its root is one of its live nodes.
    let mut x = unsafe { Node::new(b.root) };
    itr.x = x.as_ptr();
    itr.i = 0;
    itr.lvl = 0;
    itr.pos = MTPos::default();
    while !x.is_leaf() {
        itr.s[itr.lvl as usize].i = 0;
        itr.s[itr.lvl as usize].oldcol = 0;
        itr.lvl += 1;
        x = x.child(0);
        itr.x = x.as_ptr();
    }
    true
}

/// Step to the next key.
///
/// # Safety
/// `b` must be a live tree and `itr` positioned in it, or empty.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_itr_next(b: &mut MarkTree, itr: &mut MarkTreeIter) -> bool {
    // SAFETY: the caller's tree and iterator, per this function's contract.
    unsafe { marktree_itr_next_skip(b, itr, false, false, None, None) }
}

/// Step to the next key, optionally skipping the rest of the current subtree.
///
/// `skip` leaves the node the iterator is on instead of descending into it,
/// and `filter` sets `skip` by itself for a subtree holding none of the wanted
/// decoration kinds. `preload` stops one level short of a leaf, leaving
/// `itr.i` at -1, which is what the pair walk wants. `oldbase` records where
/// each level started.
///
/// # Safety
/// `b` must be a live tree and `itr` positioned in it, or empty.
pub unsafe fn marktree_itr_next_skip(
    _b: &mut MarkTree,
    itr: &mut MarkTreeIter,
    mut skip: bool,
    preload: bool,
    mut oldbase: Option<&mut [MTPos; MT_MAX_DEPTH]>,
    filter: Option<&MetaCount>,
) -> bool {
    // SAFETY: `itr` is positioned in a live tree, or empty, per the caller.
    let Some(mut x) = (unsafe { Node::from_ptr(itr.x) }) else {
        return false;
    };
    itr.i += 1;
    if let Some(filter) = filter
        && !x.is_leaf()
        && !meta_has(&x.child_meta(itr.i as usize), filter)
    {
        skip = true;
    }
    if x.is_leaf() || skip {
        if preload && x.is_leaf() && skip {
            itr.i = x.key_count() as c_int;
        } else if itr.i < x.key_count() as c_int {
            return true;
        }
        // Walk up until a node still has a key left after the one we came out
        // of, undoing that level's contribution to the absolute position.
        while itr.i >= x.key_count() as c_int {
            let Some(parent) = x.parent() else {
                itr.x = ptr::null_mut();
                return false;
            };
            x = parent;
            itr.x = x.as_ptr();
            itr.lvl -= 1;
            itr.i = itr.s[itr.lvl as usize].i;
            if itr.i > 0 {
                itr.pos.row -= x.key(itr.i as usize - 1).pos.row;
                itr.pos.col = itr.s[itr.lvl as usize].oldcol;
            }
        }
    } else {
        // The key we stepped onto is an internal one, so the next key in order
        // is the leftmost of the subtree between it and the previous key.
        while !x.is_leaf() {
            if itr.i > 0 {
                itr.s[itr.lvl as usize].oldcol = itr.pos.col;
                let sep = x.key(itr.i as usize - 1).pos;
                compose(&mut itr.pos, sep);
            }
            if itr.i == 0
                && let Some(oldbase) = oldbase.as_deref_mut()
            {
                oldbase[itr.lvl as usize + 1] = oldbase[itr.lvl as usize];
            }
            itr.s[itr.lvl as usize].i = itr.i;
            let child = x.child(itr.i as usize);
            debug_assert!(
                child.parent() == Some(x),
                "itr->x->ptr[itr->i]->parent == itr->x"
            );
            itr.lvl += 1;
            x = child;
            itr.x = x.as_ptr();
            if preload && !x.is_leaf() {
                itr.i = -1;
                break;
            }
            itr.i = 0;
            if let Some(filter) = filter
                && !x.is_leaf()
                && !meta_has(&x.child_meta(0), filter)
            {
                break;
            }
        }
    }
    true
}

/// Position `itr` at the first key at or after (row, col) that the filter
/// wants, giving up at (stop_row, stop_col).
///
/// # Safety
/// `b` must be a live tree and `meta_filter` a live `MetaCount`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_itr_get_filter(
    b: &mut MarkTree,
    row: int32_t,
    col: c_int,
    stop_row: c_int,
    stop_col: c_int,
    meta_filter: MetaFilter,
    itr: &mut MarkTreeIter,
) -> bool {
    // SAFETY: `meta_filter` is live per the caller.
    let filter = unsafe { as_filter(meta_filter) };
    if !meta_has(&b.meta_root, filter) {
        return false;
    }
    let p = MTPos { row, col };
    // SAFETY: `b` is a live tree and this is what positions `itr` in it.
    if !unsafe { marktree_itr_get_ext(b, p, itr, false, false, None, Some(filter)) } {
        return false;
    }
    // SAFETY: `b` is live and `itr` is now positioned in it.
    unsafe { marktree_itr_check_filter(b, itr, stop_row, stop_col, filter) }
}

/// Leave the subtrees the filter has nothing in, stepping out of each one
/// whose parent says so, and answer whether the iterator still names a node.
///
/// # Safety
/// `b` must be a live tree, `itr` positioned in it or empty, and
/// `meta_filter` a live `MetaCount`.
pub unsafe fn marktree_itr_step_out_filter(
    b: &mut MarkTree,
    itr: &mut MarkTreeIter,
    meta_filter: MetaFilter,
) -> bool {
    // SAFETY: `meta_filter` is live per the caller.
    let filter = unsafe { as_filter(meta_filter) };
    if !meta_has(&b.meta_root, filter) {
        itr.x = ptr::null_mut();
        return false;
    }
    // SAFETY: `itr` is positioned in `b`, or empty, per the caller.
    while let Some(x) = unsafe { Node::from_ptr(itr.x) } {
        let Some(parent) = x.parent() else {
            break;
        };
        if meta_has(&parent.child_meta(x.parent_index()), filter) {
            return true;
        }
        itr.i = x.key_count() as c_int;
        // SAFETY: `b` is live and `itr` is positioned in it. The step is
        // unfiltered on purpose: this walk does its own filtering above.
        unsafe { marktree_itr_next_skip(b, itr, true, false, None, None) };
    }
    !itr.x.is_null()
}

/// Step to the next key the filter wants, giving up at (stop_row, stop_col).
///
/// # Safety
/// `b` must be a live tree, `itr` positioned in it, and `meta_filter` a live
/// `MetaCount`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_itr_next_filter(
    b: &mut MarkTree,
    itr: &mut MarkTreeIter,
    stop_row: c_int,
    stop_col: c_int,
    meta_filter: MetaFilter,
) -> bool {
    // SAFETY: `meta_filter` is live per the caller.
    let filter = unsafe { as_filter(meta_filter) };
    // SAFETY: `b` is live and `itr` is positioned in it.
    if !unsafe { marktree_itr_next_skip(b, itr, false, false, None, Some(filter)) } {
        return false;
    }
    // SAFETY: as above.
    unsafe { marktree_itr_check_filter(b, itr, stop_row, stop_col, filter) }
}

/// Advance until the iterator is on a key the filter actually wants — the meta
/// counts only promise that the *subtree* holds one — or past the stop
/// position, which empties the iterator.
///
/// # Safety
/// `b` must be a live tree and `itr` positioned in it.
unsafe fn marktree_itr_check_filter(
    b: &mut MarkTree,
    itr: &mut MarkTreeIter,
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
        // SAFETY: `itr` is positioned in `b` per the caller, and every step
        // below either leaves it positioned or answers false.
        let pos = unsafe { marktree_itr_pos(itr) };
        if pos_leq(stop_pos, pos) {
            itr.x = ptr::null_mut();
            return false;
        }
        // SAFETY: as above.
        let k = unsafe { Node::new(itr.x) }.key(itr.i as usize);
        if !mt_end(k) && k.flags as uint32_t & key_filter != 0 {
            return true;
        }
        // SAFETY: `b` is live and `itr` is positioned in it.
        if !unsafe { marktree_itr_next_skip(b, itr, false, false, None, Some(filter)) } {
            return false;
        }
    }
}

/// Step to the previous key.
///
/// # Safety
/// `itr` must be positioned in a live tree, or empty.
pub unsafe fn marktree_itr_prev(_b: &mut MarkTree, itr: &mut MarkTreeIter) -> bool {
    // SAFETY: `itr` is positioned in a live tree, or empty, per the caller.
    let Some(mut x) = (unsafe { Node::from_ptr(itr.x) }) else {
        return false;
    };
    if x.is_leaf() {
        itr.i -= 1;
        if itr.i >= 0 {
            return true;
        }
        // Walk up until a node still has a key before the one we came out of.
        while itr.i < 0 {
            let Some(parent) = x.parent() else {
                itr.x = ptr::null_mut();
                return false;
            };
            x = parent;
            itr.x = x.as_ptr();
            itr.lvl -= 1;
            itr.i = itr.s[itr.lvl as usize].i - 1;
            if itr.i >= 0 {
                itr.pos.row -= x.key(itr.i as usize).pos.row;
                itr.pos.col = itr.s[itr.lvl as usize].oldcol;
            }
        }
    } else {
        // The previous key is the rightmost of the subtree before this one.
        while !x.is_leaf() {
            if itr.i > 0 {
                itr.s[itr.lvl as usize].oldcol = itr.pos.col;
                let sep = x.key(itr.i as usize - 1).pos;
                compose(&mut itr.pos, sep);
            }
            itr.s[itr.lvl as usize].i = itr.i;
            let child = x.child(itr.i as usize);
            debug_assert!(
                child.parent() == Some(x),
                "itr->x->ptr[itr->i]->parent == itr->x"
            );
            x = child;
            itr.x = x.as_ptr();
            itr.i = x.key_count() as c_int;
            itr.lvl += 1;
        }
        itr.i -= 1;
    }
    true
}

/// The absolute position of the key `itr` is on.
///
/// # Safety
/// `itr` must be positioned in a live tree.
pub unsafe fn marktree_itr_pos(itr: &MarkTreeIter) -> MTPos {
    // SAFETY: `itr` is positioned in a live tree per the caller.
    let mut pos = unsafe { Node::new(itr.x) }.key(itr.i as usize).pos;
    unrelative(itr.pos, &mut pos);
    pos
}

/// The key `itr` is on, at its absolute position, or [`MT_INVALID_KEY`] once
/// the walk has run off the end.
///
/// # Safety
/// `itr` must be positioned in a live tree, or empty.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_itr_current(itr: &mut MarkTreeIter) -> MTKey {
    if itr.x.is_null() {
        return MT_INVALID_KEY;
    }
    // SAFETY: `itr` is positioned in a live tree per the caller.
    let (x, pos) = unsafe { (Node::new(itr.x), marktree_itr_pos(itr)) };
    let mut key = x.key(itr.i as usize);
    key.pos = pos;
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
///
/// # Safety
/// `b` must be a live tree. Nothing here dereferences it, but the iterator
/// this leaves behind names `b`'s root, and [`marktree_itr_step_overlap`]
/// walks from there.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_itr_get_overlap(
    b: &mut MarkTree,
    row: c_int,
    col: c_int,
    itr: &mut MarkTreeIter,
) -> bool {
    if b.n_keys == 0 {
        itr.x = ptr::null_mut();
        return false;
    }
    itr.x = b.root;
    itr.i = -1;
    itr.lvl = 0;
    itr.pos = MTPos::default();
    itr.intersect_pos = MTPos { row, col };
    itr.intersect_pos_x = MTPos { row, col };
    itr.intersect_idx = 0;
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
///
/// # Safety
/// `b` must be a live tree and `itr` one [`marktree_itr_get_overlap`]
/// positioned in it.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn marktree_itr_step_overlap(
    b: &mut MarkTree,
    itr: &mut MarkTreeIter,
    pair: &mut MTPair,
) -> bool {
    // SAFETY: `itr` was positioned in `b` per the caller, so it names one of
    // `b`'s live nodes — the walk below never leaves the tree.
    let mut x = unsafe { Node::new(itr.x) };
    // Phase one: down the path, handing back each node's covering ranges.
    while itr.i == -1 {
        let set = x.intersection();
        if itr.intersect_idx < set.len() {
            let id = set.as_slice()[itr.intersect_idx];
            itr.intersect_idx += 1;
            // SAFETY: `b` is a live tree.
            let halves = unsafe {
                (
                    marktree_lookup(b, id, None),
                    marktree_lookup(b, id | MARKTREE_END_FLAG, None),
                )
            };
            *pair = mtpair_from(halves.0, halves.1);
            return true;
        }
        if x.is_leaf() {
            itr.i = 0;
            itr.s[itr.lvl as usize].i = itr.i;
            break;
        }
        let k = search_key(itr.intersect_pos_x, 0);
        itr.i = find_key(x.keys(), k).0 + 1;
        itr.s[itr.lvl as usize].i = itr.i;
        itr.s[itr.lvl as usize].oldcol = itr.pos.col;
        if itr.i > 0 {
            let sep = x.key(itr.i as usize - 1).pos;
            compose(&mut itr.pos, sep);
            relative(sep, &mut itr.intersect_pos_x);
        }
        x = x.child(itr.i as usize);
        itr.x = x.as_ptr();
        itr.lvl += 1;
        itr.i = -1;
        itr.intersect_idx = 0;
    }
    // Phase two, first half: starts in this leaf that are before the sought
    // position and whose end is not.
    while itr.i < x.key_count() as c_int && pos_less(x.key(itr.i as usize).pos, itr.intersect_pos_x)
    {
        let mut k = x.key(itr.i as usize);
        itr.i += 1;
        itr.s[itr.lvl as usize].i = itr.i;
        if !mt_start(k) {
            continue;
        }
        // SAFETY: `b` is a live tree.
        let end = unsafe { marktree_lookup(b, mt_lookup_id(k.ns, k.id, true), None) };
        if pos_less(end.pos, itr.intersect_pos) {
            continue;
        }
        unrelative(itr.pos, &mut k.pos);
        *pair = mtpair_from(k, end);
        return true;
    }
    // Second half: ends in this leaf whose start is in another node, which is
    // the case the intersection sets cannot record.
    while itr.i < x.key_count() as c_int {
        let mut k = x.key(itr.i as usize);
        itr.i += 1;
        if !mt_end(k) {
            continue;
        }
        let id = mt_lookup_id(k.ns, k.id, false);
        // SAFETY: `b` is a live tree.
        if unsafe { id2node(b, id) } == x.as_ptr() {
            continue;
        }
        unrelative(itr.pos, &mut k.pos);
        // SAFETY: `b` is a live tree.
        let start = unsafe { marktree_lookup(b, id, None) };
        if pos_leq(itr.intersect_pos, start.pos) {
            continue;
        }
        *pair = mtpair_from(start, k);
        return true;
    }
    itr.i = itr.s[itr.lvl as usize].i;
    debug_assert!(itr.i >= 0, "itr->i >= 0");
    if itr.i >= x.key_count() as c_int {
        // SAFETY: `b` is live and `itr` is positioned in it.
        unsafe { marktree_itr_next(b, itr) };
    }
    false
}

/// Park `itr`, where given, on key `i` of node `n`, and answer that key at its
/// absolute position.
///
/// The position comes from walking `n`'s ancestors, which is also where the
/// iterator's path is rebuilt, so this works for a node reached any way at all
/// — an id-map lookup, or a node a splice recorded earlier.
///
/// # Safety
/// `b` must be a live tree and `n` one of its nodes, holding a key at `i`.
pub unsafe fn marktree_itr_set_node(
    b: &mut MarkTree,
    itr: Option<&mut MarkTreeIter>,
    n: Node,
    i: c_int,
) -> MTKey {
    let mut key = n.key(i as usize);
    // SAFETY: `b` is a live tree, so its root is one of its live nodes.
    let root_level = unsafe { Node::new(b.root) }.level();
    let mut itr = itr;
    if let Some(itr) = itr.as_deref_mut() {
        itr.i = i;
        itr.x = n.as_ptr();
        itr.lvl = (root_level - n.level()) as c_int;
    }
    let mut n = n;
    while let Some(p) = n.parent() {
        let i = n.parent_index();
        debug_assert!(p.child(i) == n, "p->ptr[i] == n");
        if let Some(itr) = itr.as_deref_mut() {
            itr.s[root_level - p.level()].i = i as c_int;
        }
        if i > 0 {
            unrelative(p.key(i - 1).pos, &mut key.pos);
        }
        n = p;
    }
    if let Some(itr) = itr {
        // SAFETY: `b` is live and `itr` now names a live node of it.
        unsafe { marktree_itr_fix_pos(b, itr) };
    }
    key
}

/// Recompute `itr.pos` — and the `oldcol` of every level — by walking the path
/// the iterator recorded, after something moved the keys it was rebased on.
///
/// # Safety
/// `b` must be a live tree and `itr` positioned in it.
pub unsafe fn marktree_itr_fix_pos(b: &mut MarkTree, itr: &mut MarkTreeIter) {
    itr.pos = MTPos::default();
    // SAFETY: `b` is a live tree, so its root is one of its live nodes.
    let mut x = unsafe { Node::new(b.root) };
    for lvl in 0..itr.lvl as usize {
        itr.s[lvl].oldcol = itr.pos.col;
        let i = itr.s[lvl].i;
        if i > 0 {
            let sep = x.key(i as usize - 1).pos;
            compose(&mut itr.pos, sep);
        }
        debug_assert!(!x.is_leaf(), "x->level");
        x = x.child(i as usize);
    }
    debug_assert!(x.as_ptr() == itr.x, "x == itr->x");
}
