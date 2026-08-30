//! The line API the rest of the editor calls.
//!
//! Everything here is a thin front for the tree in
//! [`tree`](super::tree) and [`edit`](super::edit): flush whatever line
//! `ml_replace` left pending, check the line number, dispatch. The one
//! thing that lives here in its own right is the `DB_MARKED` bit, which
//! `:global` uses to remember which lines it still has to visit.

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::pos::MAXCOL;
use crate::types::{Failed, NUL};
use crate::winlayer::Buf;

/// A read-only pointer to line `lnum` of the current buffer. Never NULL.
///
/// Safe: the only promise is that the editor exists, which `curbuf` carries
/// from startup to exit, and `ml_get_buf_impl` clamps `lnum` into the
/// buffer itself. The answer is a raw pointer, so *reading* through it is
/// still the caller's business.
pub fn ml_get(lnum: linenr_T) -> *mut ::core::ffi::c_char {
    unsafe { ml_get_buf_impl(curbuf.get(), lnum, false) }
}

/// [`ml_get`] for an arbitrary buffer.
///
/// # Safety
/// `buf` must point at a buffer.
pub unsafe fn ml_get_buf(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char {
    unsafe { ml_get_buf_impl(buf, lnum, false) }
}

/// [`ml_get_buf`], but the line may be changed through the pointer.
///
/// Very limited: only the bytes already there can be rewritten. Use
/// [`ml_replace_buf`] for anything else.
///
/// # Safety
/// `buf` must point at a buffer.
pub unsafe fn ml_get_buf_mut(buf: *mut buf_T, lnum: linenr_T) -> *mut ::core::ffi::c_char {
    unsafe { ml_get_buf_impl(buf, lnum, true) }
}

/// A pointer to position `pos` of the current buffer.
///
/// # Safety
/// `pos` must be a valid position in the current buffer.
pub unsafe fn ml_get_pos(pos: *const pos_T) -> *mut ::core::ffi::c_char {
    unsafe { ml_get_buf(curbuf.get(), (*pos).lnum).offset((*pos).col as isize) }
}

/// Length of line `lnum` of the current buffer, excluding the NUL.
///
/// Safe: as [`ml_get`] -- the editor exists, and the line number is
/// clamped.
pub fn ml_get_len(lnum: linenr_T) -> colnr_T {
    unsafe { ml_get_buf_len(curbuf.get(), lnum) }
}

/// Length of the text after position `pos`, excluding the NUL.
///
/// # Safety
/// `pos` must be a valid position in the current buffer.
pub unsafe fn ml_get_pos_len(pos: *mut pos_T) -> colnr_T {
    unsafe { ml_get_buf_len(curbuf.get(), (*pos).lnum) - (*pos).col }
}

/// Length of line `lnum` of `buf`, excluding the NUL.
///
/// # Safety
/// `buf` must point at a buffer.
pub unsafe fn ml_get_buf_len(buf: *mut buf_T, lnum: linenr_T) -> colnr_T {
    // SAFETY: the caller's buffer, reached through a handle that
    // borrows it for the one access that asked and no longer.
    let mut b = unsafe { Buf::new(buf) };
    if unsafe { *ml_get_buf(buf, lnum) } == NUL as ::core::ffi::c_char {
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
pub unsafe fn gchar_pos(pos: *mut pos_T) -> ::core::ffi::c_int {
    // While searching, the column is sometimes put at the end of a line.
    if unsafe { (*pos).col } == MAXCOL as ::core::ffi::c_int
        || unsafe { (*pos).col } > ml_get_len(unsafe { (*pos).lnum })
    {
        return NUL;
    }
    unsafe { utf_ptr2char(ml_get_pos(pos)) }
}

/// Whether the line last handed out by `ml_get` is in allocated memory.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn ml_line_alloced() -> bool {
    cur_buf().b_ml.line_is_dirty()
}

/// Flush any pending change, then insert.
///
/// # Safety
/// `buf` must point at a buffer with a memline, and `line` hold `len` bytes.
unsafe fn ml_append_flush(
    buf: *mut buf_T,
    lnum: linenr_T,
    line: *mut ::core::ffi::c_char,
    len: colnr_T,
    flags: ::core::ffi::c_int,
) -> Result<(), Failed> {
    // SAFETY: the caller's buffer, reached through a handle that
    // borrows it for the one access that asked and no longer.
    let mut b = unsafe { Buf::new(buf) };
    if lnum > b.b_ml.ml_line_count {
        return Err(Failed); // lnum out of range
    }
    if b.b_ml.cached_lnum() != 0 {
        // This may invoke ml_append_int in turn.
        unsafe { ml_flush_line(buf, false) };
    }
    unsafe { ml_append_int(buf, lnum, line, len, flags) }
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
    lnum: linenr_T,
    line: *mut ::core::ffi::c_char,
    len: colnr_T,
    newfile: bool,
) -> Result<(), Failed> {
    unsafe { ml_append_flags(lnum, line, len, if newfile { ML_APPEND_NEW } else { 0 }) }
}

/// [`ml_append`] taking `ML_APPEND_` flags directly.
///
/// # Safety
/// Must run on the main thread; `line` must hold `len` bytes.
pub unsafe fn ml_append_flags(
    lnum: linenr_T,
    line: *mut ::core::ffi::c_char,
    len: colnr_T,
    flags: ::core::ffi::c_int,
) -> Result<(), Failed> {
    // During startup the memfile may still have to be created.
    if cur_buf().b_ml.ml_mfp.is_null()
        && unsafe { open_buffer(false, ::core::ptr::null_mut(), 0) }.is_err()
    {
        return Err(Failed);
    }
    unsafe { ml_append_flush(curbuf.get(), lnum, line, len, flags) }
}

/// [`ml_append`] for an arbitrary buffer, which must already have a memline.
///
/// # Safety
/// `buf` must point at a buffer; `line` must hold `len` bytes.
pub unsafe fn ml_append_buf(
    buf: *mut buf_T,
    lnum: linenr_T,
    line: *mut ::core::ffi::c_char,
    len: colnr_T,
    newfile: bool,
) -> Result<(), Failed> {
    // SAFETY: the caller's buffer, reached through a handle that
    // borrows it for the one access that asked and no longer.
    let mut b = unsafe { Buf::new(buf) };
    if b.b_ml.ml_mfp.is_null() {
        return Err(Failed);
    }
    let flags = if newfile { ML_APPEND_NEW } else { 0 };
    unsafe { ml_append_flush(buf, lnum, line, len, flags) }
}

/// Book `len` bytes at `ptr` as deleted from the current buffer, for the
/// buffer-update callbacks.
///
/// # Safety
/// Must run on the main thread; `ptr` must be NUL-terminated.
pub unsafe fn ml_add_deleted_len(ptr: *mut ::core::ffi::c_char, len: ssize_t) {
    unsafe { ml_add_deleted_len_buf(curbuf.get(), ptr, len) }
}

/// [`ml_add_deleted_len`] for an arbitrary buffer. `len` of -1 measures the
/// string.
///
/// # Safety
/// `buf` must point at a buffer; `ptr` must be NUL-terminated.
pub unsafe fn ml_add_deleted_len_buf(
    buf: *mut buf_T,
    ptr: *mut ::core::ffi::c_char,
    len_arg: ssize_t,
) {
    // SAFETY: the caller's buffer, reached through a handle that
    // borrows it for the one access that asked and no longer.
    let mut b = unsafe { Buf::new(buf) };
    if inhibit_delete_count.get() != 0 {
        return;
    }
    let maxlen = unsafe { strlen(ptr) } as ssize_t;
    let len = if len_arg == -1 || len_arg > maxlen {
        maxlen
    } else {
        len_arg
    };
    // The + 1 is the newline the line carries internally.
    unsafe { (*buf).deleted_bytes += len as size_t + 1 };
    unsafe { (*buf).deleted_bytes2 += len as size_t + 1 };
    if b.update_need_codepoints {
        unsafe {
            mb_utflen(
                ptr,
                len as size_t,
                &raw mut (*buf).deleted_codepoints,
                &raw mut (*buf).deleted_codeunits,
            )
        };
        unsafe { (*buf).deleted_codepoints += 1 }; // NL char
        unsafe { (*buf).deleted_codeunits += 1 };
    }
}

/// Replace line `lnum` of the current buffer, with buffering.
///
/// # Safety
/// Must run on the main thread; `line` must be NUL-terminated.
pub unsafe fn ml_replace(
    lnum: linenr_T,
    line: *mut ::core::ffi::c_char,
    copy: bool,
) -> Result<(), Failed> {
    unsafe { ml_replace_buf(curbuf.get(), lnum, line, copy, false) }
}

/// [`ml_replace`] with the length given, excluding the NUL.
///
/// # Safety
/// Must run on the main thread; `line` must hold `len` bytes.
pub unsafe fn ml_replace_len(
    lnum: linenr_T,
    line: *mut ::core::ffi::c_char,
    len: size_t,
    copy: bool,
) -> Result<(), Failed> {
    unsafe { ml_replace_buf_len(curbuf.get(), lnum, line, len, copy, false) }
}

/// [`ml_replace`] for an arbitrary buffer.
///
/// # Safety
/// `buf` must point at a buffer; `line` must be NULL or NUL-terminated.
pub unsafe fn ml_replace_buf(
    buf: *mut buf_T,
    lnum: linenr_T,
    line: *mut ::core::ffi::c_char,
    copy: bool,
    noalloc: bool,
) -> Result<(), Failed> {
    let len = if line.is_null() {
        -1 as ::core::ffi::c_int as size_t
    } else {
        unsafe { strlen(line) }
    };
    unsafe { ml_replace_buf_len(buf, lnum, line, len, copy, noalloc) }
}

/// Replace line `lnum` of `buf`, with buffering: the text is parked in
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
/// `buf` must point at a buffer; `line_arg` must be NULL or hold `len_arg`
/// bytes.
pub unsafe fn ml_replace_buf_len(
    buf: *mut buf_T,
    lnum: linenr_T,
    line_arg: *mut ::core::ffi::c_char,
    len_arg: size_t,
    copy: bool,
    noalloc: bool,
) -> Result<(), Failed> {
    // SAFETY: the caller's buffer, reached through a handle that
    // borrows it for the one access that asked and no longer.
    let mut b = unsafe { Buf::new(buf) };
    if line_arg.is_null() {
        return Err(Failed); // just checking...
    }
    // During startup the memfile may still have to be created.
    if b.b_ml.ml_mfp.is_null() && unsafe { open_buffer(false, ::core::ptr::null_mut(), 0) }.is_err()
    {
        return Err(Failed);
    }

    let line = if copy {
        debug_assert!(!noalloc);
        unsafe { xmemdupz(line_arg.cast(), len_arg) }.cast::<::core::ffi::c_char>()
    } else {
        line_arg
    };

    if b.b_ml.cached_lnum() != lnum {
        unsafe { ml_flush_line(buf, false) }; // another line is buffered, flush it
    }
    if b.update_callbacks.size != 0 {
        unsafe { ml_add_deleted_len_buf(buf, ml_get_buf(buf, lnum), -1) };
    }
    if let Some(old) = b.b_ml.take_owned() {
        unsafe { xfree(old.cast()) }; // free the allocated line
    }

    let len = len_arg as colnr_T + 1;
    b.b_ml.cache_replacement(line, len, lnum);
    if noalloc {
        // Upstream note: a bit of a hack, but replacing lines in a loop
        // is common and a scratch allocation per line is a lot of noise.
        unsafe { ml_flush_line(buf, true) };
    }
    Ok(())
}

/// Delete line `lnum` of `buf`.
///
/// The caller should probably also call `changed_lines`.
///
/// # Safety
/// `buf` must point at a buffer holding line `lnum`.
pub unsafe fn ml_delete_buf(buf: *mut buf_T, lnum: linenr_T, message: bool) -> Result<(), Failed> {
    unsafe { ml_flush_line(buf, false) };
    unsafe { ml_delete_int(buf, lnum, if message { ML_DEL_MESSAGE } else { 0 }) }
}

/// Delete line `lnum` of the current buffer.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn ml_delete(lnum: linenr_T) -> Result<(), Failed> {
    unsafe { ml_delete_flags(lnum, 0) }
}

/// [`ml_delete`] taking `ML_DEL_` flags.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn ml_delete_flags(lnum: linenr_T, flags: ::core::ffi::c_int) -> Result<(), Failed> {
    unsafe { ml_flush_line(curbuf.get(), false) };
    if lnum < 1 || lnum > cur_buf().b_ml.ml_line_count {
        return Err(Failed);
    }
    unsafe { ml_delete_int(curbuf.get(), lnum, flags) }
}

/// Set the [`DB_MARKED`] bit on line `lnum`.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn ml_setmarked(lnum: linenr_T) {
    if lnum < 1 || lnum > cur_buf().b_ml.ml_line_count || cur_buf().b_ml.ml_mfp.is_null() {
        return; // invalid line number
    }
    if lowest_marked.get() == 0 || lowest_marked.get() > lnum {
        lowest_marked.set(lnum);
    }
    let hp = unsafe { ml_find_line(curbuf.get(), lnum, ML_FIND) };
    if hp.is_null() {
        return;
    }
    let dp = unsafe { Db::new((*hp).bh_data.cast()) };
    unsafe {
        *db_index(dp).wrapping_offset((lnum - cur_buf().b_ml.locked_low()) as isize) |= DB_MARKED
    };
    cur_buf().b_ml.locked_is_dirty();
}

/// The first line with its [`DB_MARKED`] bit set, clearing the bit. Zero when
/// there is none left.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn ml_firstmarked() -> linenr_T {
    if cur_buf().b_ml.ml_mfp.is_null() {
        return 0;
    }
    // Start at lowest_marked: the last line a mark was found at, kept up
    // to date as lines are inserted and deleted.
    let mut lnum = lowest_marked.get();
    while lnum <= cur_buf().b_ml.ml_line_count {
        let hp = unsafe { ml_find_line(curbuf.get(), lnum, ML_FIND) };
        if hp.is_null() {
            return 0;
        }
        let dp = unsafe { Db::new((*hp).bh_data.cast()) };
        let mut i = lnum - cur_buf().b_ml.locked_low();
        while lnum <= cur_buf().b_ml.locked_high() {
            let slot = db_index(dp).wrapping_offset(i as isize);
            if unsafe { *slot } & DB_MARKED != 0 {
                unsafe { *slot &= DB_INDEX_MASK };
                cur_buf().b_ml.locked_is_dirty();
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
    if cur_buf().b_ml.ml_mfp.is_null() {
        return; // nothing to do
    }
    let mut lnum = lowest_marked.get();
    while lnum <= cur_buf().b_ml.ml_line_count {
        let hp = unsafe { ml_find_line(curbuf.get(), lnum, ML_FIND) };
        if hp.is_null() {
            return;
        }
        let dp = unsafe { Db::new((*hp).bh_data.cast()) };
        let mut i = lnum - cur_buf().b_ml.locked_low();
        while lnum <= cur_buf().b_ml.locked_high() {
            let slot = db_index(dp).wrapping_offset(i as isize);
            if unsafe { *slot } & DB_MARKED != 0 {
                unsafe { *slot &= DB_INDEX_MASK };
                cur_buf().b_ml.locked_is_dirty();
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
/// `buf` must point at a buffer; the two out-parameters must be writable.
pub unsafe fn ml_flush_deleted_bytes(
    buf: *mut buf_T,
    codepoints: *mut size_t,
    codeunits: *mut size_t,
) -> size_t {
    // SAFETY: the caller's buffer, reached through a handle that
    // borrows it for the one access that asked and no longer.
    let mut b = unsafe { Buf::new(buf) };
    let ret = b.deleted_bytes;
    unsafe { *codepoints = (*buf).deleted_codepoints };
    unsafe { *codeunits = (*buf).deleted_codeunits };
    b.deleted_bytes = 0;
    b.deleted_codepoints = 0;
    b.deleted_codeunits = 0;
    ret
}

/// Advance `lp` by one character, crossing line boundaries as needed.
///
/// Returns 1 when it moved to the next line, 2 when it moved onto the NUL at
/// the end of a line, -1 at the end of the file, and 0 otherwise.
///
/// # Safety
/// Must run on the main thread; `lp` must be a position in the current
/// buffer.
pub unsafe fn inc(lp: &mut pos_T) -> ::core::ffi::c_int {
    // While searching, the position may be set to the end of a line.
    if lp.col != MAXCOL as ::core::ffi::c_int {
        let p = unsafe { ml_get_pos(lp) };
        if unsafe { *p } != NUL as ::core::ffi::c_char {
            // Still within the line; move to the next char, which may be
            // the NUL.
            let l = unsafe { utfc_ptr2len(p) };
            lp.col += l;
            return if unsafe { *p.offset(l as isize) } != NUL as ::core::ffi::c_char {
                0
            } else {
                2
            };
        }
    }
    if lp.lnum != cur_buf().b_ml.ml_line_count {
        // There is a next line.
        lp.col = 0;
        lp.lnum += 1;
        lp.coladd = 0;
        return 1;
    }
    -1
}

/// [`inc`], but skipping the NUL at the end of a non-empty line.
///
/// # Safety
/// As [`inc`].
pub unsafe fn incl(lp: &mut pos_T) -> ::core::ffi::c_int {
    let mut r = unsafe { inc(lp) };
    if r >= 1 && lp.col != 0 {
        r = unsafe { inc(lp) };
    }
    r
}

/// Move `lp` back by one character, crossing line boundaries as needed.
///
/// Returns 1 when it moved to the previous line, -1 at the start of the file,
/// and 0 otherwise.
///
/// # Safety
/// Must run on the main thread; `lp` must be a position in the current
/// buffer.
pub unsafe fn dec(lp: &mut pos_T) -> ::core::ffi::c_int {
    lp.coladd = 0;
    if lp.col == MAXCOL as ::core::ffi::c_int {
        // Past the end of the line.
        let p = ml_get(lp.lnum);
        lp.col = ml_get_len(lp.lnum);
        lp.col -= unsafe { utf_head_off(p, p.offset(lp.col as isize)) };
        return 0;
    }
    if lp.col > 0 {
        // Still within the line.
        lp.col -= 1;
        let p = ml_get(lp.lnum);
        lp.col -= unsafe { utf_head_off(p, p.offset(lp.col as isize)) };
        return 0;
    }
    if lp.lnum > 1 {
        // There is a previous line.
        lp.lnum -= 1;
        let p = ml_get(lp.lnum);
        lp.col = ml_get_len(lp.lnum);
        lp.col -= unsafe { utf_head_off(p, p.offset(lp.col as isize)) };
        return 1;
    }
    -1 // at the start of the file
}

/// [`dec`], but skipping the NUL at the end of a non-empty line.
///
/// # Safety
/// As [`dec`].
pub unsafe fn decl(lp: &mut pos_T) -> ::core::ffi::c_int {
    let mut r = unsafe { dec(lp) };
    if r == 1 && lp.col != 0 {
        r = unsafe { dec(lp) };
    }
    r
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}
