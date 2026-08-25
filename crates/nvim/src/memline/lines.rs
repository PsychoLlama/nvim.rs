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
use crate::types::{FAIL, NUL, OK};

/// A read-only pointer to line `lnum` of the current buffer. Never NULL.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn ml_get(lnum: linenr_T) -> *mut ::core::ffi::c_char {
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
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn ml_get_len(lnum: linenr_T) -> colnr_T {
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
    unsafe {
        if *ml_get_buf(buf, lnum) == NUL as ::core::ffi::c_char {
            return 0;
        }
        debug_assert!((*buf).b_ml.ml_line_textlen > 0);
        (*buf).b_ml.ml_line_textlen - 1
    }
}

/// The codepoint at `pos`, which must either be valid or have `col` set to
/// `MAXCOL`.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn gchar_pos(pos: *mut pos_T) -> ::core::ffi::c_int {
    unsafe {
        // While searching, the column is sometimes put at the end of a line.
        if (*pos).col == MAXCOL as ::core::ffi::c_int || (*pos).col > ml_get_len((*pos).lnum) {
            return NUL;
        }
        utf_ptr2char(ml_get_pos(pos))
    }
}

/// Whether the line last handed out by `ml_get` is in allocated memory.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn ml_line_alloced() -> bool {
    unsafe { (*curbuf.get()).b_ml.ml_flags.has(MlFlags::LINE_DIRTY) }
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
) -> ::core::ffi::c_int {
    unsafe {
        if lnum > (*buf).b_ml.ml_line_count {
            return FAIL; // lnum out of range
        }
        if (*buf).b_ml.ml_line_lnum != 0 {
            // This may invoke ml_append_int in turn.
            ml_flush_line(buf, false);
        }
        ml_append_int(buf, lnum, line, len, flags)
    }
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
) -> ::core::ffi::c_int {
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
) -> ::core::ffi::c_int {
    unsafe {
        // During startup the memfile may still have to be created.
        if (*curbuf.get()).b_ml.ml_mfp.is_null()
            && open_buffer(false, ::core::ptr::null_mut(), 0) == FAIL
        {
            return FAIL;
        }
        ml_append_flush(curbuf.get(), lnum, line, len, flags)
    }
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
) -> ::core::ffi::c_int {
    unsafe {
        if (*buf).b_ml.ml_mfp.is_null() {
            return FAIL;
        }
        ml_append_flush(
            buf,
            lnum,
            line,
            len,
            if newfile { ML_APPEND_NEW } else { 0 },
        )
    }
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
    unsafe {
        if inhibit_delete_count.get() != 0 {
            return;
        }
        let maxlen = strlen(ptr) as ssize_t;
        let len = if len_arg == -1 || len_arg > maxlen {
            maxlen
        } else {
            len_arg
        };
        // The + 1 is the newline the line carries internally.
        (*buf).deleted_bytes += len as size_t + 1;
        (*buf).deleted_bytes2 += len as size_t + 1;
        if (*buf).update_need_codepoints {
            mb_utflen(
                ptr,
                len as size_t,
                &raw mut (*buf).deleted_codepoints,
                &raw mut (*buf).deleted_codeunits,
            );
            (*buf).deleted_codepoints += 1; // NL char
            (*buf).deleted_codeunits += 1;
        }
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
) -> ::core::ffi::c_int {
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
) -> ::core::ffi::c_int {
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
) -> ::core::ffi::c_int {
    unsafe {
        let len = if line.is_null() {
            -1 as ::core::ffi::c_int as size_t
        } else {
            strlen(line)
        };
        ml_replace_buf_len(buf, lnum, line, len, copy, noalloc)
    }
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
) -> ::core::ffi::c_int {
    unsafe {
        if line_arg.is_null() {
            return FAIL; // just checking...
        }
        // During startup the memfile may still have to be created.
        if (*buf).b_ml.ml_mfp.is_null() && open_buffer(false, ::core::ptr::null_mut(), 0) == FAIL {
            return FAIL;
        }

        let line = if copy {
            debug_assert!(!noalloc);
            xmemdupz(line_arg.cast(), len_arg).cast::<::core::ffi::c_char>()
        } else {
            line_arg
        };

        if (*buf).b_ml.ml_line_lnum != lnum {
            ml_flush_line(buf, false); // another line is buffered, flush it
        }
        if (*buf).update_callbacks.size != 0 {
            ml_add_deleted_len_buf(buf, ml_get_buf(buf, lnum), -1);
        }
        if (*buf).b_ml.line_is_owned() {
            xfree((*buf).b_ml.ml_line_ptr.cast()); // free the allocated line
        }

        (*buf).b_ml.ml_line_ptr = line;
        (*buf).b_ml.ml_line_textlen = len_arg as colnr_T + 1;
        (*buf).b_ml.ml_line_lnum = lnum;
        (*buf).b_ml.line_was_replaced();
        if noalloc {
            // Upstream note: a bit of a hack, but replacing lines in a loop
            // is common and a scratch allocation per line is a lot of noise.
            ml_flush_line(buf, true);
        }
        OK
    }
}

/// Delete line `lnum` of `buf`.
///
/// The caller should probably also call `changed_lines`.
///
/// # Safety
/// `buf` must point at a buffer holding line `lnum`.
pub unsafe fn ml_delete_buf(buf: *mut buf_T, lnum: linenr_T, message: bool) -> ::core::ffi::c_int {
    unsafe {
        ml_flush_line(buf, false);
        ml_delete_int(buf, lnum, if message { ML_DEL_MESSAGE } else { 0 })
    }
}

/// Delete line `lnum` of the current buffer.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn ml_delete(lnum: linenr_T) -> ::core::ffi::c_int {
    unsafe { ml_delete_flags(lnum, 0) }
}

/// [`ml_delete`] taking `ML_DEL_` flags.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn ml_delete_flags(lnum: linenr_T, flags: ::core::ffi::c_int) -> ::core::ffi::c_int {
    unsafe {
        ml_flush_line(curbuf.get(), false);
        if lnum < 1 || lnum > (*curbuf.get()).b_ml.ml_line_count {
            return FAIL;
        }
        ml_delete_int(curbuf.get(), lnum, flags)
    }
}

/// Set the [`DB_MARKED`] bit on line `lnum`.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn ml_setmarked(lnum: linenr_T) {
    unsafe {
        if lnum < 1
            || lnum > (*curbuf.get()).b_ml.ml_line_count
            || (*curbuf.get()).b_ml.ml_mfp.is_null()
        {
            return; // invalid line number
        }
        if lowest_marked.get() == 0 || lowest_marked.get() > lnum {
            lowest_marked.set(lnum);
        }
        let hp = ml_find_line(curbuf.get(), lnum, ML_FIND);
        if hp.is_null() {
            return;
        }
        let dp = (*hp).bh_data as *mut DataBlock;
        *db_index(dp).offset((lnum - (*curbuf.get()).b_ml.ml_locked_low) as isize) |= DB_MARKED;
        (*curbuf.get()).b_ml.ml_flags |= MlFlags::LOCKED_DIRTY;
    }
}

/// The first line with its [`DB_MARKED`] bit set, clearing the bit. Zero when
/// there is none left.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn ml_firstmarked() -> linenr_T {
    unsafe {
        if (*curbuf.get()).b_ml.ml_mfp.is_null() {
            return 0;
        }
        // Start at lowest_marked: the last line a mark was found at, kept up
        // to date as lines are inserted and deleted.
        let mut lnum = lowest_marked.get();
        while lnum <= (*curbuf.get()).b_ml.ml_line_count {
            let hp = ml_find_line(curbuf.get(), lnum, ML_FIND);
            if hp.is_null() {
                return 0;
            }
            let dp = (*hp).bh_data as *mut DataBlock;
            let mut i = lnum - (*curbuf.get()).b_ml.ml_locked_low;
            while lnum <= (*curbuf.get()).b_ml.ml_locked_high {
                let slot = db_index(dp).offset(i as isize);
                if *slot & DB_MARKED != 0 {
                    *slot &= DB_INDEX_MASK;
                    (*curbuf.get()).b_ml.ml_flags |= MlFlags::LOCKED_DIRTY;
                    lowest_marked.set(lnum + 1);
                    return lnum;
                }
                i += 1;
                lnum += 1;
            }
        }
        0
    }
}

/// Clear every [`DB_MARKED`] bit.
///
/// # Safety
/// Must run on the main thread, with a current buffer.
pub unsafe fn ml_clearmarked() {
    unsafe {
        if (*curbuf.get()).b_ml.ml_mfp.is_null() {
            return; // nothing to do
        }
        let mut lnum = lowest_marked.get();
        while lnum <= (*curbuf.get()).b_ml.ml_line_count {
            let hp = ml_find_line(curbuf.get(), lnum, ML_FIND);
            if hp.is_null() {
                return;
            }
            let dp = (*hp).bh_data as *mut DataBlock;
            let mut i = lnum - (*curbuf.get()).b_ml.ml_locked_low;
            while lnum <= (*curbuf.get()).b_ml.ml_locked_high {
                let slot = db_index(dp).offset(i as isize);
                if *slot & DB_MARKED != 0 {
                    *slot &= DB_INDEX_MASK;
                    (*curbuf.get()).b_ml.ml_flags |= MlFlags::LOCKED_DIRTY;
                }
                i += 1;
                lnum += 1;
            }
        }
        lowest_marked.set(0);
    }
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
    unsafe {
        let ret = (*buf).deleted_bytes;
        *codepoints = (*buf).deleted_codepoints;
        *codeunits = (*buf).deleted_codeunits;
        (*buf).deleted_bytes = 0;
        (*buf).deleted_codepoints = 0;
        (*buf).deleted_codeunits = 0;
        ret
    }
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
    unsafe {
        // While searching, the position may be set to the end of a line.
        if lp.col != MAXCOL as ::core::ffi::c_int {
            let p = ml_get_pos(lp);
            if *p != NUL as ::core::ffi::c_char {
                // Still within the line; move to the next char, which may be
                // the NUL.
                let l = utfc_ptr2len(p);
                lp.col += l;
                return if *p.offset(l as isize) != NUL as ::core::ffi::c_char {
                    0
                } else {
                    2
                };
            }
        }
        if lp.lnum != (*curbuf.get()).b_ml.ml_line_count {
            // There is a next line.
            lp.col = 0;
            lp.lnum += 1;
            lp.coladd = 0;
            return 1;
        }
        -1
    }
}

/// [`inc`], but skipping the NUL at the end of a non-empty line.
///
/// # Safety
/// As [`inc`].
pub unsafe fn incl(lp: &mut pos_T) -> ::core::ffi::c_int {
    unsafe {
        let mut r = inc(lp);
        if r >= 1 && lp.col != 0 {
            r = inc(lp);
        }
        r
    }
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
    unsafe {
        lp.coladd = 0;
        if lp.col == MAXCOL as ::core::ffi::c_int {
            // Past the end of the line.
            let p = ml_get(lp.lnum);
            lp.col = ml_get_len(lp.lnum);
            lp.col -= utf_head_off(p, p.offset(lp.col as isize));
            return 0;
        }
        if lp.col > 0 {
            // Still within the line.
            lp.col -= 1;
            let p = ml_get(lp.lnum);
            lp.col -= utf_head_off(p, p.offset(lp.col as isize));
            return 0;
        }
        if lp.lnum > 1 {
            // There is a previous line.
            lp.lnum -= 1;
            let p = ml_get(lp.lnum);
            lp.col = ml_get_len(lp.lnum);
            lp.col -= utf_head_off(p, p.offset(lp.col as isize));
            return 1;
        }
        -1 // at the start of the file
    }
}

/// [`dec`], but skipping the NUL at the end of a non-empty line.
///
/// # Safety
/// As [`dec`].
pub unsafe fn decl(lp: &mut pos_T) -> ::core::ffi::c_int {
    unsafe {
        let mut r = dec(lp);
        if r == 1 && lp.col != 0 {
            r = dec(lp);
        }
        r
    }
}
