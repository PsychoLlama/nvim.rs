//! The drawing state's pointers, wrapped.
//!
//! The drawing state hands out pointers into a slab it keeps reallocating,
//! and the drawing code holds several ranges at once while writing
//! `draw_col` through them.
//!
//! What does not have to stay raw is the *dereference*. Each wrapper below
//! makes **construction** the unsafe step and gives ordinary field access
//! after it, exactly as [`winlayer`](crate::winlayer) does for
//! windows and buffers.
//!
//! Copyright Neovim contributors. Licensed under the Apache License, Version
//! 2.0; see LICENSE.txt in the project root.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use core::ops::{Deref, DerefMut};

use crate::buffer::buf_meta_total;
use crate::types::{DecorRange, DecorRangeSlot, DecorState, MetaIndex, uint32_t};
use crate::winlayer::Buf;

// ---------------------------------------------------------------------------
// The buffer's mark counts

impl Buf {
    /// How many marks of `kind` the whole buffer holds.
    ///
    /// The cheap test every per-row question starts with: a buffer with none
    /// of the kind in question never touches the marktree.
    #[inline(always)]
    pub fn meta_total(self, kind: MetaIndex) -> uint32_t {
        buf_meta_total(self, kind)
    }
}

// ---------------------------------------------------------------------------
// The drawing state

/// A [`DecorState`] the caller has promised is live.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct DecorStateRef(*mut DecorState);

impl Deref for DecorStateRef {
    type Target = DecorState;

    #[inline(always)]
    fn deref(&self) -> &DecorState {
        // SAFETY: the constructor's promise — a live state.
        unsafe { &*self.0 }
    }
}

impl DerefMut for DecorStateRef {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut DecorState {
        // SAFETY: the constructor's promise — a live state.
        unsafe { &mut *self.0 }
    }
}

impl DecorStateRef {
    /// # Safety
    /// `state` must stay a live `DecorState` for as long as the value is
    /// used.
    #[inline(always)]
    pub const unsafe fn new(state: *mut DecorState) -> Self {
        Self(state)
    }

    #[inline(always)]
    pub fn raw(self) -> *mut DecorState {
        self.0
    }
}

/// A [`DecorRange`] in the drawing state's slab, promised live.
///
/// A handle rather than a borrow because the drawing code holds several at
/// once and writes `draw_col` through them while reading the rest of the
/// state.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Range(*mut DecorRange);

impl Deref for Range {
    type Target = DecorRange;

    #[inline(always)]
    fn deref(&self) -> &DecorRange {
        // SAFETY: the constructor's promise — a live range.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Range {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut DecorRange {
        // SAFETY: the constructor's promise — a live range.
        unsafe { &mut *self.0 }
    }
}

impl Range {
    /// # Safety
    /// `range` must stay a live `DecorRange` for as long as the value is
    /// used.
    #[inline(always)]
    pub const unsafe fn new(range: *mut DecorRange) -> Self {
        Self(range)
    }

    #[inline(always)]
    pub fn raw(self) -> *mut DecorRange {
        self.0
    }

    /// The virtual-text link this range draws, for the kinds that have one.
    #[inline(always)]
    pub fn virt(self) -> uint32_t {
        self.data.virt()
    }
}

/// The range in a slab slot.
///
/// Every caller has an index out of one of the two sorted lists, and those
/// name occupied slots; a freed slot holds a freelist link instead.
#[inline(always)]
pub fn slot_range(slot: &mut DecorRangeSlot) -> &mut DecorRange {
    match slot {
        DecorRangeSlot::Range(range) => range,
        DecorRangeSlot::Free(_) => unreachable!("decor: a sorted index names a freed slot"),
    }
}
