#![deny(unsafe_op_in_unsafe_fn)]

//! Vertical size: how many window lines a buffer line occupies.
//!
//! Built on the parent module's horizontal half plus the three things that
//! add lines of their own -- folds (which collapse many buffer lines into
//! one), diff filler, and virtual lines.

use super::*;
use crate::pos::MAXCOL;
use crate::types::NUL;

/// Whether there may be filler lines anywhere in `wp`.
///
pub(crate) fn win_may_fill(wp: Win) -> bool {
    wp.w_onebuf_opt.wo_diff != 0 && diffopt_filler()
        || buf_meta_total(wp.buffer(), MT_META_LINES) != 0
}

/// Filler lines above `lnum`: virtual lines plus, in a diff, the lines the
/// other buffer has here and this one does not.
///
/// # Safety
/// `wp` must be live.
pub(crate) unsafe fn win_get_fill(wp: Win, lnum: linenr_T) -> c_int {
    let virt_lines = unsafe {
        decor_virt_lines(
            wp.raw(),
            lnum - 1,
            lnum,
            ::core::ptr::null_mut::<c_int>(),
            ::core::ptr::null_mut::<VirtLines>(),
            true,
        )
    };

    // Be quick when there are no filler lines.
    if diffopt_filler() {
        let n = diff_check_fill(wp, lnum);
        if n > 0 {
            return virt_lines + n;
        }
    }
    virt_lines
}

/// Window lines buffer line `lnum` occupies, filler lines included.
///
/// # Safety
/// `wp` must be live and `lnum` a line of its buffer.
pub(crate) unsafe fn plines_win(wp: Win, lnum: linenr_T, limit_winheight: bool) -> c_int {
    unsafe { plines_win_nofill(wp, lnum, limit_winheight) + win_get_fill(wp, lnum) }
}

/// Window lines buffer line `lnum` occupies, filler lines excluded.
///
/// # Safety
/// `wp` must be live and `lnum` a line of its buffer.
pub(crate) unsafe fn plines_win_nofill(wp: Win, lnum: linenr_T, limit_winheight: bool) -> c_int {
    if unsafe { decor_conceal_line(wp.raw(), lnum - 1, false) } {
        return 0;
    }
    if wp.w_onebuf_opt.wo_wrap == 0 || wp.w_view_width == 0 {
        return 1;
    }
    // A folded line is handled just like an empty one.
    if line_folded(wp, lnum) {
        return 1;
    }

    let lines = unsafe { plines_win_nofold(wp, lnum) };
    if limit_winheight && lines > wp.w_view_height {
        return wp.w_view_height;
    }
    lines
}

/// Window lines physical line `lnum` occupies, ignoring folding, 'wrap' and
/// filler lines.
///
/// # Safety
/// `wp` must be live and `lnum` a line of its buffer.
pub(crate) unsafe fn plines_win_nofold(wp: Win, lnum: linenr_T) -> c_int {
    let s = unsafe { ml_get_buf(wp.w_buffer, lnum) };
    let mut csarg = CharsizeArg::default();
    let cstype = unsafe { init_charsize_arg(&mut csarg, wp, lnum, s) };
    if unsafe { *s } == NUL as c_char && csarg.virt_row < 0 {
        // Be quick for an empty line.
        return 1;
    }

    let mut col = match cstype {
        CharsizeKind::Fast => unsafe { linesize_fast(&csarg, 0, MAXCOL) },
        CharsizeKind::Regular => unsafe { linesize_regular(&mut csarg, 0, MAXCOL) },
    } as int64_t;

    // In 'list' mode the trailing '$' may take one more column.
    if wp.w_onebuf_opt.wo_list != 0 && wp.w_p_lcs_chars.eol != 0 {
        col += 1;
    }

    // Column offset for 'number', 'relativenumber' and 'foldcolumn'.
    let mut width = wp.w_view_width - unsafe { win_col_off(wp.raw()) };
    if width <= 0 {
        // Bigger than the number of screen lines.
        return 32000;
    }
    if col <= width as int64_t {
        return 1;
    }
    col -= width as int64_t;
    width += win_col_off2(wp);
    let lines = (col + (width - 1) as int64_t) / width as int64_t + 1;
    if lines > 0 && lines <= c_int::MAX as int64_t {
        lines as c_int
    } else {
        c_int::MAX
    }
}

/// Window lines used from the start of line `lnum` up to `column`.
///
/// # Safety
/// `wp` must be live and `lnum` a line of its buffer.
pub(crate) unsafe fn plines_win_col(wp: Win, lnum: linenr_T, mut column: c_long) -> c_int {
    // Filler lines above this buffer line.
    let mut lines = unsafe { win_get_fill(wp, lnum) };

    if wp.w_onebuf_opt.wo_wrap == 0 || wp.w_view_width == 0 {
        return lines + 1;
    }

    let line = unsafe { ml_get_buf(wp.w_buffer, lnum) };
    let mut csarg = CharsizeArg::default();
    let cstype = unsafe { init_charsize_arg(&mut csarg, wp, lnum, line) };

    let mut vcol: colnr_T = 0;
    let mut ci: StrCharInfo = unsafe { utf_ptr2str_char_info(line) };
    if cstype == CharsizeKind::Fast {
        let use_tabstop = csarg.use_tabstop;
        while unsafe { *ci.ptr } != NUL as c_char && {
            column -= 1;
            column >= 0
        } {
            vcol +=
                unsafe { charsize_fast_impl(wp.raw(), ci.ptr, use_tabstop, vcol, ci.chr.value) }
                    .width;
            ci = unsafe { utfc_next(ci) };
        }
    } else {
        while unsafe { *ci.ptr } != NUL as c_char && {
            column -= 1;
            column >= 0
        } {
            vcol += unsafe { charsize_regular(&mut csarg, ci.ptr, vcol, ci.chr.value) }.width;
            ci = unsafe { utfc_next(ci) };
        }
    }

    // If the current char is a TAB shown as a tab, and we are not in
    // Insert mode, "col" must be the TAB's *last* screen position. This
    // only fixes an error when the TAB wraps from one screen line to the
    // next (when 'columns' is not a multiple of 'ts') -- webb.
    let mut col = vcol;
    if ci.chr.value == TAB && State.get() & MODE_NORMAL != 0 && csarg.use_tabstop {
        col += unsafe { win_charsize(cstype, col, ci.ptr, ci.chr.value, &mut csarg) }.width - 1;
    }

    // Column offset for 'number', 'relativenumber', 'foldcolumn', etc.
    let width = wp.w_view_width - unsafe { win_col_off(wp.raw()) };
    if width <= 0 {
        return 9999;
    }

    lines += 1;
    if col > width {
        lines += (col - width) / (width + win_col_off2(wp)) + 1;
    }
    lines
}

/// Screen lines buffer line `lnum` takes, folds and topfill included.
///
/// Because of topfill this only makes sense for `lnum >= wp->w_topline`.
///
/// # Safety
/// `lnum` must be a line of `wp`'s buffer. `nextp` is set to the last line of
/// a fold, `foldedp` to whether there was one.
pub(crate) unsafe fn plines_win_full(
    wp: Win,
    mut lnum: linenr_T,
    nextp: Option<&mut linenr_T>,
    foldedp: Option<&mut bool>,
    cache: bool,
    limit_winheight: bool,
) -> c_int {
    let folded = has_folding_win(wp, lnum, Some(&mut lnum), nextp, cache, None);
    if let Some(foldedp) = foldedp {
        *foldedp = folded;
    }
    let filler_lines = if lnum == wp.w_topline {
        wp.w_topfill
    } else {
        unsafe { win_get_fill(wp, lnum) }
    };

    if unsafe { decor_conceal_line(wp.raw(), lnum - 1, false) } {
        return filler_lines;
    }

    let text_lines = if folded {
        1
    } else {
        unsafe { plines_win_nofill(wp, lnum, limit_winheight) }
    };
    text_lines + filler_lines
}

/// Window lines the range `first..=last` occupies, capped at `max`. Takes
/// folding, 'wrap', topfill and filler lines past the end of the buffer into
/// account.
///
/// Because of topfill this only makes sense for `first >= wp->w_topline`.
///
/// # Safety
/// `wp` must be live.
pub(crate) unsafe fn plines_m_win(
    wp: Win,
    mut first: linenr_T,
    last: linenr_T,
    max: c_int,
) -> c_int {
    let mut count = 0;
    while first <= last && count < max {
        let mut next = first;
        count += unsafe { plines_win_full(wp, first, Some(&mut next), None, false, false) };
        first = next + 1;
    }
    if first == unsafe { (*wp.w_buffer).b_ml.ml_line_count } + 1 {
        count += unsafe { win_get_fill(wp, first) };
    }
    max.min(count)
}

/// Total physical and filler lines in `first..=last`. Unlike
/// [`plines_m_win`], a fold is not one line and a wrapped line is not
/// several. Mainly used for scrolling offsets.
///
/// # Safety
/// `wp` must be live.
pub(crate) unsafe fn plines_m_win_fill(wp: Win, first: linenr_T, last: linenr_T) -> c_int {
    let mut count = last - first
        + 1
        + unsafe {
            decor_virt_lines(
                wp.raw(),
                first - 1,
                last,
                ::core::ptr::null_mut::<c_int>(),
                ::core::ptr::null_mut::<VirtLines>(),
                false,
            )
        };

    if diffopt_filler() {
        let mut lnum = first;
        while lnum <= last {
            // This also considers folds: no filler lines inside a fold.
            count += diff_check_fill(wp, lnum).max(0);
            lnum += 1;
        }
    }

    count.max(0)
}

/// Screen lines a range of text takes in `wp`.
///
/// `start_vcol` below zero counts all of `start_lnum` including the filler
/// lines above it; at or above zero it starts at that virtual column, rounded
/// down to a whole screen line. `end_vcol` is the mirror of that, rounded up,
/// and is overwritten with the column actually reached. `end_lnum` is
/// likewise overwritten with the last line measured, which is earlier than
/// the one passed in when `max` is reached first.
///
/// # Safety
/// `wp`, `end_lnum` and `end_vcol` must be live; `fill` may be null.
pub(crate) unsafe fn win_text_height(
    wp: Win,
    start_lnum: linenr_T,
    start_vcol: int64_t,
    end_lnum: *mut linenr_T,
    end_vcol: *mut int64_t,
    fill: *mut int64_t,
    max: int64_t,
) -> int64_t {
    let first_width = wp.w_view_width - unsafe { win_col_off(wp.raw()) };
    let width1 = first_width.max(0);
    let width2 = (first_width + win_col_off2(wp)).max(0);

    let mut height_sum_fill: int64_t = 0;
    let mut height_cur_nofill: int64_t = 0;
    let mut height_sum_nofill: int64_t = 0;
    let mut lnum = start_lnum;
    let mut cur_lnum = lnum;
    let mut cur_folded = false;

    if start_vcol >= 0 {
        let mut lnum_next = lnum;
        cur_folded = has_folding(wp, lnum, Some(&mut lnum), Some(&mut lnum_next));
        height_cur_nofill = unsafe { plines_win_nofill(wp, lnum, false) } as int64_t;
        height_sum_nofill += height_cur_nofill;
        let row_off = if start_vcol < width1 as int64_t || width2 <= 0 {
            0
        } else {
            1 + (start_vcol - width1 as int64_t) / width2 as int64_t
        };
        height_sum_nofill -= row_off.min(height_cur_nofill);
        lnum = lnum_next + 1;
    }

    while lnum <= unsafe { *end_lnum } && height_sum_nofill + height_sum_fill < max {
        let mut lnum_next = lnum;
        cur_folded = has_folding(wp, lnum, Some(&mut lnum), Some(&mut lnum_next));
        height_sum_fill += unsafe { win_get_fill(wp, lnum) } as int64_t;
        height_cur_nofill = unsafe { plines_win_nofill(wp, lnum, false) } as int64_t;
        height_sum_nofill += height_cur_nofill;
        cur_lnum = lnum;
        lnum = lnum_next + 1;
    }

    let mut vcol_end = unsafe { *end_vcol };
    let use_vcol = vcol_end >= 0 && lnum > unsafe { *end_lnum };
    if use_vcol {
        height_sum_nofill -= height_cur_nofill;
        let row_off = if vcol_end == 0 {
            0
        } else if vcol_end <= width1 as int64_t || width2 <= 0 {
            1
        } else {
            1 + (vcol_end - width1 as int64_t + width2 as int64_t - 1) / width2 as int64_t
        };
        height_sum_nofill += row_off.min(height_cur_nofill);
    }

    if cur_folded {
        vcol_end = 0;
    } else {
        let linesize = unsafe { linetabsize_eol(wp, cur_lnum) } as int64_t;
        let asked = if use_vcol { vcol_end } else { int64_t::MAX };
        vcol_end = asked.min(linesize);
    }

    let overflow = height_sum_nofill + height_sum_fill - max;
    if overflow > 0 && width2 > 0 && vcol_end > width2 as int64_t {
        vcol_end -=
            (vcol_end - width1 as int64_t) % width2 as int64_t + (overflow - 1) * width2 as int64_t;
    }

    unsafe { *end_lnum = cur_lnum };
    unsafe { *end_vcol = vcol_end };
    if !fill.is_null() {
        unsafe { *fill = height_sum_fill };
    }
    height_sum_fill + height_sum_nofill
}
