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
use crate::memfile::BlockId;
use crate::memline::MlFlags;

#[derive(Copy, Clone)]
pub struct ChunkSize {
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
    entries: Vec<ChunkSize>,
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
        self.entries.push(ChunkSize {
            mlcs_numlines: 1,
            mlcs_totalsize: 1,
        });
        self.built = true;
    }

    /// Back to one chunk of one line `size` bytes long: the first line
    /// written into an empty buffer.
    pub(crate) fn reset_to_one(&mut self, size: ::core::ffi::c_int) {
        self.entries.truncate(1);
        self.entries[0] = ChunkSize {
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
        self.entries[at] = ChunkSize {
            mlcs_numlines: lines,
            mlcs_totalsize: size,
        };
    }

    /// Start an empty chunk after the last one.
    pub(crate) fn push_empty(&mut self) {
        self.entries.push(ChunkSize {
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
/// The memfile hands a block out (`MemFile::get`) and takes it back
/// (`MemFile::put`), and the two answers `put` wants -- whether the block
/// changed, and whether the line *positions* in it changed -- are decided
/// over the whole time it is held. Upstream kept the six facts in four
/// `MemLine` fields plus two bits of `ml_flags`; they are one value now, and
/// `ml_locked` is `Some` exactly while a block is out.
///
/// The block is held by its [`BlockId`], never by its bytes: a memline
/// call that holds a block may reach the memfile again (another `get`, a
/// split's `new_block`, a sync) and the handle is what survives that -- the
/// memfile neither moves nor drops a locked block, and a handle to one that
/// is gone is refused rather than read.
///
/// It is deliberately **not** a `Drop` guard. The block is held *across*
/// calls -- a run of reads keeps the same one locked, which is the whole
/// point of it -- and by the time a `Buffer` is dropped `ml_close` has
/// closed the memfile. What makes it a guard is that there is exactly one
/// acquire (`ml_find_line`'s walk, through [`MemLine::lock`]) and exactly one
/// release ([`MemLine::unlock`]), and that the release arguments travel with
/// the block instead of in flag bits that outlive it.
pub(crate) struct LockedBlock {
    /// What `MemFile::get` handed out.
    pub id: BlockId,
    /// The first line the block holds.
    pub low: LineNr,
    /// The last line it holds, *after* the insert or delete the walk that
    /// locked it is making room for.
    pub high: LineNr,
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

/// Where the line `ml_get` last handed out lives.
#[derive(Default)]
pub(crate) enum CachedText {
    /// Nothing is cached.
    #[default]
    None,
    /// In a data block: `start` bytes into block `id`'s page. Not
    /// necessarily the block that is locked now -- a walk for a line
    /// elsewhere can lock another one under a cached line -- which is why
    /// the block is named.
    ///
    /// `text` is the same place as the pointer form answers it, taken when
    /// the line was unpacked: a page never moves while its block is in
    /// memory, so handing it out again costs no lookup. Only
    /// [`MemLine::cached_text`] reads it, and only to answer it; the bytes
    /// themselves are always reached through `id`.
    Block {
        id: BlockId,
        start: u32,
        text: *mut ::core::ffi::c_char,
    },
    /// A replacement the memline owns, waiting for `ml_flush_line` to write
    /// it back: the bytes, the NUL among them.
    Owned(Vec<u8>),
    /// One of the two literals a failed read answers.
    Placeholder(&'static ::core::ffi::CStr),
}

/// The one line `ml_get` last handed out, and whether the memline owns the
/// memory it is in.
///
/// A line read out of a data block lives *in* that block; a line
/// `ml_replace` put here is the memline's own, waiting for `ml_flush_line`
/// to write it back. Upstream told the two apart with `ML_LINE_DIRTY` and
/// `ML_ALLOCATED` in `ml_flags`, so the rule "free `ml_line_ptr` iff one of
/// these is set" lived in prose next to four loose fields. It is the type
/// now: an owned line is a `Vec` in [`CachedText::Owned`], dropped when it
/// is replaced or written back, and a line in a block is a place, not a
/// pointer.
///
/// (Upstream's `ML_ALLOCATED` -- a copy taken because the block was about to
/// be released -- is only ever set under `ML_GET_ALLOC_LINES`, a build-time
/// debugging switch that is off in every shipped build; it was not ported.)
#[derive(Default)]
pub(crate) struct LineCache {
    /// `ml_line_lnum`: which line is cached, or 0 for none.
    lnum: LineNr,
    /// `ml_line_ptr`: where its text is. Stale whenever `lnum` is 0.
    text: CachedText,
    /// `ml_line_textlen`: the text's length *including* the NUL that stands
    /// for the line break.
    textlen: ColNr,
    /// `ml_line_offset`: the byte offset of the line's start in the buffer,
    /// remembered so a run of small edits to one line computes it once.
    /// Zero means "not worked out".
    offset: size_t,
}

#[derive(Copy, Clone)]
pub struct InfoPtr {
    pub ip_bnum: BlockNr,
    pub ip_low: LineNr,
    pub ip_high: LineNr,
    pub ip_index: ::core::ffi::c_int,
}
pub struct MemLine {
    pub ml_line_count: LineNr,
    /// The memfile holding the lines, or `None` while the buffer is not
    /// loaded. The buffer owns it: [`crate::memline::ml_close`] is where it
    /// ends.
    pub ml_mfp: Option<Box<MemFile>>,
    /// The path from the root of the block tree down to the block
    /// [`MemLine::ml_locked`] names, one entry per pointer block, the
    /// root first. Upstream grew this by hand (`ml_stack` plus
    /// `ml_stack_top` and `ml_stack_size`); the length *is* the top, and
    /// the capacity is nobody's business.
    ///
    /// Truncating it to zero is how every failure path says "the tree
    /// moved under me"; the entries are plain data and cost nothing to
    /// drop.
    pub ml_stack: Vec<InfoPtr>,
    pub ml_flags: MlFlags,
    /// The line `ml_get` last handed out.
    pub(crate) ml_line: LineCache,
    /// The data block the walk is holding, if it is holding one.
    pub(crate) ml_locked: Option<LockedBlock>,
    /// The chunk index that keeps a byte-offset lookup from walking every
    /// block.
    pub(crate) ml_chunks: MlChunks,
}

impl MemLine {
    /// A closed memline, which is what a fresh `Buffer` holds.
    ///
    /// A zeroed `Buffer` is a valid one everywhere *except* the owned
    /// collections in here -- an empty `Vec` holds a non-null dangling
    /// pointer, not a zero one -- so `alloc_unregistered_buffer` writes this
    /// over the zeroes.
    pub fn closed() -> Self {
        MemLine {
            ml_line_count: 0,
            ml_mfp: None,
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
    pub fn stack_at(&self, idx: usize) -> InfoPtr {
        self.ml_stack[idx]
    }

    /// Overwrite entry `idx`.
    pub fn stack_set(&mut self, idx: usize, entry: InfoPtr) {
        self.ml_stack[idx] = entry;
    }

    /// Record which entry of the pointer block at `idx` the walk went down.
    pub fn stack_set_index(&mut self, idx: usize, index: ::core::ffi::c_int) {
        self.ml_stack[idx].ip_index = index;
    }

    /// Correct the last line entry `idx` covers, after lines were added to
    /// or removed from the block below it.
    pub fn stack_add_high(&mut self, idx: usize, count: LineNr) {
        self.ml_stack[idx].ip_high += count;
    }

    /// Push a blank entry and answer its index. Every caller fills it in.
    pub fn stack_push(&mut self) -> usize {
        self.ml_stack.push(InfoPtr {
            ip_bnum: 0,
            ip_low: 0,
            ip_high: 0,
            ip_index: 0,
        });
        self.ml_stack.len() - 1
    }

    /// Drop the deepest entry, if there is one.
    pub fn stack_pop(&mut self) -> Option<InfoPtr> {
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

    /// Whether the memline is open: the buffer is loaded.
    pub fn is_open(&self) -> bool {
        self.ml_mfp.is_some()
    }

    /// The memfile, which an open memline has.
    ///
    /// # Panics
    /// When the memline is closed: every caller has just checked.
    #[inline]
    pub(crate) fn mfp(&mut self) -> &mut MemFile {
        self.ml_mfp.as_deref_mut().expect("an open memline")
    }

    /// Which line is cached, or 0 for none: upstream's `ml_line_lnum`.
    pub fn cached_lnum(&self) -> LineNr {
        self.ml_line.lnum
    }

    /// The cached line's text, NUL-terminated: upstream's `ml_line_ptr`.
    /// Null when nothing is cached.
    ///
    /// The pointer form, for the callers that still hold a line as one; it
    /// is good until the next call into the memline.
    pub fn cached_text(&mut self) -> *mut ::core::ffi::c_char {
        match &mut self.ml_line.text {
            CachedText::None => ::core::ptr::null_mut(),
            CachedText::Owned(text) => text.as_mut_ptr().cast(),
            CachedText::Placeholder(text) => text.as_ptr().cast_mut(),
            CachedText::Block { text, .. } => *text,
        }
    }

    /// The cached line's bytes, its NUL left out. Empty when nothing is
    /// cached.
    pub(crate) fn cached_bytes(&self) -> &[u8] {
        let len = usize::try_from(self.ml_line.textlen - 1).unwrap_or(0);
        match &self.ml_line.text {
            CachedText::None => &[],
            CachedText::Owned(text) => &text[..len],
            CachedText::Placeholder(text) => text.to_bytes(),
            &CachedText::Block { id, start, .. } => {
                let page = self.ml_mfp.as_deref().expect("an open memline").page(id);
                &page[start as usize..start as usize + len]
            }
        }
    }

    /// [`Self::cached_bytes`] and the NUL after them.
    pub(crate) fn cached_bytes_with_nul(&self) -> &[u8] {
        let len = usize::try_from(self.ml_line.textlen).unwrap_or(0).max(1);
        match &self.ml_line.text {
            CachedText::None => b"\0",
            CachedText::Owned(text) => &text[..len],
            CachedText::Placeholder(text) => text.to_bytes_with_nul(),
            &CachedText::Block { id, start, .. } => {
                let page = self.ml_mfp.as_deref().expect("an open memline").page(id);
                &page[start as usize..start as usize + len]
            }
        }
    }

    /// [`Self::cached_bytes`], writable: only the bytes already there.
    pub(crate) fn cached_bytes_mut(&mut self) -> &mut [u8] {
        let len = usize::try_from(self.ml_line.textlen - 1).unwrap_or(0);
        match &mut self.ml_line.text {
            CachedText::None | CachedText::Placeholder(_) => &mut [],
            CachedText::Owned(text) => &mut text[..len],
            &mut CachedText::Block { id, start, .. } => {
                let page = self
                    .ml_mfp
                    .as_deref_mut()
                    .expect("an open memline")
                    .page_mut(id);
                &mut page[start as usize..start as usize + len]
            }
        }
    }

    /// The cached line's length, NUL included: upstream's
    /// `ml_line_textlen`.
    pub fn cached_len(&self) -> ColNr {
        self.ml_line.textlen
    }

    /// Say how long the cached line is without touching its text -- the
    /// callers that shorten an owned line in place.
    pub fn set_cached_len(&mut self, textlen: ColNr) {
        self.ml_line.textlen = textlen;
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

    /// Cache a line that lives *inside* data block `id`, `start` bytes in,
    /// so nothing here owns it.
    pub(crate) fn cache_block_line(
        &mut self,
        id: BlockId,
        start: u32,
        textlen: ColNr,
        lnum: LineNr,
    ) {
        let page = self
            .ml_mfp
            .as_deref_mut()
            .expect("an open memline")
            .page_mut(id);
        let text = page[start as usize..].as_mut_ptr().cast();
        self.ml_line.lnum = lnum;
        self.ml_line.text = CachedText::Block { id, start, text };
        self.ml_line.textlen = textlen;
    }

    /// Answer one of the two literals a failed read hands back, for line
    /// `lnum`.
    pub(crate) fn cache_placeholder(&mut self, text: &'static ::core::ffi::CStr, lnum: LineNr) {
        self.ml_line.lnum = lnum;
        self.ml_line.text = CachedText::Placeholder(text);
        self.ml_line.textlen = ColNr::try_from(text.count_bytes() + 1).unwrap_or(ColNr::MAX);
    }

    /// Cache a *replacement* for line `lnum`, which the memline now owns:
    /// `text` holds the line and its NUL. The block it came from is stale
    /// until the line is flushed, and the buffer is no longer the untouched
    /// empty one.
    pub(crate) fn cache_replacement(&mut self, text: Vec<u8>, lnum: LineNr) {
        let textlen = ColNr::try_from(text.len()).expect("a line shorter than INT_MAX");
        self.ml_line.lnum = lnum;
        self.ml_line.text = CachedText::Owned(text);
        self.ml_line.textlen = textlen;
        self.ml_flags.clear(MlFlags::EMPTY);
    }

    /// Say that line `lnum`, `textlen` bytes long with its NUL, is about to
    /// be written straight into its block with no copy cached: the chunk
    /// index reads the length while it happens.
    pub(crate) fn cache_pending(&mut self, lnum: LineNr, textlen: ColNr) {
        self.ml_line.lnum = lnum;
        self.ml_line.text = CachedText::None;
        self.ml_line.textlen = textlen;
        self.ml_flags.clear(MlFlags::EMPTY);
    }

    /// Swap the cached line's text for `text` (the line and its NUL), which
    /// the memline now owns, keeping the line number.
    pub fn swap_cached_text(&mut self, text: Vec<u8>) {
        let textlen = ColNr::try_from(text.len()).expect("a line shorter than INT_MAX");
        self.ml_line.text = CachedText::Owned(text);
        self.ml_line.textlen = textlen;
        self.ml_flags.clear(MlFlags::EMPTY);
    }

    /// Whether the cached text is a replacement that has to be written back:
    /// `ML_LINE_DIRTY`.
    pub fn line_is_dirty(&self) -> bool {
        matches!(self.ml_line.text, CachedText::Owned(_))
    }

    /// Take the replacement waiting to be written back, if there is one,
    /// leaving the line cached with no text.
    pub(crate) fn take_replacement(&mut self) -> Option<Vec<u8>> {
        match core::mem::take(&mut self.ml_line.text) {
            CachedText::Owned(text) => Some(text),
            other => {
                self.ml_line.text = other;
                None
            }
        }
    }

    /// Forget the cached line, and drop its text if the memline owned it.
    pub fn forget_line(&mut self) {
        self.ml_line.text = CachedText::None;
    }

    /// Nothing is cached any more.
    pub fn clear_cache(&mut self) {
        self.ml_line = LineCache::default();
    }

    /// Take a data block: `MemFile::get` handed `id` out, and it holds lines
    /// `low` through `high`. Nothing has changed in it yet, so neither it nor
    /// the index above it needs writing back.
    pub(crate) fn lock(&mut self, id: BlockId, low: LineNr, high: LineNr) {
        self.ml_locked = Some(LockedBlock {
            id,
            low,
            high,
            lineadd: 0,
            dirty: false,
            moved: false,
        });
    }

    /// Give the block back, and answer what `MemFile::put` has to be told
    /// about it. The caller does the `put`.
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
    /// back out again (`ml_find_line`'s already-locked answer).
    pub(crate) fn locked_id(&self) -> Option<BlockId> {
        self.ml_locked.as_ref().map(|locked| locked.id)
    }

    /// The first line of the locked block. Every caller has just had one
    /// back from `ml_find_line`, so the "no block" answer never arises.
    pub fn locked_low(&self) -> LineNr {
        self.ml_locked.as_ref().map_or(0, |locked| locked.low)
    }

    /// The last line of the locked block; see [`Self::locked_low`].
    pub fn locked_high(&self) -> LineNr {
        self.ml_locked.as_ref().map_or(0, |locked| locked.high)
    }

    /// A line is going into (`delta` 1) or out of (-1) the locked block
    /// after all: it holds one more or one fewer line, and owes the pointer
    /// blocks above it the same correction.
    pub fn shift_locked(&mut self, delta: ::core::ffi::c_int) {
        if let Some(locked) = self.ml_locked.as_mut() {
            locked.lineadd += delta;
            locked.high += delta as LineNr;
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
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `InfoPtr` is plain data; only the four numbers matter.
    fn entry(bnum: BlockNr, low: LineNr, high: LineNr) -> InfoPtr {
        InfoPtr {
            ip_bnum: bnum,
            ip_low: low,
            ip_high: high,
            ip_index: -1,
        }
    }

    fn with_stack(entries: &[InfoPtr]) -> MemLine {
        let mut ml = MemLine::closed();
        for &e in entries {
            let at = ml.stack_push();
            ml.stack_set(at, e);
        }
        ml
    }

    #[test]
    fn stack_push_answers_the_new_top() {
        let mut ml = MemLine::closed();
        assert_eq!(ml.stack_len(), 0);
        assert_eq!(ml.stack_push(), 0);
        assert_eq!(ml.stack_push(), 1);
        assert_eq!(ml.stack_len(), 2);
    }

    #[test]
    fn stack_entries_survive_the_growth_a_push_causes() {
        // The hazard the C carried a comment about: an entry read before a
        // push must still be readable after it. Taking entries by value is
        // what makes that true, and Miri is what proves the old pointer is
        // not being kept.
        let mut ml = MemLine::closed();
        for i in 0..64i32 {
            let at = ml.stack_push();
            ml.stack_set(at, entry(BlockNr::from(i), i, i + 1));
        }
        for i in 0..64i32 {
            let seen = ml.stack_at(usize::try_from(i).unwrap());
            assert_eq!(seen.ip_bnum, BlockNr::from(i));
            assert_eq!(seen.ip_high, i + 1);
        }
    }

    #[test]
    fn stack_truncate_keeps_the_entry_it_stops_at() {
        // `ml_pointer_add_entry` and `ml_free_data_block` both cut the stack
        // to `stack_idx + 1` and then correct entry `stack_idx` -- which is
        // the entry upstream read from *above* `ml_stack_top`.
        let mut ml = with_stack(&[entry(1, 1, 100), entry(2, 1, 50), entry(3, 1, 20)]);
        ml.stack_truncate(2);
        assert_eq!(ml.stack_len(), 2);
        ml.stack_add_high(1, 5);
        assert_eq!(ml.stack_at(1).ip_high, 55);
        ml.stack_set_index(1, 7);
        assert_eq!(ml.stack_at(1).ip_index, 7);
    }

    #[test]
    fn stack_pop_answers_none_at_the_bottom() {
        let mut ml = with_stack(&[entry(1, 1, 9)]);
        assert_eq!(ml.stack_pop().map(|e| e.ip_bnum), Some(1));
        assert!(ml.stack_pop().is_none());
        ml.stack_clear();
        ml.stack_free();
        assert_eq!(ml.stack_len(), 0);
    }

    #[test]
    fn locking_a_block_starts_it_clean() {
        let mut ml = MemLine::closed();
        assert!(!ml.is_locked());
        assert!(ml.locked_id().is_none());

        let id = BlockId::unbacked();
        ml.lock(id, 10, 20);
        assert!(ml.is_locked());
        assert_eq!(ml.locked_id(), Some(id));
        assert_eq!((ml.locked_low(), ml.locked_high()), (10, 20));

        let locked = ml.unlock().expect("just locked");
        assert!(!locked.dirty);
        assert!(!locked.moved);
        assert_eq!(locked.lineadd, 0);
        assert!(!ml.is_locked());
    }

    #[test]
    fn shifting_a_locked_block_moves_its_end_and_its_debt() {
        let mut ml = MemLine::closed();
        ml.lock(BlockId::unbacked(), 1, 5);
        ml.shift_locked(1);
        ml.shift_locked(1);
        ml.shift_locked(-1);
        assert_eq!(ml.locked_high(), 6);
        assert_eq!(ml.take_locked_lineadd(), 1);
        // Taken means paid: it must not be paid twice.
        assert_eq!(ml.take_locked_lineadd(), 0);
    }

    #[test]
    fn marking_a_block_with_nothing_locked_does_nothing() {
        // `ml_get(will_change)` can reach this with the cached line dirty
        // and no block held; upstream set bits that the next lock cleared.
        let mut ml = MemLine::closed();
        ml.locked_has_moved();
        ml.locked_is_dirty();
        ml.shift_locked(3);
        assert!(ml.unlock().is_none());
        assert_eq!(ml.forget_locked(), 0);
    }

    #[test]
    fn forgetting_a_locked_block_still_answers_its_debt() {
        let mut ml = MemLine::closed();
        ml.lock(BlockId::unbacked(), 1, 5);
        ml.shift_locked(-1);
        ml.locked_has_moved();
        assert_eq!(ml.forget_locked(), -1);
        assert!(!ml.is_locked());
    }

    #[test]
    fn a_fresh_chunk_index_is_unbuilt_and_on() {
        let mut chunks = MlChunks::default();
        assert!(!chunks.is_built());
        assert!(!chunks.is_off());
        chunks.build();
        assert!(chunks.is_built());
        assert_eq!(chunks.len(), 1);
        assert_eq!((chunks.lines(0), chunks.size(0)), (1, 1));
        chunks.switch_off();
        assert!(chunks.is_off());
        chunks.free();
        assert!(!chunks.is_built());
        assert!(!chunks.is_off());
    }

    #[test]
    fn splitting_a_chunk_duplicates_it_for_the_caller_to_divide() {
        let mut chunks = MlChunks::default();
        chunks.build();
        chunks.set(0, 800, 8000);
        chunks.split_at(0);
        assert_eq!(chunks.len(), 2);
        assert_eq!((chunks.lines(1), chunks.size(1)), (800, 8000));
        chunks.add_lines(1, -400);
        chunks.add_size(1, -4000);
        chunks.set(0, 400, 4000);
        assert_eq!((chunks.lines(0), chunks.size(0)), (400, 4000));
        assert_eq!((chunks.lines(1), chunks.size(1)), (400, 4000));
    }

    #[test]
    fn deleting_the_last_line_of_the_only_chunk_drops_it() {
        let mut chunks = MlChunks::default();
        chunks.build();
        chunks.set(0, 1, 5);
        chunks.delete_line(0, 400);
        assert_eq!(chunks.len(), 0);
        // Built, but with nothing in it -- the state a length alone cannot
        // tell from "never built".
        assert!(chunks.is_built());
    }

    #[test]
    fn a_short_chunk_merges_into_the_one_after_it() {
        let mut chunks = MlChunks::default();
        chunks.build();
        chunks.set(0, 100, 1000);
        chunks.push_empty();
        chunks.set(1, 100, 1000);
        chunks.push_empty();
        chunks.set(2, 700, 7000);
        // 99 + 100 <= 400, so chunk 0 and chunk 1 become one. The byte
        // count is the caller's business -- `ml_updatechunk` has already
        // taken the deleted line's bytes off -- so the two just add up.
        chunks.delete_line(0, 400);
        assert_eq!(chunks.len(), 2);
        assert_eq!((chunks.lines(0), chunks.size(0)), (199, 2000));
        assert_eq!((chunks.lines(1), chunks.size(1)), (700, 7000));
    }

    #[test]
    fn a_long_chunk_keeps_to_itself() {
        let mut chunks = MlChunks::default();
        chunks.build();
        chunks.set(0, 500, 5000);
        chunks.push_empty();
        chunks.set(1, 500, 5000);
        chunks.delete_line(1, 400);
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks.lines(1), 499);
    }

    #[test]
    fn a_block_line_is_not_owned_and_a_replacement_is() {
        let mut ml = MemLine::closed();
        ml.ml_mfp = Some(MemFile::in_memory());
        let id = ml.mfp().new_block(false, 1);
        ml.mfp().page_mut(id)[100..106].copy_from_slice(b"hello\0");

        ml.cache_block_line(id, 100, 6, 3);
        assert_eq!(ml.cached_lnum(), 3);
        assert_eq!(ml.cached_bytes(), b"hello");
        assert_eq!(ml.cached_len(), 6);
        assert!(!ml.line_is_dirty());
        assert!(ml.take_replacement().is_none());
        // The pointer form reads the same bytes, in the page.
        let text = ml.cached_text();
        assert_eq!(text.cast::<u8>(), ml.mfp().page_mut(id)[100..].as_mut_ptr());

        ml.cache_replacement(b"bye\0".to_vec(), 3);
        assert!(ml.line_is_dirty());
        assert_eq!(ml.cached_bytes(), b"bye");
        assert_eq!(ml.take_replacement().as_deref(), Some(&b"bye\0"[..]));
        // Handed over once only.
        assert!(ml.take_replacement().is_none());
    }

    #[test]
    fn an_owned_line_can_be_shortened_and_swapped() {
        let mut ml = MemLine::closed();
        ml.cache_replacement(b"one two\0".to_vec(), 1);
        ml.cached_bytes_mut()[3] = b'_';
        ml.set_cached_len(4);
        assert_eq!(ml.cached_bytes(), b"one");
        ml.swap_cached_text(b"two\0".to_vec());
        assert_eq!(ml.cached_bytes(), b"two");
        assert_eq!(ml.cached_lnum(), 1);
        assert!(ml.line_is_dirty());
    }

    #[test]
    fn a_placeholder_reads_as_itself() {
        let mut ml = MemLine::closed();
        ml.cache_placeholder(c"???", 9);
        assert_eq!(ml.cached_bytes(), b"???");
        assert_eq!(ml.cached_len(), 4);
        assert!(!ml.line_is_dirty());
        assert!(ml.cached_bytes_mut().is_empty());
    }

    #[test]
    fn clearing_the_cache_forgets_the_line_and_its_offset() {
        let mut ml = MemLine::closed();
        ml.cache_replacement(b"x\0".to_vec(), 9);
        ml.set_cached_offset(64);
        assert_eq!(ml.cached_offset(), 64);
        ml.clear_cache();
        assert_eq!(ml.cached_lnum(), 0);
        assert_eq!(ml.cached_offset(), 0);
        assert!(!ml.line_is_dirty());
        assert!(ml.cached_text().is_null());
    }
}
