//! The line API the rest of the editor calls.
//!
//! Everything here is a thin front for the tree in
//! [`tree`](super::tree) and [`edit`](super::edit): flush whatever line
//! `ml_replace` left pending, check the line number, dispatch. The one
//! thing that lives here in its own right is the `DB_MARKED` bit, which
//! `:global` uses to remember which lines it still has to visit.
//!
//! # Reading a line
//!
//! [`Lines`] is the way, and [`Buf::lines`] the way to one:
//!
//! ```ignore
//! let mut lines = buffer.lines();
//! let text = lines.line(lnum); // a LineText: derefs to the line without its NUL
//! ```
//!
//! A single read fits in one expression — `buffer.lines().line(lnum)`, whose
//! temporary handle lives to the end of the statement; a walk binds the
//! handle, and every line it hands out borrows it.
//!
//! [`Lines`]'s own documentation carries the borrow story — how long the
//! line is valid, what refuses while it is out, and what to do about the
//! two shapes the borrow cannot express. In one sentence: the line is a
//! guard ([`LineText`]) that keeps the cache as it is while it lives, so the
//! next line through the same handle does not compile and any other walk
//! of the memline -- another line, an `ml_*` mutation, a redraw that reads
//! one -- panics until it is dropped.
//!
//! # No pointer form
//!
//! Upstream's `ml_get`, `ml_get_buf` and `ml_get_buf_mut` answered the
//! line's *address*, good until the next memline call; they are gone, and
//! with them the page address the line cache kept beside the block id.
//! What is left of the family answers a length ([`ml_get_len`],
//! [`ml_get_buf_len`]) or a slice ([`Lines`]), and a caller that has to hold
//! text across a call that re-enters the editor takes
//! [`Lines::line_copy`].

#![forbid(unsafe_code)]

use super::*;
use crate::cstr;
use crate::fileio::ReadFlags;
use crate::mbyte::Codepoint;
use crate::memfile::MemPin;
use crate::memory::XString;
use crate::types::Failed;
use crate::winlayer::{Buf, MlRef};
pub use crate::winlayer::{LineText, LineTextMut};
use core::cell::OnceCell;
use core::ffi::{CStr, c_int};

/// A buffer's one-line cache, borrowed.
///
/// The memline keeps exactly one line unpacked per buffer: the line in the
/// data block holding it, or the replacement a pending `ml_replace` left.
/// Reading a *different* line flushes that pending write, releases the
/// locked block and locks another, so the bytes the previous read answered
/// may have moved or been freed. Upstream's rule is "use the line before you
/// ask for another one", and until now it was written down nowhere.
///
/// This is where it is written down, and all of it is held. Half by the
/// compiler: [`line`](Self::line) borrows the handle **mutably** and the
/// [`LineText`] it answers keeps that borrow alive, so holding one line
/// while asking for the next through the same handle does not compile.
///
/// The other half is a call *out* of here that reads another line behind
/// the handle's back — another `Lines`, `ml_replace`, `ml_append`,
/// `ml_delete`, or anything that redraws or runs user code. None of them
/// takes a borrow this handle could conflict with, so the guard holds it at
/// run time: while a [`LineText`] lives, its memfile counts it out and every
/// walk of the memline panics -- the cache cannot be refilled or the line
/// changed under it. Reads of the *same* line (it is cached already), of
/// the line count, of the rest of the buffer, and edits of the rest of the
/// buffer go on. The rule left to the caller is the one that keeps that
/// panic away: **drop the line before the call**. A guard lives to the end
/// of its scope, not to its last use, so a walk that re-enters the editor
/// takes the line, does its indexing, drops it (`drop(line)` or a block),
/// and calls; a walk that has to hold text across the call takes
/// [`line_copy`](Self::line_copy).
///
/// A caller that genuinely needs two lines of the *same* buffer at once —
/// `:diffget`'s comparison, `'foldmarker'`'s start-and-end scan — is asking
/// for something the cache cannot give, and copies one of them. That is not
/// a limitation of this type; it is the memline's, made visible.
///
/// The lines are the line's bytes **without** the terminating NUL, which is
/// the length `ml_get_len` reports and the length every caller wants. A
/// memline line stores a NUL byte as an `NL`, so the bytes may hold either
/// and never hold a terminator — which is exactly why the answer is `&[u8]`
/// and not `&CStr` ([`crate::cstr`] § the parameter convention). Walks that
/// used to step one past the last character and read the terminator want
/// [`crate::cstr::byte_at`], which answers `NUL` past the end.
///
#[derive(Debug)]
pub struct Lines(Buf);

impl Lines {
    /// Borrow the current buffer's line cache.
    pub fn current() -> Self {
        Lines(Buf::current())
    }

    /// Borrow `buffer`'s line cache.
    pub fn in_buffer(buffer: Buf) -> Self {
        Lines(buffer)
    }

    /// The buffer whose cache this is.
    pub fn buffer(&self) -> Buf {
        self.0
    }

    /// How many lines the buffer has.
    pub fn count(&self) -> LineNr {
        self.0.line_count()
    }

    /// Line `lnum`, without its NUL.
    ///
    /// Out-of-range line numbers answer the `???` placeholder, after the
    /// E315 complaint upstream's `ml_get` gives.
    #[inline(always)]
    #[track_caller]
    pub fn line(&mut self, lnum: LineNr) -> LineText<'_> {
        match self.0.lend_line_if_cached(lnum, false) {
            Some(text) => text,
            None => line_by_walk(self.0, lnum),
        }
    }

    /// Line `lnum`, writable in place.
    ///
    /// The bytes already there can be rewritten and nothing else, which is
    /// what a slice of the line's own length says; the old text is booked as
    /// deleted. Use `ml_replace` to change a line's length.
    pub fn line_mut(&mut self, lnum: LineNr) -> LineTextMut<'_> {
        fetch_line(self.0, lnum, true);
        self.0.lend_line_mut(self.0.ml())
    }

    /// Overwrite byte `at` of line `lnum` in place, answering the byte that
    /// was there -- **without** booking it as a change: no dirty block and no
    /// deleted-bytes accounting, which [`line_mut`](Self::line_mut) does.
    ///
    /// Only for a probe that puts the old byte back before anything else can
    /// see the line ('cinkeys'' `::` test blanks a colon while the label
    /// recognisers read the line, as upstream writes through `ml_get`'s
    /// pointer). Answers NUL, writing nothing, when `at` is past the line.
    pub fn swap_byte_unbooked(&mut self, lnum: LineNr, at: usize, byte: u8) -> u8 {
        fetch_line(self.0, lnum, false);
        match self.0.ml().cached_bytes_mut().get_mut(at) {
            Some(slot) => core::mem::replace(slot, byte),
            None => 0,
        }
    }

    /// How long line `lnum` is, without its NUL.
    ///
    /// [`ml_get_len`] as a method, and the answer callers want when they are
    /// about to name a column rather than read text — so it stays a `ColNr`
    /// and no caller has to cast a `usize` back down.
    pub fn line_len(&mut self, lnum: LineNr) -> ColNr {
        ColNr::try_from(self.line(lnum).len()).unwrap_or(ColNr::MAX)
    }

    /// Line `lnum` as a C string, from byte `at` on.
    ///
    /// A memline line holds no interior NUL — it stores one as an `NL` — so
    /// the string is the line's bytes from `at` to its end, and the shape the
    /// callers that still hand text to a C-string reader want, `vim_regexec`
    /// above all. `at` may be the line's length, which answers the empty
    /// string.
    ///
    /// # Panics
    /// When `at` is past the end of the line.
    #[track_caller]
    pub fn line_cstr(&mut self, lnum: LineNr, at: usize) -> LineText<'_, CStr> {
        let bytes = match self.0.lend_line_if_cached(lnum, true) {
            Some(text) => text,
            None => self.0.lend_line(fetch_line_lent(self.0, lnum), true),
        };
        assert!(at < bytes.len(), "column past the end of the line");
        bytes.map(|bytes| cstr::in_bytes(&bytes[at..]))
    }

    /// Line `lnum`, copied out of the cache.
    ///
    /// The escape hatch for the two shapes the borrow cannot express: a
    /// second line of the same buffer held at the same time, and text held
    /// across a call that re-enters the editor. Both are real needs and both
    /// are a copy in C as well — upstream spells them `xstrdup(ml_get(…))`,
    /// which is why a [`LineCopy`] carries the terminator that spelling did.
    pub fn line_copy(&mut self, lnum: LineNr) -> LineCopy {
        let line = self.line(lnum);
        let mut bytes = Vec::with_capacity(line.len() + 1);
        bytes.extend_from_slice(&line);
        drop(line);
        bytes.push(0);
        LineCopy(bytes)
    }
}

/// [`Lines::line`] past the line the cache holds: fill it, then lend it.
#[inline(never)]
#[track_caller]
fn line_by_walk<'a>(buffer: Buf, lnum: LineNr) -> LineText<'a> {
    let walk = fetch_line_lent(buffer, lnum);
    buffer.lend_line(walk, false)
}

/// A buffer's lines held for one running match: the regexp matcher's line
/// cursor.
///
/// [`Lines`] lends one line at a time, the next read needing the last slice
/// dropped. The matcher cannot work that way: it keeps the line it is on
/// while it reads another (the next one, a back-reference's, `\%V`'s), and
/// it reads through a shared context, so no `&mut` can stand between two
/// reads. So for as long as this lives, every line it hands out **stays
/// put**:
///
/// - the first read flushes a pending replacement back into its block and
///   pins the buffer's memfile ([`MemFile::pin`]), holding the pin
///   ([`MemPin`]) for as long as this lives;
/// - a line read out of a data block lends that block under the pin
///   ([`MemFile::lend_page`]), and a lent block refuses to be written,
///   freed or resized (a panic, always on: one compare while nothing is
///   pinned) and is kept when memory runs short; the slice is borrowed from
///   the pin, so it cannot outlive it;
/// - no replacement line can be cached ([`MemLine::cache_replacement`]
///   panics), and the memfile cannot be dropped;
/// - the pin knows its memfile, and every read and the unpin check that
///   the buffer still has it: a memline or memfile swapped or replaced
///   under the hold (`mem::swap` over two edits, or over two memline loans)
///   would otherwise lend from a memfile nobody pinned, and unpin another's
///   (a panic, always on: one compare per read).
///
/// Reading other lines meanwhile is fine — through here, through
/// [`Lines`], from anywhere: the memline's cache moves, the lent bytes do
/// not. *Changing* the buffer's text while this lives is what panics,
/// rather than leaving the match a dangling slice. The pin goes when this
/// is dropped, with the match context that owns it: after one match, or
/// after a whole line walk (a search, which changes nothing between its
/// lines; `:substitute`, which does, runs a match per line).
///
/// [`MemFile::pin`]: crate::memfile::MemFile::pin
/// [`MemFile::lend_page`]: crate::memfile::MemFile::lend_page
/// [`MemLine::cache_replacement`]: super::state::MemLine::cache_replacement
#[derive(Debug)]
pub(crate) struct HeldLines {
    buffer: Buf,
    /// The pin on the buffer's memfile, taken at the first read that finds
    /// one, so a match that reads no line pays nothing. The pin knows its
    /// memfile, and every read and the unpin check that the buffer still
    /// has it.
    pin: OnceCell<MemPin>,
}

impl HeldLines {
    /// Hold `buffer`'s lines for a match; nothing is pinned until a line
    /// is read.
    #[inline(always)]
    pub(crate) fn new(buffer: Buf) -> HeldLines {
        HeldLines {
            buffer,
            pin: OnceCell::new(),
        }
    }

    /// Line `lnum`, its NUL included, good for as long as `self` is.
    ///
    /// The NUL is part of the slice because the matcher reads it as a
    /// position: `$` matches there and a character step stops there.
    #[inline(always)]
    #[track_caller]
    pub(crate) fn line(&self, lnum: LineNr) -> &[u8] {
        let pin = match self.pin.get() {
            Some(pin) => Some(pin),
            None => self.pin(),
        };
        let walk = fetch_line_lent(self.buffer, lnum);
        match pin {
            Some(pin) => self.buffer.lend_pinned_line(walk, pin),
            // No memfile: the memline answers a static stand-in.
            None => walk
                .static_cached_bytes_with_nul()
                .expect("a memline with no memfile caches only stand-ins"),
        }
    }

    /// Pin the memfile, if the buffer has one yet (a memline that is not
    /// open answers static stand-ins, which need none).
    #[inline(never)]
    fn pin(&self) -> Option<&MemPin> {
        let buffer = self.buffer;
        if !buffer.ml().is_open() {
            return None;
        }
        // A pending replacement goes back into its block first: once
        // pinned, nothing may cache one, nor write it back.
        if buffer.ml().line_is_dirty() {
            ml_flush_line(buffer);
        }
        let pin = buffer.ml().memfile().pin();
        Some(self.pin.get_or_init(|| pin))
    }
}

impl Drop for HeldLines {
    #[inline]
    fn drop(&mut self) {
        if let Some(pin) = self.pin.take() {
            let mut ml = self.buffer.ml();
            let memfile = ml.ml_mfp.as_deref_mut();
            if std::thread::panicking() && !memfile.as_ref().is_some_and(|m| m.is_pinned_by(&pin)) {
                // The panic that is unwinding already ends the process.
                return;
            }
            memfile.expect("a pinned memfile is not dropped").unpin(pin);
            drop(ml);
        }
    }
}

/// A line taken out of the cache, with the terminator the cache had after it.
///
/// A line read from the memline is followed by a NUL in the same allocation,
/// and the walks over it lean on that: several deliberately look one byte
/// *past* the last character and expect the terminator to end them
/// ([`crate::cstr::byte_at`] is the same reader with no memory behind it, and
/// `utf_head_off` reads the byte it is asked about before deciding anything).
/// A bare `Vec<u8>` of the line's bytes does not have that byte, and the
/// walks then read the vector's uninitialised capacity — which is a real bug
/// this type exists to have made impossible once.
///
/// So the copy keeps the NUL, and the slice it derefs to leaves it out: the
/// bytes are the line's, and the byte after them is a terminator, exactly as
/// they were in the buffer.
#[derive(Debug)]
pub struct LineCopy(Vec<u8>);

impl ::core::ops::Deref for LineCopy {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.0[..self.0.len() - 1]
    }
}

impl LineCopy {
    /// The copy as a C string, for the callers that still pass a pointer.
    ///
    /// # Panics
    /// Never in practice: the memline stores a NUL byte as an `NL`, so a
    /// line holds no interior NUL and the terminator this added is the only
    /// one.
    pub fn as_cstr(&self) -> &CStr {
        CStr::from_bytes_with_nul(&self.0).expect("a buffer line holds no NUL")
    }
}

/// Length of line `lnum` of the current buffer, excluding the NUL.
///
/// Safe: the editor exists, and the line number is clamped.
pub fn ml_get_len(lnum: LineNr) -> ColNr {
    ml_get_buf_len(Buf::current(), lnum)
}

/// Length of line `lnum` of `buffer`, excluding the NUL.
pub fn ml_get_buf_len(buffer: Buf, lnum: LineNr) -> ColNr {
    fetch_line(buffer, lnum, false);
    (buffer.ml().cached_len() - 1).max(ColNr::ZERO)
}

/// The codepoint at `pos`, which must either be valid or have `col` set to
/// `MAXCOL`.
///
/// NUL past the end of the line, which is what the searches that park a
/// column at `MAXCOL` are asking about.
pub fn gchar_pos(pos: &Pos) -> Codepoint {
    // While searching, the column is sometimes put at the end of a line.
    if pos.col == ColNr::MAX {
        return Codepoint::NUL;
    }
    let mut lines = Lines::current();
    let line = lines.line(pos.lnum);
    let at = usize::try_from(pos.col).unwrap_or(0);
    if at >= line.len() {
        return Codepoint::NUL;
    }
    char_at(&line[at..])
}

/// Whether the line last handed out by `ml_get` is the memline's own
/// replacement, which may be edited in place.
pub fn ml_line_alloced() -> bool {
    Buf::current().ml().line_is_dirty()
}

/// Insert `line` (its NUL the last byte) after line `lnum` of `buffer`,
/// whose memline is open, flushing any pending change first.
fn ml_append_flush(buffer: Buf, lnum: LineNr, line: &[u8], flags: c_int) -> Result<(), LineError> {
    ml_append_flush_held(buffer, buffer.ml(), lnum, line, flags)
}

/// [`ml_append_flush`] with the memline lent: one loan for the whole
/// append when no change is pending, which is every line of a file read.
fn ml_append_flush_held(
    buffer: Buf,
    ml: MlRef<'_>,
    lnum: LineNr,
    line: &[u8],
    flags: c_int,
) -> Result<(), LineError> {
    if lnum > ml.ml_line_count {
        return Err(LineError::NoSuchLine);
    }
    if ml.cached_lnum() != LineNr::ZERO {
        drop(ml);
        // This may invoke ml_append_int in turn.
        ml_flush_line(buffer);
        return ml_append_int(buffer, lnum, line, flags);
    }
    if !ml.is_open() {
        return Err(LineError::NoSuchLine);
    }
    ml_append_held(buffer, ml, lnum, line, flags)
}

/// Append `line` — its text, then the NUL the memline counts — after line
/// `lnum` of the current buffer (0 to put it in front of the file), opening
/// the memline if need be. `flags` are `ML_APPEND_*`.
///
/// `line` must not be another line of a buffer: unlocking a block can
/// invalidate that. The memline copies the bytes and writes none of them.
///
/// The caller should probably also call `appended_lines`.
pub(crate) fn ml_append_flags(lnum: LineNr, line: &[u8], flags: c_int) -> Result<(), LineError> {
    debug_assert_eq!(line.last(), Some(&0), "the line carries its NUL");
    let buffer = Buf::current();
    let ml = buffer.ml();
    if ml.is_open() {
        return ml_append_flush_held(buffer, ml, lnum, line, flags);
    }
    drop(ml);
    // During startup the memfile may still have to be created.
    open_buffer(false, None, ReadFlags::NONE)?;
    ml_append_flush(Buf::current(), lnum, line, flags)
}

/// [`ml_append_flags`] for a line the caller holds without its terminator:
/// the memline is handed a terminated copy.
pub(crate) fn ml_append_text(lnum: LineNr, line: &[u8]) -> Result<(), LineError> {
    let mut copy = Vec::with_capacity(line.len() + 1);
    copy.extend_from_slice(line);
    copy.push(0);
    ml_append_flags(lnum, &copy, 0)
}

/// [`ml_append_flags`] for a terminated line the caller holds.
pub(crate) fn ml_append_cstr(lnum: LineNr, line: &CStr) -> Result<(), LineError> {
    ml_append_flags(lnum, line.to_bytes_with_nul(), 0)
}

/// [`ml_append_flags`] of a line the caller holds with its terminator: the
/// last byte of `line` is the NUL the memline counts, and the bytes before it
/// are the text (a NUL in the text already stored as a newline). `newfile`
/// says a new file is being read in, which records `pe_old_lnum` for
/// recovery.
pub(crate) fn ml_append_terminated(
    lnum: LineNr,
    line: &[u8],
    newfile: bool,
) -> Result<(), LineError> {
    ml_append_flags(lnum, line, if newfile { ML_APPEND_NEW } else { 0 })
}

/// Append `line` (its terminator the last byte) after line `lnum` of
/// `buffer`, which must already have a memline. `newfile` as
/// [`ml_append_terminated`]'s.
pub fn ml_append_buf_terminated(
    buffer: Buf,
    lnum: LineNr,
    line: &[u8],
    newfile: bool,
) -> Result<(), LineError> {
    debug_assert_eq!(line.last(), Some(&0), "the line carries its NUL");
    if !buffer.ml().is_open() {
        return Err(LineError::NoSuchLine);
    }
    ml_append_flush(buffer, lnum, line, if newfile { ML_APPEND_NEW } else { 0 })
}

/// [`ml_append_buf_terminated`] for a line the caller holds without its
/// terminator: the memline is handed a terminated copy.
pub fn ml_append_bytes(buffer: Buf, lnum: LineNr, line: &[u8]) -> Result<(), LineError> {
    let mut copy = Vec::with_capacity(line.len() + 1);
    copy.extend_from_slice(line);
    copy.push(0);
    ml_append_buf_terminated(buffer, lnum, &copy, false)
}

/// Book the bytes of `text`, up to `len` of them, as deleted from the current
/// buffer, for the buffer-update callbacks.
pub fn ml_add_deleted_len(text: &[u8], len: usize) {
    ml_add_deleted_len_buf(Buf::current(), text, len);
}

/// [`ml_add_deleted_len`] for an arbitrary buffer.
pub fn ml_add_deleted_len_buf(buffer: Buf, text: &[u8], len: usize) {
    book_deleted(buffer, deleted_counts(buffer, text, len));
}

/// What deleting `text` (its first `len` bytes) adds to `buffer`'s deleted
/// counts, measured without writing the buffer: a caller holding a read of
/// it measures first and books after letting go. `None` while deletes are
/// not counted.
pub(crate) fn deleted_counts(buffer: Buf, text: &[u8], len: usize) -> Option<DeletedCounts> {
    if INHIBIT_DELETE_COUNT.get() != 0 {
        return None;
    }
    let text = &text[..len.min(text.len())];
    let units = buffer
        .update_need_codepoints()
        .then(|| utf_lengths(text, text.len()));
    Some(DeletedCounts {
        bytes: text.len(),
        units,
    })
}

/// [`deleted_counts`]' answer: bytes, and code points and units when the
/// buffer's watchers want them.
pub(crate) struct DeletedCounts {
    bytes: usize,
    units: Option<(usize, usize)>,
}

/// Add what [`deleted_counts`] measured to `buffer`'s counts.
pub(crate) fn book_deleted(buffer: Buf, counts: Option<DeletedCounts>) {
    let Some(counts) = counts else {
        return;
    };
    // The + 1 is the newline the line carries internally.
    buffer.set_deleted_bytes(buffer.deleted_bytes() + counts.bytes + 1);
    buffer.set_deleted_bytes2(buffer.deleted_bytes2() + counts.bytes + 1);
    if let Some((points, units)) = counts.units {
        buffer.set_deleted_codepoints(buffer.deleted_codepoints() + points + 1); // NL char
        buffer.set_deleted_codeunits(buffer.deleted_codeunits() + units + 1);
    }
}

/// Make line `lnum` of `buffer` the one the cache holds, as a replacement is
/// about to: open the memline if it is not, flush another pending line, and
/// book the old text as deleted.
fn begin_replace(buffer: Buf, lnum: LineNr) -> Result<(), Failed> {
    // During startup the memfile may still have to be created.
    if !buffer.ml().is_open() {
        open_buffer(false, None, ReadFlags::NONE)?;
    }
    if buffer.ml().cached_lnum() != lnum {
        ml_flush_line(buffer); // another line is buffered, flush it
    }
    if crate::buffer_updates::has_callbacks(buffer) {
        let mut lines = Lines(buffer);
        let text = lines.line(lnum);
        let counts = deleted_counts(buffer, &text, text.len());
        drop(text);
        book_deleted(buffer, counts);
    }
    Ok(())
}

impl Buf {
    /// Replace line `lnum` of `self` with `line` (its NUL the last byte),
    /// which the memline takes over, with buffering: the text is parked in the
    /// line cache and only written back by [`ml_flush_line`].
    ///
    /// The caller should probably also call `changed_lines`, unless it uses
    /// `update_screen(UPD_NOT_VALID)`.
    fn replace_line(self, lnum: LineNr, line: Vec<u8>) -> Result<(), Failed> {
        begin_replace(self, lnum)?;
        self.ml().cache_replacement(line, lnum);
        Ok(())
    }
}

/// Replace line `lnum` of `buffer` with `line`, which the memline copies.
pub(crate) fn ml_replace_buf_text(buffer: Buf, lnum: LineNr, line: &[u8]) -> Result<(), Failed> {
    let mut copy = Vec::with_capacity(line.len() + 1);
    copy.extend_from_slice(line);
    copy.push(0);
    buffer.replace_line(lnum, copy)
}

/// Replace line `lnum` of `buffer` with `line`, which the memline takes over.
pub(crate) fn ml_replace_buf_owned(buffer: Buf, lnum: LineNr, line: XString) -> Result<(), Failed> {
    buffer.replace_line(lnum, line.into_vec())
}

/// [`ml_replace_buf_owned`] for the current buffer.
pub(crate) fn ml_replace_owned(lnum: LineNr, line: XString) -> Result<(), Failed> {
    ml_replace_buf_owned(Buf::current(), lnum, line)
}

/// Replace line `lnum` of `buffer` with `line` and write it into its block
/// at once, without the copy the line cache would take: replacing lines in
/// a loop is common, and a scratch allocation per line is a lot of noise.
///
/// `line` holds the text and, last, its NUL.
pub(crate) fn ml_replace_buf_now(buffer: Buf, lnum: LineNr, line: &[u8]) -> Result<(), Failed> {
    debug_assert_eq!(line.last(), Some(&0), "the line carries its NUL");
    begin_replace(buffer, lnum)?;
    let text = line;
    buffer.ml().cache_pending(
        lnum,
        ColNr::try_from(text.len()).expect("a line shorter than INT_MAX"),
    );
    ml_flush_text(buffer, lnum, text);
    buffer.ml().clear_cache();
    Ok(())
}

/// Delete line `lnum` of `buffer`.
///
/// The caller should probably also call `changed_lines`.
pub fn ml_delete_buf(buffer: Buf, lnum: LineNr, message: bool) -> Result<(), LineError> {
    ml_flush_line(buffer);
    ml_delete_int(buffer, lnum, if message { ML_DEL_MESSAGE } else { 0 })
}

/// Delete line `lnum` of the current buffer.
pub fn ml_delete(lnum: LineNr) -> Result<(), LineError> {
    ml_delete_flags(lnum, 0)
}

/// [`ml_delete`] taking `ML_DEL_` flags.
pub fn ml_delete_flags(lnum: LineNr, flags: c_int) -> Result<(), LineError> {
    ml_flush_line(Buf::current());
    if lnum < LineNr::ONE || lnum > Buf::current().line_count() {
        return Err(LineError::NoSuchLine);
    }
    ml_delete_int(Buf::current(), lnum, flags)
}

/// Set the [`DB_MARKED`] bit on line `lnum`.
pub fn ml_setmarked(lnum: LineNr) {
    let buffer = Buf::current();
    if lnum < LineNr::ONE || lnum > buffer.line_count() || !buffer.ml().is_open() {
        return; // invalid line number
    }
    if LOWEST_MARKED.get() == LineNr::ZERO || LOWEST_MARKED.get() > lnum {
        LOWEST_MARKED.set(lnum);
    }
    let Some(id) = ml_find_line(buffer, lnum, ML_FIND) else {
        return;
    };
    let idx = (lnum - buffer.ml().locked_low()).cast_unsigned() as usize;
    DataBlock(buffer.ml().page_mut(id)).set_marked(idx, true);
    buffer.ml().locked_is_dirty();
}

/// Walk the lines from `LOWEST_MARKED` on, clearing every [`DB_MARKED`] bit
/// found. With `first`, stop at the first one and answer its line; otherwise
/// answer 0 once all are clear.
fn clear_marks(first: bool) -> LineNr {
    let buffer = Buf::current();
    // Nothing marked yet leaves `LOWEST_MARKED` at 0, and line 0 is line
    // 1's block. Upstream starts there anyway and reads index entry -1 --
    // the top half of the block's line count, which is never marked -- so
    // starting at line 1 skips only a read that could find nothing.
    let mut lnum = LOWEST_MARKED.get().max(LineNr::ONE);
    while lnum <= buffer.line_count() {
        let Some(id) = ml_find_line(buffer, lnum, ML_FIND) else {
            return LineNr::ZERO;
        };
        let mut held = buffer.ml();
        let (low, high) = (held.locked_low(), held.locked_high());
        let mut data = DataBlock(held.page_mut(id));
        let mut found = None;
        while lnum <= high {
            let idx = (lnum - low).cast_unsigned() as usize;
            if data.is_marked(idx) {
                data.set_marked(idx, false);
                found = Some(lnum);
                if first {
                    break;
                }
            }
            lnum += 1;
        }
        drop(held);
        if found.is_some() {
            buffer.ml().locked_is_dirty();
        }
        if first && let Some(found) = found {
            LOWEST_MARKED.set(found + 1);
            return found;
        }
    }
    LineNr::ZERO
}

/// The first line with its [`DB_MARKED`] bit set, clearing the bit. Zero when
/// there is none left.
pub fn ml_firstmarked() -> LineNr {
    if !Buf::current().ml_is_open() {
        return LineNr::ZERO;
    }
    // Start at lowest_marked: the last line a mark was found at, kept up
    // to date as lines are inserted and deleted.
    clear_marks(true)
}

/// Clear every [`DB_MARKED`] bit.
pub fn ml_clearmarked() {
    if !Buf::current().ml_is_open() {
        return; // nothing to do
    }
    let _ = clear_marks(false);
    LOWEST_MARKED.set(LineNr::ZERO);
}

/// Take and reset the deleted-byte counters the buffer-update callbacks
/// report: the bytes, codepoints and code units deleted since the last take.
pub fn ml_flush_deleted_bytes(buffer: Buf) -> (size_t, size_t, size_t) {
    let taken = (
        buffer.deleted_bytes(),
        buffer.deleted_codepoints(),
        buffer.deleted_codeunits(),
    );
    buffer.set_deleted_bytes(0);
    buffer.set_deleted_codepoints(0);
    buffer.set_deleted_codeunits(0);
    taken
}

/// Where a one-character step ([`inc`], [`dec`] and their cursor forms) left
/// a position. Upstream answered `0`, `1`, `2` and `-1`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Moved {
    /// Onto another character of the same line.
    InLine,
    /// Onto the next line, or the previous one.
    NewLine,
    /// Onto the NUL past the end of the line (forward only).
    OntoNul,
    /// Nowhere: the position was already at the end of the buffer, or
    /// its start.
    Stuck,
}

impl Moved {
    /// Whether the step went nowhere.
    #[inline]
    pub(crate) fn is_stuck(self) -> bool {
        self == Moved::Stuck
    }

    /// Whether the step left the character's line, or got to its end:
    /// upstream's `>= 1`.
    #[inline]
    pub(crate) fn left_the_line(self) -> bool {
        matches!(self, Moved::NewLine | Moved::OntoNul)
    }
}

/// Advance `pos` by one character, crossing line boundaries as needed.
pub(crate) fn inc(pos: &mut Pos) -> Moved {
    // While searching, the position may be set to the end of a line.
    if pos.col != ColNr::MAX {
        let mut lines = Lines::current();
        let line = lines.line(pos.lnum);
        let at = usize::try_from(pos.col).unwrap_or(0);
        if at < line.len() {
            // Still within the line; move to the next char, which may be
            // the one past the end.
            let l = cluster_len(&line[at..]);
            pos.col += ColNr::try_from(l).unwrap_or(ColNr::ZERO).get();
            return if at + l < line.len() {
                Moved::InLine
            } else {
                Moved::OntoNul
            };
        }
    }
    if pos.lnum != Buf::current().line_count() {
        // There is a next line.
        pos.col = ColNr::ZERO;
        pos.lnum += 1;
        pos.coladd = 0;
        return Moved::NewLine;
    }
    Moved::Stuck
}

/// [`inc`], but skipping the NUL at the end of a non-empty line.
pub(crate) fn incl(pos: &mut Pos) -> Moved {
    let mut r = inc(pos);
    if r.left_the_line() && pos.col != ColNr::ZERO {
        r = inc(pos);
    }
    r
}

/// Move `pos` back by one character, crossing line boundaries as needed.
/// Never [`Moved::OntoNul`].
pub(crate) fn dec(pos: &mut Pos) -> Moved {
    pos.coladd = 0;
    // Step `pos.col` back off a trail byte, and -- when `to_end` is set --
    // onto the last character of the line first. One read of the line does
    // both, where upstream paid `ml_get` and `ml_get_len` separately.
    let snap = |pos: &mut Pos, to_end: bool| {
        let mut lines = Lines::current();
        let line = lines.line(pos.lnum);
        if to_end {
            pos.col = ColNr::try_from(line.len()).unwrap_or(ColNr::MAX);
        }
        let at = usize::try_from(pos.col).unwrap_or(0);
        pos.col -= ColNr::try_from(head_off(&line, at))
            .unwrap_or(ColNr::ZERO)
            .get();
        drop(line);
    };
    if pos.col == ColNr::MAX {
        // Past the end of the line.
        snap(pos, true);
        return Moved::InLine;
    }
    if pos.col > ColNr::ZERO {
        // Still within the line.
        pos.col -= 1;
        snap(pos, false);
        return Moved::InLine;
    }
    if pos.lnum > LineNr::ONE {
        // There is a previous line.
        pos.lnum -= 1;
        snap(pos, true);
        return Moved::NewLine;
    }
    Moved::Stuck // at the start of the file
}

/// [`dec`], but skipping the NUL at the end of a non-empty line.
pub(crate) fn decl(pos: &mut Pos) -> Moved {
    let mut r = dec(pos);
    if r == Moved::NewLine && pos.col != ColNr::ZERO {
        r = dec(pos);
    }
    r
}
