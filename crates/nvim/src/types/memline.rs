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

/// The chunk index: a run of between `MLCS_MINL` and `MLCS_MAXL`
/// consecutive lines with their total byte size, so `ml_find_line_or_offset`
/// can skip whole runs instead of visiting every block.
///
/// Upstream spelled it `ml_chunksize` (the array), `ml_numchunks` (its
/// capacity) and `ml_usedchunks` (its length, or **-1** for "a walk failed,
/// the counts are wrong, stop using it"). The capacity is the `Vec`'s
/// business; the length is the `Vec`'s length; and the two states that are
/// not a length -- never built, and switched off -- are their own flags,
/// because they are not the same thing: deleting the only chunk of a
/// one-chunk table leaves a *built* table with no entries.
///
/// All of the arithmetic lives here, in a file that forbids `unsafe`.
#[derive(Default)]
pub(crate) struct MlChunks {
    entries: Vec<chunksize_T>,
    /// `ml_chunksize != NULL`.
    built: bool,
    /// `ml_usedchunks == -1`.
    off: bool,
}

impl MlChunks {
    /// Whether a failed walk has switched the index off. It stays off until
    /// the memline is reopened.
    pub(crate) fn is_off(&self) -> bool {
        self.off
    }

    /// Switch it off: a tree walk could not find a line it had counted, so
    /// nothing in here can be trusted.
    pub(crate) fn switch_off(&mut self) {
        self.off = true;
    }

    /// Whether the table has been allocated at all.
    pub(crate) fn is_built(&self) -> bool {
        self.built
    }

    /// Build it, holding the one line a fresh memline has.
    pub(crate) fn build(&mut self) {
        self.entries.clear();
        self.entries.reserve(100);
        self.entries.push(chunksize_T {
            mlcs_numlines: 1,
            mlcs_totalsize: 1,
        });
        self.built = true;
    }

    /// Back to one chunk of one line `size` bytes long: the first line
    /// written into an empty buffer.
    pub(crate) fn reset_to_one(&mut self, size: ::core::ffi::c_int) {
        self.entries.truncate(1);
        self.entries[0] = chunksize_T {
            mlcs_numlines: 1,
            mlcs_totalsize: size,
        };
    }

    /// Forget the table and give its memory back, at `ml_close`.
    pub(crate) fn free(&mut self) {
        *self = Self::default();
    }

    /// How many chunks are live: upstream's `ml_usedchunks`.
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    /// Lines in chunk `at`.
    pub(crate) fn lines(&self, at: usize) -> ::core::ffi::c_int {
        self.entries[at].mlcs_numlines
    }

    /// Bytes in chunk `at`.
    pub(crate) fn size(&self, at: usize) -> ::core::ffi::c_int {
        self.entries[at].mlcs_totalsize
    }

    /// Add `delta` lines to chunk `at`.
    pub(crate) fn add_lines(&mut self, at: usize, delta: ::core::ffi::c_int) {
        self.entries[at].mlcs_numlines += delta;
    }

    /// Add `delta` bytes to chunk `at`.
    pub(crate) fn add_size(&mut self, at: usize, delta: ::core::ffi::c_int) {
        self.entries[at].mlcs_totalsize += delta;
    }

    /// Overwrite chunk `at`.
    pub(crate) fn set(&mut self, at: usize, lines: ::core::ffi::c_int, size: ::core::ffi::c_int) {
        self.entries[at] = chunksize_T {
            mlcs_numlines: lines,
            mlcs_totalsize: size,
        };
    }

    /// Start an empty chunk after the last one.
    pub(crate) fn push_empty(&mut self) {
        self.entries.push(chunksize_T {
            mlcs_numlines: 0,
            mlcs_totalsize: 0,
        });
    }

    /// Chunk `at` is about to be cut in two: put a copy of it next to
    /// itself, for the caller to divide the lines and bytes between.
    pub(crate) fn split_at(&mut self, at: usize) {
        self.entries.insert(at, self.entries[at]);
    }

    /// A line came out of chunk `at`, which may leave it and a neighbour
    /// short enough between them to be merged. `min_lines` is `MLCS_MINL`.
    ///
    /// The whole of upstream's `ml_updatechunk(ML_CHNK_DELLINE)` tail, and
    /// pure table arithmetic -- no block is read, so none of it has to be
    /// unsafe any more.
    pub(crate) fn delete_line(&mut self, at: usize, min_lines: ::core::ffi::c_int) {
        let mut at = at;
        self.add_lines(at, -1);

        if at + 1 < self.len() && self.lines(at) + self.lines(at + 1) <= min_lines {
            // Merge with the chunk after it instead: step onto that one, so
            // the collapse below folds it into this one.
            at += 1;
        } else if at == 0 && self.lines(0) <= 0 {
            // The first chunk emptied and there is nothing before it to
            // merge into, so drop it.
            self.entries.remove(0);
            return;
        } else if at == 0
            || (self.lines(at) > 10 && self.lines(at) + self.lines(at - 1) > min_lines)
        {
            return;
        }

        // Collapse this chunk into the one before it.
        let gone = self.entries[at];
        self.add_lines(at - 1, gone.mlcs_numlines);
        self.add_size(at - 1, gone.mlcs_totalsize);
        self.entries.remove(at);
    }
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

/// The one line `ml_get` last handed out, and whether the memline owns the
/// memory it is in.
///
/// A line read out of a data block is a pointer *into* that block; a line
/// `ml_replace` put here is the memline's own allocation, waiting for
/// `ml_flush_line` to write it back. Upstream told the two apart with
/// `ML_LINE_DIRTY` and `ML_ALLOCATED` in `ml_flags`, so the rule "free
/// `ml_line_ptr` iff one of these is set" lived in prose next to four loose
/// fields. It is the type now: the text cannot be set without saying which
/// kind it is, and the pointer only comes back out of [`Self::take_owned`],
/// which clears the ownership as it hands it over.
///
/// There is no `Drop`. `ml_get` hands `ptr` to every caller in the editor,
/// and the three places that free it (`ml_flush_line`, `ml_replace_buf_len`
/// and `ml_close`) each do so at a point they have chosen; a destructor
/// would claim an exclusive ownership the API does not have.
#[derive(Default)]
pub(crate) struct LineCache {
    /// `ml_line_lnum`: which line is cached, or 0 for none.
    lnum: linenr_T,
    /// `ml_line_ptr`: its text, NUL-terminated. Stale whenever `lnum` is 0.
    ptr: *mut ::core::ffi::c_char,
    /// `ml_line_textlen`: the text's length *including* the NUL that stands
    /// for the line break.
    textlen: colnr_T,
    /// `ml_line_offset`: the byte offset of the line's start in the buffer,
    /// remembered so a run of small edits to one line computes it once.
    /// Zero means "not worked out".
    offset: size_t,
    /// `ML_LINE_DIRTY`: the text is a *replacement* for what the block
    /// holds, and has to be written back before it is dropped.
    dirty: bool,
    /// `ML_ALLOCATED`: the text is the memline's own copy of what the block
    /// holds, taken because the block was about to be released.
    ///
    /// Nothing sets it. Upstream only does so under `ML_GET_ALLOC_LINES`, a
    /// build-time debugging switch that is off in every shipped build and
    /// was not ported; the field is kept because the *reading* half --
    /// "owned means free it" -- is real, and because a port of that switch
    /// would have nowhere else to put it.
    allocated: bool,
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
    /// The line `ml_get` last handed out.
    pub(crate) ml_line: LineCache,
    /// The data block the walk is holding, if it is holding one.
    pub(crate) ml_locked: Option<LockedBlock>,
    /// The chunk index that keeps a byte-offset lookup from walking every
    /// block.
    pub(crate) ml_chunks: MlChunks,
}

impl memline_T {
    /// A closed memline, which is what a fresh `buf_T` holds.
    ///
    /// A zeroed `buf_T` is a valid one everywhere *except* the owned
    /// collections in here -- an empty `Vec` holds a non-null dangling
    /// pointer, not a zero one -- so `alloc_unregistered_buffer` writes this
    /// over the zeroes.
    pub fn closed() -> Self {
        memline_T {
            ml_line_count: 0,
            ml_mfp: ::core::ptr::null_mut(),
            ml_stack: Vec::new(),
            ml_flags: MlFlags::NONE,
            ml_line: LineCache::default(),
            ml_locked: None,
            ml_chunks: MlChunks::default(),
        }
    }

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

    /// Which line is cached, or 0 for none: upstream's `ml_line_lnum`.
    pub fn cached_lnum(&self) -> linenr_T {
        self.ml_line.lnum
    }

    /// The cached line's text: upstream's `ml_line_ptr`. Stale unless
    /// [`Self::cached_lnum`] is non-zero.
    pub fn cached_text(&self) -> *mut ::core::ffi::c_char {
        self.ml_line.ptr
    }

    /// The cached line's length, NUL included: upstream's
    /// `ml_line_textlen`.
    pub fn cached_len(&self) -> colnr_T {
        self.ml_line.textlen
    }

    /// Say how long the cached line is without touching its text -- the two
    /// `ml_get` failure paths, which answer a static placeholder, and the
    /// callers that shorten the text in place.
    pub fn set_cached_len(&mut self, textlen: colnr_T) {
        self.ml_line.textlen = textlen;
    }

    /// Say which line the placeholder `ml_get` just answered stands for.
    pub fn set_cached_lnum(&mut self, lnum: linenr_T) {
        self.ml_line.lnum = lnum;
    }

    /// The cached line's byte offset in the buffer, or 0 for "not worked
    /// out": upstream's `ml_line_offset`.
    pub fn cached_offset(&self) -> size_t {
        self.ml_line.offset
    }

    /// Remember the cached line's byte offset.
    pub fn set_cached_offset(&mut self, offset: size_t) {
        self.ml_line.offset = offset;
    }

    /// Cache a line that lives *inside* the locked data block, so nothing
    /// here owns it.
    pub fn cache_block_line(
        &mut self,
        text: *mut ::core::ffi::c_char,
        textlen: colnr_T,
        lnum: linenr_T,
    ) {
        self.ml_line = LineCache {
            lnum,
            ptr: text,
            textlen,
            offset: self.ml_line.offset,
            dirty: false,
            allocated: false,
        };
    }

    /// Cache a *replacement* for line `lnum`, in memory the memline now
    /// owns: the block it came from is stale until the line is flushed, and
    /// the buffer is no longer the untouched empty one.
    pub fn cache_replacement(
        &mut self,
        text: *mut ::core::ffi::c_char,
        textlen: colnr_T,
        lnum: linenr_T,
    ) {
        self.ml_line = LineCache {
            lnum,
            ptr: text,
            textlen,
            offset: self.ml_line.offset,
            dirty: true,
            allocated: false,
        };
        self.ml_flags.clear(MlFlags::EMPTY);
    }

    /// Swap the cached line's text for `text`, which the memline now owns,
    /// keeping the line number. Answers the old text if it was owned, for
    /// the caller to free.
    #[must_use]
    pub fn swap_cached_text(
        &mut self,
        text: *mut ::core::ffi::c_char,
        textlen: colnr_T,
    ) -> Option<*mut ::core::ffi::c_char> {
        let old = self.take_owned();
        self.ml_line.ptr = text;
        self.ml_line.textlen = textlen;
        self.ml_line.dirty = true;
        old
    }

    /// Whether the cached text is a replacement that has to be written back:
    /// `ML_LINE_DIRTY`.
    pub fn line_is_dirty(&self) -> bool {
        self.ml_line.dirty
    }

    /// Whether the cached text is the memline's own allocation rather than a
    /// pointer into the locked block, so that dropping it means freeing it.
    ///
    /// Either kind says so: a rewritten line is always rebuilt into fresh
    /// memory, and a copied one was taken out of a block that went away.
    pub fn line_is_owned(&self) -> bool {
        self.ml_line.dirty || self.ml_line.allocated
    }

    /// Give up the cached text if the memline owns it, so the caller can
    /// free it. The ownership is cleared either way; the pointer itself
    /// stays cached, because callers still read it back.
    #[must_use]
    pub fn take_owned(&mut self) -> Option<*mut ::core::ffi::c_char> {
        debug_assert!(!self.line_is_owned() || self.ml_line.lnum != 0);
        let owned = self.line_is_owned().then_some(self.ml_line.ptr);
        self.ml_line.dirty = false;
        self.ml_line.allocated = false;
        owned
    }

    /// Forget that the memline owns the cached text -- the caller has freed
    /// it, or handed the ownership on.
    pub fn forget_line(&mut self) {
        let _ = self.take_owned();
    }

    /// Nothing is cached any more.
    pub fn clear_cache(&mut self) {
        self.ml_line = LineCache::default();
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

    /// Record that the cached text is a *replacement* for the line it was
    /// read as, and that the buffer is no longer the untouched empty one.
    pub fn line_was_replaced(&mut self) {
        self.ml_line.dirty = true;
        self.ml_flags.clear(MlFlags::EMPTY);
    }
}
