//! `r` -- overwriting every character in the region with one character.
//!
//! Three things make this more than a memset:
//!
//! * the replacement can be a *different width* from what it replaces, in
//!   bytes and in screen cells, so a line often has to be rebuilt rather than
//!   patched -- [`pbyte`] is the fast path that only applies when both sides
//!   are one byte, and [`replace_character`] the general one;
//! * a blockwise replace has to pad short lines out to the block's edge and
//!   re-lay any TAB the block splits, which is why [`replace_block_line`]
//!   builds a whole new line;
//! * `r<CR>` does not replace with a character at all, it *splits* the line,
//!   and `CTRL-V <CR>` (`REPLACE_CR_NCHAR`) asks for the literal carriage
//!   return instead -- which is the whole job of the `had_ctrl_v_cr` flag.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::memline::MlFlags;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_void};

use super::*;
use crate::ex_docmd::cmdmod_has;
use crate::types::{FAIL, NUL, OK};

/// Overwrite the single byte at `lp` with `c`.
///
/// Only for a one-byte character replacing a one-byte character; anything else
/// changes the line's length and has to go through [`replace_character`].
///
/// # Safety
/// `lp` must name a line of the current buffer.
pub(crate) unsafe fn pbyte(mut lp: pos_T, c: c_int) {
    debug_assert!(c <= c_int::from(u8::MAX));
    // SAFETY: the caller's promise -- `lp` names a line of the current
    // buffer, and the column is clamped to that line below before the write.
    let p = unsafe { ml_get_buf_mut(curbuf.get(), lp.lnum) };
    let len = cur_buf().b_ml.cached_len();

    // Safety check: the caller's column may be past the line.
    if lp.col >= len {
        lp.col = if len > 1 { len - 2 } else { 0 };
    }
    unsafe { *p.offset(lp.col as isize) = c as c_char };
    if curbuf_splice_pending.get() == 0 {
        let row = lp.lnum as c_int - 1;
        unsafe { extmark_splice_cols(curbuf.get(), row, lp.col, 1, 1, kExtmarkUndo) };
    }
}

/// Replace the character under the cursor with `c`, whatever the two widths.
///
/// Goes through Replace mode's own insert so that a multi-byte character on
/// either side is handled; leaves the cursor back on the replaced character.
///
/// # Safety
/// The cursor must name a valid position in the current buffer.
unsafe fn replace_character(c: c_int) {
    let saved = State.get();
    State.set(MODE_REPLACE);
    // SAFETY: the caller's promise -- the cursor names a valid position.
    unsafe { ins_char(c) };
    State.set(saved);
    // Back up onto the character just replaced.
    dec_cursor();
}

/// `r` over the operator's region.
///
/// `c` is the replacement, or `REPLACE_CR_NCHAR`/`REPLACE_NL_NCHAR` for the
/// `CTRL-V <CR>`/`CTRL-V <NL>` spellings that mean the literal byte rather
/// than a line split.
///
/// # Safety
/// `oap` must point to a live `oparg_T` describing a region of the current
/// buffer.
pub(crate) unsafe fn op_replace(oap: *mut oparg_T, mut c: c_int) -> c_int {
    // SAFETY: the caller's promise -- a live `oparg_T` of the current buffer.
    let mut oap = unsafe { Op::new(oap) };
    if cur_buf().b_ml.ml_flags.has(MlFlags::EMPTY) || oap.empty {
        return OK;
    }

    // CTRL-V CR / CTRL-V NL: put the byte in, do not split the line.
    let mut had_ctrl_v_cr = false;
    if c == REPLACE_CR_NCHAR {
        had_ctrl_v_cr = true;
        c = CAR;
    } else if c == REPLACE_NL_NCHAR {
        had_ctrl_v_cr = true;
        c = NL;
    }

    unsafe { mb_adjust_opend(oap.raw()) };

    let (above, below) = (oap.start.lnum - 1, oap.end.lnum + 1);
    if u_save(above, below) == FAIL {
        return FAIL;
    }

    if oap.motion_type == kMTBlockWise {
        replace_block(oap, c, had_ctrl_v_cr);
    } else {
        replace_chars(oap, c);
    }

    cur_win().w_cursor = oap.start;
    let (lnum, col, last) = (oap.start.lnum, oap.start.col, oap.end.lnum + 1);
    check_cursor(unsafe { Win::current() });
    changed_lines(cur_buf(), lnum, col, last, 0, true);

    if !cmdmod_has(CmdModFlags::LOCKMARKS) {
        cur_buf().b_op_start = oap.start;
        cur_buf().b_op_end = oap.end;
    }
    OK
}

/// The blockwise arm: one rebuilt line per line the block reaches.
///
/// `oap` must be blockwise.
fn replace_block(oap: Op, c: c_int, had_ctrl_v_cr: bool) {
    let mut bd = block_def::ZERO;
    bd.is_MAX = c_int::from(cur_win().w_curswant == MAXCOL);
    while cur_win().w_cursor.lnum <= oap.end.lnum {
        // Make sure the cursor position is valid for `block_prep`.
        cur_win().w_cursor.col = 0;
        // SAFETY: the cursor walks the region, so its line is the buffer's.
        let lnum = cur_win().w_cursor.lnum;
        unsafe { block_prep(oap.raw(), &raw mut bd, lnum, true) };
        if bd.textlen != 0 || (op_virtual() && bd.is_MAX == 0) {
            replace_block_line(oap, &mut bd, c, had_ctrl_v_cr);
        }
        cur_win().w_cursor.lnum += 1;
    }
}

/// One line of the blockwise arm.
///
/// Splitting a TAB the block only partly covers can make the line *longer*, so
/// the line is rebuilt: the text before the block, `startspaces` pad, the
/// replacement repeated, `endspaces` pad, and the text after. With `\r` or
/// `\n` and no CTRL-V there is no "after": the tail becomes a new line.
///
/// `oap` and `bd` must describe the cursor line, as [`block_prep`] left them.
fn replace_block_line(mut oap: Op, bd: &mut block_def, c: c_int, had_ctrl_v_cr: bool) {
    // Replacing with `\r`/`\n` splits the line rather than overwriting.
    let splits_line = !had_ctrl_v_cr && (c == '\r' as c_int || c == '\n' as c_int);

    // When the block starts in virtual space, that offset counts as
    // pre-padding. (Upstream also keeps a running count `n` of the extra
    // characters a split TAB needs, here and just below; nothing reads it,
    // so only this side effect on `startspaces` is carried over.)
    // SAFETY: `bd` describes the cursor line, so `bd.textstart` is inside it.
    if op_virtual() && bd.is_short != 0 && unsafe { *bd.textstart } as c_int == NUL {
        let mut vpos = pos_T {
            lnum: cur_win().w_cursor.lnum,
            col: 0,
            coladd: 0,
        };
        // SAFETY: a live current window, and a local position in the cursor's
        // own line.
        unsafe { getvpos(Win::current(), Pos::new(&raw mut vpos), oap.start_vcol) };
        bd.startspaces += vpos.coladd;
    }

    // How many characters to replace.
    let mut numc = oap.end_vcol - oap.start_vcol + 1;
    if bd.is_short != 0 && (!op_virtual() || bd.is_MAX != 0) {
        numc -= (oap.end_vcol - bd.end_vcol) + 1;
    }
    // A double-wide character only fits half as many times.
    if unsafe { utf_char2cells(c) } > 1 {
        if numc & 1 != 0 && bd.is_short == 0 {
            bd.endspaces += 1;
        }
        numc /= 2;
    }

    let mut num_chars = numc;
    numc *= utf_char2len(c);

    let mut oldp = get_cursor_line_ptr();
    let oldlen = get_cursor_line_len();

    let mut newp_size = bd.textcol as size_t + bd.startspaces as size_t;
    if !splits_line {
        newp_size += numc as size_t;
        if bd.is_short == 0 {
            newp_size += (bd.endspaces + oldlen - bd.textcol - bd.textlen) as size_t;
        }
    }
    // SAFETY: `newp_size` counts the prefix, the pre-spaces, the repeated
    // replacement, the post-spaces and the tail -- exactly what is written.
    let newp = unsafe { xmallocz(newp_size) } as *mut c_char;

    // Up to the replaced part, then the pre-spaces.
    unsafe {
        memmove(
            newp as *mut c_void,
            oldp as *const c_void,
            bd.textcol as size_t,
        )
    };
    oldp = unsafe { oldp.offset((bd.textcol + bd.textlen) as isize) };
    let at = unsafe { newp.offset(bd.textcol as isize) } as *mut c_void;
    unsafe { memset(at, ' ' as c_int, bd.startspaces as size_t) };

    // What is left of the line after the block, NUL included.
    let col = oldlen - bd.textcol - bd.textlen + 1;
    debug_assert!(col >= 0);

    let mut after_p: *mut c_char = ::core::ptr::null_mut();
    let mut after_p_len: size_t = 0;
    let mut newrows = 0;
    let mut newcols = 0;
    if !splits_line {
        let mut newp_len = bd.textcol + bd.startspaces;
        while num_chars > 0 {
            num_chars -= 1;
            newp_len += unsafe { utf_char2bytes(c, newp.offset(newp_len as isize)) };
        }
        if bd.is_short == 0 {
            let at = unsafe { newp.offset(newp_len as isize) } as *mut c_void;
            unsafe { memset(at, ' ' as c_int, bd.endspaces as size_t) };
            newp_len += bd.endspaces;
            let tail = unsafe { newp.offset(newp_len as isize) } as *mut c_void;
            unsafe { memmove(tail, oldp as *const c_void, col as size_t) };
        }
        newcols = newp_len - bd.textcol;
    } else {
        // The tail becomes the next line.
        after_p_len = col as size_t;
        after_p = unsafe { xmalloc(after_p_len) } as *mut c_char;
        unsafe { memmove(after_p as *mut c_void, oldp as *const c_void, after_p_len) };
        newrows = 1;
    }

    let baselnum = cur_win().w_cursor.lnum;
    unsafe { ml_replace(baselnum, newp, false) };
    curbuf_splice_pending.set(curbuf_splice_pending.get() + 1);
    if !after_p.is_null() {
        let len = after_p_len as colnr_T;
        unsafe { ml_append(cur_win().w_cursor.lnum, after_p, len, false) };
        cur_win().w_cursor.lnum += 1;
        unsafe { appended_lines_mark(cur_win().w_cursor.lnum, 1) };
        oap.end.lnum += 1;
        unsafe { xfree(after_p as *mut c_void) };
    }
    curbuf_splice_pending.set(curbuf_splice_pending.get() - 1);
    let old_bytes = bd.textlen as bcount_t;
    let new_bytes = (newrows + newcols) as bcount_t;
    let row = baselnum as c_int - 1;
    let (col, len) = (bd.textcol, bd.textlen);
    let op = kExtmarkUndo;
    unsafe {
        extmark_splice(
            curbuf.get(),
            row,
            col,
            0,
            len,
            old_bytes,
            newrows,
            newcols,
            new_bytes,
            op,
        )
    };
}

/// The charwise and linewise arm: walk the region a character at a time.
///
/// `oap` must be charwise or linewise.
fn replace_chars(mut oap: Op, c: c_int) {
    // SAFETY: the cursor walks the region, so it names a position of the
    // current buffer at every step, which is what each of these asks for.
    if oap.motion_type == kMTLineWise {
        oap.start.col = 0;
        cur_win().w_cursor.col = 0;
        oap.end.col = ml_get_len(oap.end.lnum);
        if oap.end.col != 0 {
            oap.end.col -= 1;
        }
    } else if !oap.inclusive {
        unsafe { dec(&mut oap.end) };
    }

    while ltoreq(cur_win().w_cursor, oap.end) {
        let mut done = false;

        let under_cursor = gchar_cursor();
        if under_cursor != NUL {
            let new_byte_len = utf_char2len(c);
            let old_byte_len = unsafe { utfc_ptr2len(get_cursor_pos_ptr()) };

            if new_byte_len > 1 || old_byte_len > 1 {
                // Slow, but it handles a single-byte character replacing a
                // multi-byte one and the other way around.
                if cur_win().w_cursor.lnum == oap.end.lnum {
                    oap.end.col += new_byte_len - old_byte_len;
                }
                unsafe { replace_character(c) };
                done = true;
            } else {
                if under_cursor == TAB {
                    // Breaking the TAB moves the end, so remember where it
                    // was in columns first.
                    let mut end_vcol = 0;
                    if cur_win().w_cursor.lnum == oap.end.lnum {
                        end_vcol = unsafe { getviscol2(oap.end.col, oap.end.coladd) };
                    }
                    unsafe { coladvance_force(getviscol()) };
                    if cur_win().w_cursor.lnum == oap.end.lnum {
                        // SAFETY: a live current window, and the operator's
                        // end position in the cursor's own line.
                        unsafe { getvpos(Win::current(), oap.end(), end_vcol) };
                    }
                }
                // With `coladd` set the cursor may now be just past a TAB.
                if gchar_cursor() != NUL {
                    unsafe { pbyte(cur_win().w_cursor, c) };
                    done = true;
                }
            }
        }

        if !done && op_virtual() && cur_win().w_cursor.lnum == oap.end.lnum {
            replace_virtual_tail(oap, c);
        }

        // On to the next character; stop at the end of the file.
        if inc_cursor() == -1 {
            break;
        }
    }
}

/// 'virtualedit' only: replace the columns past the end of the last line.
///
/// Reached when the region extends into virtual space, where there is no
/// character to overwrite; `coladvance_force` fills the line out with spaces
/// first and those are then replaced.
///
/// The cursor must be on `oap.end.lnum`.
fn replace_virtual_tail(oap: Op, c: c_int) {
    let mut virtcols = oap.end.coladd;
    if cur_win().w_cursor.lnum == oap.start.lnum
        && oap.start.col == oap.end.col
        && oap.start.coladd != 0
    {
        virtcols -= oap.start.coladd;
    }

    // `oap.end` has been trimmed, so it is effectively inclusive: the extra
    // +1 is what keeps the NUL byte from being trampled.
    // SAFETY: the cursor is on `oap.end.lnum`, a line of the current buffer,
    // and `coladvance_force` fills it out to the column being replaced.
    let endcol = unsafe { getviscol2(oap.end.col, oap.end.coladd) };
    unsafe { coladvance_force(endcol + 1) };
    cur_win().w_cursor.col -= virtcols + 1;
    while virtcols >= 0 {
        if utf_char2len(c) > 1 {
            unsafe { replace_character(c) };
        } else {
            unsafe { pbyte(cur_win().w_cursor, c) };
        }
        if unsafe { inc(&mut cur_win().w_cursor) } == -1 {
            break;
        }
        virtcols -= 1;
    }
}

/// The buffer the editor is working in.
fn cur_buf() -> Buf {
    // SAFETY: `curbuf` is set from startup to exit.
    unsafe { Buf::current() }
}

/// The window the editor is working in.
fn cur_win() -> Win {
    // SAFETY: `curwin` is set from startup to exit.
    unsafe { Win::current() }
}
