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
    pub ml_stack: *mut infoptr_T,
    pub ml_stack_top: ::core::ffi::c_int,
    pub ml_stack_size: ::core::ffi::c_int,
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
