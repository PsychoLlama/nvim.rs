//! Blockwise `I` and `A`, and `c` everywhere.
//!
//! All three run Insert mode *once*, on the block's first line, and then copy
//! what was typed into the rest of the block. Nothing records what was typed:
//! it is recovered afterwards by comparing the first line's length against
//! [`BlockInsertPre`], measured just before `edit()` ran -- which is why both
//! [`op_insert`] and [`op_change`] are a "before" half, an `edit()`, and an
//! "after" half that has to cope with everything Insert mode may have done in
//! between.
//!
//! The awkward cases the after-half exists for:
//!
//! * 'autoindent' or `=` may have changed the *indent* of the first line, so
//!   the block's column has moved and the indent itself must not be counted as
//!   inserted text;
//! * the user may have moved the cursor before typing, so the insert did not
//!   start where the block did (`b_op_start_orig` against `oap.start`);
//! * `A` on a block opened with `$` has no fixed right edge, so the insert's
//!   own start column is the only reference there is;
//! * Insert mode may have been left with CTRL-C (`got_int`) or on another
//!   line, in which case there is nothing sensible to replay and the operator
//!   quietly stops.
//!
//! [`adjust_cursor_eol`] is the shared tail that puts the cursor back on a
//! legal column afterwards.

#![deny(unsafe_op_in_unsafe_fn)]

use crate::memline::MlFlags;
use core::ffi::{c_char, c_int, c_uint, c_void};

use super::*;
use crate::types::{FAIL, NUL, OK};

/// The first line, measured just before Insert mode runs.
///
/// [`op_insert`] recovers what was typed by measuring the line again
/// afterwards and subtracting; everything here exists so that the difference
/// is the *text* and not an indent change.
struct BlockInsertPre {
    /// Indent of the first line in bytes.
    ind_pre_col: colnr_T,
    /// Indent of the first line in screen columns.
    ind_pre_vcol: c_int,
    /// Bytes of the first line from the block's column onwards (past the
    /// block's text as well, for `A`).
    pre_textlen: c_int,
}

/// Blockwise `I` and `A`.
///
/// # Safety
/// `oap` must point to a live `oparg_T` describing a region of the current
/// buffer.
pub(crate) unsafe fn op_insert(oap: *mut oparg_T, count1: c_int) {
    unsafe {
        let mut bd = block_def::ZERO;
        // `edit()` changes `w_curswant`; record it now, for `A`.
        bd.is_MAX = c_int::from((*curwin.get()).w_curswant == MAXCOL);

        // The Visual block is still marked; get rid of it now.
        (*curwin.get()).w_cursor.lnum = (*oap).start.lnum;
        redraw_curbuf_later(UPD_INVERTED);
        update_screen();

        let mut pre = BlockInsertPre {
            ind_pre_col: 0,
            ind_pre_vcol: 0,
            pre_textlen: 0,
        };
        if (*oap).motion_type == kMTBlockWise {
            match measure_before_insert(oap, &mut bd) {
                Some(measured) => pre = measured,
                None => return,
            }
        }

        if (*oap).op_type == OP_APPEND && !move_cursor_for_append(oap, &mut bd) {
            return;
        }

        let t1 = (*oap).start;
        let start_insert = (*curwin.get()).w_cursor;
        edit(NUL, false, count1);

        // When a TAB was inserted and the characters in front of it were
        // folded into it too, the cursor's column may have been *reduced*.
        if t1.lnum == (*curbuf.get()).b_op_start_orig.lnum
            && lt((*curbuf.get()).b_op_start_orig, t1)
        {
            (*oap).start = (*curbuf.get()).b_op_start_orig;
        }

        // The user moved off the line, or left Insert mode with CTRL-C: there
        // is nothing to replay.
        if (*curwin.get()).w_cursor.lnum != (*oap).start.lnum || got_int.get() {
            return;
        }

        if (*oap).motion_type == kMTBlockWise {
            replay_insert(oap, &mut bd, &mut pre, start_insert);
        }
    }
}

/// The blockwise half of `op_insert`'s prologue: put the cursor on a real
/// column and measure the first line.
///
/// `None` means undo could not be prepared and the operator must stop.
///
/// # Safety
/// `oap` must point to a live blockwise `oparg_T`.
unsafe fn measure_before_insert(oap: *mut oparg_T, bd: &mut block_def) -> Option<BlockInsertPre> {
    unsafe {
        // With 'virtualedit' the spaces have to go in before `block_prep`
        // runs. When only "block" is set, virtual edit is already off here,
        // but `coladvance_force` still needs it -- and it reads the
        // *window-local* 'virtualedit', so that is what gets overridden.
        if (*curwin.get()).w_cursor.coladd > 0 {
            let old_ve_flags: c_uint = (*curwin.get()).w_onebuf_opt.wo_ve_flags;
            if u_save_cursor() == FAIL {
                return None;
            }
            (*curwin.get()).w_onebuf_opt.wo_ve_flags = kOptVeFlagAll as c_uint;
            coladvance_force(if (*oap).op_type == OP_APPEND {
                (*oap).end_vcol + 1
            } else {
                getviscol()
            });
            if (*oap).op_type == OP_APPEND {
                (*curwin.get()).w_cursor.col -= 1;
            }
            (*curwin.get()).w_onebuf_opt.wo_ve_flags = old_ve_flags;
        }

        block_prep(oap, &raw mut *bd, (*oap).start.lnum, true);
        let mut pre_textlen = ml_get_len((*oap).start.lnum) - bd.textcol;
        if (*oap).op_type == OP_APPEND {
            pre_textlen -= bd.textlen;
        }
        Some(BlockInsertPre {
            ind_pre_col: getwhitecols_curline() as colnr_T,
            ind_pre_vcol: get_indent(),
            pre_textlen,
        })
    }
}

/// Put the cursor where `A` should start typing; false means give up.
///
/// # Safety
/// `oap` must point to a live `oparg_T`; `bd` must describe its first line.
unsafe fn move_cursor_for_append(oap: *mut oparg_T, bd: &mut block_def) -> bool {
    unsafe {
        if (*oap).motion_type == kMTBlockWise && (*curwin.get()).w_cursor.coladd == 0 {
            // To the character right of the block.
            (*curwin.get()).w_set_curswant = true;
            while *get_cursor_pos_ptr() as c_int != NUL
                && (*curwin.get()).w_cursor.col < bd.textcol + bd.textlen
            {
                (*curwin.get()).w_cursor.col += 1;
            }
            if bd.is_short != 0 && bd.is_MAX == 0 {
                // The first line was too short: pad it out and say so in `bd`.
                if u_save_cursor() == FAIL {
                    return false;
                }
                for _ in 0..bd.endspaces {
                    ins_char(' ' as c_int);
                }
                bd.textlen += bd.endspaces;
            }
        } else {
            (*curwin.get()).w_cursor = (*oap).end;
            check_cursor_col(curwin.get());
            // Works just like `i` on the next character.
            if *ml_get((*curwin.get()).w_cursor.lnum) as c_int != NUL
                && (*oap).start_vcol != (*oap).end_vcol
            {
                inc_cursor();
            }
        }
        true
    }
}

/// Copy what was typed on the block's first line into the rest of the block.
///
/// Everything before the `block_insert` call is re-measuring: the indent may
/// have changed, the insert may not have started where the block did, and with
/// `$` the block has no right edge to measure against.
///
/// # Safety
/// `oap` must point to a live blockwise `oparg_T` whose first line is the
/// cursor line.
unsafe fn replay_insert(
    oap: *mut oparg_T,
    bd: &mut block_def,
    pre: &mut BlockInsertPre,
    start_insert: pos_T,
) {
    unsafe {
        // If indenting kicked in the first line has moved -- but only count it
        // when the indent actually grew.
        let mut ind_post_vcol = 0;
        let mut did_indent = false;
        let ind_post_col = getwhitecols_curline() as colnr_T;
        if (*curbuf.get()).b_op_start.col > pre.ind_pre_col && ind_post_col > pre.ind_pre_col {
            bd.textcol += ind_post_col - pre.ind_pre_col;
            ind_post_vcol = get_indent();
            bd.start_vcol += ind_post_vcol - pre.ind_pre_vcol;
            did_indent = true;
        }

        // The user may have moved the cursor before typing; try to move the
        // block to match. Only when the difference is not the indent's doing.
        if (*oap).start.lnum == (*curbuf.get()).b_op_start_orig.lnum
            && bd.is_MAX == 0
            && !did_indent
        {
            let t = getviscol2(
                (*curbuf.get()).b_op_start_orig.col,
                (*curbuf.get()).b_op_start_orig.coladd,
            );
            let orig_at =
                (*curbuf.get()).b_op_start_orig.col + (*curbuf.get()).b_op_start_orig.coladd;
            let block_at = (*oap).start.col + (*oap).start.coladd;

            if (*oap).op_type == OP_INSERT && block_at != orig_at {
                (*oap).start.col = (*curbuf.get()).b_op_start_orig.col;
                pre.pre_textlen -= t - (*oap).start_vcol;
                (*oap).start_vcol = t;
            } else if (*oap).op_type == OP_APPEND && block_at >= orig_at {
                (*oap).start.col = (*curbuf.get()).b_op_start_orig.col;
                // Back to what `pre_textlen` would have been for an insert.
                pre.pre_textlen += bd.textlen;
                pre.pre_textlen -= t - (*oap).start_vcol;
                (*oap).start_vcol = t;
                (*oap).op_type = OP_INSERT;
            }
        }

        // Spaces and tabs in the indent may have turned into other spaces and
        // tabs, so measure the starting column again. Not with `$`, where the
        // end of the line has moved anyway.
        let shift_for_indent = did_indent && bd.textcol - ind_post_col > 0;
        if shift_for_indent {
            (*oap).start.col += ind_post_col - pre.ind_pre_col;
            (*oap).start_vcol += ind_post_vcol - pre.ind_pre_vcol;
            (*oap).end.col += ind_post_col - pre.ind_pre_col;
            (*oap).end_vcol += ind_post_vcol - pre.ind_pre_vcol;
        }
        let mut bd2 = block_def::ZERO;
        block_prep(oap, &raw mut bd2, (*oap).start.lnum, true);
        if shift_for_indent {
            // `oap` is used below, so put it back.
            (*oap).start.col -= ind_post_col - pre.ind_pre_col;
            (*oap).start_vcol -= ind_post_vcol - pre.ind_pre_vcol;
            (*oap).end.col -= ind_post_col - pre.ind_pre_col;
            (*oap).end_vcol -= ind_post_vcol - pre.ind_pre_vcol;
        }
        if bd.is_MAX == 0 || bd2.textlen < bd.textlen {
            if (*oap).op_type == OP_APPEND {
                pre.pre_textlen += bd2.textlen - bd.textlen;
                if bd2.endspaces != 0 {
                    bd2.textlen -= 1;
                }
            }
            bd.textcol = bd2.textcol;
            bd.textlen = bd2.textlen;
        }

        // A later `ml_get` flushes the line data, so the inserted text has to
        // be copied out before anything else touches the buffer.
        let mut firstline = ml_get((*oap).start.lnum);
        let mut len = ml_get_len((*oap).start.lnum);
        let mut add = bd.textcol;
        // How far the cursor was moved during the insert.
        let mut offset: colnr_T = 0;
        if (*oap).op_type == OP_APPEND {
            add += bd.textlen;
            // The cursor may have been moved during the insert when `$` was
            // used, and then the block has no right edge to measure from.
            if bd.is_MAX != 0
                && start_insert.lnum == (*Insstart.ptr()).lnum
                && start_insert.col > (*Insstart.ptr()).col
            {
                offset = start_insert.col - (*Insstart.ptr()).col;
                add -= offset;
                if (*oap).end_vcol <= offset {
                    // Moved outside the Visual block; nothing sensible to do.
                    return;
                }
                (*oap).end_vcol -= offset + 1;
            }
        }
        // A short line: point at the NUL.
        add = add.min(len);
        firstline = firstline.offset(add as isize);
        len -= add;

        let ins_len = len - pre.pre_textlen - offset;
        if pre.pre_textlen >= 0 && ins_len > 0 {
            let ins_text = xmemdupz(firstline as *const c_void, ins_len as size_t) as *mut c_char;
            if u_save((*oap).start.lnum, (*oap).end.lnum + 1) == OK {
                block_insert(
                    oap,
                    ins_text,
                    ins_len as size_t,
                    (*oap).op_type == OP_INSERT,
                    &raw mut *bd,
                );
            }
            (*curwin.get()).w_cursor.col = (*oap).start.col;
            check_cursor(curwin.get());
            xfree(ins_text as *mut c_void);
        }
    }
}

/// `c` -- delete the region, then insert.
///
/// Answers true when `edit()` returned because of a CTRL-O command.
///
/// # Safety
/// `oap` must point to a live `oparg_T` describing a region of the current
/// buffer.
pub(crate) unsafe fn op_change(oap: *mut oparg_T) -> c_int {
    unsafe {
        let mut l = (*oap).start.col;
        if (*oap).motion_type == kMTLineWise {
            l = 0;
            // Like opening a new line: do smart indent.
            can_si.set(may_do_si());
        }

        // Delete the region first. In an empty buffer there is nothing to
        // delete, only undo to prepare.
        if (*curbuf.get()).b_ml.ml_flags.has(MlFlags::EMPTY) {
            if u_save_cursor() == FAIL {
                return 0;
            }
        } else if op_delete(oap).is_err() {
            return 0;
        }

        if l > (*curwin.get()).w_cursor.col
            && *ml_get((*curwin.get()).w_cursor.lnum) as c_int != NUL
            && !op_virtual()
        {
            inc_cursor();
        }

        let mut bd = block_def::ZERO;
        let mut pre_textlen = 0;
        let mut pre_indent = 0;
        if (*oap).motion_type == kMTBlockWise {
            // Add the spaces before measuring the line's length.
            if op_virtual() && ((*curwin.get()).w_cursor.coladd > 0 || gchar_cursor() == NUL) {
                coladvance_force(getviscol());
            }
            let firstline = ml_get((*oap).start.lnum);
            pre_textlen = ml_get_len((*oap).start.lnum);
            pre_indent = getwhitecols(firstline) as c_int;
            bd.textcol = (*curwin.get()).w_cursor.col;
        }

        if (*oap).motion_type == kMTLineWise {
            fix_indent();
        }

        // Reset `finish_op` now: it must not be set inside `edit()`.
        let save_finish_op = finish_op.get();
        finish_op.set(false);
        let retval = c_int::from(edit(NUL, false, 1));
        finish_op.set(save_finish_op);

        // Copy the new text to the rest of a Visual block. Not when Insert
        // mode ended with CTRL-C.
        if (*oap).motion_type == kMTBlockWise
            && (*oap).start.lnum != (*oap).end.lnum
            && !got_int.get()
        {
            replay_change(oap, &mut bd, pre_textlen, pre_indent);
        }

        auto_format(false, true);
        retval
    }
}

/// Copy what `c` inserted on the block's first line into the rest of the
/// block.
///
/// # Safety
/// `oap` must point to a live blockwise `oparg_T`; `bd.textcol` must be the
/// column the insert started at.
unsafe fn replay_change(
    oap: *mut oparg_T,
    bd: &mut block_def,
    mut pre_textlen: c_int,
    pre_indent: c_int,
) {
    unsafe {
        // Auto-indenting may have changed the indent. If the cursor was past
        // the indent, that change is not part of the inserted text.
        let firstline = ml_get((*oap).start.lnum);
        if bd.textcol > pre_indent {
            let new_indent = getwhitecols(firstline) as c_int;
            pre_textlen += new_indent - pre_indent;
            bd.textcol += new_indent - pre_indent;
        }

        let ins_len = ml_get_len((*oap).start.lnum) - pre_textlen;
        if ins_len <= 0 {
            return;
        }

        // A later `ml_get` flushes the line data, so take a copy first.
        let ins_text = xmalloc(ins_len as size_t + 1) as *mut c_char;
        xmemcpyz(
            ins_text as *mut c_void,
            firstline.offset(bd.textcol as isize) as *const c_void,
            ins_len as size_t,
        );

        let mut linenr = (*oap).start.lnum + 1;
        while linenr <= (*oap).end.lnum {
            block_prep(oap, &raw mut *bd, linenr, true);
            if bd.is_short == 0 || op_virtual() {
                // When the block starts in virtual space, that offset is
                // padding in front of the text.
                let mut vpos = pos_T {
                    lnum: linenr,
                    col: 0,
                    coladd: 0,
                };
                if bd.is_short != 0 {
                    getvpos(curwin.get(), &raw mut vpos, (*oap).start_vcol);
                }

                let oldp = ml_get(linenr);
                let newp = xmalloc(
                    ml_get_len(linenr) as size_t + vpos.coladd as size_t + ins_len as size_t + 1,
                ) as *mut c_char;
                // Up to the block's column, then the pad, then the text.
                memmove(
                    newp as *mut c_void,
                    oldp as *const c_void,
                    bd.textcol as size_t,
                );
                let mut newlen = bd.textcol;
                memset(
                    newp.offset(newlen as isize) as *mut c_void,
                    ' ' as c_int,
                    vpos.coladd as size_t,
                );
                newlen += vpos.coladd;
                memmove(
                    newp.offset(newlen as isize) as *mut c_void,
                    ins_text as *const c_void,
                    ins_len as size_t,
                );
                newlen += ins_len;
                strcpy(
                    newp.offset(newlen as isize),
                    oldp.offset(bd.textcol as isize),
                );
                ml_replace(linenr, newp, false);
                extmark_splice_cols(
                    curbuf.get(),
                    linenr as c_int - 1,
                    bd.textcol,
                    0,
                    vpos.coladd + ins_len,
                    kExtmarkUndo,
                );
            }
            linenr += 1;
        }

        check_cursor(curwin.get());
        changed_lines(
            curbuf.get(),
            (*oap).start.lnum + 1,
            0,
            (*oap).end.lnum + 1,
            0,
            true,
        );
        xfree(ins_text as *mut c_void);
    }
}

/// Move the cursor left off the NUL past the end of the line, when it should
/// not be sitting there.
///
/// # Safety
/// Operates on the current window's cursor.
pub unsafe fn adjust_cursor_eol() {
    unsafe {
        let cur_ve_flags = get_ve_flags(curwin.get());
        let adj_cursor = (*curwin.get()).w_cursor.col > 0
            && gchar_cursor() == NUL
            && cur_ve_flags & kOptVeFlagOnemore as c_uint == 0
            && cur_ve_flags & kOptVeFlagAll as c_uint == 0
            && !(restart_edit.get() != 0 || State.get() & MODE_INSERT != 0);
        if !adj_cursor {
            return;
        }

        // Onto the last character in the line.
        dec_cursor();

        if cur_ve_flags == kOptVeFlagAll as c_uint {
            // `coladd` becomes the width of that last character.
            let mut scol: colnr_T = 0;
            let mut ecol: colnr_T = 0;
            getvcol(
                curwin.get(),
                &raw mut (*curwin.get()).w_cursor,
                &raw mut scol,
                ::core::ptr::null_mut(),
                &raw mut ecol,
            );
            (*curwin.get()).w_cursor.coladd = ecol - scol + 1;
        }
    }
}
