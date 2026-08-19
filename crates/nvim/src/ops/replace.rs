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
    unsafe {
        debug_assert!(c <= c_int::from(u8::MAX));
        let p = ml_get_buf_mut(curbuf.get(), lp.lnum);
        let len = (*curbuf.get()).b_ml.ml_line_textlen;

        // Safety check: the caller's column may be past the line.
        if lp.col >= len {
            lp.col = if len > 1 { len - 2 } else { 0 };
        }
        *p.offset(lp.col as isize) = c as c_char;
        if curbuf_splice_pending.get() == 0 {
            extmark_splice_cols(
                curbuf.get(),
                lp.lnum as c_int - 1,
                lp.col,
                1,
                1,
                kExtmarkUndo,
            );
        }
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
    unsafe {
        let saved = State.get();
        State.set(MODE_REPLACE);
        ins_char(c);
        State.set(saved);
        // Back up onto the character just replaced.
        dec_cursor();
    }
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
    unsafe {
        if (*curbuf.get()).b_ml.ml_flags & ML_EMPTY != 0 || (*oap).empty {
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

        mb_adjust_opend(oap);

        if u_save((*oap).start.lnum - 1, (*oap).end.lnum + 1) == FAIL {
            return FAIL;
        }

        if (*oap).motion_type == kMTBlockWise {
            replace_block(oap, c, had_ctrl_v_cr);
        } else {
            replace_chars(oap, c);
        }

        (*curwin.get()).w_cursor = (*oap).start;
        check_cursor(curwin.get());
        changed_lines(
            curbuf.get(),
            (*oap).start.lnum,
            (*oap).start.col,
            (*oap).end.lnum + 1,
            0,
            true,
        );

        if !cmdmod_has(CmdModFlags::LOCKMARKS) {
            (*curbuf.get()).b_op_start = (*oap).start;
            (*curbuf.get()).b_op_end = (*oap).end;
        }
        OK
    }
}

/// The blockwise arm: one rebuilt line per line the block reaches.
///
/// # Safety
/// `oap` must point to a live blockwise `oparg_T`.
unsafe fn replace_block(oap: *mut oparg_T, c: c_int, had_ctrl_v_cr: bool) {
    unsafe {
        let mut bd = block_def::ZERO;
        bd.is_MAX = c_int::from((*curwin.get()).w_curswant == MAXCOL);
        while (*curwin.get()).w_cursor.lnum <= (*oap).end.lnum {
            // Make sure the cursor position is valid for `block_prep`.
            (*curwin.get()).w_cursor.col = 0;
            block_prep(oap, &raw mut bd, (*curwin.get()).w_cursor.lnum, true);
            if bd.textlen != 0 || (virtual_op.get() != 0 && bd.is_MAX == 0) {
                replace_block_line(oap, &mut bd, c, had_ctrl_v_cr);
            }
            (*curwin.get()).w_cursor.lnum += 1;
        }
    }
}

/// One line of the blockwise arm.
///
/// Splitting a TAB the block only partly covers can make the line *longer*, so
/// the line is rebuilt: the text before the block, `startspaces` pad, the
/// replacement repeated, `endspaces` pad, and the text after. With `\r` or
/// `\n` and no CTRL-V there is no "after": the tail becomes a new line.
///
/// # Safety
/// `oap` and `bd` must describe the cursor line, as [`block_prep`] left them.
unsafe fn replace_block_line(oap: *mut oparg_T, bd: &mut block_def, c: c_int, had_ctrl_v_cr: bool) {
    unsafe {
        // Replacing with `\r`/`\n` splits the line rather than overwriting.
        let splits_line = !had_ctrl_v_cr && (c == '\r' as c_int || c == '\n' as c_int);

        // When the block starts in virtual space, that offset counts as
        // pre-padding. (Upstream also keeps a running count `n` of the extra
        // characters a split TAB needs, here and just below; nothing reads it,
        // so only this side effect on `startspaces` is carried over.)
        if virtual_op.get() != 0 && bd.is_short != 0 && *bd.textstart as c_int == NUL {
            let mut vpos = pos_T {
                lnum: (*curwin.get()).w_cursor.lnum,
                col: 0,
                coladd: 0,
            };
            getvpos(curwin.get(), &raw mut vpos, (*oap).start_vcol);
            bd.startspaces += vpos.coladd;
        }

        // How many characters to replace.
        let mut numc = (*oap).end_vcol - (*oap).start_vcol + 1;
        if bd.is_short != 0 && (virtual_op.get() == 0 || bd.is_MAX != 0) {
            numc -= ((*oap).end_vcol - bd.end_vcol) + 1;
        }
        // A double-wide character only fits half as many times.
        if utf_char2cells(c) > 1 {
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
        let newp = xmallocz(newp_size) as *mut c_char;

        // Up to the replaced part, then the pre-spaces.
        memmove(
            newp as *mut c_void,
            oldp as *const c_void,
            bd.textcol as size_t,
        );
        oldp = oldp.offset((bd.textcol + bd.textlen) as isize);
        memset(
            newp.offset(bd.textcol as isize) as *mut c_void,
            ' ' as c_int,
            bd.startspaces as size_t,
        );

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
                newp_len += utf_char2bytes(c, newp.offset(newp_len as isize));
            }
            if bd.is_short == 0 {
                memset(
                    newp.offset(newp_len as isize) as *mut c_void,
                    ' ' as c_int,
                    bd.endspaces as size_t,
                );
                newp_len += bd.endspaces;
                memmove(
                    newp.offset(newp_len as isize) as *mut c_void,
                    oldp as *const c_void,
                    col as size_t,
                );
            }
            newcols = newp_len - bd.textcol;
        } else {
            // The tail becomes the next line.
            after_p_len = col as size_t;
            after_p = xmalloc(after_p_len) as *mut c_char;
            memmove(after_p as *mut c_void, oldp as *const c_void, after_p_len);
            newrows = 1;
        }

        ml_replace((*curwin.get()).w_cursor.lnum, newp, false);
        *curbuf_splice_pending.ptr() += 1;
        let baselnum = (*curwin.get()).w_cursor.lnum;
        if !after_p.is_null() {
            ml_append(
                (*curwin.get()).w_cursor.lnum,
                after_p,
                after_p_len as colnr_T,
                false,
            );
            (*curwin.get()).w_cursor.lnum += 1;
            appended_lines_mark((*curwin.get()).w_cursor.lnum, 1);
            (*oap).end.lnum += 1;
            xfree(after_p as *mut c_void);
        }
        *curbuf_splice_pending.ptr() -= 1;
        extmark_splice(
            curbuf.get(),
            baselnum as c_int - 1,
            bd.textcol,
            0,
            bd.textlen,
            bd.textlen as bcount_t,
            newrows,
            newcols,
            (newrows + newcols) as bcount_t,
            kExtmarkUndo,
        );
    }
}

/// The charwise and linewise arm: walk the region a character at a time.
///
/// # Safety
/// `oap` must point to a live charwise or linewise `oparg_T`.
unsafe fn replace_chars(oap: *mut oparg_T, c: c_int) {
    unsafe {
        if (*oap).motion_type == kMTLineWise {
            (*oap).start.col = 0;
            (*curwin.get()).w_cursor.col = 0;
            (*oap).end.col = ml_get_len((*oap).end.lnum);
            if (*oap).end.col != 0 {
                (*oap).end.col -= 1;
            }
        } else if !(*oap).inclusive {
            dec(&raw mut (*oap).end);
        }

        while ltoreq((*curwin.get()).w_cursor, (*oap).end) {
            let mut done = false;

            let under_cursor = gchar_cursor();
            if under_cursor != NUL {
                let new_byte_len = utf_char2len(c);
                let old_byte_len = utfc_ptr2len(get_cursor_pos_ptr());

                if new_byte_len > 1 || old_byte_len > 1 {
                    // Slow, but it handles a single-byte character replacing a
                    // multi-byte one and the other way around.
                    if (*curwin.get()).w_cursor.lnum == (*oap).end.lnum {
                        (*oap).end.col += new_byte_len - old_byte_len;
                    }
                    replace_character(c);
                    done = true;
                } else {
                    if under_cursor == TAB {
                        // Breaking the TAB moves the end, so remember where it
                        // was in columns first.
                        let mut end_vcol = 0;
                        if (*curwin.get()).w_cursor.lnum == (*oap).end.lnum {
                            end_vcol = getviscol2((*oap).end.col, (*oap).end.coladd);
                        }
                        coladvance_force(getviscol());
                        if (*curwin.get()).w_cursor.lnum == (*oap).end.lnum {
                            getvpos(curwin.get(), &raw mut (*oap).end, end_vcol);
                        }
                    }
                    // With `coladd` set the cursor may now be just past a TAB.
                    if gchar_cursor() != NUL {
                        pbyte((*curwin.get()).w_cursor, c);
                        done = true;
                    }
                }
            }

            if !done && virtual_op.get() != 0 && (*curwin.get()).w_cursor.lnum == (*oap).end.lnum {
                replace_virtual_tail(oap, c);
            }

            // On to the next character; stop at the end of the file.
            if inc_cursor() == -1 {
                break;
            }
        }
    }
}

/// 'virtualedit' only: replace the columns past the end of the last line.
///
/// Reached when the region extends into virtual space, where there is no
/// character to overwrite; `coladvance_force` fills the line out with spaces
/// first and those are then replaced.
///
/// # Safety
/// `oap` must point to a live `oparg_T`; the cursor must be on `oap.end.lnum`.
unsafe fn replace_virtual_tail(oap: *mut oparg_T, c: c_int) {
    unsafe {
        let mut virtcols = (*oap).end.coladd;
        if (*curwin.get()).w_cursor.lnum == (*oap).start.lnum
            && (*oap).start.col == (*oap).end.col
            && (*oap).start.coladd != 0
        {
            virtcols -= (*oap).start.coladd;
        }

        // `oap.end` has been trimmed, so it is effectively inclusive: the extra
        // +1 is what keeps the NUL byte from being trampled.
        coladvance_force(getviscol2((*oap).end.col, (*oap).end.coladd) + 1);
        (*curwin.get()).w_cursor.col -= virtcols + 1;
        while virtcols >= 0 {
            if utf_char2len(c) > 1 {
                replace_character(c);
            } else {
                pbyte((*curwin.get()).w_cursor, c);
            }
            if inc(&raw mut (*curwin.get()).w_cursor) == -1 {
                break;
            }
            virtcols -= 1;
        }
    }
}
