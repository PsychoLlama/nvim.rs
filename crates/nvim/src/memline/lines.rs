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
//! let text = lines.line(lnum); // &[u8], the line without its NUL
//! ```
//!
//! A single read fits in one expression — `buffer.lines().line(lnum)`, whose
//! temporary handle lives to the end of the statement; a walk binds the
//! handle, and every slice it hands out is bounded by it.
//!
//! [`Lines`]'s own documentation carries the borrow story — how long the
//! slice is valid, what invalidates it, and what to do about the two shapes
//! the borrow cannot express. In one sentence: the slice lives exactly as
//! long as the `&mut Lines` it came from, so the next line, any `ml_*`
//! mutation and any re-entry into the editor all need it dropped first.
//!
//! # The pointer forms that stay
//!
//! Four of upstream's `ml_get*` spellings are left, and the ratchet's
//! `ml_get_raw` needle counts every call to one -- *including* the calls
//! spelled through an alias: [`Buf::line_raw`], [`Buf::line_mut_raw`],
//! [`Buf::line_len_raw`] and `cursor`'s four `get_cursor_{line,pos}_{ptr,len}`
//! accessors are the same read with a shorter name, and the needle sees them
//! by name. This is what each form is for; anything not on this list has been
//! deleted (`ml_get_pos`, `ml_get_pos_len`, and `ml_get_cursor`, which never
//! had a caller here).
//!
//! | form | why it stays |
//! | --- | --- |
//! | [`ml_get_buf`] | **the floor** of the pointer form: it unpacks the line
//!   and answers where it lives. [`Lines::line`] unpacks the same way and
//!   answers the bytes instead. |
//! | [`ml_get_buf_mut`] | the same read, marking the line dirty.
//!   [`Buf::line_mut_raw`] is written on top of it, as [`Lines::line_mut`] is
//!   on the same unpacking, and the read also books the old text as deleted —
//!   so a caller that wants only the bytes must *not* reach for it. |
//! | [`ml_get_buf_len`] | a line's length without a slice to measure. It
//!   dispatches through `ml_get_buf` and then reads the cache's own length,
//!   which is why a caller that is about to read the line should take
//!   [`Lines::line`] and ask the slice instead of paying two dispatches. |
//! | [`ml_get`], [`ml_get_len`] | the current buffer's spelling of the two
//!   above. They exist for the callers that still hold a line as a pointer,
//!   and they go when those do. |
//!
//! The count that is *not* the floor is the call sites outside this module.
//! Those are the pointer walks that have not been rewritten yet; the ones
//! left are listed in the phase's handoff, and each is a walk whose *callee*
//! still takes a `*mut c_char`.
//!
//! **The floor is 22 calls.** Five are in this file -- the forms above,
//! written in terms of `ml_get_buf_impl` and `fetch_line` -- and seventeen
//! are in `cursor/mod.rs`, where the `get_cursor_{line,pos}_{ptr,len}`
//! accessors spell the same read for the line the cursor is on. Nothing under those can
//! answer a slice, so the number does not go to zero; everything the needle
//! counts beyond 22 is a call site elsewhere, and each one retires when the
//! callee it feeds stops taking a pointer. `ml_replace` and `ml_append` are
//! the two that most of them are walking toward.

#![forbid(unsafe_code)]

use super::*;
use crate::cstr;
use crate::fileio::ReadFlags;
use crate::memory::XString;
use crate::pos::MAXCOL;
use crate::types::{Failed, NUL};
use crate::winlayer::Buf;
use core::ffi::{CStr, c_char, c_int};

/// A read-only pointer to line `lnum` of the current buffer. Never NULL.
///
/// Safe: the only promise is that the editor exists, which `curbuf` carries
/// from startup to exit, and the read clamps `lnum` into the buffer itself.
/// The answer is a raw pointer, so *reading* through it is still the
/// caller's business.
///
/// Prefer [`Lines::line`]: see [the module's floor](self#the-pointer-forms-that-stay).
pub fn ml_get(lnum: LineNr) -> *mut c_char {
    ml_get_buf_impl(Buf::current(), lnum, false)
}

/// [`ml_get`] for an arbitrary buffer.
pub fn ml_get_buf(buffer: Buf, lnum: LineNr) -> *mut c_char {
    ml_get_buf_impl(buffer, lnum, false)
}

/// [`ml_get_buf`], but the line may be changed through the pointer.
///
/// Very limited: only the bytes already there can be rewritten. Use
/// [`ml_replace_buf_text`] for anything else.
pub fn ml_get_buf_mut(buffer: Buf, lnum: LineNr) -> *mut c_char {
    ml_get_buf_impl(buffer, lnum, true)
}

/// A buffer's one-line cache, borrowed.
///
/// The memline keeps exactly one line unpacked per buffer: the line in the
/// data block holding it, or the replacement a pending `ml_replace` left.
/// Reading a *different* line flushes that pending write, releases the
/// locked block and locks another, so the bytes the previous read answered
/// may have moved or been freed. Upstream's rule is "use the line before you
/// ask for another one", and until now it was written down nowhere.
///
/// This is where it is written down, and as much of it as the compiler can
/// hold is held: [`line`](Self::line) borrows the handle **mutably** and the
/// slice it answers keeps that borrow alive, so holding one line while
/// asking for the next does not compile. That is the whole of the cache's
/// one-line limit, expressed as a lifetime.
///
/// What the compiler still cannot see is a call *out* of here that reads
/// another line behind the handle's back — `ml_replace`, `ml_append`,
/// `ml_delete`, or anything that redraws or runs user code. Not one of them
/// takes a borrow this handle could conflict with, so that half of the
/// contract is the caller's, and it is one rule: **the borrow ends before
/// the call**. A walk that has to re-enter the editor takes the line, does
/// its indexing, drops the slice, and calls; a walk that has to hold text
/// across the call takes [`line_copy`](Self::line_copy).
///
/// A caller that genuinely needs two lines of the *same* buffer at once —
/// `:diffget`'s comparison, `'foldmarker'`'s start-and-end scan — is asking
/// for something the cache cannot give, and copies one of them. That is not
/// a limitation of this type; it is the memline's, made visible.
///
/// The slices are the line's bytes **without** the terminating NUL, which is
/// the length `ml_get_len` reports and the length every caller wants. A
/// memline line stores a NUL byte as an `NL`, so the bytes may hold either
/// and never hold a terminator — which is exactly why the answer is `&[u8]`
/// and not `&CStr` ([`crate::cstr`] § the parameter convention). Walks that
/// used to step one past the last character and read the terminator want
/// [`crate::cstr::byte_at`], which answers `NUL` past the end.
///
/// [`Buf::line_raw`] is the same read as a raw pointer, for callers that still
/// want one; [`ml_get`] and kin are the free-function spelling. Both stay
/// while any caller still holds a line as a pointer.
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
        self.0.b_ml.ml_line_count
    }

    /// Line `lnum`, without its NUL.
    ///
    /// Out-of-range line numbers answer the `???` placeholder [`ml_get`]
    /// hands back, and complain the same way; there is no failure case here
    /// that the pointer form does not have.
    pub fn line(&mut self, lnum: LineNr) -> &[u8] {
        fetch_line(self.0, lnum, false);
        self.0.b_ml.cached_bytes()
    }

    /// Line `lnum`, writable in place.
    ///
    /// Exactly as limited as [`ml_get_buf_mut`]: the bytes already there can
    /// be rewritten and nothing else, which is what a slice of the line's own
    /// length says. Use `ml_replace` to change a line's length.
    pub fn line_mut(&mut self, lnum: LineNr) -> &mut [u8] {
        fetch_line(self.0, lnum, true);
        self.0.b_ml.cached_bytes_mut()
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
    pub fn line_cstr(&mut self, lnum: LineNr, at: usize) -> &CStr {
        fetch_line(self.0, lnum, false);
        let bytes = self.0.b_ml.cached_bytes_with_nul();
        assert!(at < bytes.len(), "column past the end of the line");
        cstr::in_bytes(&bytes[at..])
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
        bytes.extend_from_slice(line);
        bytes.push(NUL as u8);
        LineCopy(bytes)
    }
}

/// Line `lnum` of `buffer`, its NUL included, borrowed for as long as the
/// handle is rather than for as long as a [`Lines`] borrow.
///
/// This is upstream's `ml_get_buf` contract with a slice in place of the
/// pointer: the bytes are good until the next read of a *different* line of
/// the buffer, and nothing in the type says so. [`Lines`] says so, and is
/// what everything else reads lines through; this exists for the one reader
/// that holds its line across a stretch of work it knows reads no other —
/// the regexp matcher, whose cursor is an offset into the line it is on and
/// which fetches the line again, through here, after anything that might
/// have moved it (`\%V`'s virtual columns, `\%'m`'s mark lookup, a
/// back-reference to another line). See `regexp::rex` for that side of it.
///
/// The NUL is part of the slice because the matcher reads it as a position:
/// `$` matches there and a character step stops there.
pub(crate) fn held_line(buffer: &Buf, lnum: LineNr) -> &[u8] {
    fetch_line(*buffer, lnum, false);
    buffer.b_ml.cached_bytes_with_nul()
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
/// Safe: as [`ml_get`] -- the editor exists, and the line number is
/// clamped.
pub fn ml_get_len(lnum: LineNr) -> ColNr {
    ml_get_buf_len(Buf::current(), lnum)
}

/// Length of line `lnum` of `buffer`, excluding the NUL.
pub fn ml_get_buf_len(buffer: Buf, lnum: LineNr) -> ColNr {
    fetch_line(buffer, lnum, false);
    (buffer.b_ml.cached_len() - 1).max(0)
}

/// The codepoint at `pos`, which must either be valid or have `col` set to
/// `MAXCOL`.
///
/// NUL past the end of the line, which is what the searches that park a
/// column at `MAXCOL` are asking about.
pub fn gchar_pos(pos: &Pos) -> c_int {
    // While searching, the column is sometimes put at the end of a line.
    if pos.col == MAXCOL as c_int {
        return NUL;
    }
    let mut lines = Lines::current();
    let line = lines.line(pos.lnum);
    let at = usize::try_from(pos.col).unwrap_or(0);
    if at >= line.len() {
        return NUL;
    }
    char_at(&line[at..])
}

/// Whether the line last handed out by `ml_get` is the memline's own
/// replacement, which may be edited in place.
pub fn ml_line_alloced() -> bool {
    Buf::current().b_ml.line_is_dirty()
}

/// Insert `line` (its NUL the last byte) after line `lnum` of `buffer`,
/// whose memline is open, flushing any pending change first.
fn ml_append_flush(buffer: Buf, lnum: LineNr, line: &[u8], flags: c_int) -> Result<(), Failed> {
    if lnum > buffer.b_ml.ml_line_count {
        return Err(Failed); // lnum out of range
    }
    if buffer.b_ml.cached_lnum() != 0 {
        // This may invoke ml_append_int in turn.
        ml_flush_line(buffer);
    }
    ml_append_int(buffer, lnum, line, flags)
}

/// Append `line` — its text, then the NUL the memline counts — after line
/// `lnum` of the current buffer (0 to put it in front of the file), opening
/// the memline if need be. `flags` are `ML_APPEND_*`.
///
/// `line` must not be another line of a buffer: unlocking a block can
/// invalidate that. The memline copies the bytes and writes none of them.
///
/// The caller should probably also call `appended_lines`.
pub(crate) fn ml_append_flags(lnum: LineNr, line: &[u8], flags: c_int) -> Result<(), Failed> {
    debug_assert_eq!(line.last(), Some(&0), "the line carries its NUL");
    // During startup the memfile may still have to be created.
    if !Buf::current().b_ml.is_open() && open_buffer(false, None, ReadFlags::NONE).is_err() {
        return Err(Failed);
    }
    ml_append_flush(Buf::current(), lnum, line, flags)
}

/// [`ml_append_flags`] for a line the caller holds without its terminator:
/// the memline is handed a terminated copy.
pub(crate) fn ml_append_text(lnum: LineNr, line: &[u8]) -> Result<(), Failed> {
    let mut copy = Vec::with_capacity(line.len() + 1);
    copy.extend_from_slice(line);
    copy.push(0);
    ml_append_flags(lnum, &copy, 0)
}

/// [`ml_append_flags`] for a terminated line the caller holds.
pub(crate) fn ml_append_cstr(lnum: LineNr, line: &CStr) -> Result<(), Failed> {
    ml_append_flags(lnum, line.to_bytes_with_nul(), 0)
}

/// [`ml_append_flags`] of a line the caller holds with its terminator: the
/// last byte of `line` is the NUL the memline counts, and the bytes before it
/// are the text (a NUL in the text already stored as a newline). `newfile`
/// says a new file is being read in, which records `pe_old_lnum` for
/// recovery.
pub(crate) fn ml_append_terminated(lnum: LineNr, line: &[u8], newfile: bool) -> Result<(), Failed> {
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
) -> Result<(), Failed> {
    debug_assert_eq!(line.last(), Some(&0), "the line carries its NUL");
    if !buffer.b_ml.is_open() {
        return Err(Failed);
    }
    ml_append_flush(buffer, lnum, line, if newfile { ML_APPEND_NEW } else { 0 })
}

/// [`ml_append_buf_terminated`] for a line the caller holds without its
/// terminator: the memline is handed a terminated copy.
pub fn ml_append_bytes(buffer: Buf, lnum: LineNr, line: &[u8]) -> Result<(), Failed> {
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
pub fn ml_add_deleted_len_buf(mut buffer: Buf, text: &[u8], len: usize) {
    if inhibit_delete_count.get() != 0 {
        return;
    }
    let text = &text[..len.min(text.len())];
    // The + 1 is the newline the line carries internally.
    buffer.deleted_bytes += text.len() + 1;
    buffer.deleted_bytes2 += text.len() + 1;
    if buffer.update_need_codepoints {
        let (points, units) = utf_lengths(text, text.len());
        buffer.deleted_codepoints += points + 1; // NL char
        buffer.deleted_codeunits += units + 1;
    }
}

/// Make line `lnum` of `buffer` the one the cache holds, as a replacement is
/// about to: open the memline if it is not, flush another pending line, and
/// book the old text as deleted.
fn begin_replace(buffer: Buf, lnum: LineNr) -> Result<(), Failed> {
    // During startup the memfile may still have to be created.
    if !buffer.b_ml.is_open() {
        open_buffer(false, None, ReadFlags::NONE)?;
    }
    if buffer.b_ml.cached_lnum() != lnum {
        ml_flush_line(buffer); // another line is buffered, flush it
    }
    if buffer.update_callbacks.size != 0 {
        fetch_line(buffer, lnum, false);
        let text = buffer.b_ml.cached_bytes();
        ml_add_deleted_len_buf(buffer, text, text.len());
    }
    Ok(())
}

/// Replace line `lnum` of `buffer` with `line` (its NUL the last byte),
/// which the memline takes over, with buffering: the text is parked in the
/// line cache and only written back by [`ml_flush_line`].
///
/// The caller should probably also call `changed_lines`, unless it uses
/// `update_screen(UPD_NOT_VALID)`.
fn replace_line(mut buffer: Buf, lnum: LineNr, line: Vec<u8>) -> Result<(), Failed> {
    begin_replace(buffer, lnum)?;
    buffer.b_ml.cache_replacement(line, lnum);
    Ok(())
}

/// Replace line `lnum` of `buffer` with `line`, which the memline copies.
pub(crate) fn ml_replace_buf_text(buffer: Buf, lnum: LineNr, line: &[u8]) -> Result<(), Failed> {
    let mut copy = Vec::with_capacity(line.len() + 1);
    copy.extend_from_slice(line);
    copy.push(0);
    replace_line(buffer, lnum, copy)
}

/// Replace line `lnum` of `buffer` with `line`, which the memline takes over.
pub(crate) fn ml_replace_buf_owned(buffer: Buf, lnum: LineNr, line: XString) -> Result<(), Failed> {
    replace_line(buffer, lnum, line.into_vec())
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
pub(crate) fn ml_replace_buf_now(mut buffer: Buf, lnum: LineNr, line: &[u8]) -> Result<(), Failed> {
    debug_assert_eq!(line.last(), Some(&0), "the line carries its NUL");
    begin_replace(buffer, lnum)?;
    let text = line;
    buffer.b_ml.cache_pending(
        lnum,
        ColNr::try_from(text.len()).expect("a line shorter than INT_MAX"),
    );
    ml_flush_text(buffer, lnum, text);
    buffer.b_ml.clear_cache();
    Ok(())
}

/// Delete line `lnum` of `buffer`.
///
/// The caller should probably also call `changed_lines`.
pub fn ml_delete_buf(buffer: Buf, lnum: LineNr, message: bool) -> Result<(), Failed> {
    ml_flush_line(buffer);
    ml_delete_int(buffer, lnum, if message { ML_DEL_MESSAGE } else { 0 })
}

/// Delete line `lnum` of the current buffer.
pub fn ml_delete(lnum: LineNr) -> Result<(), Failed> {
    ml_delete_flags(lnum, 0)
}

/// [`ml_delete`] taking `ML_DEL_` flags.
pub fn ml_delete_flags(lnum: LineNr, flags: c_int) -> Result<(), Failed> {
    ml_flush_line(Buf::current());
    if lnum < 1 || lnum > Buf::current().b_ml.ml_line_count {
        return Err(Failed);
    }
    ml_delete_int(Buf::current(), lnum, flags)
}

/// Set the [`DB_MARKED`] bit on line `lnum`.
pub fn ml_setmarked(lnum: LineNr) {
    let mut buffer = Buf::current();
    if lnum < 1 || lnum > buffer.b_ml.ml_line_count || !buffer.b_ml.is_open() {
        return; // invalid line number
    }
    if lowest_marked.get() == 0 || lowest_marked.get() > lnum {
        lowest_marked.set(lnum);
    }
    let Some(id) = ml_find_line(buffer, lnum, ML_FIND) else {
        return;
    };
    let idx = (lnum - buffer.b_ml.locked_low()) as usize;
    DataBlock(buffer.b_ml.page_mut(id)).set_marked(idx, true);
    buffer.b_ml.locked_is_dirty();
}

/// Walk the lines from `lowest_marked` on, clearing every [`DB_MARKED`] bit
/// found. With `first`, stop at the first one and answer its line; otherwise
/// answer 0 once all are clear.
fn clear_marks(first: bool) -> LineNr {
    let mut buffer = Buf::current();
    // Nothing marked yet leaves `lowest_marked` at 0, and line 0 is line
    // 1's block. Upstream starts there anyway and reads index entry -1 --
    // the top half of the block's line count, which is never marked -- so
    // starting at line 1 skips only a read that could find nothing.
    let mut lnum = lowest_marked.get().max(1);
    while lnum <= buffer.b_ml.ml_line_count {
        let Some(id) = ml_find_line(buffer, lnum, ML_FIND) else {
            return 0;
        };
        let (low, high) = (buffer.b_ml.locked_low(), buffer.b_ml.locked_high());
        let mut data = DataBlock(buffer.b_ml.page_mut(id));
        let mut found = None;
        while lnum <= high {
            let idx = (lnum - low) as usize;
            if data.is_marked(idx) {
                data.set_marked(idx, false);
                found = Some(lnum);
                if first {
                    break;
                }
            }
            lnum += 1;
        }
        if found.is_some() {
            buffer.b_ml.locked_is_dirty();
        }
        if first && let Some(found) = found {
            lowest_marked.set(found + 1);
            return found;
        }
    }
    0
}

/// The first line with its [`DB_MARKED`] bit set, clearing the bit. Zero when
/// there is none left.
pub fn ml_firstmarked() -> LineNr {
    if !Buf::current().b_ml.is_open() {
        return 0;
    }
    // Start at lowest_marked: the last line a mark was found at, kept up
    // to date as lines are inserted and deleted.
    clear_marks(true)
}

/// Clear every [`DB_MARKED`] bit.
pub fn ml_clearmarked() {
    if !Buf::current().b_ml.is_open() {
        return; // nothing to do
    }
    let _ = clear_marks(false);
    lowest_marked.set(0);
}

/// Take and reset the deleted-byte counters the buffer-update callbacks
/// report: the bytes, codepoints and code units deleted since the last take.
pub fn ml_flush_deleted_bytes(mut buffer: Buf) -> (size_t, size_t, size_t) {
    let taken = (
        buffer.deleted_bytes,
        buffer.deleted_codepoints,
        buffer.deleted_codeunits,
    );
    buffer.deleted_bytes = 0;
    buffer.deleted_codepoints = 0;
    buffer.deleted_codeunits = 0;
    taken
}

/// Advance `pos` by one character, crossing line boundaries as needed.
///
/// Returns 1 when it moved to the next line, 2 when it moved onto the NUL at
/// the end of a line, -1 at the end of the file, and 0 otherwise.
pub fn inc(pos: &mut Pos) -> c_int {
    // While searching, the position may be set to the end of a line.
    if pos.col != MAXCOL as c_int {
        let mut lines = Lines::current();
        let line = lines.line(pos.lnum);
        let at = usize::try_from(pos.col).unwrap_or(0);
        if at < line.len() {
            // Still within the line; move to the next char, which may be
            // the one past the end.
            let l = cluster_len(&line[at..]);
            pos.col += ColNr::try_from(l).unwrap_or(0);
            return if at + l < line.len() { 0 } else { 2 };
        }
    }
    if pos.lnum != Buf::current().b_ml.ml_line_count {
        // There is a next line.
        pos.col = 0;
        pos.lnum += 1;
        pos.coladd = 0;
        return 1;
    }
    -1
}

/// [`inc`], but skipping the NUL at the end of a non-empty line.
pub fn incl(pos: &mut Pos) -> c_int {
    let mut r = inc(pos);
    if r >= 1 && pos.col != 0 {
        r = inc(pos);
    }
    r
}

/// Move `pos` back by one character, crossing line boundaries as needed.
///
/// Returns 1 when it moved to the previous line, -1 at the start of the file,
/// and 0 otherwise.
pub fn dec(pos: &mut Pos) -> c_int {
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
        pos.col -= ColNr::try_from(head_off(line, at)).unwrap_or(0);
    };
    if pos.col == MAXCOL as c_int {
        // Past the end of the line.
        snap(pos, true);
        return 0;
    }
    if pos.col > 0 {
        // Still within the line.
        pos.col -= 1;
        snap(pos, false);
        return 0;
    }
    if pos.lnum > 1 {
        // There is a previous line.
        pos.lnum -= 1;
        snap(pos, true);
        return 1;
    }
    -1 // at the start of the file
}

/// [`dec`], but skipping the NUL at the end of a non-empty line.
pub fn decl(pos: &mut Pos) -> c_int {
    let mut r = dec(pos);
    if r == 1 && pos.col != 0 {
        r = dec(pos);
    }
    r
}
