//! Byte offsets: `line2byte`, `byte2line` and `'go'`'s
//! character count.
//!
//! The tree does not store byte offsets, so `ml_find_line_or_offset` walks it
//! adding up block sizes. `ml_updatechunk` maintains the `b_ml.ml_chunks`
//! accelerator that keeps that walk from being O(lines) on every call: a run
//! of between `MLCS_MINL` and `MLCS_MAXL` consecutive lines, with their total
//! byte size, so the walk can skip whole runs and only visit blocks inside
//! the run the answer falls in.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_int;

use super::*;
use crate::pos::MAXCOL;
use crate::winlayer::Win;

/// Where `ml_updatechunk` last left off. Appending runs down the buffer in
/// order, so the next call almost always wants the same chunk or the next
/// one, and the search from chunk zero can be skipped.
///
/// Setting `ml_upd_lastbuf` to null invalidates the lot.
static ml_upd_lastbuf: GlobalCell<*mut buf_T> = GlobalCell::new(core::ptr::null_mut());
static ml_upd_lastline: GlobalCell<linenr_T> = GlobalCell::new(0);
static ml_upd_lastcurline: GlobalCell<linenr_T> = GlobalCell::new(0);
static ml_upd_lastcurix: GlobalCell<usize> = GlobalCell::new(0);

/// Keep the chunk table up to date for a line that was added, removed or
/// resized.
///
/// `updtype` is [`ML_CHNK_ADDLINE`] (add `len` to the chunk, splitting it if
/// it got too long — careful, this can call `ml_find_line`),
/// [`ML_CHNK_DELLINE`] (subtract `len`, possibly merging the chunk away) or
/// [`ML_CHNK_UPDLINE`] (add `len` as a signed quantity).
///
/// # Safety
/// `buf` must point at a buffer.
pub(crate) unsafe fn ml_updatechunk(
    buf: *mut buf_T,
    line: linenr_T,
    len_arg: c_int,
    updtype: c_int,
) {
    unsafe {
        let mut curline = ml_upd_lastcurline.get();
        let mut curix = ml_upd_lastcurix.get();

        if (*buf).b_ml.ml_chunks.is_off() || len_arg == 0 {
            return;
        }
        if !(*buf).b_ml.ml_chunks.is_built() {
            (*buf).b_ml.ml_chunks.build();
        }

        if updtype == ML_CHNK_UPDLINE && (*buf).b_ml.ml_line_count == 1 {
            // First line in an empty buffer, from ml_flush_line: reset.
            let textlen = (*buf).b_ml.cached_len();
            (*buf).b_ml.ml_chunks.reset_to_one(textlen);
            return;
        }

        // Find the chunk the line belongs to; `curline` ends up at the start
        // of it.
        if buf != ml_upd_lastbuf.get()
            || line != ml_upd_lastline.get() + 1
            || updtype != ML_CHNK_ADDLINE
        {
            curline = 1;
            curix = 0;
            while curix + 1 < (*buf).b_ml.ml_chunks.len()
                && line >= curline + (*buf).b_ml.ml_chunks.lines(curix)
            {
                curline += (*buf).b_ml.ml_chunks.lines(curix);
                curix += 1;
            }
        } else if curix + 1 < (*buf).b_ml.ml_chunks.len()
            && line >= curline + (*buf).b_ml.ml_chunks.lines(curix)
        {
            // The cached position is one chunk stale; step it on.
            curline += (*buf).b_ml.ml_chunks.lines(curix);
            curix += 1;
        }

        let len = if updtype == ML_CHNK_DELLINE {
            -len_arg
        } else {
            len_arg
        };
        (*buf).b_ml.ml_chunks.add_size(curix, len);

        if updtype == ML_CHNK_ADDLINE {
            if !ml_chunk_addline(buf, line, curline, curix) {
                return;
            }
        } else if updtype == ML_CHNK_DELLINE {
            ml_upd_lastbuf.set(core::ptr::null_mut()); // force a recalc
            (*buf).b_ml.ml_chunks.delete_line(curix, MLCS_MINL);
            return;
        }

        ml_upd_lastbuf.set(buf);
        ml_upd_lastline.set(line);
        ml_upd_lastcurline.set(curline);
        ml_upd_lastcurix.set(curix);
    }
}

/// A line was added to chunk `curix`. Returns false if the caller must return
/// without caching its position — either the chunk was split (which
/// invalidates the cache) or the walk failed.
///
/// # Safety
/// `buf` must point at a buffer whose chunk index has a chunk `curix`.
unsafe fn ml_chunk_addline(
    buf: *mut buf_T,
    line: linenr_T,
    curline: linenr_T,
    curix: usize,
) -> bool {
    unsafe {
        (*buf).b_ml.ml_chunks.add_lines(curix, 1);

        if (*buf).b_ml.ml_chunks.lines(curix) >= MLCS_MAXL {
            return ml_chunk_split(buf, curix, curline);
        }

        if (*buf).b_ml.ml_chunks.lines(curix) >= MLCS_MINL
            && curix + 1 == (*buf).b_ml.ml_chunks.len()
            && (*buf).b_ml.ml_line_count - line <= 1
        {
            // This is the last chunk and starting another one after it is
            // cheap right now, so do it and save the walk later on.
            (*buf).b_ml.ml_chunks.push_empty();
            if line != (*buf).b_ml.ml_line_count {
                // The line is just before the last one, so move the last
                // line's size over. This is the common case while loading a
                // file.
                let hp = ml_find_line(buf, (*buf).b_ml.ml_line_count, ML_FIND);
                if hp.is_null() {
                    (*buf).b_ml.ml_chunks.switch_off();
                    return false;
                }
                let dp = (*hp).bh_data as *mut DataBlock;
                let rest = if (*dp).db_line_count == 1 {
                    (*dp).db_txt_end.wrapping_sub((*dp).db_txt_start) as c_int
                } else {
                    db_line_start(dp, ((*dp).db_line_count - 2) as c_int) as c_int
                        - (*dp).db_txt_start as c_int
                };
                (*buf).b_ml.ml_chunks.set(curix + 1, 1, rest);
                (*buf).b_ml.ml_chunks.add_size(curix, -rest);
                (*buf).b_ml.ml_chunks.add_lines(curix, -1);
            }
        }
        true
    }
}

/// Chunk `curix` grew past [`MLCS_MAXL`] lines: cut it in two at
/// [`MLCS_MINL`], measuring the first half by walking the data blocks it
/// covers.
///
/// Always returns false — either it failed, or it invalidated the cached
/// position. Both mean the caller returns without caching.
///
/// # Safety
/// `buf` must point at a buffer whose chunk index has a chunk `curix`.
unsafe fn ml_chunk_split(buf: *mut buf_T, curix: usize, curline_arg: linenr_T) -> bool {
    unsafe {
        (*buf).b_ml.ml_chunks.split_at(curix);

        // Total size of the first MLCS_MINL lines of the chunk.
        let mut curline = curline_arg;
        let mut size = 0;
        let mut linecnt = 0;
        while curline < (*buf).b_ml.ml_line_count && linecnt < MLCS_MINL {
            let hp = ml_find_line(buf, curline, ML_FIND);
            if hp.is_null() {
                (*buf).b_ml.ml_chunks.switch_off();
                return false;
            }
            let dp = (*hp).bh_data as *mut DataBlock;
            let count = (*buf).b_ml.locked_high() - (*buf).b_ml.locked_low() + 1;
            let idx = curline - (*buf).b_ml.locked_low();
            curline = (*buf).b_ml.locked_high() + 1;

            // Index of the last line of this block that still counts.
            let rest = count - idx;
            let end_idx = if linecnt + rest > MLCS_MINL {
                let end_idx = idx + MLCS_MINL - linecnt - 1;
                linecnt = MLCS_MINL;
                end_idx
            } else {
                linecnt += rest;
                count - 1
            };

            // First line in the block has its text at the end of it.
            let text_end = if idx == 0 {
                (*dp).db_txt_end as c_int
            } else {
                db_line_start(dp, idx - 1) as c_int
            };
            size += text_end - db_line_start(dp, end_idx) as c_int;
        }

        (*buf).b_ml.ml_chunks.add_lines(curix + 1, -linecnt);
        (*buf).b_ml.ml_chunks.add_size(curix + 1, -size);
        (*buf).b_ml.ml_chunks.set(curix, linecnt, size);
        ml_upd_lastbuf.set(core::ptr::null_mut()); // force a recalc
        false
    }
}

/// Translate between line numbers and byte offsets, in whichever direction
/// the caller left blank.
///
/// With `lnum > 0` this returns the byte offset of the start of that line
/// (and `offp` should be NULL). With `lnum == 0` it returns the line holding
/// byte offset `*offp`, and writes the remaining column offset back through
/// `offp`.
///
/// `no_ff` counts one byte per line break whatever `'fileformat'` says, which
/// is what byte tracking wants.
///
/// Returns -1 when there is nothing to answer from.
///
/// # Safety
/// `buf` must point at a buffer, and `offp` be NULL or writable.
pub unsafe fn ml_find_line_or_offset(
    buf: *mut buf_T,
    lnum: linenr_T,
    offp: *mut c_int,
    no_ff: bool,
) -> c_int {
    unsafe {
        let ffdos = (!no_ff && get_fileformat(buf) == EOL_DOS) as c_int;
        let mut extra = 0;

        // Take care of the cached line first, and only if it is before the
        // requested line. The value for the cached line is then cached in
        // turn: the extmark code wants the byte offset of the line being
        // edited, so a run of small edits to one line computes it once.
        //
        // Caching does not work with 'fileformat', which is no problem for
        // byte tracking (it ignores 'fileformat'), but a line2byte() call
        // does invalidate the cache for the time being.
        let can_cache = lnum != 0 && ffdos == 0 && (*buf).b_ml.cached_lnum() == lnum;
        if lnum == 0 || (*buf).b_ml.cached_lnum() < lnum || !no_ff {
            ml_flush_line(curbuf.get(), false);
        } else if can_cache && (*buf).b_ml.cached_offset() > 0 {
            return (*buf).b_ml.cached_offset() as c_int;
        }

        if (*buf).b_ml.ml_chunks.is_off() || !(*buf).b_ml.ml_chunks.is_built() || lnum < 0 {
            // The memline is empty. If it is open at all it still behaves as
            // though it held one empty line.
            if no_ff && !(*buf).b_ml.ml_mfp.is_null() && (lnum == 1 || lnum == 2) {
                return lnum - 1;
            }
            return -1;
        }

        let offset = if offp.is_null() { 0 } else { *offp };
        if lnum == 0 && offset <= 0 {
            // Not a "find offset", and offset 0 must be in line 1.
            return 1;
        }

        // Skip whole chunks, up to but not including the one the answer is
        // in. The last chunk is special: it never qualifies.
        let chunks = &(*buf).b_ml.ml_chunks;
        let mut curline: linenr_T = 1;
        let mut curix = 0usize;
        let mut size = 0;
        while curix + 1 < chunks.len()
            && (lnum != 0 && lnum >= curline + chunks.lines(curix)
                || offset != 0 && offset > size + chunks.size(curix) + ffdos * chunks.lines(curix))
        {
            curline += chunks.lines(curix);
            size += chunks.size(curix);
            if offset != 0 && ffdos != 0 {
                size += chunks.lines(curix);
            }
            curix += 1;
        }

        // Then walk the data blocks of that chunk line by line.
        while lnum != 0 && curline < lnum || offset != 0 && size < offset {
            // The bounds test has to come first: ml_find_line complains about
            // a line past the end.
            if curline > (*buf).b_ml.ml_line_count {
                return -1;
            }
            let hp = ml_find_line(buf, curline, ML_FIND);
            if hp.is_null() {
                return -1;
            }
            let dp = (*hp).bh_data as *mut DataBlock;
            let count = (*buf).b_ml.locked_high() - (*buf).b_ml.locked_low() + 1;
            let start_idx = curline - (*buf).b_ml.locked_low();
            let mut idx = start_idx;
            // First line in the block has its text at the end of it.
            let text_end = if idx == 0 {
                (*dp).db_txt_end as c_int
            } else {
                db_line_start(dp, idx - 1) as c_int
            };

            // Index of the last line of this block to account for.
            if lnum != 0 {
                if curline + (count - idx) >= lnum {
                    idx += lnum - curline - 1;
                } else {
                    idx = count - 1;
                }
            } else {
                extra = 0;
                while offset >= size + text_end - db_line_start(dp, idx) as c_int + ffdos {
                    if ffdos != 0 {
                        size += 1;
                    }
                    if idx == count - 1 {
                        // The offset is past the last line of the block.
                        extra = 1;
                        break;
                    }
                    idx += 1;
                }
            }

            let len = text_end - db_line_start(dp, idx) as c_int;
            size += len;
            if offset != 0 && size >= offset {
                if size + ffdos == offset {
                    *offp = 0;
                } else if idx == start_idx {
                    *offp = offset - size + len;
                } else {
                    *offp = offset - size + len - (text_end - db_line_start(dp, idx - 1) as c_int);
                }
                curline += idx - start_idx + extra;
                if curline > (*buf).b_ml.ml_line_count {
                    return -1; // exactly one byte beyond the end
                }
                return curline;
            }
            curline = (*buf).b_ml.locked_high() + 1;
        }

        if lnum != 0 {
            // Count the extra CR characters.
            if ffdos != 0 {
                size += lnum - 1;
            }
            // Do not count the last line break with 'noeol' and either 'bin'
            // or 'nofixeol'.
            if ((*buf).b_p_fixeol == 0 || (*buf).b_p_bin != 0)
                && (*buf).b_p_eol == 0
                && lnum > (*buf).b_ml.ml_line_count
            {
                size -= ffdos + 1;
            }
        }

        if can_cache && size > 0 {
            (*buf).b_ml.set_cached_offset(size as size_t);
        }
        size
    }
}

/// Move the cursor to byte `cnt` of the buffer.
///
/// # Safety
/// Must run on the main thread, with a current buffer and window.
pub unsafe fn goto_byte(cnt: c_int) {
    unsafe {
        let mut boff = cnt;
        ml_flush_line(curbuf.get(), false); // the cached line may be dirty
        setpcmark();
        if boff != 0 {
            boff -= 1;
        }
        let lnum = ml_find_line_or_offset(curbuf.get(), 0, &raw mut boff, false);
        if lnum < 1 {
            // Past the end.
            (*curwin.get()).w_cursor.lnum = (*curbuf.get()).b_ml.ml_line_count;
            (*curwin.get()).w_curswant = MAXCOL as c_int;
            coladvance(Win::current(), MAXCOL as c_int);
        } else {
            (*curwin.get()).w_cursor.lnum = lnum;
            (*curwin.get()).w_cursor.col = boff;
            (*curwin.get()).w_cursor.coladd = 0;
            (*curwin.get()).w_set_curswant = true;
        }
        check_cursor(Win::current());

        // Make sure the cursor is on the first byte of a multi-byte char.
        mb_adjust_cursor();
    }
}
