//! The decoration pointers, wrapped.
//!
//! Decorations are reached through raw pointers everywhere — a mark's
//! `ext` branch names a [`DecorVirtText`] chain, the sign/highlight items live
//! in one global store and chain through indices into it, and the drawing
//! state hands out pointers into a slab it keeps reallocating. The pointers
//! have to stay raw: [`decor_put_sh`](super::decor_put_sh) invalidates every
//! item pointer, the drawing code holds several ranges at once while writing
//! `draw_col` through them, and decoration providers re-enter through Lua.
//!
//! What does not have to stay raw is the *dereference*. Each wrapper below
//! makes **construction** the unsafe step and gives ordinary field access
//! after it, exactly as [`winlayer`](crate::winlayer) does for
//! windows and buffers. The chain walks ([`Virt::chain`], [`Sh::chain`]) then
//! cost one promise instead of one per link.
//!
//! Copyright Neovim contributors. Licensed under the Apache License, Version
//! 2.0; see LICENSE.txt in the project root.

#![deny(unsafe_op_in_unsafe_fn)]

use ::core::ffi::c_int;
use ::core::iter;
use ::core::ops::{Deref, DerefMut};

use crate::buffer::buf_meta_total;
use crate::decoration::{
    DECOR_ID_INVALID, decor_item, kVTHide, kVTIsLines, kVTLinesAbove, mt_decor_virt,
};
use crate::types::{
    DecorInline, DecorRange, DecorRangeSlot, DecorSignHighlight, DecorState, DecorVirtText, MTKey,
    MetaIndex, VirtLines, VirtText, uint32_t,
};
use crate::winlayer::Buf;

// ---------------------------------------------------------------------------
// Virtual text

/// One link of a mark's virtual-text chain, promised live.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Virt(*mut DecorVirtText);

impl Deref for Virt {
    type Target = DecorVirtText;

    #[inline(always)]
    fn deref(&self) -> &DecorVirtText {
        // SAFETY: the constructor's promise — a live virtual text.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Virt {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut DecorVirtText {
        // SAFETY: the constructor's promise — a live virtual text.
        unsafe { &mut *self.0 }
    }
}

impl Virt {
    /// # Safety
    /// `vt` must stay a live virtual text for as long as the value is used.
    #[inline(always)]
    pub const unsafe fn new(vt: *mut DecorVirtText) -> Self {
        Self(vt)
    }

    /// The virtual text `vt` names, `None` for null — which is how a chain
    /// ends.
    ///
    /// # Safety
    /// `vt` must be null, or stay a live virtual text for as long as the
    /// value is used.
    #[inline(always)]
    pub const unsafe fn from_raw(vt: *mut DecorVirtText) -> Option<Self> {
        if vt.is_null() { None } else { Some(Self(vt)) }
    }

    #[inline(always)]
    pub fn raw(self) -> *mut DecorVirtText {
        self.0
    }

    /// The next link, if any.
    #[inline(always)]
    pub fn next(self) -> Option<Self> {
        // A live chain's `next` is a live virtual text or null.
        let next = self.next;
        (!next.is_null()).then_some(Self(next))
    }

    /// This link and every one after it.
    ///
    /// # Safety
    /// `first` must be null or the head of a live chain.
    pub unsafe fn chain(first: *mut DecorVirtText) -> impl Iterator<Item = Self> {
        // SAFETY: the caller's chain head.
        let head = unsafe { Self::from_raw(first) };
        iter::successors(head, |vt| vt.next())
    }

    /// Whether this is a block of whole virtual *lines* rather than inline
    /// virtual *text* — the two share the chain.
    #[inline(always)]
    pub fn is_lines(self) -> bool {
        self.flags as c_int & kVTIsLines as c_int != 0
    }

    /// Whether a virtual-lines block is drawn above the line its mark sits
    /// on rather than below it.
    #[inline(always)]
    pub fn lines_above(self) -> bool {
        self.flags as c_int & kVTLinesAbove as c_int != 0
    }

    /// Whether an overlay virtual text hides itself over a concealed cell.
    #[inline(always)]
    pub fn hides_over_concealed(self) -> bool {
        self.flags as c_int & kVTHide as c_int != 0
    }

    /// The block of virtual lines this link carries.
    #[inline(always)]
    pub fn lines(self) -> VirtLines {
        // SAFETY: `is_lines` says the union holds the `virt_lines` branch,
        // and both branches are the same plain vector type anyway.
        unsafe { self.data.virt_lines }
    }

    /// The chunks of inline virtual text this link carries.
    #[inline(always)]
    pub fn text(self) -> VirtText {
        // SAFETY: as `lines` — the two branches are the same layout.
        unsafe { self.data.virt_text }
    }

    /// The chunk vector itself, for the code that frees it.
    #[inline(always)]
    pub fn text_ptr(self) -> *mut VirtText {
        // The union's branches share an address.
        unsafe { &raw mut (*self.0).data.virt_text }
    }
}

// ---------------------------------------------------------------------------
// Sign and highlight items

/// One entry of the decoration item store, promised live.
///
/// Live means "since the last [`decor_put_sh`](super::decor_put_sh)": the
/// store is one `Vec` and adding to it moves every entry.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Sh(*mut DecorSignHighlight);

impl Deref for Sh {
    type Target = DecorSignHighlight;

    #[inline(always)]
    fn deref(&self) -> &DecorSignHighlight {
        // SAFETY: the constructor's promise — a live item.
        unsafe { &*self.0 }
    }
}

impl DerefMut for Sh {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut DecorSignHighlight {
        // SAFETY: the constructor's promise — a live item.
        unsafe { &mut *self.0 }
    }
}

impl Sh {
    /// # Safety
    /// `sh` must stay a live sign/highlight item for as long as the value is
    /// used.
    #[inline(always)]
    pub const unsafe fn new(sh: *mut DecorSignHighlight) -> Self {
        Self(sh)
    }

    /// # Safety
    /// `sh` must be null, or stay live for as long as the value is used.
    #[inline(always)]
    pub const unsafe fn from_raw(sh: *mut DecorSignHighlight) -> Option<Self> {
        if sh.is_null() { None } else { Some(Self(sh)) }
    }

    /// Item `idx` of the store.
    ///
    /// Safe: [`decor_item`](super::decor_item) panics on an index the store
    /// never handed out, and what it answers stays live until the next
    /// [`decor_put_sh`](super::decor_put_sh) — the same contract every
    /// caller of `decor_item` already works to.
    #[inline(always)]
    pub fn at(idx: uint32_t) -> Self {
        Self(decor_item(idx))
    }

    #[inline(always)]
    pub fn raw(self) -> *mut DecorSignHighlight {
        self.0
    }

    /// The next item of this decoration's chain, if any.
    #[inline(always)]
    pub fn next_item(self) -> Option<Self> {
        (self.next != DECOR_ID_INVALID).then(|| Self::at(self.next))
    }

    /// Every sign/highlight item of `decor`, in chain order. Empty for an
    /// inline decoration, which has no store entry at all.
    pub fn chain(decor: DecorInline) -> impl Iterator<Item = Self> {
        // SAFETY: `ext` says the union holds the `ext` branch.
        let first = decor.ext.then(|| unsafe { decor.data.ext.sh_idx });
        let head = first.filter(|idx| *idx != DECOR_ID_INVALID).map(Self::at);
        iter::successors(head, |sh| sh.next_item())
    }
}

/// Every virtual text `mark` carries, in chain order.
///
/// Safe: a key read out of a live marktree names a live chain, which is the
/// only way a caller has one.
pub fn mark_virt_chain(mark: MTKey) -> impl Iterator<Item = Virt> {
    // SAFETY: as above — `mt_decor_virt` answers null or the head of the
    // live chain the mark owns.
    unsafe { Virt::chain(mt_decor_virt(mark)) }
}

// ---------------------------------------------------------------------------
// The buffer's mark counts

impl Buf {
    /// How many marks of `kind` the whole buffer holds.
    ///
    /// The cheap test every per-row question starts with: a buffer with none
    /// of the kind in question never touches the marktree.
    #[inline(always)]
    pub fn meta_total(self, kind: MetaIndex) -> uint32_t {
        // SAFETY: a live buffer.
        unsafe { buf_meta_total(self.raw(), kind) }
    }
}

// ---------------------------------------------------------------------------
// The drawing state

/// A [`DecorState`] the caller has promised is live.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct State(*mut DecorState);

impl Deref for State {
    type Target = DecorState;

    #[inline(always)]
    fn deref(&self) -> &DecorState {
        // SAFETY: the constructor's promise — a live state.
        unsafe { &*self.0 }
    }
}

impl DerefMut for State {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut DecorState {
        // SAFETY: the constructor's promise — a live state.
        unsafe { &mut *self.0 }
    }
}

impl State {
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

    /// The virtual text this range draws, for the kinds that have one.
    #[inline(always)]
    pub fn virt(self) -> Virt {
        // SAFETY: the caller checked `kind`; a range's `vt` is live for as
        // long as the range is.
        Virt(unsafe { self.data.vt })
    }
}

/// The range half of a slab slot.
///
/// The union's other branch is the freelist link, which overwrites only the
/// first field; every caller here has an index out of one of the two sorted
/// lists, which name occupied slots.
#[inline(always)]
pub fn slot_range(slot: &mut DecorRangeSlot) -> &mut DecorRange {
    // SAFETY: a slot reached through `ranges_i` holds a range.
    unsafe { &mut slot.range }
}
