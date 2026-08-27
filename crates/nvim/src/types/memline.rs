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
    pub ml_locked: *mut bhdr_T,
    pub ml_locked_low: linenr_T,
    pub ml_locked_high: linenr_T,
    pub ml_locked_lineadd: ::core::ffi::c_int,
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

    /// A freshly locked data block: nothing has changed in it yet, so
    /// neither it nor the index above it needs writing back.
    pub fn block_is_clean(&mut self) {
        self.ml_flags
            .clear(MlFlags::LOCKED_DIRTY | MlFlags::LOCKED_POS);
    }

    /// Record that `ml_line_ptr` now holds a *replacement* for the line it
    /// was read as: the block it came from is stale until the line is
    /// flushed, and the buffer is no longer the untouched empty one.
    pub fn line_was_replaced(&mut self) {
        self.ml_flags |= MlFlags::LINE_DIRTY;
        self.ml_flags.clear(MlFlags::EMPTY);
    }
}
