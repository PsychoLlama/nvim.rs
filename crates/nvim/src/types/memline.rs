#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

// Canonical type definitions, hoisted out of the per-module copies c2rust
// emitted. One definition per logical type; every module re-exports here.
use super::*;
use crate::memline::MlFlags;

#[derive(Copy, Clone)]
pub struct chunksize_T {
    pub mlcs_numlines: ::core::ffi::c_int,
    pub mlcs_totalsize: ::core::ffi::c_int,
}
/// The data block [`crate::memline::ml_find_line`] is holding, and what has
/// been done to it since.
///
/// The memfile hands a block out (`mf_get`) and takes it back (`mf_put`), and
/// the two answers `mf_put` wants -- whether the block changed, and whether
/// the line *positions* in it changed -- are decided over the whole time it
/// is held. Upstream kept the six facts in four `memline_T` fields plus two
/// bits of `ml_flags`; they are one value now, and `ml_locked` is `Some`
/// exactly while a block is out.
///
/// It is deliberately **not** a `Drop` guard. The block is held *across*
/// calls -- a run of reads keeps the same one locked, which is the whole
/// point of it -- and by the time a `buf_T` is dropped `ml_close` has run
/// `mf_close`, which freed every `bhdr_T` in the memfile; a `Drop` that put
/// the block back would be a use-after-free. What makes it a guard is that
/// there is exactly one acquire (`ml_find_line`'s walk, through
/// [`memline_T::lock`]) and exactly one release ([`memline_T::unlock`]),
/// and that the release arguments travel with the block instead of in flag
/// bits that outlive it.
pub(crate) struct LockedBlock {
    /// What `mf_get` handed out.
    pub hp: *mut bhdr_T,
    /// The first line the block holds.
    pub low: linenr_T,
    /// The last line it holds, *after* the insert or delete the walk that
    /// locked it is making room for.
    pub high: linenr_T,
    /// Lines added to (or, negative, removed from) the block that the
    /// pointer blocks above it have not been told about yet.
    pub lineadd: ::core::ffi::c_int,
    /// Upstream's `ML_LOCKED_DIRTY`: the block changed and has to be
    /// written back.
    pub dirty: bool,
    /// Upstream's `ML_LOCKED_POS`: the line positions in it changed too, so
    /// the pointer block above has to be corrected as well.
    pub moved: bool,
}

#[derive(Copy, Clone)]
pub struct infoptr_T {
    pub ip_bnum: blocknr_T,
    pub ip_low: linenr_T,
    pub ip_high: linenr_T,
    pub ip_index: ::core::ffi::c_int,
}
pub struct memline_T {
    pub ml_line_count: linenr_T,
    pub ml_mfp: *mut memfile_T,
    /// The path from the root of the block tree down to the block
    /// [`memline_T::ml_locked`] names, one entry per pointer block, the
    /// root first. Upstream grew this by hand (`ml_stack` plus
    /// `ml_stack_top` and `ml_stack_size`); the length *is* the top, and
    /// the capacity is nobody's business.
    ///
    /// Truncating it to zero is how every failure path says "the tree
    /// moved under me"; the entries are plain data and cost nothing to
    /// drop.
    pub ml_stack: Vec<infoptr_T>,
    pub ml_flags: MlFlags,
    pub ml_line_textlen: colnr_T,
    pub ml_line_lnum: linenr_T,
    pub ml_line_ptr: *mut ::core::ffi::c_char,
    pub ml_line_offset: size_t,
    pub ml_line_offset_ff: ::core::ffi::c_int,
    /// The data block the walk is holding, if it is holding one.
    pub(crate) ml_locked: Option<LockedBlock>,
    pub ml_chunksize: *mut chunksize_T,
    pub ml_numchunks: ::core::ffi::c_int,
    pub ml_usedchunks: ::core::ffi::c_int,
}

impl memline_T {
    /// How deep the block stack is: upstream's `ml_stack_top`.
    pub fn stack_len(&self) -> usize {
        self.ml_stack.len()
    }

    /// Entry `idx` of the block stack, by value.
    ///
    /// Every entry is [`Copy`] and no method hands out a reference into the
    /// stack, so no caller can be holding one when a push moves it.
    pub fn stack_at(&self, idx: usize) -> infoptr_T {
        self.ml_stack[idx]
    }

    /// Overwrite entry `idx`.
    pub fn stack_set(&mut self, idx: usize, entry: infoptr_T) {
        self.ml_stack[idx] = entry;
    }

    /// Record which entry of the pointer block at `idx` the walk went down.
    pub fn stack_set_index(&mut self, idx: usize, index: ::core::ffi::c_int) {
        self.ml_stack[idx].ip_index = index;
    }

    /// Correct the last line entry `idx` covers, after lines were added to
    /// or removed from the block below it.
    pub fn stack_add_high(&mut self, idx: usize, count: linenr_T) {
        self.ml_stack[idx].ip_high += count;
    }

    /// Push a blank entry and answer its index. Every caller fills it in.
    pub fn stack_push(&mut self) -> usize {
        self.ml_stack.push(infoptr_T {
            ip_bnum: 0,
            ip_low: 0,
            ip_high: 0,
            ip_index: 0,
        });
        self.ml_stack.len() - 1
    }

    /// Drop the deepest entry, if there is one.
    pub fn stack_pop(&mut self) -> Option<infoptr_T> {
        self.ml_stack.pop()
    }

    /// Keep only the first `len` entries.
    pub fn stack_truncate(&mut self, len: usize) {
        self.ml_stack.truncate(len);
    }

    /// Forget the whole path: what every failure says, and what a walk that
    /// has to start again at the root says.
    pub fn stack_clear(&mut self) {
        self.ml_stack.clear();
    }

    /// Give the stack's memory back. Only [`crate::memline::ml_close`] does
    /// this; every other reset keeps the allocation for the next walk.
    pub fn stack_free(&mut self) {
        self.ml_stack = Vec::new();
    }

    /// Whether the line `ml_line_ptr` names is the memline's own allocation
    /// rather than a pointer into the locked block, so that dropping it
    /// means freeing it.
    ///
    /// Either flag says so: `LINE_DIRTY` because a rewritten line is always
    /// rebuilt into fresh memory, `ALLOCATED` because `ml_get` copies a line
    /// out of a block it is about to release.
    pub fn line_is_owned(&self) -> bool {
        self.ml_flags.has(MlFlags::LINE_DIRTY | MlFlags::ALLOCATED)
    }

    /// Forget the line `ml_line_ptr` names -- the caller has freed it, or
    /// handed ownership on.
    pub fn forget_line(&mut self) {
        self.ml_flags
            .clear(MlFlags::LINE_DIRTY | MlFlags::ALLOCATED);
    }

    /// Take a data block: `mf_get` handed `hp` out, and it holds lines `low`
    /// through `high`. Nothing has changed in it yet, so neither it nor the
    /// index above it needs writing back.
    pub fn lock(&mut self, hp: *mut bhdr_T, low: linenr_T, high: linenr_T) {
        self.ml_locked = Some(LockedBlock {
            hp,
            low,
            high,
            lineadd: 0,
            dirty: false,
            moved: false,
        });
    }

    /// Give the block back, and answer what `mf_put` has to be told about
    /// it. The caller does the `mf_put`: this type knows nothing about the
    /// memfile.
    #[must_use]
    pub(crate) fn unlock(&mut self) -> Option<LockedBlock> {
        self.ml_locked.take()
    }

    /// Forget the block without giving it back -- for the one caller that
    /// freed it outright (`ml_free_data_block`) and still needs the lines it
    /// owes the pointer blocks above.
    #[must_use]
    pub fn forget_locked(&mut self) -> ::core::ffi::c_int {
        self.ml_locked.take().map_or(0, |locked| locked.lineadd)
    }

    /// Whether a block is being held.
    pub fn is_locked(&self) -> bool {
        self.ml_locked.is_some()
    }

    /// The held block itself, for the one caller that hands the same one
    /// back out again (`ml_find_line`'s already-locked answer). Null when
    /// nothing is held, which upstream's `ml_locked` was too.
    pub fn locked_hp(&self) -> *mut bhdr_T {
        self.ml_locked
            .as_ref()
            .map_or(::core::ptr::null_mut(), |locked| locked.hp)
    }

    /// The first line of the locked block. Every caller has just had one
    /// back from `ml_find_line`, so the "no block" answer never arises.
    pub fn locked_low(&self) -> linenr_T {
        self.ml_locked.as_ref().map_or(0, |locked| locked.low)
    }

    /// The last line of the locked block; see [`Self::locked_low`].
    pub fn locked_high(&self) -> linenr_T {
        self.ml_locked.as_ref().map_or(0, |locked| locked.high)
    }

    /// A line is going into (`delta` 1) or out of (-1) the locked block
    /// after all: it holds one more or one fewer line, and owes the pointer
    /// blocks above it the same correction.
    pub fn shift_locked(&mut self, delta: ::core::ffi::c_int) {
        if let Some(locked) = self.ml_locked.as_mut() {
            locked.lineadd += delta;
            locked.high += delta as linenr_T;
        }
    }

    /// Take the lines the locked block owes the pointer blocks above it,
    /// because the caller is about to correct them by hand.
    pub fn take_locked_lineadd(&mut self) -> ::core::ffi::c_int {
        match self.ml_locked.as_mut() {
            Some(locked) => core::mem::replace(&mut locked.lineadd, 0),
            None => 0,
        }
    }

    /// The locked block changed and has to be written back.
    pub fn locked_is_dirty(&mut self) {
        if let Some(locked) = self.ml_locked.as_mut() {
            locked.dirty = true;
        }
    }

    /// The locked block changed and its line positions moved with it, so the
    /// pointer block above needs correcting too.
    pub fn locked_has_moved(&mut self) {
        if let Some(locked) = self.ml_locked.as_mut() {
            locked.dirty = true;
            locked.moved = true;
        }
    }

    /// Record that `ml_line_ptr` now holds a *replacement* for the line it
    /// was read as: the block it came from is stale until the line is
    /// flushed, and the buffer is no longer the untouched empty one.
    pub fn line_was_replaced(&mut self) {
        self.ml_flags |= MlFlags::LINE_DIRTY;
        self.ml_flags.clear(MlFlags::EMPTY);
    }
}
