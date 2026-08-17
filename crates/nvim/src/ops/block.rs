//! Blockwise geometry: turning a CTRL-V region into per-line byte ranges.
//!
//! A blockwise operator sees a rectangle of *screen columns* and the buffer
//! holds bytes, so every blockwise operation starts here.
//!
//! [`block_prep`] does the translation for one line, filling a `block_def`
//! with where the block starts and ends in that line, how many columns of
//! padding belong on either side because a TAB or a wide character straddles
//! an edge, and whether the line is too short to reach the block at all. Its
//! answer means slightly different things per operator, which is upstream's
//! own note and worth repeating:
//!
//! * for a **delete**, `textlen` covers the first and last characters that are
//!   even *partly* deleted, and `startspaces`/`endspaces` are the columns of
//!   those characters that survive;
//! * for a **yank** or `g~`, `textlen` covers only whole characters, and
//!   `startspaces`/`endspaces` are the columns of the straddling characters
//!   that are taken.
//!
//! [`charwise_block_prep`] answers the same question for a *charwise* region
//! being treated as a block (`getregion()` and the register API),
//! [`get_op_vcol`] decides the column pair an operator's region spans, and
//! [`block_insert`] is the write half: `I` and `A` putting the same text into
//! every line of the block.
//!
//! [`reset_lbr`]/[`restore_lbr`] bracket all of it, because 'linebreak'
//! changes what `getvcol` answers and every column here has to be measured
//! with it off.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};

use super::*;

/// Insert `s` into every line of the block below the first, before the block
/// (`b_insert`, for `I`) or after it (for `A`).
///
/// The first line was inserted into interactively; this is the replay over the
/// rest, and it runs once, after Insert mode has been left. The caller
/// prepares undo.
///
/// # Safety
/// `oap` and `bdp` must point to live structs; `s` must be `slen` readable
/// bytes.
pub(crate) unsafe fn block_insert(
    oap: *mut oparg_T,
    s: *const c_char,
    slen: size_t,
    b_insert: bool,
    bdp: *mut block_def,
) {
    unsafe {
        let old_state = State.get();
        // Not MODE_REPLACE, whatever the user was in.
        State.set(MODE_INSERT);

        let mut lnum = (*oap).start.lnum + 1;
        while lnum <= (*oap).end.lnum {
            block_prep(oap, bdp, lnum, true);
            if (*bdp).is_short != 0 && b_insert {
                // `I` on a line that ends before the block starts.
                lnum += 1;
                continue;
            }

            let mut oldp = ml_get(lnum);

            // `spaces` is non-zero when a TAB has to be cut, and `count` the
            // extra spaces that replace it. `ts_val` is the cut character's
            // width.
            let ts_val;
            let mut count = 0;
            let mut spaces = 0;
            let mut offset: colnr_T;
            if b_insert {
                ts_val = (*bdp).start_char_vcols;
                spaces = (*bdp).startspaces;
                if spaces != 0 {
                    count = ts_val - 1;
                }
                offset = (*bdp).textcol;
            } else if (*bdp).is_short == 0 {
                // Append, with padding after the block.
                ts_val = (*bdp).end_char_vcols;
                spaces = if (*bdp).endspaces != 0 {
                    ts_val - (*bdp).endspaces
                } else {
                    0
                };
                if spaces != 0 {
                    count = ts_val - 1;
                }
                offset = (*bdp).textcol + (*bdp).textlen - c_int::from(spaces != 0);
            } else {
                // Append past the end of a short line: pad out to the block's
                // edge, unless `$` made the block open-ended.
                ts_val = (*bdp).end_char_vcols;
                if (*bdp).is_MAX == 0 {
                    spaces = (*oap).end_vcol - (*bdp).end_vcol + 1;
                }
                count = spaces;
                offset = (*bdp).textcol + (*bdp).textlen;
            }

            if spaces > 0 {
                // Do not copy part of a multi-byte character.
                offset -= utf_head_off(oldp, oldp.offset(offset as isize));
            }
            // Can go negative when the cursor was moved.
            spaces = spaces.max(0);
            debug_assert!(count >= 0);

            // The allocation has to match exactly what is copied below.
            let extra = if spaces > 0 && (*bdp).is_short == 0 {
                (ts_val - spaces) as size_t
            } else {
                0
            };
            let newp = xmalloc(
                ml_get_len(lnum) as size_t + spaces as size_t + slen + extra + count as size_t + 1,
            ) as *mut c_char;

            // Up to the shifted part.
            memmove(newp as *mut c_void, oldp as *const c_void, offset as size_t);
            oldp = oldp.offset(offset as isize);
            let startcol = offset;

            // Pre-padding, then the new text.
            memset(
                newp.offset(offset as isize) as *mut c_void,
                ' ' as c_int,
                spaces as size_t,
            );
            memmove(
                newp.offset((offset + spaces as colnr_T) as isize) as *mut c_void,
                s as *const c_void,
                slen,
            );
            offset += slen as colnr_T;

            let mut skipped = 0;
            if spaces > 0 && (*bdp).is_short == 0 {
                if *oldp as c_int == TAB {
                    // Post-padding: the rest of the TAB being split, which is
                    // then dropped rather than copied.
                    memset(
                        newp.offset((offset + spaces as colnr_T) as isize) as *mut c_void,
                        ' ' as c_int,
                        (ts_val - spaces) as size_t,
                    );
                    oldp = oldp.offset(1);
                    count += 1;
                    skipped = 1;
                } else {
                    // Not a TAB, so no extra spaces.
                    count = spaces;
                }
            }
            if spaces > 0 {
                offset += count;
            }
            strcpy(newp.offset(offset as isize), oldp);

            ml_replace(lnum, newp, false);
            extmark_splice_cols(
                curbuf.get(),
                lnum as c_int - 1,
                startcol,
                skipped,
                offset - startcol,
                kExtmarkUndo,
            );

            if lnum == (*oap).end.lnum {
                // `']` goes to the end of the block, not the end of the insert
                // in the first line.
                (*curbuf.get()).b_op_end.lnum = (*oap).end.lnum;
                (*curbuf.get()).b_op_end.col = offset;
                if (*curbuf.get()).b_visual.vi_end.coladd != 0 {
                    (*curbuf.get()).b_visual.vi_end.col += (*curbuf.get()).b_visual.vi_end.coladd;
                    (*curbuf.get()).b_visual.vi_end.coladd = 0;
                }
            }
            lnum += 1;
        }

        State.set(old_state);

        // Only if lines past the first were actually modified, which is the
        // loop's own bound.
        if (*oap).start.lnum < (*oap).end.lnum {
            changed_lines(
                curbuf.get(),
                (*oap).start.lnum + 1,
                0,
                (*oap).end.lnum + 1,
                0,
                true,
            );
        }
    }
}

/// Turn 'linebreak' off, answering whether it was on.
///
/// Pass the answer to [`restore_lbr`]. Every column measured for a blockwise
/// operation has to be measured with 'linebreak' off, because it changes what
/// `getvcol` answers.
///
/// # Safety
/// Touches the current window.
pub unsafe fn reset_lbr() -> bool {
    unsafe {
        if (*curwin.get()).w_onebuf_opt.wo_lbr == 0 {
            return false;
        }
        (*curwin.get()).w_onebuf_opt.wo_lbr = false_0;
        // Changing 'linebreak' may require w_virtcol to be recomputed.
        (*curwin.get()).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL);
        true
    }
}

/// Put 'linebreak' back if [`reset_lbr`] took it off.
///
/// # Safety
/// Touches the current window.
pub unsafe fn restore_lbr(lbr_saved: bool) {
    unsafe {
        if (*curwin.get()).w_onebuf_opt.wo_lbr != 0 || !lbr_saved {
            return;
        }
        (*curwin.get()).w_onebuf_opt.wo_lbr = true_0;
        (*curwin.get()).w_valid &= !(VALID_WROW | VALID_WCOL | VALID_VIRTCOL);
    }
}

/// Where the block sits in one line: fill `bdp` for `lnum`.
///
/// `is_del` selects the delete reading of the answer described in the module
/// doc; every other operator passes false.
///
/// Note that this converts a partly selected multi-byte character to spaces,
/// like a partly selected TAB.
///
/// # Safety
/// `oap` and `bdp` must point to live structs, and `lnum` must be a line of
/// the current buffer.
pub unsafe fn block_prep(oap: *mut oparg_T, bdp: *mut block_def, lnum: linenr_T, is_del: bool) {
    unsafe {
        // Unwanted line breaks would move every column measured below.
        let lbr_saved = reset_lbr();

        // Everything but `textcol`, `textstart` (written at the end) and
        // `is_MAX` (the caller's, meaning the block was opened with `$`).
        (*bdp).startspaces = 0;
        (*bdp).endspaces = 0;
        (*bdp).textlen = 0;
        (*bdp).start_vcol = 0;
        (*bdp).end_vcol = 0;
        (*bdp).is_short = false_0;
        (*bdp).is_oneChar = false_0;
        (*bdp).pre_whitesp = 0;
        (*bdp).pre_whitesp_c = 0;
        (*bdp).end_char_vcols = 0;
        (*bdp).start_char_vcols = 0;

        let line = ml_get(lnum);
        let mut prev_pstart = line;

        // Walk to the block's left edge, remembering the run of white space in
        // front of it (`shift_block` widens exactly that run).
        let mut incr = 0;
        let mut csarg = CharsizeArg::default();
        let mut cstype = init_charsize_arg(&mut csarg, curwin.get(), lnum, line);
        let mut ci: StrCharInfo = utf_ptr2StrCharInfo(line);
        let mut vcol = (*bdp).start_vcol;
        while vcol < (*oap).start_vcol && *ci.ptr as c_int != NUL {
            incr = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &mut csarg).width;
            vcol += incr;
            if ascii_iswhite(ci.chr.value) {
                (*bdp).pre_whitesp += incr;
                (*bdp).pre_whitesp_c += 1;
            } else {
                (*bdp).pre_whitesp = 0;
                (*bdp).pre_whitesp_c = 0;
            }
            prev_pstart = ci.ptr;
            ci = utfc_next(ci);
        }
        (*bdp).start_vcol = vcol;
        let mut pstart = ci.ptr;
        (*bdp).start_char_vcols = incr;

        if (*bdp).start_vcol < (*oap).start_vcol {
            // The line ends before the block starts.
            (*bdp).end_vcol = (*bdp).start_vcol;
            (*bdp).is_short = true_0;
            if !is_del || (*oap).op_type == OP_APPEND {
                (*bdp).endspaces = (*oap).end_vcol - (*oap).start_vcol + 1;
            }
        } else {
            (*bdp).startspaces = (*bdp).start_vcol - (*oap).start_vcol;
            if is_del && (*bdp).startspaces != 0 {
                (*bdp).startspaces = (*bdp).start_char_vcols - (*bdp).startspaces;
            }
            let mut pend = pstart;
            (*bdp).end_vcol = (*bdp).start_vcol;

            if (*bdp).end_vcol > (*oap).end_vcol {
                // The whole block is inside one character -- a wide TAB.
                (*bdp).is_oneChar = true_0;
                if (*oap).op_type == OP_INSERT {
                    (*bdp).endspaces = (*bdp).start_char_vcols - (*bdp).startspaces;
                } else if (*oap).op_type == OP_APPEND {
                    (*bdp).startspaces += (*oap).end_vcol - (*oap).start_vcol + 1;
                    (*bdp).endspaces = (*bdp).start_char_vcols - (*bdp).startspaces;
                } else {
                    (*bdp).startspaces = (*oap).end_vcol - (*oap).start_vcol + 1;
                    if is_del && (*oap).op_type != OP_LSHIFT {
                        // Summing the two into `startspaces` does not work for
                        // a Visual replace, so the TAB is split in two.
                        (*bdp).startspaces =
                            (*bdp).start_char_vcols - ((*bdp).start_vcol - (*oap).start_vcol);
                        (*bdp).endspaces = (*bdp).end_vcol - (*oap).end_vcol - 1;
                    }
                }
            } else {
                // Walk on to the block's right edge.
                cstype = init_charsize_arg(&mut csarg, curwin.get(), lnum, line);
                ci = utf_ptr2StrCharInfo(pend);
                vcol = (*bdp).end_vcol;
                let mut prev_pend = pend;
                while vcol <= (*oap).end_vcol && *ci.ptr as c_int != NUL {
                    prev_pend = ci.ptr;
                    incr = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &mut csarg).width;
                    vcol += incr;
                    ci = utfc_next(ci);
                }
                (*bdp).end_vcol = vcol;
                pend = ci.ptr;

                if (*bdp).end_vcol <= (*oap).end_vcol
                    && (!is_del || (*oap).op_type == OP_APPEND || (*oap).op_type == OP_REPLACE)
                {
                    // The line ends inside the block. Filling it out to the
                    // block's width is the alternative, and it is deliberately
                    // not done: it leaves trailing white space behind.
                    (*bdp).is_short = true_0;
                    if (*oap).op_type == OP_APPEND || virtual_op.get() != 0 {
                        (*bdp).endspaces =
                            (*oap).end_vcol - (*bdp).end_vcol + c_int::from((*oap).inclusive);
                    }
                } else if (*bdp).end_vcol > (*oap).end_vcol {
                    (*bdp).endspaces = (*bdp).end_vcol - (*oap).end_vcol - 1;
                    if !is_del && (*bdp).endspaces != 0 {
                        (*bdp).endspaces = incr - (*bdp).endspaces;
                        if pend != pstart {
                            pend = prev_pend;
                        }
                    }
                }
            }

            (*bdp).end_char_vcols = incr;
            if is_del && (*bdp).startspaces != 0 {
                pstart = prev_pstart;
            }
            (*bdp).textlen = pend.offset_from(pstart) as c_int;
        }

        (*bdp).textcol = pstart.offset_from(line) as colnr_T;
        (*bdp).textstart = pstart;
        restore_lbr(lbr_saved);
    }
}

/// Where a *charwise* region sits in one line: fill `bdp` for `lnum`.
///
/// The region is `start`..`end` rather than a rectangle, so only the first and
/// last lines are clipped; everything between is the whole line. The
/// 'virtualedit' arms are what make it more than that: an end inside a TAB
/// becomes `endspaces` columns of padding, and a region that starts and ends
/// inside the *same* character is `is_oneChar` with no text at all.
///
/// # Safety
/// `bdp` must point to a live struct and `lnum` must be a line of the current
/// buffer.
pub unsafe fn charwise_block_prep(
    mut start: pos_T,
    mut end: pos_T,
    bdp: *mut block_def,
    lnum: linenr_T,
    inclusive: bool,
) {
    unsafe {
        let p = ml_get(lnum);
        let plen = ml_get_len(lnum);

        (*bdp).startspaces = 0;
        (*bdp).endspaces = 0;
        (*bdp).is_oneChar = false_0;
        (*bdp).start_char_vcols = 0;

        let mut startcol: colnr_T = 0;
        let mut endcol: colnr_T = MAXCOL;
        let mut cs: colnr_T = 0;
        let mut ce: colnr_T = 0;

        if lnum == start.lnum {
            startcol = start.col;
            if virtual_op.get() != 0 {
                getvcol(
                    curwin.get(),
                    &raw mut start,
                    &raw mut cs,
                    ::core::ptr::null_mut(),
                    &raw mut ce,
                );
                if ce != cs && start.coladd > 0 {
                    // Part of a TAB is selected -- but do not double-count it.
                    (*bdp).start_char_vcols = ce - cs + 1;
                    (*bdp).startspaces = ((*bdp).start_char_vcols - start.coladd).max(0);
                    startcol += 1;
                }
            }
        }

        if lnum == end.lnum {
            endcol = end.col;
            if virtual_op.get() != 0 {
                getvcol(
                    curwin.get(),
                    &raw mut end,
                    &raw mut cs,
                    ::core::ptr::null_mut(),
                    &raw mut ce,
                );
                // No padding for a double-width character: `endcol` is then on
                // the last byte of the character, not past it.
                if *p.offset(endcol as isize) as c_int == NUL
                    || (cs + end.coladd < ce && utf_head_off(p, p.offset(endcol as isize)) == 0)
                {
                    if start.lnum == end.lnum && start.col == end.col {
                        // The whole region is inside one character.
                        (*bdp).is_oneChar = true_0;
                        (*bdp).startspaces = end.coladd - start.coladd + c_int::from(inclusive);
                        endcol = startcol;
                    } else {
                        (*bdp).endspaces = end.coladd + c_int::from(inclusive);
                        endcol -= c_int::from(inclusive);
                    }
                }
            }
        }

        if endcol == MAXCOL {
            endcol = ml_get_len(lnum);
        }
        (*bdp).textlen = if startcol > endcol || (*bdp).is_oneChar != 0 {
            0
        } else {
            endcol - startcol + c_int::from(inclusive)
        };
        (*bdp).textcol = startcol;
        (*bdp).textstart = if startcol <= plen {
            p.offset(startcol as isize)
        } else {
            p
        };
    }
}

/// Compute `oap`'s `start_vcol`/`end_vcol` and square its corners up.
///
/// Only does anything for a CTRL-V selection. Afterwards `oap.start` and
/// `oap.end` are the block's upper-left and lower-right corners as *character*
/// positions, which is what every blockwise operator then works from.
///
/// `initial` is false when replaying, and asks for the 'selection' adjustment
/// to be skipped. `redo_visual_vcol` is the recorded width a `.` replay uses
/// instead of measuring the selection again.
///
/// # Safety
/// `oap` must point to a live `oparg_T` describing a region of the current
/// buffer.
pub(crate) unsafe fn get_op_vcol(oap: *mut oparg_T, redo_visual_vcol: colnr_T, initial: bool) {
    unsafe {
        if VIsual_mode.get() != Ctrl_V
            || (!initial && (*oap).end.col < (*curwin.get()).w_view_width)
        {
            return;
        }

        (*oap).motion_type = kMTBlockWise;
        // Do not let the end land on a trail byte.
        mark_mb_adjustpos((*curwin.get()).w_buffer, &raw mut (*oap).end);

        getvvcol(
            curwin.get(),
            &raw mut (*oap).start,
            &raw mut (*oap).start_vcol,
            ::core::ptr::null_mut(),
            &raw mut (*oap).end_vcol,
        );
        if !redo_VIsual_busy.get() {
            let mut start: colnr_T = 0;
            let mut end: colnr_T = 0;
            getvvcol(
                curwin.get(),
                &raw mut (*oap).end,
                &raw mut start,
                ::core::ptr::null_mut(),
                &raw mut end,
            );
            (*oap).start_vcol = (*oap).start_vcol.min(start);
            if end > (*oap).end_vcol {
                if initial
                    && *p_sel.get() as c_int == 'e' as c_int
                    && start >= 1
                    && start > (*oap).end_vcol
                {
                    (*oap).end_vcol = start - 1;
                } else {
                    (*oap).end_vcol = end;
                }
            }
        }

        if (*curwin.get()).w_curswant == MAXCOL {
            // `$` was used: the block's right edge is the longest line's.
            (*curwin.get()).w_cursor.col = MAXCOL;
            (*oap).end_vcol = 0;
            (*curwin.get()).w_cursor.lnum = (*oap).start.lnum;
            while (*curwin.get()).w_cursor.lnum <= (*oap).end.lnum {
                let mut end: colnr_T = 0;
                getvvcol(
                    curwin.get(),
                    &raw mut (*curwin.get()).w_cursor,
                    ::core::ptr::null_mut(),
                    ::core::ptr::null_mut(),
                    &raw mut end,
                );
                (*oap).end_vcol = (*oap).end_vcol.max(end);
                (*curwin.get()).w_cursor.lnum += 1;
            }
        } else if redo_VIsual_busy.get() {
            (*oap).end_vcol = (*oap).start_vcol + redo_visual_vcol - 1;
        }

        // Turn the column pair back into the block's two corner *positions*.
        (*curwin.get()).w_cursor.lnum = (*oap).end.lnum;
        coladvance(curwin.get(), (*oap).end_vcol);
        (*oap).end = (*curwin.get()).w_cursor;

        (*curwin.get()).w_cursor = (*oap).start;
        coladvance(curwin.get(), (*oap).start_vcol);
        (*oap).start = (*curwin.get()).w_cursor;
    }
}
