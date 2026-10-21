//! `<` and `>` -- moving an indent left or right.
//!
//! Three shapes of one operation:
//!
//! * [`shift_line`] re-indents *one* line by a multiple of the shift width.
//!   Which width that is takes three options to answer -- 'shiftwidth' if it
//!   is non-zero, else 'vartabstop' if it is set, else 'tabstop' -- and with
//!   'vartabstop' the answer depends on *where* the indent already is, which
//!   is what [`get_vts`] and [`get_vts_sum`] are for.
//! * [`op_shift`] runs that over a linewise region, one line at a time.
//! * [`shift_block`] is the blockwise case, and it is not a re-indent at all:
//!   it inserts or removes white space at the block's left edge, in the middle
//!   of the line. Its two directions have almost nothing in common, so they
//!   are two functions ([`shift_block_right`] and [`shift_block_left`]); each
//!   builds a whole replacement line and reports the byte span it changed, so
//!   that the driver can do the one `ml_replace` and the one extmark splice.

#![forbid(unsafe_code)]

use crate::memory::XString;
use crate::winlayer::{Buf, Win};
use core::ffi::{c_int, c_ulong};

use super::*;
use crate::edit::BeginlineOpts;
use crate::ex_docmd::cmdmod_has;
use crate::mbyte::{c_char_info, c_cluster_len, c_next_cluster};
use crate::types::{IOSIZE, NUL};

/// `<` and `>` over the operator's region.
///
/// `curs_top` leaves the cursor on the first line (`>>`) rather than the last
/// (`:>`); a blockwise shift ignores it and goes back to where it started.
/// `op` describes a region of the current buffer; the loop walks it, so every
/// line it reaches is one of it, and the cursor is on that line throughout.
pub fn op_shift(op: &OpArg, curs_top: bool, amount: c_int) {
    let (above, below) = (op.start.lnum - 1, op.end.lnum + 1);
    if u_save(above, below).is_err() {
        return;
    }

    let mut block_col: ColNr = 0;
    if op.motion_type == kMTBlockWise {
        block_col = Win::current().w_cursor.col;
    }

    for _ in 0..op.line_count {
        let lnum = Win::current().w_cursor.lnum;
        let first_char = c_int::from(byte_at(Lines::current().line(lnum), 0));
        if first_char == NUL {
            // Empty line: nothing to indent, but the cursor still has to
            // land somewhere legal.
            Win::current().w_cursor.col = 0;
        } else if op.motion_type == kMTBlockWise {
            shift_block(op, amount);
        } else if first_char != '#' as c_int || !preprocs_left() {
            // A line starting with '#' stays put when 'smartindent' or
            // 'cindent' says preprocessor lines keep column 0.
            let left = op.op_type == OpType::Lshift;
            shift_line(left, p_sr(), amount, false);
        }
        Win::current().w_cursor.lnum += 1;
    }

    if op.motion_type == kMTBlockWise {
        Win::current().w_cursor.lnum = op.start.lnum;
        Win::current().w_cursor.col = block_col;
    } else if curs_top {
        Win::current().w_cursor.lnum = op.start.lnum;
        // `shift_line` may have moved the column.
        beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
    } else {
        Win::current().w_cursor.lnum -= 1;
    }
    // The cursor line must not be in a closed fold.
    fold_open_cursor();

    if op.line_count as OptInt > p_report() {
        // Two plural forms, nested: "line"/"lines" on the line count and
        // "time"/"times" on the shift count, which is why the outer
        // `ngettext` chooses between two already-translated formats.
        let op_char = if op.op_type == OpType::Rshift {
            c">"
        } else {
            c"<"
        };
        let lines = op.line_count as c_ulong;
        let single = ngettext(
            c"%ld line %sed %d time",
            c"%ld line %sed %d times",
            amount as c_ulong,
        );
        let plural = ngettext(
            c"%ld lines %sed %d time",
            c"%ld lines %sed %d times",
            amount as c_ulong,
        );
        let fmt = ngettext(single, plural, lines);
        // The "N lines >ed M times" report, cut where upstream's shared
        // `IObuff` would cut it.
        let report = crate::formatted!(
            IOSIZE as usize,
            fmt.to_bytes(),
            op.line_count as int64_t,
            op_char,
            amount,
        );
        msg_keep_text(XString::from_bytes(&report).as_cstr(), 0);
    }

    if !cmdmod_has(CmdModFlags::LOCKMARKS) {
        Buf::current().b_op_start = op.start;
        Buf::current().b_op_end.lnum = op.end.lnum;
        Buf::current().b_op_end.col = Lines::current().line_len(op.end.lnum);
        if Buf::current().b_op_end.col > 0 {
            Buf::current().b_op_end.col -= 1;
        }
    }

    let (first, last) = (op.start.lnum, op.end.lnum + 1);
    changed_lines(Buf::current(), first, 0, last, 0, true);
}

/// Width of the `index`-th 'vartabstop' stop, the last one repeating forever.
///
/// `vts[0]` is the count and the stops themselves start at 1, so index 0 is
/// "before the first stop" and answers 0.
fn get_vts(vts: &[c_int], index: c_int) -> c_int {
    let count = vts[0];
    if index < 1 {
        0
    } else if index <= count {
        vts[index as usize]
    } else {
        vts[count as usize]
    }
}

/// Column of the `index`-th 'vartabstop' stop: the sum of every stop up to and
/// including it, with the last stop repeating past the end of the array.
fn get_vts_sum(vts: &[c_int], index: c_int) -> c_int {
    let count = vts[0];
    let mut sum = 0;
    let mut i = 1;
    while i <= index && i <= count {
        sum += vts[i as usize];
        i += 1;
    }
    // Stops beyond the array all have the last stop's width.
    if i <= index {
        sum += vts[count as usize] * (index - count);
    }
    sum
}

/// The new indent for a shift of `amount` steps of `sw_val` columns.
///
/// `round` is 'shiftround': land on a multiple of the step rather than moving
/// by whole steps from wherever the line already is.
///
/// `count` is the line's current indent in columns.
fn get_new_sw_indent(
    left: bool,
    round: bool,
    amount: int64_t,
    sw_val: int64_t,
    count: int64_t,
) -> int64_t {
    if !round {
        // Original vi behaviour: move by whole steps from here.
        return if left {
            (count - sw_val * amount).max(0)
        } else {
            count + sw_val * amount
        };
    }

    let mut steps = int64_t::from(crate::math::trim_to_int(count / sw_val));
    let extra = crate::math::trim_to_int(count % sw_val);
    // Shifting left off a partial step spends the shift on the remainder.
    let amount = if extra != 0 && left {
        amount - 1
    } else {
        amount
    };
    if left {
        steps = (steps - amount).max(0);
    } else {
        steps += amount;
    }
    steps * sw_val
}

/// The new indent for a shift of `amount` 'vartabstop' stops.
///
/// `indent` is the line's current indent in columns.
fn get_new_vts_indent(
    left: bool,
    round: bool,
    amount: c_int,
    vts: &[c_int],
    indent: int64_t,
) -> int64_t {
    // Walk out to the first stop past the current indent, then step back
    // one: `vtsi` is the stop at or to the left of it.
    let mut vtsi = 0;
    let mut vts_indent = 0;
    let mut ts = 0;
    while int64_t::from(vts_indent) <= indent {
        vtsi += 1;
        ts = get_vts(vts, vtsi);
        vts_indent += ts;
    }
    vts_indent -= ts;
    vtsi -= 1;

    // Extra indent to the right of that stop.
    let offset = indent - int64_t::from(vts_indent);

    let stop = |index: c_int| int64_t::from(get_vts_sum(vts, index));
    if round {
        // 'shiftround': land on a stop, keeping no remainder. Shifting
        // left off a partial stop spends the shift on the remainder.
        if !left {
            stop(vtsi + amount)
        } else if offset == 0 {
            stop(vtsi - amount)
        } else {
            stop(vtsi - (amount - 1))
        }
    } else if !left {
        stop(vtsi + amount) + offset
    } else if amount > vtsi {
        0
    } else {
        stop(vtsi - amount) + offset
    }
}

/// Shift the cursor line `amount` steps left or right.
///
/// The step is 'shiftwidth' if it is non-zero, else 'vartabstop' if it is set,
/// else 'tabstop'. ('softtabstop' and 'varsofttabstop' deliberately play no
/// part: the documentation says nothing about them here and Vim does not
/// consult them either.)
///
/// `round` is 'shiftround'. `call_changed_bytes` is false only for the callers
/// that report the change themselves.
pub fn shift_line(left: bool, round: bool, amount: c_int, call_changed_bytes: bool) {
    let sw_val = Buf::current().b_p_sw;
    let ts_val = Buf::current().b_p_ts;
    let now = int64_t::from(get_indent());

    let amount = int64_t::from(amount);
    // 'vartabstop' is only asked for when 'shiftwidth' is zero, and then
    // copied: the list is the buffer's.
    let count = if sw_val != 0 {
        get_new_sw_indent(left, round, amount, sw_val, now)
    } else {
        match Buf::current().tab_stop_list() {
            None => get_new_sw_indent(left, round, amount, ts_val, now),
            Some(vts) => get_new_vts_indent(left, round, amount as c_int, &vts, now),
        }
    };

    let indent = crate::math::trim_to_int(count);
    if State.get() & VREPLACE_FLAG != 0 {
        change_indent(INDENT_SET as c_int, indent, 0, call_changed_bytes);
    } else {
        let flags = if call_changed_bytes {
            SIN_CHANGED as c_int
        } else {
            0
        };
        set_indent(indent, flags);
    }
}

/// The replacement line one direction of [`shift_block`] built, and the byte
/// span of the cursor line it stands in for.
///
/// `old_len`/`new_len` are what the extmark splice at `start_col` is told, and
/// they are *not* the whole line: everything either side of the shift is
/// copied verbatim.
struct ShiftedLine {
    /// Replacement for the whole line; `ml_replace` takes it.
    line: Vec<u8>,
    /// Byte offset in the old line at which the change starts.
    start_col: c_int,
    /// Bytes replaced there.
    old_len: c_int,
    /// Bytes written in their place.
    new_len: c_int,
}

/// Shift one line of a blockwise region, leaving the cursor on the block's
/// first character.
///
/// `op` must be blockwise, and the cursor must name the line.
fn shift_block(op: &OpArg, amount: c_int) {
    let left = op.op_type == OpType::Lshift;
    let old_state = State.get();
    let old_col = Win::current().w_cursor.col;
    let sw_val = get_sw_value_indent(Buf::current(), left);
    let old_p_ri = p_ri();

    // No 'revins' and no MODE_REPLACE while we rebuild the indent.
    P_RI.set(false);
    State.set(MODE_INSERT);

    let mut bd = BlockDef::ZERO;
    let lnum = Win::current().w_cursor.lnum;
    block_prep(&mut bd, op, lnum, true);
    if bd.is_short != 0 {
        return;
    }

    // Screen columns to insert or remove. `sw_val` is at least 1, so the
    // division is the multiplication's overflow check and nothing else.
    let total = (amount as u32).wrapping_mul(sw_val as u32) as c_int;
    if total / sw_val != amount {
        return;
    }

    let shifted = {
        let mut lines = Lines::current();
        let line = lines.line(lnum);
        if left {
            shift_block_left(op, &mut bd, line, total)
        } else {
            shift_block_right(&mut bd, line, total)
        }
    };

    let _ = ml_replace_owned(lnum, XString::from_vec(shifted.line));
    changed_bytes(lnum, bd.textcol);
    let (at, old, new) = (shifted.start_col, shifted.old_len, shifted.new_len);
    let (buffer, row) = (Buf::current(), lnum as c_int - 1);
    extmark_splice_cols(buffer, row, at, old, new, kExtmarkUndo);

    State.set(old_state);
    Win::current().w_cursor.col = old_col;
    P_RI.set(old_p_ri);
}

/// `>` on one line of a blockwise region: widen the white space in front of
/// the block by `total` screen columns.
///
/// The work is: measure every white-space column already there (including the
/// part of a TAB the block splits), add `total`, and re-lay the whole run as
/// TABs and spaces under 'expandtab'/'vartabstop'. `bd.textcol` is moved back
/// to where the run starts, because the caller reports the change from there.
///
/// `old` is the cursor line and `bd` describes it, as [`block_prep`] left it.
fn shift_block_right(bd: &mut BlockDef, old: &[u8], mut total: c_int) -> ShiftedLine {
    // The block's first byte, which every walk below starts from.
    let block_at = (bd.textcol as usize).min(old.len());

    // All the virtual white space up to and including a split TAB.
    total += bd.pre_whitesp;
    let mut ws_vcol = bd.start_vcol - bd.pre_whitesp;
    let mut text_at = block_at;
    if bd.startspaces != 0 {
        if c_cluster_len(&old[text_at..]) == 1 {
            text_at += 1;
        } else {
            // A multi-byte character straddles the edge: it cannot be
            // rebuilt as white space, so start the run at the block.
            ws_vcol = 0;
            bd.startspaces = 0;
        }
    }

    // Add the width of the white space that follows the block's edge.
    let lnum = Win::current().w_cursor.lnum;
    let base = &old[text_at..];
    let (mut csarg, cstype) = CharsizeArg::new(Win::current(), lnum, base);
    let mut at = 0;
    let mut chr = c_char_info(base, 0);
    let mut vcol = bd.start_vcol as c_int;
    while ascii_iswhite(chr.value) {
        let incr = win_charsize(cstype, vcol, at, chr.value, &mut csarg).width;
        (at, chr) = c_next_cluster(base, at, chr);
        total += incr;
        vcol += incr;
    }
    // `total` is now all the white space wanted, and the text resumes at the
    // first non-white character in the block.
    let rest = text_at + at;
    bd.start_vcol = vcol;

    let (tabs, spaces) = if Buf::current().b_p_et == 0 {
        Buf::current().tab_fill(ws_vcol, ws_vcol + total)
    } else {
        (0, total)
    };

    // Allow for a split TAB in front of the block.
    let col_pre = bd.pre_whitesp_c - c_int::from(bd.startspaces != 0);
    bd.textcol -= col_pre;

    let keep = (bd.textcol as usize).min(old.len());
    let mut line = Vec::with_capacity(keep + (tabs + spaces) as usize + old.len() - rest + 1);
    line.extend_from_slice(&old[..keep]);
    line.resize(line.len() + tabs as usize, TAB as u8);
    line.resize(line.len() + spaces as usize, b' ');
    line.extend_from_slice(&old[rest..]);

    let shifted = ShiftedLine {
        start_col: bd.textcol,
        old_len: (rest - block_at) as c_int + col_pre,
        new_len: tabs + spaces,
        line,
    };
    debug_assert!(
        shifted.new_len - shifted.old_len == shifted.line.len() as c_int - old.len() as c_int
    );
    shifted
}

/// `<` on one line of a blockwise region: narrow the white space in front of
/// the block by up to `total` screen columns.
///
/// Unlike the `>` direction this does not re-lay the indent. It finds the
/// first non-white character displayed at or after the block's edge, works out
/// which column it should move back to, and keeps everything up to the last
/// character that still fits verbatim -- so the only thing rewritten is the
/// gap, and a TAB the destination lands inside becomes `fill` spaces.
///
/// `old` is the cursor line and `bd` describes it, as [`block_prep`] left it.
fn shift_block_left(op: &OpArg, bd: &mut BlockDef, old: &[u8], total: c_int) -> ShiftedLine {
    // The block's first byte, which every walk below starts from.
    let block_at = (bd.textcol as usize).min(old.len());
    let base = &old[block_at..];

    // Find the first non-white character displayed after the block's start
    // column, and the width of the white space in front of it. When
    // `startspaces` is set, the block's first character is the one its edge
    // splits, so the search starts after it.
    let mut non_white = 0;
    if bd.startspaces != 0 {
        non_white = c_cluster_len(base);
    }
    let mut non_white_col = bd.start_vcol;
    let lnum = Win::current().w_cursor.lnum;
    let (mut csarg, mut cstype) = CharsizeArg::new(Win::current(), lnum, base);
    while ascii_iswhite(c_int::from(byte_at(base, non_white))) {
        let c = int32_t::from(base[non_white]);
        non_white_col += win_charsize(cstype, non_white_col, non_white, c, &mut csarg).width;
        non_white += 1;
    }

    // Shift by `total`, or by all the white space there is if that is less.
    let block_space_width = non_white_col - op.start_vcol;
    let destination_col = non_white_col - block_space_width.min(total);

    // How much of the line can be reused unchanged. When `startspaces` is
    // set, the block's first character *precedes* the block, so its width
    // comes off to get its column.
    let mut verbatim_copy_width = bd.start_vcol;
    if bd.startspaces != 0 {
        verbatim_copy_width -= bd.start_char_vcols;
    }
    (csarg, cstype) = CharsizeArg::new(Win::current(), 0, base);
    let mut at = 0;
    let mut chr = c_char_info(base, 0);
    while verbatim_copy_width < destination_col {
        let w = verbatim_copy_width;
        let incr = win_charsize(cstype, w, at, chr.value, &mut csarg).width;
        if verbatim_copy_width + incr > destination_col {
            break;
        }
        verbatim_copy_width += incr;
        (at, chr) = c_next_cluster(base, at, chr);
    }

    // A destination inside a TAB leaves a gap the TAB used to cover.
    debug_assert!(destination_col - verbatim_copy_width >= 0);
    let fill = destination_col - verbatim_copy_width;

    // The part of the line left of the shift, which is not being shifted,
    // then `fill` spaces, then the rest of the line from `non_white`.
    let fixedlen = (block_at + at).min(old.len());
    let rest = (block_at + non_white).min(old.len());
    let mut line = Vec::with_capacity(fixedlen + fill as usize + old.len() - rest + 1);
    line.extend_from_slice(&old[..fixedlen]);
    line.resize(line.len() + fill as usize, b' ');
    line.extend_from_slice(&old[rest..]);

    let shifted = ShiftedLine {
        start_col: fixedlen as c_int,
        old_len: bd.textcol + non_white as c_int - fixedlen as c_int,
        new_len: fill,
        line,
    };
    debug_assert!(
        shifted.new_len - shifted.old_len == shifted.line.len() as c_int - old.len() as c_int
    );
    shifted
}

/// Run the linewise operator an Ex command spells -- `:delete`, `:yank`,
/// `:<`, `:>` -- over `op`, whose lines are an Ex range the command line
/// has already checked against the current buffer.
pub(crate) fn run_ex_operator(op: &mut OpArg, amount: c_int) {
    match op.op_type {
        OpType::Delete => {
            // `:delete` reports its own refusals.
            let _ = op_delete(op);
        }
        OpType::Yank => {
            op_yank(op, true);
        }
        _ => op_shift(op, false, amount),
    }
}
