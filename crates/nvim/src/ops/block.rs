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

use crate::winlayer::{Buf, Pos, Win};
use core::ffi::{c_char, c_int, c_void};

use super::*;
use crate::r#move::WinValid;
use crate::normal::{sel_exclusive, visual_mode};
use crate::types::NUL;

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
    // SAFETY: the caller's promise -- both point to live structs.
    let (oap, bdp) = unsafe { (&mut *oap, &mut *bdp) };
    let old_state = State.get();
    // Not MODE_REPLACE, whatever the user was in.
    State.set(MODE_INSERT);

    let mut lnum = oap.start.lnum + 1;
    while lnum <= oap.end.lnum {
        // SAFETY: `lnum` walks the region, which is of the current buffer.
        unsafe { block_prep(&raw mut *oap, &raw mut *bdp, lnum, true) };
        if bdp.is_short != 0 && b_insert {
            // `I` on a line that ends before the block starts.
            lnum += 1;
            continue;
        }

        // SAFETY: `lnum` is a line of the current buffer.
        let mut oldp = ml_get(lnum);
        let oldlen = ml_get_len(lnum) as size_t;

        // `spaces` is non-zero when a TAB has to be cut, and `count` the
        // extra spaces that replace it. `ts_val` is the cut character's
        // width.
        let ts_val;
        let mut count = 0;
        let mut spaces = 0;
        let mut offset: colnr_T;
        if b_insert {
            ts_val = bdp.start_char_vcols;
            spaces = bdp.startspaces;
            if spaces != 0 {
                count = ts_val - 1;
            }
            offset = bdp.textcol;
        } else if bdp.is_short == 0 {
            // Append, with padding after the block.
            ts_val = bdp.end_char_vcols;
            spaces = if bdp.endspaces != 0 {
                ts_val - bdp.endspaces
            } else {
                0
            };
            if spaces != 0 {
                count = ts_val - 1;
            }
            offset = bdp.textcol + bdp.textlen - c_int::from(spaces != 0);
        } else {
            // Append past the end of a short line: pad out to the block's
            // edge, unless `$` made the block open-ended.
            ts_val = bdp.end_char_vcols;
            if bdp.is_MAX == 0 {
                spaces = oap.end_vcol - bdp.end_vcol + 1;
            }
            count = spaces;
            offset = bdp.textcol + bdp.textlen;
        }

        if spaces > 0 {
            // Do not copy part of a multi-byte character.
            // SAFETY: `offset` is a byte index into the line.
            offset -= unsafe { utf_head_off(oldp, oldp.offset(offset as isize)) };
        }
        // Can go negative when the cursor was moved.
        spaces = spaces.max(0);
        debug_assert!(count >= 0);

        // The allocation has to match exactly what is copied below.
        let extra = if spaces > 0 && bdp.is_short == 0 {
            (ts_val - spaces) as size_t
        } else {
            0
        };
        let size = oldlen + spaces as size_t + slen + extra + count as size_t + 1;
        // SAFETY: `size` counts every byte the rebuild below writes, the
        // terminating NUL included.
        let newp = unsafe { xmalloc(size) } as *mut c_char;

        let startcol = offset;
        let mut skipped = 0;
        // SAFETY: `newp` was allocated for exactly what is written here, and
        // `oldp` is a NUL-terminated buffer line at least `offset` bytes long.
        // Up to the shifted part.
        unsafe { newp.cast::<u8>().copy_from(oldp.cast(), offset as size_t) };
        oldp = unsafe { oldp.offset(offset as isize) };

        // Pre-padding, then the new text.
        let pad = unsafe { newp.offset(offset as isize) } as *mut c_void;
        unsafe { pad.cast::<u8>().write_bytes(b' ', spaces as size_t) };
        let at = unsafe { newp.offset((offset + spaces as colnr_T) as isize) } as *mut c_void;
        unsafe { at.cast::<u8>().copy_from(s.cast(), slen) };
        offset += slen as colnr_T;

        if spaces > 0 && bdp.is_short == 0 {
            if unsafe { *oldp } as c_int == TAB {
                // Post-padding: the rest of the TAB being split, which is
                // then dropped rather than copied.
                let tail =
                    unsafe { newp.offset((offset + spaces as colnr_T) as isize) } as *mut c_void;
                let into = tail.cast::<u8>();
                unsafe { into.write_bytes(b' ', (ts_val - spaces) as size_t) };
                oldp = unsafe { oldp.offset(1) };
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
        unsafe { strcpy(newp.offset(offset as isize), oldp) };

        let _ = unsafe { ml_replace(lnum, newp, false) };
        let splice = offset - startcol;
        unsafe {
            extmark_splice_cols(
                curbuf.get(),
                lnum as c_int - 1,
                startcol,
                skipped,
                splice,
                kExtmarkUndo,
            )
        };

        if lnum == oap.end.lnum {
            // `']` goes to the end of the block, not the end of the insert
            // in the first line.
            cur_buf().b_op_end.lnum = oap.end.lnum;
            cur_buf().b_op_end.col = offset;
            if cur_buf().b_visual.vi_end.coladd != 0 {
                cur_buf().b_visual.vi_end.col += cur_buf().b_visual.vi_end.coladd;
                cur_buf().b_visual.vi_end.coladd = 0;
            }
        }
        lnum += 1;
    }

    State.set(old_state);

    // Only if lines past the first were actually modified, which is the
    // loop's own bound.
    if oap.start.lnum < oap.end.lnum {
        let (first, last) = (oap.start.lnum + 1, oap.end.lnum + 1);
        // SAFETY: both name lines of the current buffer.
        changed_lines(cur_buf(), first, 0, last, 0, true);
    }
}

/// Turn 'linebreak' off, answering whether it was on.
///
/// Pass the answer to [`restore_lbr`]. Every column measured for a blockwise
/// operation has to be measured with 'linebreak' off, because it changes what
/// `getvcol` answers.
///
/// Safe: the only thing it touches is the current window, which is what
/// [`cur_win`] already promises.
pub fn reset_lbr() -> bool {
    if cur_win().w_onebuf_opt.wo_lbr == 0 {
        return false;
    }
    cur_win().w_onebuf_opt.wo_lbr = 0;
    // Changing 'linebreak' may require w_virtcol to be recomputed.
    cur_win()
        .w_valid
        .clear(WinValid::WROW | WinValid::WCOL | WinValid::VIRTCOL);
    true
}

/// Put 'linebreak' back if [`reset_lbr`] took it off.
///
/// Safe for the same reason [`reset_lbr`] is.
pub fn restore_lbr(lbr_saved: bool) {
    if cur_win().w_onebuf_opt.wo_lbr != 0 || !lbr_saved {
        return;
    }
    cur_win().w_onebuf_opt.wo_lbr = 1;
    cur_win()
        .w_valid
        .clear(WinValid::WROW | WinValid::WCOL | WinValid::VIRTCOL);
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
    // SAFETY: the caller's promise -- both point to live structs.
    let (oap, bdp) = unsafe { (&mut *oap, &mut *bdp) };
    // Unwanted line breaks would move every column measured below.
    let lbr_saved = reset_lbr();

    // Everything but `textcol`, `textstart` (written at the end) and
    // `is_MAX` (the caller's, meaning the block was opened with `$`).
    bdp.startspaces = 0;
    bdp.endspaces = 0;
    bdp.textlen = 0;
    bdp.start_vcol = 0;
    bdp.end_vcol = 0;
    bdp.is_short = 0;
    bdp.is_oneChar = 0;
    bdp.pre_whitesp = 0;
    bdp.pre_whitesp_c = 0;
    bdp.end_char_vcols = 0;
    bdp.start_char_vcols = 0;

    // SAFETY: `lnum` is a line of the current buffer, so the line is a live
    // NUL-terminated string and every walk below stays inside it.
    let line = ml_get(lnum);
    let mut prev_pstart = line;

    // Walk to the block's left edge, remembering the run of white space in
    // front of it (`shift_block` widens exactly that run).
    let mut incr = 0;
    let mut csarg = CharsizeArg::default();
    let mut cstype = unsafe { init_charsize_arg(&mut csarg, Win::new(curwin.get()), lnum, line) };
    let mut ci: StrCharInfo = unsafe { utf_ptr2str_char_info(line) };
    let mut vcol = bdp.start_vcol;
    while vcol < oap.start_vcol && unsafe { *ci.ptr } as c_int != NUL {
        incr = unsafe { win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &mut csarg) }.width;
        vcol += incr;
        if ascii_iswhite(ci.chr.value) {
            bdp.pre_whitesp += incr;
            bdp.pre_whitesp_c += 1;
        } else {
            bdp.pre_whitesp = 0;
            bdp.pre_whitesp_c = 0;
        }
        prev_pstart = ci.ptr;
        ci = unsafe { utfc_next(ci) };
    }
    bdp.start_vcol = vcol;
    let mut pstart = ci.ptr;
    bdp.start_char_vcols = incr;

    if bdp.start_vcol < oap.start_vcol {
        // The line ends before the block starts.
        bdp.end_vcol = bdp.start_vcol;
        bdp.is_short = 1;
        if !is_del || oap.op_type == OpType::Append {
            bdp.endspaces = oap.end_vcol - oap.start_vcol + 1;
        }
    } else {
        bdp.startspaces = bdp.start_vcol - oap.start_vcol;
        if is_del && bdp.startspaces != 0 {
            bdp.startspaces = bdp.start_char_vcols - bdp.startspaces;
        }
        let mut pend = pstart;
        bdp.end_vcol = bdp.start_vcol;

        if bdp.end_vcol > oap.end_vcol {
            // The whole block is inside one character -- a wide TAB.
            bdp.is_oneChar = 1;
            if oap.op_type == OpType::Insert {
                bdp.endspaces = bdp.start_char_vcols - bdp.startspaces;
            } else if oap.op_type == OpType::Append {
                bdp.startspaces += oap.end_vcol - oap.start_vcol + 1;
                bdp.endspaces = bdp.start_char_vcols - bdp.startspaces;
            } else {
                bdp.startspaces = oap.end_vcol - oap.start_vcol + 1;
                if is_del && oap.op_type != OpType::Lshift {
                    // Summing the two into `startspaces` does not work for
                    // a Visual replace, so the TAB is split in two.
                    bdp.startspaces = bdp.start_char_vcols - (bdp.start_vcol - oap.start_vcol);
                    bdp.endspaces = bdp.end_vcol - oap.end_vcol - 1;
                }
            }
        } else {
            // Walk on to the block's right edge.
            cstype = unsafe { init_charsize_arg(&mut csarg, Win::new(curwin.get()), lnum, line) };
            ci = unsafe { utf_ptr2str_char_info(pend) };
            vcol = bdp.end_vcol;
            let mut prev_pend = pend;
            while vcol <= oap.end_vcol && unsafe { *ci.ptr } as c_int != NUL {
                prev_pend = ci.ptr;
                incr =
                    unsafe { win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &mut csarg) }.width;
                vcol += incr;
                ci = unsafe { utfc_next(ci) };
            }
            bdp.end_vcol = vcol;
            pend = ci.ptr;

            if bdp.end_vcol <= oap.end_vcol
                && (!is_del || oap.op_type == OpType::Append || oap.op_type == OpType::Replace)
            {
                // The line ends inside the block. Filling it out to the
                // block's width is the alternative, and it is deliberately
                // not done: it leaves trailing white space behind.
                bdp.is_short = 1;
                if oap.op_type == OpType::Append || op_virtual() {
                    bdp.endspaces = oap.end_vcol - bdp.end_vcol + c_int::from(oap.inclusive);
                }
            } else if bdp.end_vcol > oap.end_vcol {
                bdp.endspaces = bdp.end_vcol - oap.end_vcol - 1;
                if !is_del && bdp.endspaces != 0 {
                    bdp.endspaces = incr - bdp.endspaces;
                    if pend != pstart {
                        pend = prev_pend;
                    }
                }
            }
        }

        bdp.end_char_vcols = incr;
        if is_del && bdp.startspaces != 0 {
            pstart = prev_pstart;
        }
        // SAFETY: `pstart` and `pend` are both inside `line`.
        bdp.textlen = unsafe { pend.offset_from(pstart) } as c_int;
    }

    bdp.textcol = unsafe { pstart.offset_from(line) } as colnr_T;
    bdp.textstart = pstart;
    restore_lbr(lbr_saved);
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
    // SAFETY: the caller's promise -- `bdp` is live and `lnum` is a line of
    // the current buffer, so `p` is a live NUL-terminated line of `plen`
    // bytes and every index taken of it below is a column of that line.
    let bdp = unsafe { &mut *bdp };
    let p = ml_get(lnum);
    let plen = ml_get_len(lnum);

    bdp.startspaces = 0;
    bdp.endspaces = 0;
    bdp.is_oneChar = 0;
    bdp.start_char_vcols = 0;

    let mut startcol: colnr_T = 0;
    let mut endcol: colnr_T = MAXCOL;

    if lnum == start.lnum {
        startcol = start.col;
        if op_virtual() {
            let at = unsafe { Pos::new(&raw mut start) };
            let (cs, ce) = cur_win().vcol_span(at);
            if ce != cs && start.coladd > 0 {
                // Part of a TAB is selected -- but do not double-count it.
                bdp.start_char_vcols = ce - cs + 1;
                bdp.startspaces = (bdp.start_char_vcols - start.coladd).max(0);
                startcol += 1;
            }
        }
    }

    if lnum == end.lnum {
        endcol = end.col;
        if op_virtual() {
            let at = unsafe { Pos::new(&raw mut end) };
            let (cs, ce) = cur_win().vcol_span(at);
            // No padding for a double-width character: `endcol` is then on
            // the last byte of the character, not past it.
            let mid_char = || unsafe { utf_head_off(p, p.offset(endcol as isize)) } == 0;
            if unsafe { *p.offset(endcol as isize) } as c_int == NUL
                || (cs + end.coladd < ce && mid_char())
            {
                if start.lnum == end.lnum && start.col == end.col {
                    // The whole region is inside one character.
                    bdp.is_oneChar = 1;
                    bdp.startspaces = end.coladd - start.coladd + c_int::from(inclusive);
                    endcol = startcol;
                } else {
                    bdp.endspaces = end.coladd + c_int::from(inclusive);
                    endcol -= c_int::from(inclusive);
                }
            }
        }
    }

    if endcol == MAXCOL {
        endcol = plen;
    }
    bdp.textlen = if startcol > endcol || bdp.is_oneChar != 0 {
        0
    } else {
        endcol - startcol + c_int::from(inclusive)
    };
    bdp.textcol = startcol;
    bdp.textstart = if startcol <= plen {
        unsafe { p.offset(startcol as isize) }
    } else {
        p
    };
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
pub(crate) fn get_op_vcol(mut oap: Op, redo_visual_vcol: colnr_T, initial: bool) {
    if !visual_mode().is_block() || (!initial && oap.end.col < cur_win().w_view_width) {
        return;
    }

    oap.motion_type = kMTBlockWise;
    // Do not let the end land on a trail byte.
    cur_win().buffer().snap_to_char(oap.end());

    (oap.start_vcol, oap.end_vcol) = cur_win().virtual_vcol_span(oap.start());
    if !redo_VIsual_busy.get() {
        let (start, end) = cur_win().virtual_vcol_span(oap.end());
        oap.start_vcol = oap.start_vcol.min(start);
        if end > oap.end_vcol {
            if initial && sel_exclusive() && start >= 1 && start > oap.end_vcol {
                oap.end_vcol = start - 1;
            } else {
                oap.end_vcol = end;
            }
        }
    }

    if cur_win().w_curswant == MAXCOL {
        // `$` was used: the block's right edge is the longest line's.
        cur_win().w_cursor.col = MAXCOL;
        oap.end_vcol = 0;
        cur_win().w_cursor.lnum = oap.start.lnum;
        let cursor = cur_win().cursor();
        while cur_win().w_cursor.lnum <= oap.end.lnum {
            let (_, end) = cur_win().virtual_vcol_span(cursor);
            oap.end_vcol = oap.end_vcol.max(end);
            cur_win().w_cursor.lnum += 1;
        }
    } else if redo_VIsual_busy.get() {
        oap.end_vcol = oap.start_vcol + redo_visual_vcol - 1;
    }

    // Turn the column pair back into the block's two corner *positions*.
    cur_win().w_cursor.lnum = oap.end.lnum;
    cur_win().coladvance(oap.end_vcol);
    oap.end = cur_win().w_cursor;

    cur_win().w_cursor = oap.start;
    cur_win().coladvance(oap.start_vcol);
    oap.start = cur_win().w_cursor;
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
