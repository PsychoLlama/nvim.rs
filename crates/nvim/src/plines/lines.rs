#![forbid(unsafe_code)]
#![deny(
    clippy::cast_lossless,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::ptr_as_ptr
)]

//! Vertical size: how many window lines a buffer line occupies.
//!
//! Built on the parent module's horizontal half plus the three things that
//! add lines of their own -- folds (which collapse many buffer lines into
//! one), diff filler, and virtual lines.

use super::*;

/// Whether there may be filler lines anywhere in `window`.
///
pub(crate) fn win_may_fill(window: Win) -> bool {
    window.w_onebuf_opt.wo_diff != 0 && diffopt_filler()
        || buf_meta_total(window.buffer(), MT_META_LINES) != 0
}

/// Filler lines above `lnum`: virtual lines plus, in a diff, the lines the
/// other buffer has here and this one does not.
pub(crate) fn win_get_fill(window: Win, lnum: LineNr) -> c_int {
    let virt_lines = window.virt_lines((lnum - 1).get(), lnum.get(), None, None, true);

    // Be quick when there are no filler lines.
    if diffopt_filler() {
        let n = diff_check_fill(window, lnum);
        if n > 0 {
            return virt_lines + n;
        }
    }
    virt_lines
}

/// Window lines buffer line `lnum` occupies, filler lines included.
pub(crate) fn plines_win(window: Win, lnum: LineNr, limit_winheight: bool) -> c_int {
    plines_win_nofill(window, lnum, limit_winheight) + win_get_fill(window, lnum)
}

/// Window lines buffer line `lnum` occupies, filler lines excluded.
pub(crate) fn plines_win_nofill(window: Win, lnum: LineNr, limit_winheight: bool) -> c_int {
    if decor_conceal_line(window, (lnum - 1).get(), false) {
        return 0;
    }
    if window.w_onebuf_opt.wo_wrap == 0 || window.w_view_width == 0 {
        return 1;
    }
    // A folded line is handled just like an empty one.
    if line_folded(window, lnum) {
        return 1;
    }

    let lines = plines_win_nofold(window, lnum);
    if limit_winheight && lines > window.w_view_height {
        return window.w_view_height;
    }
    lines
}

/// Window lines physical line `lnum` occupies, ignoring folding, 'wrap' and
/// filler lines.
pub(crate) fn plines_win_nofold(window: Win, lnum: LineNr) -> c_int {
    let mut text = window.buffer().lines();
    let s = text.line(lnum);
    let (mut csarg, cstype) = CharsizeArg::new(window, lnum, s);
    if s.is_empty() && csarg.virt_row < 0 {
        // Be quick for an empty line.
        return 1;
    }

    let mut col = int64_t::from(match cstype {
        CharsizeKind::Fast => linesize_fast(&csarg, 0, ColNr::MAX),
        CharsizeKind::Regular => linesize_regular(&mut csarg, 0, ColNr::MAX),
    });

    // In 'list' mode the trailing '$' may take one more column.
    if window.w_onebuf_opt.wo_list != 0 && window.w_p_lcs_chars.eol != 0 {
        col += 1;
    }

    // Column offset for 'number', 'relativenumber' and 'foldcolumn'.
    let mut width = window.w_view_width - window.col_off();
    if width <= 0 {
        // Bigger than the number of screen lines.
        return 32000;
    }
    if col <= int64_t::from(width) {
        return 1;
    }
    col -= int64_t::from(width);
    width += win_col_off2(window);
    let lines = (col + int64_t::from(width - 1)) / int64_t::from(width) + 1;
    match c_int::try_from(lines) {
        Ok(lines) if lines > 0 => lines,
        _ => c_int::MAX,
    }
}

/// Window lines used from the start of line `lnum` up to `column`.
pub(crate) fn plines_win_col(window: Win, lnum: LineNr, mut column: c_long) -> c_int {
    // Filler lines above this buffer line.
    let mut lines = win_get_fill(window, lnum);

    if window.w_onebuf_opt.wo_wrap == 0 || window.w_view_width == 0 {
        return lines + 1;
    }

    let mut text = window.buffer().lines();
    let line = text.line(lnum);
    let (mut csarg, cstype) = CharsizeArg::new(window, lnum, line);

    let mut vcol: ColNr = ColNr::ZERO;
    let mut at = 0;
    let mut chr = c_char_info(line, 0);
    if cstype == CharsizeKind::Fast {
        let use_tabstop = csarg.use_tabstop;
        while !ends_at(line, at, chr) && {
            column -= 1;
            column >= 0
        } {
            vcol += charsize_fast_impl(window, line, at, use_tabstop, vcol, chr.value).width;
            (at, chr) = c_next_cluster(line, at, chr);
        }
    } else {
        while !ends_at(line, at, chr) && {
            column -= 1;
            column >= 0
        } {
            vcol += charsize_regular(&mut csarg, at, vcol, chr.value).width;
            (at, chr) = c_next_cluster(line, at, chr);
        }
    }

    // If the current char is a TAB shown as a tab, and we are not in
    // Insert mode, "col" must be the TAB's *last* screen position. This
    // only fixes an error when the TAB wraps from one screen line to the
    // next (when 'columns' is not a multiple of 'ts') -- webb.
    let mut col = vcol;
    if chr.value == TAB && STATE.get() & MODE_NORMAL != 0 && csarg.use_tabstop {
        col += win_charsize(cstype, col.get(), at, chr.value, &mut csarg).width - 1;
    }

    // Column offset for 'number', 'relativenumber', 'foldcolumn', etc.
    let width = window.w_view_width - window.col_off();
    if width <= 0 {
        return 9999;
    }

    lines += 1;
    if col > ColNr::new(width) {
        lines += (col - width).get() / (width + win_col_off2(window)) + 1;
    }
    lines
}

impl Win {
    /// Screen lines buffer line `lnum` takes, folds and topfill included.
    ///
    /// Because of topfill this only makes sense for `lnum >= wp->w_topline`.
    pub(crate) fn screen_lines_full(
        self,
        mut lnum: LineNr,
        nextp: Option<&mut LineNr>,
        foldedp: Option<&mut bool>,
        cache: bool,
        limit_winheight: bool,
    ) -> c_int {
        let folded = self.has_folding_with(lnum, Some(&mut lnum), nextp, cache, None);
        if let Some(foldedp) = foldedp {
            *foldedp = folded;
        }
        let filler_lines = if lnum == self.w_topline {
            self.w_topfill
        } else {
            win_get_fill(self, lnum)
        };

        if decor_conceal_line(self, (lnum - 1).get(), false) {
            return filler_lines;
        }

        let text_lines = if folded {
            1
        } else {
            plines_win_nofill(self, lnum, limit_winheight)
        };
        text_lines + filler_lines
    }

    /// Screen lines a range of text takes in `self`.
    ///
    /// `start_vcol` below zero counts all of `start_lnum` including the filler
    /// lines above it; at or above zero it starts at that virtual column, rounded
    /// down to a whole screen line. `end_vcol` is the mirror of that, rounded up,
    /// and is overwritten with the column actually reached. `end_lnum` is
    /// likewise overwritten with the last line measured, which is earlier than
    /// the one passed in when `max` is reached first. `fill`, when given, gets
    /// the filler lines counted.
    pub(crate) fn text_height(
        self,
        start_lnum: LineNr,
        start_vcol: int64_t,
        end_lnum: &mut LineNr,
        end_vcol: &mut int64_t,
        fill: Option<&mut int64_t>,
        max: int64_t,
    ) -> int64_t {
        let first_width = self.w_view_width - self.col_off();
        let width1 = first_width.max(0);
        let width2 = (first_width + win_col_off2(self)).max(0);

        let mut height_sum_fill: int64_t = 0;
        let mut height_cur_nofill: int64_t = 0;
        let mut height_sum_nofill: int64_t = 0;
        let mut lnum = start_lnum;
        let mut cur_lnum = lnum;
        let mut cur_folded = false;

        if start_vcol >= 0 {
            let mut lnum_next = lnum;
            cur_folded = self.has_folding(lnum, Some(&mut lnum), Some(&mut lnum_next));
            height_cur_nofill = int64_t::from(plines_win_nofill(self, lnum, false));
            height_sum_nofill += height_cur_nofill;
            let row_off = if start_vcol < int64_t::from(width1) || width2 <= 0 {
                0
            } else {
                1 + (start_vcol - int64_t::from(width1)) / int64_t::from(width2)
            };
            height_sum_nofill -= row_off.min(height_cur_nofill);
            lnum = lnum_next + 1;
        }

        while lnum <= *end_lnum && height_sum_nofill + height_sum_fill < max {
            let mut lnum_next = lnum;
            cur_folded = self.has_folding(lnum, Some(&mut lnum), Some(&mut lnum_next));
            height_sum_fill += int64_t::from(win_get_fill(self, lnum));
            height_cur_nofill = int64_t::from(plines_win_nofill(self, lnum, false));
            height_sum_nofill += height_cur_nofill;
            cur_lnum = lnum;
            lnum = lnum_next + 1;
        }

        let mut vcol_end = *end_vcol;
        let use_vcol = vcol_end >= 0 && lnum > *end_lnum;
        if use_vcol {
            height_sum_nofill -= height_cur_nofill;
            let row_off = if vcol_end == 0 {
                0
            } else if vcol_end <= int64_t::from(width1) || width2 <= 0 {
                1
            } else {
                1 + (vcol_end - int64_t::from(width1) + int64_t::from(width2) - 1)
                    / int64_t::from(width2)
            };
            height_sum_nofill += row_off.min(height_cur_nofill);
        }

        if cur_folded {
            vcol_end = 0;
        } else {
            let linesize = int64_t::from(linetabsize_eol(self, cur_lnum));
            let asked = if use_vcol { vcol_end } else { int64_t::MAX };
            vcol_end = asked.min(linesize);
        }

        let overflow = height_sum_nofill + height_sum_fill - max;
        if overflow > 0 && width2 > 0 && vcol_end > int64_t::from(width2) {
            vcol_end -= (vcol_end - int64_t::from(width1)) % int64_t::from(width2)
                + (overflow - 1) * int64_t::from(width2);
        }

        *end_lnum = cur_lnum;
        *end_vcol = vcol_end;
        if let Some(fill) = fill {
            *fill = height_sum_fill;
        }
        height_sum_fill + height_sum_nofill
    }
}

/// Window lines the range `first..=last` occupies, capped at `max`. Takes
/// folding, 'wrap', topfill and filler lines past the end of the buffer into
/// account.
///
/// Because of topfill this only makes sense for `first >= wp->w_topline`.
pub(crate) fn plines_m_win(window: Win, mut first: LineNr, last: LineNr, max: c_int) -> c_int {
    let mut count = 0;
    while first <= last && count < max {
        let mut next = first;
        count += window.screen_lines_full(first, Some(&mut next), None, false, false);
        first = next + 1;
    }
    if first == window.w_buffer.b_ml.ml_line_count + 1 {
        count += win_get_fill(window, first);
    }
    max.min(count)
}

/// Total physical and filler lines in `first..=last`. Unlike
/// [`plines_m_win`], a fold is not one line and a wrapped line is not
/// several. Mainly used for scrolling offsets.
pub(crate) fn plines_m_win_fill(window: Win, first: LineNr, last: LineNr) -> c_int {
    let mut count =
        last - first + 1 + window.virt_lines((first - 1).get(), last.get(), None, None, false);

    if diffopt_filler() {
        let mut lnum = first;
        while lnum <= last {
            // This also considers folds: no filler lines inside a fold.
            count += diff_check_fill(window, lnum).max(0);
            lnum += 1;
        }
    }

    count.max(0)
}
