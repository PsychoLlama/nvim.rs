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

use core::ffi::{c_char, c_int, c_ulong, c_void};

use super::*;
use crate::types::{FAIL, IOSIZE, NUL};

/// `<` and `>` over the operator's region.
///
/// `curs_top` leaves the cursor on the first line (`>>`) rather than the last
/// (`:>`); a blockwise shift ignores it and goes back to where it started.
///
/// # Safety
/// `oap` must point to a live `oparg_T` describing a region of the current
/// buffer.
pub unsafe fn op_shift(oap: *mut oparg_T, curs_top: bool, amount: c_int) {
    unsafe {
        if u_save((*oap).start.lnum - 1, (*oap).end.lnum + 1) == FAIL {
            return;
        }

        let mut block_col: colnr_T = 0;
        if (*oap).motion_type == kMTBlockWise {
            block_col = (*curwin.get()).w_cursor.col;
        }

        for _ in 0..(*oap).line_count {
            let first_char = *get_cursor_line_ptr() as u8 as c_int;
            if first_char == NUL {
                // Empty line: nothing to indent, but the cursor still has to
                // land somewhere legal.
                (*curwin.get()).w_cursor.col = 0;
            } else if (*oap).motion_type == kMTBlockWise {
                shift_block(oap, amount);
            } else if first_char != '#' as c_int || !preprocs_left() {
                // A line starting with '#' stays put when 'smartindent' or
                // 'cindent' says preprocessor lines keep column 0.
                shift_line((*oap).op_type == OP_LSHIFT, p_sr.get() != 0, amount, false);
            }
            (*curwin.get()).w_cursor.lnum += 1;
        }

        if (*oap).motion_type == kMTBlockWise {
            (*curwin.get()).w_cursor.lnum = (*oap).start.lnum;
            (*curwin.get()).w_cursor.col = block_col;
        } else if curs_top {
            (*curwin.get()).w_cursor.lnum = (*oap).start.lnum;
            // `shift_line` may have moved the column.
            beginline(BL_SOL as c_int | BL_FIX as c_int);
        } else {
            (*curwin.get()).w_cursor.lnum -= 1;
        }
        // The cursor line must not be in a closed fold.
        foldOpenCursor();

        if (*oap).line_count as OptInt > p_report.get() {
            // Two plural forms, nested: "line"/"lines" on the line count and
            // "time"/"times" on the shift count, which is why the outer
            // `ngettext` chooses between two already-translated formats.
            let op = if (*oap).op_type == OP_RSHIFT {
                c">".as_ptr()
            } else {
                c"<".as_ptr()
            };
            let msg_line_single = ngettext(
                c"%ld line %sed %d time".as_ptr(),
                c"%ld line %sed %d times".as_ptr(),
                amount as c_ulong,
            );
            let msg_line_plural = ngettext(
                c"%ld lines %sed %d time".as_ptr(),
                c"%ld lines %sed %d times".as_ptr(),
                amount as c_ulong,
            );
            vim_snprintf(
                IObuff.ptr() as *mut c_char,
                IOSIZE as size_t,
                ngettext(
                    msg_line_single,
                    msg_line_plural,
                    (*oap).line_count as c_ulong,
                ),
                (*oap).line_count as int64_t,
                op,
                amount,
            );
            msg_keep(IObuff.ptr() as *mut c_char, 0, true, false);
        }

        if (*cmdmod.ptr()).cmod_flags & CMOD_LOCKMARKS as c_int == 0 {
            (*curbuf.get()).b_op_start = (*oap).start;
            (*curbuf.get()).b_op_end.lnum = (*oap).end.lnum;
            (*curbuf.get()).b_op_end.col = ml_get_len((*oap).end.lnum);
            if (*curbuf.get()).b_op_end.col > 0 {
                (*curbuf.get()).b_op_end.col -= 1;
            }
        }

        changed_lines(
            curbuf.get(),
            (*oap).start.lnum,
            0,
            (*oap).end.lnum + 1,
            0,
            true,
        );
    }
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
/// # Safety
/// Reads the current line's indent through the cursor.
unsafe fn get_new_sw_indent(left: bool, round: bool, amount: int64_t, sw_val: int64_t) -> int64_t {
    unsafe {
        let count = int64_t::from(get_indent());

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
    unsafe {
        let sw_val = (*curbuf.get()).b_p_sw;
        let ts_val = (*curbuf.get()).b_p_ts;
        let vts_array = (*curbuf.get()).b_p_vts_array as *const c_int;
        // `vts_array[0]` is the count, so the whole array is one longer.
        let vts = (!vts_array.is_null() && *vts_array != 0)
            .then(|| ::core::slice::from_raw_parts(vts_array, *vts_array as usize + 1));

        let count = match vts {
            _ if sw_val != 0 => get_new_sw_indent(left, round, int64_t::from(amount), sw_val),
            None => get_new_sw_indent(left, round, int64_t::from(amount), ts_val),
            Some(vts) => get_new_vts_indent(left, round, amount, vts, int64_t::from(get_indent())),
        };

        let indent = crate::math::trim_to_int(count);
        if State.get() & VREPLACE_FLAG != 0 {
            change_indent(INDENT_SET as c_int, indent, false_0, call_changed_bytes);
        } else {
            set_indent(
                indent,
                if call_changed_bytes {
                    SIN_CHANGED as c_int
                } else {
                    0
                },
            );
        }
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
/// # Safety
/// `oap` must point to a live blockwise `oparg_T`; the cursor names the line.
pub(crate) unsafe fn shift_block(oap: *mut oparg_T, amount: c_int) {
    unsafe {
        let left = (*oap).op_type == OP_LSHIFT;
        let old_state = State.get();
        let old_col = (*curwin.get()).w_cursor.col;
        let sw_val = get_sw_value_indent(curbuf.get(), left);
        let old_p_ri = p_ri.get();

        // No 'revins' and no MODE_REPLACE while we rebuild the indent.
        p_ri.set(0);
        State.set(MODE_INSERT);

        let mut bd = block_def::ZERO;
        block_prep(oap, &raw mut bd, (*curwin.get()).w_cursor.lnum, true);
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

        ml_replace((*curwin.get()).w_cursor.lnum, shifted.line, false);
        changed_bytes((*curwin.get()).w_cursor.lnum, bd.textcol);
        extmark_splice_cols(
            curbuf.get(),
            (*curwin.get()).w_cursor.lnum as c_int - 1,
            shifted.start_col,
            shifted.old_len,
            shifted.new_len,
            kExtmarkUndo,
        );

        State.set(old_state);
        (*curwin.get()).w_cursor.col = old_col;
        p_ri.set(old_p_ri);
    }
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
/// # Safety
/// `bd` must describe the cursor line, as [`block_prep`] left it.
unsafe fn shift_block_right(bd: &mut block_def, mut total: c_int) -> ShiftedLine {
    unsafe {
        let old_p = get_cursor_line_ptr();
        let old_line_len = get_cursor_line_len();
        let ts_val = (*curbuf.get()).b_p_ts as c_int;

        // All the virtual white space up to and including a split TAB.
        total += bd.pre_whitesp;
        let mut ws_vcol = bd.start_vcol - bd.pre_whitesp;
        let old_textstart = bd.textstart;
        if bd.startspaces != 0 {
            if utfc_ptr2len(bd.textstart) == 1 {
                bd.textstart = bd.textstart.offset(1);
            } else {
                // A multi-byte character straddles the edge: it cannot be
                // rebuilt as white space, so start the run at the block.
                ws_vcol = 0;
                bd.startspaces = 0;
            }
        }

        // Add the width of the white space that follows the block's edge.
        let mut csarg = CharsizeArg::default();
        let cstype = init_charsize_arg(
            &mut csarg,
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            bd.textstart,
        );
        let mut ci: StrCharInfo = utf_ptr2StrCharInfo(bd.textstart);
        let mut vcol = bd.start_vcol as c_int;
        while ascii_iswhite(ci.chr.value) {
            let incr = win_charsize(cstype, vcol, ci.ptr, ci.chr.value, &mut csarg).width;
            ci = utfc_next(ci);
            total += incr;
            vcol += incr;
        }
        bd.textstart = ci.ptr;
        bd.start_vcol = vcol;

        // `total` is now all the white space wanted and `bd.textstart` points
        // at the first non-white character in the block.
        let mut tabs = 0;
        let mut spaces = 0;
        if (*curbuf.get()).b_p_et == 0 {
            tabstop_fromto(
                ws_vcol,
                ws_vcol + total,
                ts_val,
                (*curbuf.get()).b_p_vts_array,
                &raw mut tabs,
                &raw mut spaces,
            );
        } else {
            spaces = total;
        }

        // Allow for a split TAB in front of the block.
        let col_pre = bd.pre_whitesp_c - c_int::from(bd.startspaces != 0);
        bd.textcol -= col_pre;

        let new_line_len =
            bd.textcol + tabs + spaces + (old_line_len - bd.textstart.offset_from(old_p) as c_int);
        let newp = xmalloc(new_line_len as size_t + 1) as *mut c_char;
        memmove(
            newp as *mut c_void,
            old_p as *const c_void,
            bd.textcol as size_t,
        );
        memset(
            newp.offset(bd.textcol as isize) as *mut c_void,
            TAB,
            tabs as size_t,
        );
        memset(
            newp.offset((bd.textcol + tabs) as isize) as *mut c_void,
            ' ' as c_int,
            spaces as size_t,
        );
        strcpy(
            newp.offset((bd.textcol + tabs + spaces) as isize),
            bd.textstart,
        );

        let shifted = ShiftedLine {
            line: newp,
            start_col: bd.textcol,
            old_len: bd.textstart.offset_from(old_textstart) as c_int + col_pre,
            new_len: tabs + spaces,
        };
        debug_assert!(shifted.new_len - shifted.old_len == new_line_len - old_line_len);
        shifted
    }
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
/// # Safety
/// `bd` must describe the cursor line, as [`block_prep`] left it.
unsafe fn shift_block_left(oap: *mut oparg_T, bd: &mut block_def, total: c_int) -> ShiftedLine {
    unsafe {
        let old_p = get_cursor_line_ptr();
        let old_line_len = get_cursor_line_len();

        // Find the first non-white character displayed after the block's start
        // column, and the width of the white space in front of it. When
        // `startspaces` is set, `textstart` is the character the block's edge
        // splits, so the search starts after it.
        let mut non_white = bd.textstart;
        if bd.startspaces != 0 {
            non_white = non_white.offset(utfc_ptr2len(non_white) as isize);
        }
        let mut non_white_col = bd.start_vcol;
        let mut csarg = CharsizeArg::default();
        let mut cstype = init_charsize_arg(
            &mut csarg,
            curwin.get(),
            (*curwin.get()).w_cursor.lnum,
            bd.textstart,
        );
        while ascii_iswhite(*non_white as c_int) {
            non_white_col += win_charsize(
                cstype,
                non_white_col,
                non_white,
                *non_white as u8 as int32_t,
                &mut csarg,
            )
            .width;
            non_white = non_white.offset(1);
        }

        // Shift by `total`, or by all the white space there is if that is less.
        let block_space_width = non_white_col - (*oap).start_vcol;
        let destination_col = non_white_col - block_space_width.min(total);

        // How much of the line can be reused unchanged. When `startspaces` is
        // set, `textstart` is the character *preceding* the block, so its width
        // comes off to get its column.
        let mut verbatim_copy_width = bd.start_vcol;
        if bd.startspaces != 0 {
            verbatim_copy_width -= bd.start_char_vcols;
        }
        cstype = init_charsize_arg(&mut csarg, curwin.get(), 0, bd.textstart);
        let mut ci: StrCharInfo = utf_ptr2StrCharInfo(bd.textstart);
        while verbatim_copy_width < destination_col {
            let incr = win_charsize(
                cstype,
                verbatim_copy_width,
                ci.ptr,
                ci.chr.value,
                &mut csarg,
            )
            .width;
            if verbatim_copy_width + incr > destination_col {
                break;
            }
            verbatim_copy_width += incr;
            ci = utfc_next(ci);
        }
        let verbatim_copy_end = ci.ptr;

        // A destination inside a TAB leaves a gap the TAB used to cover.
        debug_assert!(destination_col - verbatim_copy_width >= 0);
        let fill = destination_col - verbatim_copy_width;

        debug_assert!(verbatim_copy_end.offset_from(old_p) >= 0);
        // The part of the line left of the shift, which is not being shifted.
        let fixedlen = verbatim_copy_end.offset_from(old_p) as c_int;
        // The replacement line is that part, then `fill` spaces, then the rest
        // of the line from `non_white`.
        let new_line_len = fixedlen + fill + (old_line_len - non_white.offset_from(old_p) as c_int);

        let newp = xmalloc(new_line_len as size_t + 1) as *mut c_char;
        memmove(
            newp as *mut c_void,
            old_p as *const c_void,
            fixedlen as size_t,
        );
        memset(
            newp.offset(fixedlen as isize) as *mut c_void,
            ' ' as c_int,
            fill as size_t,
        );
        strcpy(newp.offset((fixedlen + fill) as isize), non_white);

        let shifted = ShiftedLine {
            line: newp,
            start_col: fixedlen,
            old_len: bd.textcol + non_white.offset_from(bd.textstart) as c_int - fixedlen,
            new_len: fill,
        };
        debug_assert!(shifted.new_len - shifted.old_len == new_line_len - old_line_len);
        shifted
    }
}
