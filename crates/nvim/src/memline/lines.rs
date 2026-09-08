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
//! [`Lines`]'s own documentation carries the borrow story — how long the
//! slice is valid, what invalidates it, and what to do about the two shapes
//! the borrow cannot express. In one sentence: the slice lives exactly as
//! long as the `&mut Lines` it came from, so the next line, any `ml_*`
//! mutation and any re-entry into the editor all need it dropped first.

#![deny(unsafe_op_in_unsafe_fn)]
#![allow(unsafe_code)]

use super::*;
use crate::cstr;
use crate::pos::MAXCOL;
use crate::types::{Failed, NUL};
use crate::winlayer::Buf;

/// A read-only pointer to line `lnum` of the current buffer. Never NULL.
///
/// Safe: the only promise is that the editor exists, which `curbuf` carries
/// from startup to exit, and `ml_get_buf_impl` clamps `lnum` into the
/// buffer itself. The answer is a raw pointer, so *reading* through it is
/// still the caller's business.
pub fn ml_get(lnum: LineNr) -> *mut ::core::ffi::c_char {
    unsafe { ml_get_buf_impl(Buf::current(), lnum, false) }
}

/// [`ml_get`] for an arbitrary buffer.
///
/// # Safety
/// `buffer` must point at a buffer.
pub unsafe fn ml_get_buf(buffer: Buf, lnum: LineNr) -> *mut ::core::ffi::c_char {
    unsafe { ml_get_buf_impl(buffer, lnum, false) }
}

/// [`ml_get_buf`], but the line may be changed through the pointer.
///
/// Very limited: only the bytes already there can be rewritten. Use
/// [`ml_replace_buf`] for anything else.
///
/// # Safety
/// `buffer` must point at a buffer.
pub unsafe fn ml_get_buf_mut(buffer: Buf, lnum: LineNr) -> *mut ::core::ffi::c_char {
    unsafe { ml_get_buf_impl(buffer, lnum, true) }
}

/// A buffer's one-line cache, borrowed.
///
/// `ml_get` answers a pointer *into* the data block holding the line — or
/// into the buffer a pending `ml_replace` left — and the memline keeps
/// exactly one line unpacked per buffer. Reading a *different* line flushes
/// that pending write, releases the locked block and locks another, so the
/// bytes the previous read answered may have moved or been freed. Upstream's
/// rule is "use the line before you ask for another one", and until now it
/// was written down nowhere.
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
/// [`Buf::line`] is the same read as a raw pointer, for callers that still
/// want one; [`ml_get`] and kin are the free-function spelling. Both stay
/// while any caller still holds a line as a pointer, and
/// `ml_get_placeholder`'s `static` — the `???` an out-of-range line answers
/// — goes with them: it is a pointer with nowhere to live otherwise, and
/// nothing here can retire it before the last pointer caller does.
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
        let buf = self.0.raw();
        // SAFETY: a live buffer.
        let buf = unsafe { Buf::new(buf) };
        // SAFETY: a live buffer. `ml_get_buf` never answers NULL, and the
        // second call is a cache hit on the line the first one just read, so
        // it is that line's length: the slice is the line. The borrow of
        // `self` is what keeps the next read from invalidating it.
        unsafe {
            let text = ml_get_buf(buf, lnum).cast::<u8>();
            ::core::slice::from_raw_parts(text, to_len(ml_get_buf_len(buf, lnum)))
        }
    }

    /// Line `lnum`, writable in place.
    ///
    /// Exactly as limited as [`ml_get_buf_mut`]: the bytes already there can
    /// be rewritten and nothing else, which is what a slice of the line's own
    /// length says. Use `ml_replace` to change a line's length.
    pub fn line_mut(&mut self, lnum: LineNr) -> &mut [u8] {
        let buf = self.0.raw();
        // SAFETY: a live buffer.
        let buf = unsafe { Buf::new(buf) };
        // SAFETY: as [`Lines::line`] -- the first call marks the line dirty
        // and the second is a cache hit on it -- and the borrow is
        // exclusive, so no shared slice of the same cache can be alive.
        unsafe {
            let text = ml_get_buf_mut(buf, lnum).cast::<u8>();
            ::core::slice::from_raw_parts_mut(text, to_len(ml_get_buf_len(buf, lnum)))
        }
    }

    /// Line `lnum`, copied out of the cache.
    ///
    /// The escape hatch for the two shapes the borrow cannot express: a
    /// second line of the same buffer held at the same time, and text held
    /// across a call that re-enters the editor. Both are real needs and both
    /// are a copy in C as well — upstream spells them `xstrdup(ml_get(…))`.
    pub fn line_copy(&mut self, lnum: LineNr) -> Vec<u8> {
        self.line(lnum).to_vec()
    }
}

/// A line length as a slice length. `ml_get_buf_len` answers 0 for an empty
/// line and never less, so the clamp is a formality the type asks for.
fn to_len(len: ColNr) -> usize {
    usize::try_from(len).unwrap_or(0)
}

/// A pointer to position `pos` of the current buffer.
///
/// # Safety
/// `pos` must be a valid position in the current buffer.
pub unsafe fn ml_get_pos(pos: *const Pos) -> *mut ::core::ffi::c_char {
    unsafe { ml_get_buf(Buf::current(), (*pos).lnum).offset((*pos).col as isize) }
}

/// Length of line `lnum` of the current buffer, excluding the NUL.
///
/// Safe: as [`ml_get`] -- the editor exists, and the line number is
/// clamped.
pub fn ml_get_len(lnum: LineNr) -> ColNr {
    unsafe { ml_get_buf_len(Buf::current(), lnum) }
}

/// Length of the text after position `pos`, excluding the NUL.
///
/// # Safety
/// `pos` must be a valid position in the current buffer.
pub unsafe fn ml_get_pos_len(pos: *mut Pos) -> ColNr {
    unsafe { ml_get_buf_len(Buf::current(), (*pos).lnum) - (*pos).col }
}

/// Length of line `lnum` of `buffer`, excluding the NUL.
///
/// # Safety
/// `buffer` must point at a buffer.
pub unsafe fn ml_get_buf_len(buffer: Buf, lnum: LineNr) -> ColNr {
    // SAFETY: the caller's buffer, reached through a handle that
    // borrows it for the one access that asked and no longer.
    let b = buffer;
    if unsafe { *ml_get_buf(buffer, lnum) } == NUL as ::core::ffi::c_char {
        return 0;
    }
    debug_assert!(b.b_ml.cached_len() > 0);
    (b.b_ml.cached_len()) - 1
}

/// The codepoint at `pos`, which must either be valid or have `col` set to
/// `MAXCOL`.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn gchar_pos(pos: *mut Pos) -> ::core::ffi::c_int {
    // While searching, the column is sometimes put at the end of a line.
    if unsafe { (*pos).col } == MAXCOL as ::core::ffi::c_int
        || unsafe { (*pos).col } > ml_get_len(unsafe { (*pos).lnum })
    {
        return NUL;
    }
    unsafe { utf_ptr2char(ml_get_pos(pos)) }
}

/// Whether the line last handed out by `ml_get` is in allocated memory.
pub fn ml_line_alloced() -> bool {
    Buf::current().b_ml.line_is_dirty()
}

/// Flush any pending change, then insert.
///
/// # Safety
/// `buffer` must point at a buffer with a memline, and `line` hold `len` bytes.
unsafe fn ml_append_flush(
    buffer: Buf,
    lnum: LineNr,
    line: *mut ::core::ffi::c_char,
    len: ColNr,
    flags: ::core::ffi::c_int,
) -> Result<(), Failed> {
    // SAFETY: the caller's buffer, reached through a handle that
    // borrows it for the one access that asked and no longer.
    let b = buffer;
    if lnum > b.b_ml.ml_line_count {
        return Err(Failed); // lnum out of range
    }
    if b.b_ml.cached_lnum() != 0 {
        // This may invoke ml_append_int in turn.
        unsafe { ml_flush_line(buffer, false) };
    }
    unsafe { ml_append_int(buffer, lnum, line, len, flags) }
}

/// Append a line after `lnum` of the current buffer (0 to put it in front of
/// the file).
///
/// `line` need not be allocated, but must not be another line of a buffer —
/// unlocking a block can invalidate that. `len` includes the NUL, or is 0 to
/// measure it. `newfile` says a new file is being read in, which records
/// `pe_old_lnum` for recovery.
///
/// The caller should probably also call `appended_lines`.
///
/// # Safety
/// Must run on the main thread; `line` must hold `len` bytes.
pub unsafe fn ml_append(
    lnum: LineNr,
    line: *mut ::core::ffi::c_char,
    len: ColNr,
    newfile: bool,
) -> Result<(), Failed> {
    unsafe { ml_append_flags(lnum, line, len, if newfile { ML_APPEND_NEW } else { 0 }) }
}

/// [`ml_append`] taking `ML_APPEND_` flags directly.
///
/// # Safety
/// Must run on the main thread; `line` must hold `len` bytes.
pub unsafe fn ml_append_flags(
    lnum: LineNr,
    line: *mut ::core::ffi::c_char,
    len: ColNr,
    flags: ::core::ffi::c_int,
) -> Result<(), Failed> {
    // During startup the memfile may still have to be created.
    if Buf::current().b_ml.ml_mfp.is_null()
        && unsafe { open_buffer(false, ::core::ptr::null_mut(), 0) }.is_err()
    {
        return Err(Failed);
    }
    unsafe { ml_append_flush(Buf::current(), lnum, line, len, flags) }
}

/// [`ml_append`] for an arbitrary buffer, which must already have a memline.
///
/// # Safety
/// `buffer` must point at a buffer; `line` must hold `len` bytes.
pub unsafe fn ml_append_buf(
    buffer: Buf,
    lnum: LineNr,
    line: *mut ::core::ffi::c_char,
    len: ColNr,
    newfile: bool,
) -> Result<(), Failed> {
    // SAFETY: the caller's buffer, reached through a handle that
    // borrows it for the one access that asked and no longer.
    let b = buffer;
    if b.b_ml.ml_mfp.is_null() {
        return Err(Failed);
    }
    let flags = if newfile { ML_APPEND_NEW } else { 0 };
    unsafe { ml_append_flush(buffer, lnum, line, len, flags) }
}

/// Book `len` bytes at `text` as deleted from the current buffer, for the
/// buffer-update callbacks.
///
/// # Safety
/// Must run on the main thread; `text` must be NUL-terminated.
pub unsafe fn ml_add_deleted_len(text: *mut ::core::ffi::c_char, len: ssize_t) {
    unsafe { ml_add_deleted_len_buf(Buf::current(), text, len) }
}

/// [`ml_add_deleted_len`] for an arbitrary buffer. `len` of -1 measures the
/// string.
///
/// # Safety
/// `buffer` must point at a buffer; `text` must be NUL-terminated.
pub unsafe fn ml_add_deleted_len_buf(
    mut buffer: Buf,
    text: *mut ::core::ffi::c_char,
    len_arg: ssize_t,
) {
    // SAFETY: the caller's buffer, reached through a handle that
    // borrows it for the one access that asked and no longer.
    let b = buffer;
    if inhibit_delete_count.get() != 0 {
        return;
    }
    let maxlen = unsafe { cstr::bytes_at(text) }.len() as ssize_t;
    let len = if len_arg == -1 || len_arg > maxlen {
        maxlen
    } else {
        len_arg
    };
    // The + 1 is the newline the line carries internally.
    buffer.deleted_bytes += len as size_t + 1;
    buffer.deleted_bytes2 += len as size_t + 1;
    if b.update_need_codepoints {
        unsafe {
            mb_utflen(
                text,
                len as size_t,
                &raw mut buffer.deleted_codepoints,
                &raw mut buffer.deleted_codeunits,
            )
        };
        buffer.deleted_codepoints += 1; // NL char
        buffer.deleted_codeunits += 1;
    }
}

/// Replace line `lnum` of the current buffer, with buffering.
///
/// # Safety
/// Must run on the main thread; `line` must be NUL-terminated.
pub unsafe fn ml_replace(
    lnum: LineNr,
    line: *mut ::core::ffi::c_char,
    copy: bool,
) -> Result<(), Failed> {
    unsafe { ml_replace_buf(Buf::current(), lnum, line, copy, false) }
}

/// [`ml_replace`] with the length given, excluding the NUL.
///
/// # Safety
/// Must run on the main thread; `line` must hold `len` bytes.
pub unsafe fn ml_replace_len(
    lnum: LineNr,
    line: *mut ::core::ffi::c_char,
    len: size_t,
    copy: bool,
) -> Result<(), Failed> {
    unsafe { ml_replace_buf_len(Buf::current(), lnum, line, len, copy, false) }
}

/// [`ml_replace`] for an arbitrary buffer.
///
/// # Safety
/// `buffer` must point at a buffer; `line` must be NULL or NUL-terminated.
pub unsafe fn ml_replace_buf(
    buffer: Buf,
    lnum: LineNr,
    line: *mut ::core::ffi::c_char,
    copy: bool,
    noalloc: bool,
) -> Result<(), Failed> {
    let len = if line.is_null() {
        -1 as ::core::ffi::c_int as size_t
    } else {
        unsafe { cstr::bytes_at(line) }.len()
    };
    unsafe { ml_replace_buf_len(buffer, lnum, line, len, copy, noalloc) }
}

/// Replace line `lnum` of `buffer`, with buffering: the text is parked in
/// `ml_line_ptr` and only written back by [`ml_flush_line`].
///
/// `copy` duplicates `line`; otherwise `line` is taken over, and may be freed
/// to make room for text properties. `noalloc` says the caller owns the
/// memory and it must not be freed at all — the line is flushed straight
/// back out instead. `len_arg` excludes the NUL.
///
/// The caller should probably also call `changed_lines`, unless it uses
/// `update_screen(UPD_NOT_VALID)`.
///
/// # Safety
/// `buffer` must point at a buffer; `line_arg` must be NULL or hold `len_arg`
/// bytes.
pub unsafe fn ml_replace_buf_len(
    buffer: Buf,
    lnum: LineNr,
    line_arg: *mut ::core::ffi::c_char,
    len_arg: size_t,
    copy: bool,
    noalloc: bool,
) -> Result<(), Failed> {
    // SAFETY: the caller's buffer, reached through a handle that
    // borrows it for the one access that asked and no longer.
    let mut b = buffer;
    if line_arg.is_null() {
        return Err(Failed); // just checking...
    }
    // During startup the memfile may still have to be created.
    if b.b_ml.ml_mfp.is_null() {
        unsafe { open_buffer(false, ::core::ptr::null_mut(), 0) }?;
    }

    let line = if copy {
        debug_assert!(!noalloc);
        unsafe { xmemdupz(line_arg.cast(), len_arg) }.cast::<::core::ffi::c_char>()
    } else {
        line_arg
    };

    if b.b_ml.cached_lnum() != lnum {
        unsafe { ml_flush_line(buffer, false) }; // another line is buffered, flush it
    }
    if b.update_callbacks.size != 0 {
        unsafe { ml_add_deleted_len_buf(buffer, ml_get_buf(buffer, lnum), -1) };
    }
    if let Some(old) = b.b_ml.take_owned() {
        unsafe { xfree(old.cast()) }; // free the allocated line
    }

    let len = len_arg as ColNr + 1;
    b.b_ml.cache_replacement(line, len, lnum);
    if noalloc {
        // Upstream note: a bit of a hack, but replacing lines in a loop
        // is common and a scratch allocation per line is a lot of noise.
        unsafe { ml_flush_line(buffer, true) };
    }
    Ok(())
}

/// Delete line `lnum` of `buffer`.
///
/// The caller should probably also call `changed_lines`.
///
/// # Safety
/// `buffer` must point at a buffer holding line `lnum`.
pub unsafe fn ml_delete_buf(buffer: Buf, lnum: LineNr, message: bool) -> Result<(), Failed> {
    unsafe { ml_flush_line(buffer, false) };
    unsafe { ml_delete_int(buffer, lnum, if message { ML_DEL_MESSAGE } else { 0 }) }
}

/// Delete line `lnum` of the current buffer.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn ml_delete(lnum: LineNr) -> Result<(), Failed> {
    unsafe { ml_delete_flags(lnum, 0) }
}

/// [`ml_delete`] taking `ML_DEL_` flags.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn ml_delete_flags(lnum: LineNr, flags: ::core::ffi::c_int) -> Result<(), Failed> {
    unsafe { ml_flush_line(Buf::current(), false) };
    if lnum < 1 || lnum > Buf::current().b_ml.ml_line_count {
        return Err(Failed);
    }
    unsafe { ml_delete_int(Buf::current(), lnum, flags) }
}

/// Set the [`DB_MARKED`] bit on line `lnum`.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn ml_setmarked(lnum: LineNr) {
    if lnum < 1 || lnum > Buf::current().b_ml.ml_line_count || Buf::current().b_ml.ml_mfp.is_null()
    {
        return; // invalid line number
    }
    if lowest_marked.get() == 0 || lowest_marked.get() > lnum {
        lowest_marked.set(lnum);
    }
    let hp = unsafe { ml_find_line(Buf::current(), lnum, ML_FIND) };
    if hp.is_null() {
        return;
    }
    let dp = unsafe { Db::new((*hp).bh_data.cast()) };
    let idx = (lnum - Buf::current().b_ml.locked_low()) as isize;
    unsafe { *db_index(dp).wrapping_offset(idx) |= DB_MARKED };
    Buf::current().b_ml.locked_is_dirty();
}

/// The first line with its [`DB_MARKED`] bit set, clearing the bit. Zero when
/// there is none left.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn ml_firstmarked() -> LineNr {
    if Buf::current().b_ml.ml_mfp.is_null() {
        return 0;
    }
    // Start at lowest_marked: the last line a mark was found at, kept up
    // to date as lines are inserted and deleted.
    let mut lnum = lowest_marked.get();
    while lnum <= Buf::current().b_ml.ml_line_count {
        let hp = unsafe { ml_find_line(Buf::current(), lnum, ML_FIND) };
        if hp.is_null() {
            return 0;
        }
        let dp = unsafe { Db::new((*hp).bh_data.cast()) };
        let mut i = lnum - Buf::current().b_ml.locked_low();
        while lnum <= Buf::current().b_ml.locked_high() {
            let slot = db_index(dp).wrapping_offset(i as isize);
            if unsafe { *slot } & DB_MARKED != 0 {
                unsafe { *slot &= DB_INDEX_MASK };
                Buf::current().b_ml.locked_is_dirty();
                lowest_marked.set(lnum + 1);
                return lnum;
            }
            i += 1;
            lnum += 1;
        }
    }
    0
}

/// Clear every [`DB_MARKED`] bit.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn ml_clearmarked() {
    if Buf::current().b_ml.ml_mfp.is_null() {
        return; // nothing to do
    }
    let mut lnum = lowest_marked.get();
    while lnum <= Buf::current().b_ml.ml_line_count {
        let hp = unsafe { ml_find_line(Buf::current(), lnum, ML_FIND) };
        if hp.is_null() {
            return;
        }
        let dp = unsafe { Db::new((*hp).bh_data.cast()) };
        let mut i = lnum - Buf::current().b_ml.locked_low();
        while lnum <= Buf::current().b_ml.locked_high() {
            let slot = db_index(dp).wrapping_offset(i as isize);
            if unsafe { *slot } & DB_MARKED != 0 {
                unsafe { *slot &= DB_INDEX_MASK };
                Buf::current().b_ml.locked_is_dirty();
            }
            i += 1;
            lnum += 1;
        }
    }
    lowest_marked.set(0);
}

/// Take and reset the deleted-byte counters the buffer-update callbacks
/// report.
///
/// # Safety
/// `buffer` must point at a buffer; the two out-parameters must be writable.
pub unsafe fn ml_flush_deleted_bytes(
    buffer: Buf,
    codepoints: *mut size_t,
    codeunits: *mut size_t,
) -> size_t {
    // SAFETY: the caller's buffer, reached through a handle that
    // borrows it for the one access that asked and no longer.
    let mut b = buffer;
    let ret = b.deleted_bytes;
    unsafe { *codepoints = buffer.deleted_codepoints };
    unsafe { *codeunits = buffer.deleted_codeunits };
    b.deleted_bytes = 0;
    b.deleted_codepoints = 0;
    b.deleted_codeunits = 0;
    ret
}

/// Advance `pos` by one character, crossing line boundaries as needed.
///
/// Returns 1 when it moved to the next line, 2 when it moved onto the NUL at
/// the end of a line, -1 at the end of the file, and 0 otherwise.
///
/// # Safety
/// Must run on the main thread; `pos` must be a position in the current
/// buffer.
pub unsafe fn inc(pos: &mut Pos) -> ::core::ffi::c_int {
    // While searching, the position may be set to the end of a line.
    if pos.col != MAXCOL as ::core::ffi::c_int {
        let p = unsafe { ml_get_pos(pos) };
        if unsafe { *p } != NUL as ::core::ffi::c_char {
            // Still within the line; move to the next char, which may be
            // the NUL.
            let l = unsafe { utfc_ptr2len(p) };
            pos.col += l;
            return if unsafe { *p.offset(l as isize) } != NUL as ::core::ffi::c_char {
                0
            } else {
                2
            };
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
///
/// # Safety
/// As [`inc`].
pub unsafe fn incl(pos: &mut Pos) -> ::core::ffi::c_int {
    let mut r = unsafe { inc(pos) };
    if r >= 1 && pos.col != 0 {
        r = unsafe { inc(pos) };
    }
    r
}

/// Move `pos` back by one character, crossing line boundaries as needed.
///
/// Returns 1 when it moved to the previous line, -1 at the start of the file,
/// and 0 otherwise.
///
/// # Safety
/// Must run on the main thread; `pos` must be a position in the current
/// buffer.
pub unsafe fn dec(pos: &mut Pos) -> ::core::ffi::c_int {
    pos.coladd = 0;
    if pos.col == MAXCOL as ::core::ffi::c_int {
        // Past the end of the line.
        let p = ml_get(pos.lnum);
        pos.col = ml_get_len(pos.lnum);
        pos.col -= unsafe { utf_head_off(p, p.offset(pos.col as isize)) };
        return 0;
    }
    if pos.col > 0 {
        // Still within the line.
        pos.col -= 1;
        let p = ml_get(pos.lnum);
        pos.col -= unsafe { utf_head_off(p, p.offset(pos.col as isize)) };
        return 0;
    }
    if pos.lnum > 1 {
        // There is a previous line.
        pos.lnum -= 1;
        let p = ml_get(pos.lnum);
        pos.col = ml_get_len(pos.lnum);
        pos.col -= unsafe { utf_head_off(p, p.offset(pos.col as isize)) };
        return 1;
    }
    -1 // at the start of the file
}

/// [`dec`], but skipping the NUL at the end of a non-empty line.
///
/// # Safety
/// As [`dec`].
pub unsafe fn decl(pos: &mut Pos) -> ::core::ffi::c_int {
    let mut r = unsafe { dec(pos) };
    if r == 1 && pos.col != 0 {
        r = unsafe { dec(pos) };
    }
    r
}
