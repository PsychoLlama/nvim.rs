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

#![forbid(unsafe_code)]

use crate::winlayer::{Buf, Win};
use core::ffi::c_int;

use super::*;
use crate::mbyte::{c_char_info, c_next_cluster, snap_pos_to_char};
use crate::r#move::WinValid;
use crate::normal::{sel_exclusive, visual_mode};
use crate::plines::{getvcol, getvvcol};
use crate::types::CharInfo;

/// Insert `s` into every line of the block below the first, before the block
/// (`b_insert`, for `I`) or after it (for `A`).
///
/// The first line was inserted into interactively; this is the replay over the
/// rest, and it runs once, after Insert mode has been left. The caller
/// prepares undo.
pub(crate) fn block_insert(op: &OpArg, s: &[u8], b_insert: bool, bdp: &mut BlockDef) {
    let old_state = State.get();
    // Not MODE_REPLACE, whatever the user was in.
    State.set(MODE_INSERT);

    let mut lnum = op.start.lnum + 1;
    while lnum <= op.end.lnum {
        block_prep(bdp, op, lnum, true);
        if bdp.is_short != 0 && b_insert {
            // `I` on a line that ends before the block starts.
            lnum += 1;
            continue;
        }

        // `spaces` is non-zero when a TAB has to be cut, and `count` the
        // extra spaces that replace it. `ts_val` is the cut character's
        // width.
        let ts_val;
        let mut count = 0;
        let mut spaces = 0;
        let mut offset: ColNr;
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
            if bdp.is_max == 0 {
                spaces = op.end_vcol - bdp.end_vcol + 1;
            }
            count = spaces;
            offset = bdp.textcol + bdp.textlen;
        }

        // The rebuilt line, and where the tail of the old one resumes. The
        // borrow ends before `ml_replace` writes it back.
        let mut skipped = 0;
        let (line, startcol) = {
            let mut lines = Lines::current();
            let oldp = lines.line(lnum);

            if spaces > 0 {
                // Do not copy part of a multi-byte character.
                offset -= head_off(oldp, offset as usize) as ColNr;
            }
            // Can go negative when the cursor was moved.
            spaces = spaces.max(0);
            debug_assert!(count >= 0);

            let at = usize::try_from(offset).unwrap_or(0).min(oldp.len());
            let mut line = Vec::with_capacity(oldp.len() + spaces as usize + s.len() + 1);
            line.extend_from_slice(&oldp[..at]);
            line.resize(line.len() + spaces as usize, b' ');
            line.extend_from_slice(s);

            let startcol = offset;
            offset += s.len() as ColNr;
            let mut tail = at;
            if spaces > 0 && bdp.is_short == 0 {
                if byte_at(oldp, at) == TAB as u8 {
                    // Post-padding: the rest of the TAB being split, which
                    // is then dropped rather than copied.
                    line.resize(line.len() + (ts_val - spaces) as usize, b' ');
                    tail = at + 1;
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
            line.extend_from_slice(&oldp[tail.min(oldp.len())..]);
            (line, startcol)
        };

        let _ = ml_replace_buf_text(Buf::current(), lnum, &line);
        let splice = offset - startcol;
        extmark_splice_cols(
            Buf::current(),
            lnum as c_int - 1,
            startcol,
            skipped,
            splice,
            kExtmarkUndo,
        );

        if lnum == op.end.lnum {
            // `']` goes to the end of the block, not the end of the insert
            // in the first line.
            Buf::current().b_op_end.lnum = op.end.lnum;
            Buf::current().b_op_end.col = offset;
            if Buf::current().b_visual.vi_end.coladd != 0 {
                Buf::current().b_visual.vi_end.col += Buf::current().b_visual.vi_end.coladd;
                Buf::current().b_visual.vi_end.coladd = 0;
            }
        }
        lnum += 1;
    }

    State.set(old_state);

    // Only if lines past the first were actually modified, which is the
    // loop's own bound.
    if op.start.lnum < op.end.lnum {
        let (first, last) = (op.start.lnum + 1, op.end.lnum + 1);
        changed_lines(Buf::current(), first, 0, last, 0, true);
    }
}

/// Turn 'linebreak' off, answering whether it was on.
///
/// Pass the answer to [`restore_lbr`]. Every column measured for a blockwise
/// operation has to be measured with 'linebreak' off, because it changes what
/// `getvcol` answers.
///
/// Safe: the only thing it touches is the current window, which is what
/// [`Win::current`] already promises.
pub fn reset_lbr() -> bool {
    if Win::current().w_onebuf_opt.wo_lbr == 0 {
        return false;
    }
    Win::current().w_onebuf_opt.wo_lbr = 0;
    // Changing 'linebreak' may require w_virtcol to be recomputed.
    Win::current()
        .w_valid
        .clear(WinValid::WROW | WinValid::WCOL | WinValid::VIRTCOL);
    true
}

/// Put 'linebreak' back if [`reset_lbr`] took it off.
///
/// Safe for the same reason [`reset_lbr`] is.
pub fn restore_lbr(lbr_saved: bool) {
    if Win::current().w_onebuf_opt.wo_lbr != 0 || !lbr_saved {
        return;
    }
    Win::current().w_onebuf_opt.wo_lbr = 1;
    Win::current()
        .w_valid
        .clear(WinValid::WROW | WinValid::WCOL | WinValid::VIRTCOL);
}

/// Where the block sits in one line: fill `bd` for `lnum` of the current
/// buffer.
///
/// `is_del` selects the delete reading of the answer described in the module
/// doc; every other operator passes false.
///
/// Note that this converts a partly selected multi-byte character to spaces,
/// like a partly selected TAB.
pub fn block_prep(bd: &mut BlockDef, op: &OpArg, lnum: LineNr, is_del: bool) {
    let mut lines = Lines::current();
    block_prep_line(bd, op, lnum, lines.line(lnum), is_del);
}

/// [`block_prep`] of `line`, which is line `lnum` of the current buffer, for
/// a caller that holds it already.
pub(crate) fn block_prep_line(
    bd: &mut BlockDef,
    op: &OpArg,
    lnum: LineNr,
    line: &[u8],
    is_del: bool,
) {
    // Unwanted line breaks would move every column measured below.
    let lbr_saved = reset_lbr();

    // Everything but `textcol` (written at the end) and `is_max` (the
    // caller's, meaning the block was opened with `$`).
    bd.startspaces = 0;
    bd.endspaces = 0;
    bd.textlen = 0;
    bd.start_vcol = 0;
    bd.end_vcol = 0;
    bd.is_short = 0;
    bd.is_one_char = 0;
    bd.pre_whitesp = 0;
    bd.pre_whitesp_c = 0;
    bd.end_char_vcols = 0;
    bd.start_char_vcols = 0;

    // The walks end at the line's end, where the decoder answers a zero
    // codepoint; the byte test only runs then.
    let at_end = |at: usize, chr: CharInfo| chr.value == 0 && byte_at(line, at) == 0;

    // Walk to the block's left edge, remembering the run of white space in
    // front of it (`shift_block` widens exactly that run).
    let mut incr = 0;
    let (mut csarg, mut cstype) = CharsizeArg::new(Win::current(), lnum, line);
    let mut prev_pstart = 0;
    let mut at = 0;
    let mut chr = c_char_info(line, 0);
    let mut vcol = bd.start_vcol;
    while vcol < op.start_vcol && !at_end(at, chr) {
        incr = win_charsize(cstype, vcol, at, chr.value, &mut csarg).width;
        vcol += incr;
        if ascii_iswhite(chr.value) {
            bd.pre_whitesp += incr;
            bd.pre_whitesp_c += 1;
        } else {
            bd.pre_whitesp = 0;
            bd.pre_whitesp_c = 0;
        }
        prev_pstart = at;
        (at, chr) = c_next_cluster(line, at, chr);
    }
    bd.start_vcol = vcol;
    let mut pstart = at;
    bd.start_char_vcols = incr;

    if bd.start_vcol < op.start_vcol {
        // The line ends before the block starts.
        bd.end_vcol = bd.start_vcol;
        bd.is_short = 1;
        if !is_del || op.op_type == OpType::Append {
            bd.endspaces = op.end_vcol - op.start_vcol + 1;
        }
    } else {
        bd.startspaces = bd.start_vcol - op.start_vcol;
        if is_del && bd.startspaces != 0 {
            bd.startspaces = bd.start_char_vcols - bd.startspaces;
        }
        let mut pend = pstart;
        bd.end_vcol = bd.start_vcol;

        if bd.end_vcol > op.end_vcol {
            // The whole block is inside one character -- a wide TAB.
            bd.is_one_char = 1;
            if op.op_type == OpType::Insert {
                bd.endspaces = bd.start_char_vcols - bd.startspaces;
            } else if op.op_type == OpType::Append {
                bd.startspaces += op.end_vcol - op.start_vcol + 1;
                bd.endspaces = bd.start_char_vcols - bd.startspaces;
            } else {
                bd.startspaces = op.end_vcol - op.start_vcol + 1;
                if is_del && op.op_type != OpType::Lshift {
                    // Summing the two into `startspaces` does not work for
                    // a Visual replace, so the TAB is split in two.
                    bd.startspaces = bd.start_char_vcols - (bd.start_vcol - op.start_vcol);
                    bd.endspaces = bd.end_vcol - op.end_vcol - 1;
                }
            }
        } else {
            // Walk on to the block's right edge.
            (csarg, cstype) = CharsizeArg::new(Win::current(), lnum, line);
            at = pend;
            chr = c_char_info(line, pend);
            vcol = bd.end_vcol;
            let mut prev_pend = pend;
            while vcol <= op.end_vcol && !at_end(at, chr) {
                prev_pend = at;
                incr = win_charsize(cstype, vcol, at, chr.value, &mut csarg).width;
                vcol += incr;
                (at, chr) = c_next_cluster(line, at, chr);
            }
            bd.end_vcol = vcol;
            pend = at;

            if bd.end_vcol <= op.end_vcol
                && (!is_del || op.op_type == OpType::Append || op.op_type == OpType::Replace)
            {
                // The line ends inside the block. Filling it out to the
                // block's width is the alternative, and it is deliberately
                // not done: it leaves trailing white space behind.
                bd.is_short = 1;
                if op.op_type == OpType::Append || op_virtual() {
                    bd.endspaces = op.end_vcol - bd.end_vcol + c_int::from(op.inclusive);
                }
            } else if bd.end_vcol > op.end_vcol {
                bd.endspaces = bd.end_vcol - op.end_vcol - 1;
                if !is_del && bd.endspaces != 0 {
                    bd.endspaces = incr - bd.endspaces;
                    if pend != pstart {
                        pend = prev_pend;
                    }
                }
            }
        }

        bd.end_char_vcols = incr;
        if is_del && bd.startspaces != 0 {
            pstart = prev_pstart;
        }
        bd.textlen = (pend - pstart) as c_int;
    }

    bd.textcol = pstart as ColNr;
    restore_lbr(lbr_saved);
}

/// Where a *charwise* region sits in one line: fill `bd` for `lnum` of the
/// current buffer.
///
/// The region is `start`..`end` rather than a rectangle, so only the first and
/// last lines are clipped; everything between is the whole line. The
/// 'virtualedit' arms are what make it more than that: an end inside a TAB
/// becomes `endspaces` columns of padding, and a region that starts and ends
/// inside the *same* character is `is_one_char` with no text at all.
///
/// A corner on `lnum` must be a column of that line, its end included.
fn charwise_block_prep(
    mut start: Pos,
    mut end: Pos,
    bd: &mut BlockDef,
    lnum: LineNr,
    inclusive: bool,
) {
    let plen = Lines::current().line_len(lnum);

    bd.startspaces = 0;
    bd.endspaces = 0;
    bd.is_one_char = 0;
    bd.start_char_vcols = 0;

    let mut startcol: ColNr = 0;
    let mut endcol: ColNr = MAXCOL;

    if lnum == start.lnum {
        startcol = start.col;
        if op_virtual() {
            let (cs, _, ce) = getvcol(Win::current(), &mut start);
            if ce != cs && start.coladd > 0 {
                // Part of a TAB is selected -- but do not double-count it.
                bd.start_char_vcols = ce - cs + 1;
                bd.startspaces = (bd.start_char_vcols - start.coladd).max(0);
                startcol += 1;
            }
        }
    }

    if lnum == end.lnum {
        endcol = end.col;
        if op_virtual() {
            let (cs, _, ce) = getvcol(Win::current(), &mut end);
            let mut lines = Lines::current();
            let p = lines.line(lnum);
            let at = usize::try_from(endcol).unwrap_or(0);
            // No padding for a double-width character: `endcol` is then on
            // the last byte of the character, not past it.
            if byte_at(p, at) == 0 || (cs + end.coladd < ce && head_off(p, at) == 0) {
                if start.lnum == end.lnum && start.col == end.col {
                    // The whole region is inside one character.
                    bd.is_one_char = 1;
                    bd.startspaces = end.coladd - start.coladd + c_int::from(inclusive);
                    endcol = startcol;
                } else {
                    bd.endspaces = end.coladd + c_int::from(inclusive);
                    endcol -= c_int::from(inclusive);
                }
            }
        }
    }

    if endcol == MAXCOL {
        endcol = plen;
    }
    bd.textlen = if startcol > endcol || bd.is_one_char != 0 {
        0
    } else {
        endcol - startcol + c_int::from(inclusive)
    };
    bd.textcol = startcol;
}

/// [`block_prep`] of `op` on line `lnum` of the current buffer, for a caller
/// that only reads the answer.
pub(crate) fn block_def(op: &OpArg, lnum: LineNr) -> BlockDef {
    let mut bd = BlockDef::ZERO;
    block_prep(&mut bd, op, lnum, false);
    bd
}

/// [`block_def`] of `line`, which is line `lnum` of the current buffer.
pub(crate) fn block_def_of(op: &OpArg, lnum: LineNr, line: &[u8]) -> BlockDef {
    let mut bd = BlockDef::ZERO;
    block_prep_line(&mut bd, op, lnum, line, false);
    bd
}

/// [`charwise_block_prep`] of line `lnum` of the current buffer, for a
/// caller that only reads the answer. A corner on `lnum` past the end of the
/// line is read as its end.
pub(crate) fn charwise_block_def(start: Pos, end: Pos, lnum: LineNr, inclusive: bool) -> BlockDef {
    let len = ml_get_len(lnum);
    let onto_line = |pos: Pos| Pos {
        col: if pos.lnum == lnum {
            pos.col.clamp(0, len)
        } else {
            pos.col
        },
        ..pos
    };
    let mut bd = BlockDef::ZERO;
    charwise_block_prep(onto_line(start), onto_line(end), &mut bd, lnum, inclusive);
    bd
}

/// Compute `op`'s `start_vcol`/`end_vcol` and square its corners up.
///
/// Only does anything for a CTRL-V selection. Afterwards `op.start` and
/// `op.end` are the block's upper-left and lower-right corners as *character*
/// positions, which is what every blockwise operator then works from.
///
/// `initial` is false when replaying, and asks for the 'selection' adjustment
/// to be skipped. `redo_visual_vcol` is the recorded width a `.` replay uses
/// instead of measuring the selection again.
pub(crate) fn get_op_vcol(op: &mut OpArg, redo_visual_vcol: ColNr, initial: bool) {
    if !visual_mode().is_block() || (!initial && op.end.col < Win::current().w_view_width) {
        return;
    }

    op.motion_type = kMTBlockWise;
    // Do not let the end land on a trail byte.
    op.end = snap_pos_to_char(Win::current().buffer(), op.end);

    let (start, _, end) = getvvcol(Win::current(), &mut op.start);
    (op.start_vcol, op.end_vcol) = (start, end);
    if !redo_VIsual_busy.get() {
        let (start, _, end) = getvvcol(Win::current(), &mut op.end);
        op.start_vcol = op.start_vcol.min(start);
        if end > op.end_vcol {
            if initial && sel_exclusive() && start >= 1 && start > op.end_vcol {
                op.end_vcol = start - 1;
            } else {
                op.end_vcol = end;
            }
        }
    }

    if Win::current().w_curswant == MAXCOL {
        // `$` was used: the block's right edge is the longest line's.
        Win::current().w_cursor.col = MAXCOL;
        op.end_vcol = 0;
        Win::current().w_cursor.lnum = op.start.lnum;
        while Win::current().w_cursor.lnum <= op.end.lnum {
            let (_, end) = Win::current().virtual_vcol_span(Win::current().cursor());
            op.end_vcol = op.end_vcol.max(end);
            Win::current().w_cursor.lnum += 1;
        }
    } else if redo_VIsual_busy.get() {
        op.end_vcol = op.start_vcol + redo_visual_vcol - 1;
    }

    // Turn the column pair back into the block's two corner *positions*.
    Win::current().w_cursor.lnum = op.end.lnum;
    Win::current().coladvance(op.end_vcol);
    op.end = Win::current().w_cursor;

    Win::current().w_cursor = op.start;
    Win::current().coladvance(op.start_vcol);
    op.start = Win::current().w_cursor;
}
