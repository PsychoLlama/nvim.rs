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

#![deny(unsafe_op_in_unsafe_fn)]

use crate::winlayer::{Buf, Win};
use core::ffi::{c_char, c_int, c_ulong, c_void};

use super::*;
use crate::edit::BeginlineOpts;
use crate::ex_docmd::cmdmod_has;
use crate::types::{IOSIZE, NUL};

/// `<` and `>` over the operator's region.
///
/// `curs_top` leaves the cursor on the first line (`>>`) rather than the last
/// (`:>`); a blockwise shift ignores it and goes back to where it started.
///
/// # Safety
/// `oap` must point to a live `oparg_T` describing a region of the current
/// buffer.
pub unsafe fn op_shift(oap: *mut oparg_T, curs_top: bool, amount: c_int) {
    // SAFETY: the caller's promise -- a live `oparg_T` of the current buffer.
    // The loop walks the region, so every line it reaches is one of it, and
    // the cursor is on that line throughout.
    let oap = unsafe { Op::new(oap) };
    // The "N lines >ed M times" report; upstream shares `IObuff` for it.
    let mut report = [0 as c_char; IOSIZE as usize];
    let (above, below) = (oap.start.lnum - 1, oap.end.lnum + 1);
    if u_save(above, below).is_err() {
        return;
    }

    let mut block_col: colnr_T = 0;
    if oap.motion_type == kMTBlockWise {
        block_col = cur_win().w_cursor.col;
    }

    for _ in 0..oap.line_count {
        let first_char = unsafe { *get_cursor_line_ptr() } as u8 as c_int;
        if first_char == NUL {
            // Empty line: nothing to indent, but the cursor still has to
            // land somewhere legal.
            cur_win().w_cursor.col = 0;
        } else if oap.motion_type == kMTBlockWise {
            shift_block(oap, amount);
        } else if first_char != '#' as c_int || !unsafe { preprocs_left() } {
            // A line starting with '#' stays put when 'smartindent' or
            // 'cindent' says preprocessor lines keep column 0.
            let left = oap.op_type == OpType::Lshift;
            unsafe { shift_line(left, p_sr.get() != 0, amount, false) };
        }
        cur_win().w_cursor.lnum += 1;
    }

    if oap.motion_type == kMTBlockWise {
        cur_win().w_cursor.lnum = oap.start.lnum;
        cur_win().w_cursor.col = block_col;
    } else if curs_top {
        cur_win().w_cursor.lnum = oap.start.lnum;
        // `shift_line` may have moved the column.
        beginline(BeginlineOpts::SOL | BeginlineOpts::FIX);
    } else {
        cur_win().w_cursor.lnum -= 1;
    }
    // The cursor line must not be in a closed fold.
    unsafe { fold_open_cursor() };

    if oap.line_count as OptInt > p_report.get() {
        // Two plural forms, nested: "line"/"lines" on the line count and
        // "time"/"times" on the shift count, which is why the outer
        // `ngettext` chooses between two already-translated formats.
        let op = if oap.op_type == OpType::Rshift {
            c">".as_ptr()
        } else {
            c"<".as_ptr()
        };
        let lines = oap.line_count as c_ulong;
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
        let out = report.as_mut_ptr();
        unsafe {
            vim_snprintf(
                out,
                IOSIZE as size_t,
                fmt.as_ptr(),
                oap.line_count as int64_t,
                op,
                amount,
            )
        };
        unsafe { msg_keep(out, 0, true, false) };
    }

    if !cmdmod_has(CmdModFlags::LOCKMARKS) {
        cur_buf().b_op_start = oap.start;
        cur_buf().b_op_end.lnum = oap.end.lnum;
        cur_buf().b_op_end.col = ml_get_len(oap.end.lnum);
        if cur_buf().b_op_end.col > 0 {
            cur_buf().b_op_end.col -= 1;
        }
    }

    let (first, last) = (oap.start.lnum, oap.end.lnum + 1);
    changed_lines(cur_buf(), first, 0, last, 0, true);
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
///
/// # Safety
/// Operates on the cursor line of the current buffer.
pub unsafe fn shift_line(left: bool, round: bool, amount: c_int, call_changed_bytes: bool) {
    let sw_val = cur_buf().b_p_sw;
    let ts_val = cur_buf().b_p_ts;
    let vts_array = cur_buf().b_p_vts_array as *const c_int;
    // `vts_array[0]` is the count, so the whole array is one longer.
    // SAFETY: 'vartabstop' is stored as its own length followed by that many
    // stops, and the cursor is on a line of the current buffer.
    let stops = if vts_array.is_null() {
        0
    } else {
        unsafe { *vts_array }
    };
    let vts = (stops != 0)
        .then(|| unsafe { ::core::slice::from_raw_parts(vts_array, stops as usize + 1) });
    let now = int64_t::from(get_indent());

    let amount = int64_t::from(amount);
    let count = match vts {
        _ if sw_val != 0 => get_new_sw_indent(left, round, amount, sw_val, now),
        None => get_new_sw_indent(left, round, amount, ts_val, now),
        Some(vts) => get_new_vts_indent(left, round, amount as c_int, vts, now),
    };

    let indent = crate::math::trim_to_int(count);
    if State.get() & VREPLACE_FLAG != 0 {
        unsafe { change_indent(INDENT_SET as c_int, indent, 0, call_changed_bytes) };
    } else {
        let flags = if call_changed_bytes {
            SIN_CHANGED as c_int
        } else {
            0
        };
        unsafe { set_indent(indent, flags) };
    }
}

/// The replacement line one direction of [`shift_block`] built, and the byte
/// span of the cursor line it stands in for.
///
/// `old_len`/`new_len` are what the extmark splice at `start_col` is told, and
/// they are *not* the whole line: everything either side of the shift is
/// copied verbatim.
struct ShiftedLine {
    /// Allocated replacement for the whole line; `ml_replace` takes it.
    line: *mut c_char,
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
/// `oap` must be blockwise, and the cursor must name the line.
fn shift_block(oap: Op, amount: c_int) {
    // SAFETY: the cursor line is a line of the current buffer, and `bd`
    // describes it once `block_prep` has run.
    let left = oap.op_type == OpType::Lshift;
    let old_state = State.get();
    let old_col = cur_win().w_cursor.col;
    let sw_val = unsafe { get_sw_value_indent(curbuf.get(), left) };
    let old_p_ri = p_ri.get();

    // No 'revins' and no MODE_REPLACE while we rebuild the indent.
    p_ri.set(0);
    State.set(MODE_INSERT);

    let mut bd = block_def::ZERO;
    let lnum = cur_win().w_cursor.lnum;
    unsafe { block_prep(oap.raw(), &raw mut bd, lnum, true) };
    if bd.is_short != 0 {
        return;
    }

    // Screen columns to insert or remove. `sw_val` is at least 1, so the
    // division is the multiplication's overflow check and nothing else.
    let total = (amount as u32).wrapping_mul(sw_val as u32) as c_int;
    if total / sw_val != amount {
        return;
    }

    let shifted = if left {
        shift_block_left(oap, &mut bd, total)
    } else {
        shift_block_right(&mut bd, total)
    };

    let _ = unsafe { ml_replace(lnum, shifted.line, false) };
    unsafe { changed_bytes(lnum, bd.textcol) };
    let (at, old, new) = (shifted.start_col, shifted.old_len, shifted.new_len);
    unsafe { extmark_splice_cols(curbuf.get(), lnum as c_int - 1, at, old, new, kExtmarkUndo) };

    State.set(old_state);
    cur_win().w_cursor.col = old_col;
    p_ri.set(old_p_ri);
}

/// `>` on one line of a blockwise region: widen the white space in front of
/// the block by `total` screen columns.
///
/// The work is: measure every white-space column already there (including the
/// part of a TAB the block splits), add `total`, and re-lay the whole run as
/// TABs and spaces under 'expandtab'/'vartabstop'. `bd` is advanced to the
/// first non-white character as a side effect, and its `textcol` back to where
/// the run starts, because the caller reports the change from there.
///
/// `bd` must describe the cursor line, as [`block_prep`] left it.
fn shift_block_right(bd: &mut block_def, mut total: c_int) -> ShiftedLine {
    // SAFETY: `bd` describes the cursor line, so `bd.textstart` is inside it
    // and every walk below stops at a non-white character or its NUL.
    let old_p = get_cursor_line_ptr();
    let old_line_len = get_cursor_line_len();
    let ts_val = cur_buf().b_p_ts as c_int;

    // All the virtual white space up to and including a split TAB.
    total += bd.pre_whitesp;
    let mut ws_vcol = bd.start_vcol - bd.pre_whitesp;
    let old_textstart = bd.textstart;
    if bd.startspaces != 0 {
        if unsafe { utfc_ptr2len(bd.textstart) } == 1 {
            bd.textstart = unsafe { bd.textstart.offset(1) };
        } else {
            // A multi-byte character straddles the edge: it cannot be
            // rebuilt as white space, so start the run at the block.
            ws_vcol = 0;
            bd.startspaces = 0;
        }
    }

    // Add the width of the white space that follows the block's edge.
    let mut csarg = CharsizeArg::default();
    let lnum = cur_win().w_cursor.lnum;
    let cstype =
        unsafe { init_charsize_arg(&mut csarg, Win::new(curwin.get()), lnum, bd.textstart) };
    let mut ci: StrCharInfo = unsafe { utf_ptr2str_char_info(bd.textstart) };
    let mut vcol = bd.start_vcol as c_int;
    while ascii_iswhite(ci.chr.value) {
        let incr = unsafe { win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &mut csarg) }.width;
        ci = unsafe { utfc_next(ci) };
        total += incr;
        vcol += incr;
    }
    bd.textstart = ci.ptr;
    bd.start_vcol = vcol;

    // `total` is now all the white space wanted and `bd.textstart` points
    // at the first non-white character in the block.
    let mut tabs = 0;
    let mut spaces = 0;
    if cur_buf().b_p_et == 0 {
        let (vts, tp, sp) = (cur_buf().b_p_vts_array, &raw mut tabs, &raw mut spaces);
        unsafe { tabstop_fromto(ws_vcol, ws_vcol + total, ts_val, vts, tp, sp) };
    } else {
        spaces = total;
    }

    // Allow for a split TAB in front of the block.
    let col_pre = bd.pre_whitesp_c - c_int::from(bd.startspaces != 0);
    bd.textcol -= col_pre;

    // SAFETY: `newp` is sized for the kept prefix, the new white space and
    // the rest of the line, which is exactly what is written into it.
    let (newp, new_line_len, old_len) = unsafe {
        let kept = old_line_len - bd.textstart.offset_from(old_p) as c_int;
        let new_line_len = bd.textcol + tabs + spaces + kept;
        let newp = xmalloc(new_line_len as size_t + 1) as *mut c_char;
        newp.cast::<u8>()
            .copy_from(old_p.cast(), bd.textcol as size_t);
        let at_tabs = newp.offset(bd.textcol as isize) as *mut c_void;
        at_tabs
            .cast::<u8>()
            .write_bytes((TAB) as u8, tabs as size_t);
        let at_spaces = newp.offset((bd.textcol + tabs) as isize) as *mut c_void;
        at_spaces.cast::<u8>().write_bytes(b' ', spaces as size_t);
        strcpy(
            newp.offset((bd.textcol + tabs + spaces) as isize),
            bd.textstart,
        );
        let old_len = bd.textstart.offset_from(old_textstart) as c_int + col_pre;
        (newp, new_line_len, old_len)
    };

    let shifted = ShiftedLine {
        line: newp,
        start_col: bd.textcol,
        old_len,
        new_len: tabs + spaces,
    };
    debug_assert!(shifted.new_len - shifted.old_len == new_line_len - old_line_len);
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
/// `bd` must describe the cursor line, as [`block_prep`] left it.
fn shift_block_left(oap: Op, bd: &mut block_def, total: c_int) -> ShiftedLine {
    // SAFETY: `bd` describes the cursor line, so `bd.textstart` is inside it
    // and both walks below stop at a character the line really holds.
    let old_p = get_cursor_line_ptr();
    let old_line_len = get_cursor_line_len();

    // Find the first non-white character displayed after the block's start
    // column, and the width of the white space in front of it. When
    // `startspaces` is set, `textstart` is the character the block's edge
    // splits, so the search starts after it.
    let mut non_white = bd.textstart;
    if bd.startspaces != 0 {
        non_white = unsafe { non_white.offset(utfc_ptr2len(non_white) as isize) };
    }
    let mut non_white_col = bd.start_vcol;
    let mut csarg = CharsizeArg::default();
    let lnum = cur_win().w_cursor.lnum;
    let mut cstype =
        unsafe { init_charsize_arg(&mut csarg, Win::new(curwin.get()), lnum, bd.textstart) };
    while ascii_iswhite(unsafe { *non_white } as c_int) {
        let c = unsafe { *non_white } as u8 as int32_t;
        let at = non_white;
        non_white_col += unsafe { win_charsize(cstype, non_white_col, at, c, &mut csarg) }.width;
        non_white = unsafe { non_white.offset(1) };
    }

    // Shift by `total`, or by all the white space there is if that is less.
    let block_space_width = non_white_col - oap.start_vcol;
    let destination_col = non_white_col - block_space_width.min(total);

    // How much of the line can be reused unchanged. When `startspaces` is
    // set, `textstart` is the character *preceding* the block, so its width
    // comes off to get its column.
    let mut verbatim_copy_width = bd.start_vcol;
    if bd.startspaces != 0 {
        verbatim_copy_width -= bd.start_char_vcols;
    }
    cstype = unsafe { init_charsize_arg(&mut csarg, Win::new(curwin.get()), 0, bd.textstart) };
    let mut ci: StrCharInfo = unsafe { utf_ptr2str_char_info(bd.textstart) };
    while verbatim_copy_width < destination_col {
        let w = verbatim_copy_width;
        let incr = unsafe { win_charsize(cstype, w, ci.ptr, ci.chr.value, &mut csarg) }.width;
        if verbatim_copy_width + incr > destination_col {
            break;
        }
        verbatim_copy_width += incr;
        ci = unsafe { utfc_next(ci) };
    }
    let verbatim_copy_end = ci.ptr;

    // A destination inside a TAB leaves a gap the TAB used to cover.
    debug_assert!(destination_col - verbatim_copy_width >= 0);
    let fill = destination_col - verbatim_copy_width;

    // SAFETY: `newp` is sized for the kept prefix, the `fill` spaces and the
    // rest of the line, which is exactly what is written into it.
    let (newp, new_line_len, fixedlen, old_len) = unsafe {
        debug_assert!(verbatim_copy_end.offset_from(old_p) >= 0);
        // The part of the line left of the shift, which is not being shifted.
        let fixedlen = verbatim_copy_end.offset_from(old_p) as c_int;
        // The replacement line is that part, then `fill` spaces, then the
        // rest of the line from `non_white`.
        let kept = old_line_len - non_white.offset_from(old_p) as c_int;
        let new_line_len = fixedlen + fill + kept;
        let newp = xmalloc(new_line_len as size_t + 1) as *mut c_char;
        newp.cast::<u8>()
            .copy_from(old_p.cast(), fixedlen as size_t);
        let at = newp.offset(fixedlen as isize) as *mut c_void;
        at.cast::<u8>().write_bytes(b' ', fill as size_t);
        strcpy(newp.offset((fixedlen + fill) as isize), non_white);
        let moved = non_white.offset_from(bd.textstart) as c_int;
        (newp, new_line_len, fixedlen, bd.textcol + moved - fixedlen)
    };

    let shifted = ShiftedLine {
        line: newp,
        start_col: fixedlen,
        old_len,
        new_len: fill,
    };
    debug_assert!(shifted.new_len - shifted.old_len == new_line_len - old_line_len);
    shifted
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
