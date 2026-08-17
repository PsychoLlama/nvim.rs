//! [`Cursor`]: a marktree iterator, positioned in the tree it walks.
//!
//! The twin of [`Node`](super::node::Node). `Node` made a *node* reachable
//! without a promise per dereference; this does the same for a *walk*. The
//! iterator functions all want two things at once — the tree and an iterator
//! already positioned in it — which a signature cannot say, so every consumer
//! outside this family paid a promise per step. A `Cursor` states the pair
//! once, at construction, and every step after that is ordinary checked code.
//!
//! # Why two raw pointers and not two references
//!
//! Both halves are re-derived for the duration of one call, in [`parts`], and
//! never held across one. That is deliberate, and it is the same trade
//! `Rex` makes in `regexp/rex.rs`:
//!
//! * an iterator outlives the calls it is stepped by — `DecorState` keeps one
//!   between rows of a redraw — so a `&mut MarkTreeIter` would have to live
//!   inside a `GlobalCell` that other code reaches through the same cell;
//! * the walks here run decoration providers, which is Lua, which can place
//!   and delete marks. Upstream keeps walking afterwards (and
//!   `decor_state_invalidate` is the machinery that copes), so a `&mut
//!   MarkTree` spanning the callback would be a borrow the editor really does
//!   invalidate underneath itself, not a theoretical one.
//!
//! The `PhantomData` still books the *iterator* as uniquely borrowed for the
//! cursor's life, so a caller cannot step the same walk two ways at once.
//!
//! Copyright Neovim contributors. Licensed under the Apache License, Version
//! 2.0; see LICENSE.txt in the project root.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;
use core::marker::PhantomData;

use crate::marktree::iter::{
    marktree_itr_current, marktree_itr_get, marktree_itr_get_filter, marktree_itr_get_overlap,
    marktree_itr_next, marktree_itr_next_filter, marktree_itr_step_out_filter,
    marktree_itr_step_overlap,
};
use crate::marktree::key::MT_INVALID_KEY;
use crate::marktree::marktree_lookup_ns;
use crate::marktree::meta::MetaCount;
use crate::marktree::pair::marktree_get_altpos;
use crate::types::{MTKey, MTPair, MTPos, MarkTree, MarkTreeIter, int32_t, uint32_t};
use crate::winlayer::Buf;

/// A walk over one marktree.
///
/// Built from a tree and an iterator that belongs to it; from then on every
/// step is a safe call. See the module documentation for why it holds
/// pointers rather than references.
pub struct Cursor<'a> {
    tree: *mut MarkTree,
    itr: *mut MarkTreeIter,
    /// Books `itr` as uniquely borrowed: two cursors cannot step one walk.
    walk: PhantomData<&'a mut MarkTreeIter>,
}

/// `buf`'s marktree.
///
/// Safe: [`Buf`] has already promised a live buffer, and `b_marktree` is one
/// of its own fields. Dereferencing the answer is what needs a promise.
pub fn tree_of(mut buf: Buf) -> *mut MarkTree {
    buf.b_marktree.as_mut_ptr()
}

/// The mark `ns`/`id` names in `buf` — the `end` half when asked for it —
/// without moving any walk.
pub fn lookup_ns(buf: Buf, ns: uint32_t, id: uint32_t, end: bool) -> MTKey {
    // SAFETY: a live buffer's marktree, and the lookup writes no iterator.
    unsafe { marktree_lookup_ns(&mut *tree_of(buf), ns, id, end, None) }
}

impl<'a> Cursor<'a> {
    /// A walk over `buf`'s marks, stepped with `itr`.
    ///
    /// Safe: [`Buf`] has already promised a live buffer, whose `b_marktree`
    /// is therefore a live tree, and the borrow is a live iterator — either
    /// fresh (all-zero, which is where a walk starts) or one this same
    /// buffer positioned earlier.
    pub fn in_buffer(buf: Buf, itr: &'a mut MarkTreeIter) -> Self {
        Self {
            tree: tree_of(buf),
            itr,
            walk: PhantomData,
        }
    }

    /// # Safety
    /// `tree` must be a live marktree and `itr` a live iterator that is
    /// either fresh or already positioned in *that* tree, both for `'a`.
    pub const unsafe fn from_raw(tree: *mut MarkTree, itr: *mut MarkTreeIter) -> Self {
        Self {
            tree,
            itr,
            walk: PhantomData,
        }
    }

    /// The tree and the iterator, for the length of one call.
    fn parts(&mut self) -> (&mut MarkTree, &mut MarkTreeIter) {
        // SAFETY: the constructor's promise — a live tree and a live iterator
        // in it. `&mut self` makes the pair unique for the call, and it is
        // never held past the statement that asked for it.
        unsafe { (&mut *self.tree, &mut *self.itr) }
    }

    /// Whether the walk has run off the end of the tree.
    pub fn is_empty(&mut self) -> bool {
        self.parts().1.x.is_null()
    }

    /// Position the walk at the first mark at or after (`row`, `col`).
    pub fn seek(&mut self, row: int32_t, col: c_int) -> bool {
        let (tree, itr) = self.parts();
        // SAFETY: a live tree; this is what positions the iterator in it.
        unsafe { marktree_itr_get(tree, row, col, itr) }
    }

    /// [`Cursor::seek`], descending only into subtrees `filter` wants and
    /// giving up at (`stop_row`, `stop_col`).
    pub fn seek_filter(
        &mut self,
        row: int32_t,
        col: c_int,
        stop_row: c_int,
        stop_col: c_int,
        filter: &'static MetaCount,
    ) -> bool {
        let (tree, itr) = self.parts();
        // SAFETY: a live tree, and `filter` is a `'static` count array.
        unsafe { marktree_itr_get_filter(tree, row, col, stop_row, stop_col, filter.as_ptr(), itr) }
    }

    /// Position the walk to enumerate the ranges *covering* (`row`, `col`) —
    /// the ones that started earlier and reach into it. Follow with
    /// [`Cursor::step_overlap`] until it answers `None`, which leaves the
    /// walk an ordinary one positioned at (`row`, `col`).
    pub fn seek_overlap(&mut self, row: c_int, col: c_int) -> bool {
        let (tree, itr) = self.parts();
        // SAFETY: a live tree; this is what positions the iterator in it.
        unsafe { marktree_itr_get_overlap(tree, row, col, itr) }
    }

    /// One more range covering the position [`Cursor::seek_overlap`] was
    /// given, or `None` once they are exhausted.
    pub fn step_overlap(&mut self) -> Option<MTPair> {
        // Only read back when the step answers true, which is when it has
        // written every field; the seed is the family's own "no mark".
        let mut pair = MTPair {
            start: MT_INVALID_KEY,
            end_pos: MTPos::default(),
            end_right_gravity: false,
        };
        let (tree, itr) = self.parts();
        // SAFETY: a live tree with the iterator positioned in it by
        // `seek_overlap`, which is what the type of `self` promises.
        let more = unsafe { marktree_itr_step_overlap(tree, itr, &mut pair) };
        more.then_some(pair)
    }

    /// Leave the subtrees `filter` has nothing in, and answer whether the
    /// walk still names a node.
    pub fn step_out_filter(&mut self, filter: &'static MetaCount) -> bool {
        let (tree, itr) = self.parts();
        // SAFETY: a live tree with the iterator positioned in it, and
        // `filter` is a `'static` count array.
        unsafe { marktree_itr_step_out_filter(tree, itr, filter.as_ptr()) }
    }

    /// The mark the walk is on, or [`MT_INVALID_KEY`](super::key::MT_INVALID_KEY)
    /// once it has run off the end.
    pub fn current(&mut self) -> MTKey {
        let itr = self.parts().1;
        // SAFETY: the iterator is positioned in a live tree, or empty.
        unsafe { marktree_itr_current(itr) }
    }

    /// Step to the next mark.
    pub fn next(&mut self) -> bool {
        let (tree, itr) = self.parts();
        // SAFETY: a live tree with the iterator positioned in it.
        unsafe { marktree_itr_next(tree, itr) }
    }

    /// Step to the next mark `filter` wants, giving up at (`stop_row`,
    /// `stop_col`).
    pub fn next_filter(
        &mut self,
        stop_row: c_int,
        stop_col: c_int,
        filter: &'static MetaCount,
    ) -> bool {
        let (tree, itr) = self.parts();
        // SAFETY: a live tree with the iterator positioned in it, and
        // `filter` is a `'static` count array.
        unsafe { marktree_itr_next_filter(tree, itr, stop_row, stop_col, filter.as_ptr()) }
    }

    /// Where the other half of `mark`'s pair sits — `mark`'s own position
    /// when it is unpaired. Does not move the walk.
    pub fn altpos(&mut self, mark: MTKey) -> MTPos {
        let tree = self.parts().0;
        // SAFETY: a live tree, and `mark` was read out of it.
        unsafe { marktree_get_altpos(tree, mark, None) }
    }
}

/// The marks the walk yields from where it stands, stepping with `next`,
/// until one is past (`row`) or the tree runs out.
///
/// The shape both `decoration/` and `plines.rs` spell out: `seek`, then a
/// loop that reads the mark, stops at a row bound, and steps.
pub struct Marks<'c, 'a> {
    cursor: &'c mut Cursor<'a>,
    step: Step,
    done: bool,
}

/// How a [`Marks`] walk advances.
enum Step {
    /// Plain `marktree_itr_next`.
    All,
    /// `marktree_itr_next_filter` up to a stop position.
    Filtered {
        stop_row: c_int,
        stop_col: c_int,
        filter: &'static MetaCount,
    },
}

impl<'c, 'a> Iterator for Marks<'c, 'a> {
    type Item = MTKey;

    fn next(&mut self) -> Option<MTKey> {
        if self.done || self.cursor.is_empty() {
            return None;
        }
        let mark = self.cursor.current();
        self.done = match self.step {
            Step::All => !self.cursor.next(),
            Step::Filtered {
                stop_row,
                stop_col,
                filter,
            } => !self.cursor.next_filter(stop_row, stop_col, filter),
        };
        Some(mark)
    }
}

impl<'a> Cursor<'a> {
    /// Every mark from where the walk stands, in tree order.
    pub fn marks<'c>(&'c mut self) -> Marks<'c, 'a> {
        Marks {
            cursor: self,
            step: Step::All,
            done: false,
        }
    }

    /// Every mark `filter` wants from where the walk stands, up to
    /// (`stop_row`, `stop_col`).
    pub fn filtered_marks<'c>(
        &'c mut self,
        stop_row: c_int,
        stop_col: c_int,
        filter: &'static MetaCount,
    ) -> Marks<'c, 'a> {
        Marks {
            cursor: self,
            step: Step::Filtered {
                stop_row,
                stop_col,
                filter,
            },
            done: false,
        }
    }

    /// Every range covering the position [`Cursor::seek_overlap`] was given.
    pub fn overlaps<'c>(&'c mut self) -> impl Iterator<Item = MTPair> + 'c
    where
        'a: 'c,
    {
        ::core::iter::from_fn(move || self.step_overlap())
    }
}
