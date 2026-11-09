//! [`Cursor`]: a marktree iterator, positioned in the tree it walks.
//!
//! The iterator functions all want two things at once — the tree and an
//! iterator already positioned in it. A `Cursor` pairs a buffer with an
//! iterator once, at construction, and every step after that borrows the
//! buffer's tree for that one call.
//!
//! # Why a buffer handle and not a `&mut MarkTree`
//!
//! The tree is re-borrowed through the [`Buf`] for the duration of one call,
//! and never held across one. That is deliberate:
//!
//! * an iterator outlives the calls it is stepped by — `DecorState` keeps one
//!   between rows of a redraw — so a `&mut MarkTreeIter` would have to live
//!   inside a `GlobalCell` that other code reaches through the same cell;
//! * the walks here run decoration providers, which is Lua, which can place
//!   and delete marks. Upstream keeps walking afterwards (and
//!   `Buf::invalidate_decor_state` is the machinery that copes), so a `&mut
//!   MarkTree` spanning the callback would be a borrow the editor really does
//!   invalidate underneath itself, not a theoretical one. An iterator names
//!   its node by id, so one a callback left stale reads a wrong key at worst.
//!
//! Copyright Neovim contributors. Licensed under the Apache License, Version
//! 2.0; see LICENSE.txt in the project root.

#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

use core::ffi::c_int;

use crate::marktree::iter::{
    marktree_itr_current, marktree_itr_get, marktree_itr_get_filter, marktree_itr_get_overlap,
    marktree_itr_next, marktree_itr_next_filter, marktree_itr_step_out_filter,
    marktree_itr_step_overlap,
};
use crate::marktree::key::MT_INVALID_KEY;
use crate::marktree::marktree_lookup_ns;
use crate::marktree::meta::MetaCount;
use crate::marktree::pair::marktree_get_altpos;
use crate::marktree::{MTKey, MTPair, MTPos, MarkTree, MarkTreeIter};
use crate::os::libc::{int32_t, uint32_t};
use crate::winlayer::Buf;

/// A walk over one buffer's marktree.
///
/// Built from a buffer and an iterator that belongs to its tree; from then on
/// every step is a safe call. See the module documentation for why it holds
/// the buffer handle rather than a reference to the tree.
#[derive(Debug)]
pub struct Cursor<'a> {
    buffer: Buf,
    iter: &'a mut MarkTreeIter,
}

/// The mark `ns`/`id` names in `buffer` — the `end` half when asked for it —
/// without moving any walk.
pub fn lookup_ns(mut buffer: Buf, ns: uint32_t, id: uint32_t, end: bool) -> MTKey {
    marktree_lookup_ns(&mut buffer.b_marktree, ns, id, end, None)
}

impl<'a> Cursor<'a> {
    /// A walk over `buffer`'s marks, stepped with `iter` — either fresh
    /// (positioned nowhere, which is where a walk starts) or one this same
    /// buffer positioned earlier.
    pub fn in_buffer(buffer: Buf, iter: &'a mut MarkTreeIter) -> Self {
        Self { buffer, iter }
    }

    /// The tree and the iterator, for the length of one call.
    fn parts(&mut self) -> (&mut MarkTree, &mut MarkTreeIter) {
        (&mut self.buffer.b_marktree, &mut *self.iter)
    }

    /// Whether the walk has run off the end of the tree.
    pub fn is_empty(&mut self) -> bool {
        !self.iter.is_positioned()
    }

    /// Position the walk at the first mark at or after (`row`, `col`).
    pub fn seek(&mut self, row: int32_t, col: c_int) -> bool {
        let (tree, iter) = self.parts();
        marktree_itr_get(tree, row, col, iter)
    }

    /// [`Cursor::seek`], descending only into subtrees `filter` wants and
    /// giving up at (`stop_row`, `stop_col`).
    pub fn seek_filter(
        &mut self,
        row: int32_t,
        col: c_int,
        stop_row: c_int,
        stop_col: c_int,
        filter: &MetaCount,
    ) -> bool {
        let (tree, iter) = self.parts();
        marktree_itr_get_filter(tree, row, col, stop_row, stop_col, filter, iter)
    }

    /// Position the walk to enumerate the ranges *covering* (`row`, `col`) —
    /// the ones that started earlier and reach into it. Follow with
    /// [`Cursor::step_overlap`] until it answers `None`, which leaves the
    /// walk an ordinary one positioned at (`row`, `col`).
    pub fn seek_overlap(&mut self, row: c_int, col: c_int) -> bool {
        let (tree, iter) = self.parts();
        marktree_itr_get_overlap(tree, row, col, iter)
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
        let (tree, iter) = self.parts();
        let more = marktree_itr_step_overlap(tree, iter, &mut pair);
        more.then_some(pair)
    }

    /// Leave the subtrees `filter` has nothing in, and answer whether the
    /// walk still names a node.
    pub fn step_out_filter(&mut self, filter: &MetaCount) -> bool {
        let (tree, iter) = self.parts();
        marktree_itr_step_out_filter(tree, iter, filter)
    }

    /// The mark the walk is on, or [`MT_INVALID_KEY`]
    /// once it has run off the end.
    pub fn current(&mut self) -> MTKey {
        let (tree, iter) = self.parts();
        marktree_itr_current(tree, iter)
    }

    /// Step to the next mark. Not `Iterator::next` — a cursor answers
    /// "is there one" and leaves the mark itself to [`current`](Self::current).
    pub fn step(&mut self) -> bool {
        let (tree, iter) = self.parts();
        marktree_itr_next(tree, iter)
    }

    /// Step to the next mark `filter` wants, giving up at (`stop_row`,
    /// `stop_col`).
    pub fn step_filter(&mut self, stop_row: c_int, stop_col: c_int, filter: &MetaCount) -> bool {
        let (tree, iter) = self.parts();
        marktree_itr_next_filter(tree, iter, stop_row, stop_col, filter)
    }

    /// Where the other half of `mark`'s pair sits — `mark`'s own position
    /// when it is unpaired. Does not move the walk.
    pub fn altpos(&mut self, mark: MTKey) -> MTPos {
        let tree = self.parts().0;
        marktree_get_altpos(tree, mark, None)
    }
}

/// The marks the walk yields from where it stands, stepping with `next`,
/// until one is past (`row`) or the tree runs out.
///
/// The shape both `decoration/` and `plines.rs` spell out: `seek`, then a
/// loop that reads the mark, stops at a row bound, and steps.
#[derive(Debug)]
pub struct Marks<'c, 'a> {
    cursor: &'c mut Cursor<'a>,
    step: Step,
    done: bool,
}

/// How a [`Marks`] walk advances.
#[derive(Debug)]
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
            Step::All => !self.cursor.step(),
            Step::Filtered {
                stop_row,
                stop_col,
                filter,
            } => !self.cursor.step_filter(stop_row, stop_col, filter),
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
