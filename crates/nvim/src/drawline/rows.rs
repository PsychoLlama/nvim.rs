//! One screen row of a buffer line: the info columns that start it and the
//! hand-off to the grid that ends it.
//!
//! A buffer line occupies as many rows as it takes. Each one begins with the
//! columns left of the text — the fold column, the sign column, the number
//! column or `'statuscolumn'`, then `'breakindent'` and `'showbreak'` — and
//! ends either because the row filled up ([`Cells::finish_screen_line`],
//! which starts the next one) or because the text ran out
//! ([`Cells::finish_line`], which does not).

#![deny(unsafe_op_in_unsafe_fn)]

use super::*;
use crate::decoration::{kHlModeCombine, kHlModeReplace, kVLLeftcol, kVLScroll};
use crate::grid::{SLF_INC_VCOL, SLF_WRAP};
use crate::r#move::WinValid;
use crate::option::cpo_has;
use crate::types::{CpoFlag, NUL};

impl Cells {
    /// Draw everything left of the text on this screen row: the fold, sign and
    /// number columns or `'statuscolumn'`, then `'breakindent'` and
    /// `'showbreak'`.
    ///
    /// # Safety
    /// `wp` must be live and `f` must hold the caller's frame.
    pub(super) unsafe fn draw_columns(
        &mut self,
        wlv: &mut WinLineVars,
        wp: *mut win_T,
        f: &LineFrame,
    ) -> Step {
        // SAFETY: the caller's window and frame.
        unsafe {
            if self.cul_screenline {
                wlv.cursorline_attr = 0;
                wlv.line_attr = self.line_attr_save;
                wlv.line_attr_lowprio = self.line_attr_lowprio_save;
            }
            debug_assert!(wlv.off == 0);

            if wp == cmdwin_win.get() {
                wlv.draw_col_fill(
                    schar_from_ascii(cmdwin_type.get() as u8),
                    1,
                    win_hl_attr(wp, HLF_AT),
                );
            }

            if wlv.filler_todo > 0 {
                // How many of the filler rows still to draw are virtual lines
                // rather than diff filler: the virtual ones come last.
                let index = wlv.filler_todo - (wlv.filler_lines - wlv.n_virt_lines);
                if index > 0 {
                    self.virt_line_index = self.virt_lines.size as ::core::ffi::c_int - index;
                    debug_assert!(self.virt_line_index >= 0);
                    self.virt_line_flags =
                        (*self.virt_lines.items.offset(self.virt_line_index as isize)).flags;
                }
            }

            if self.virt_line_index >= 0
                && self.virt_line_flags & kVLLeftcol as ::core::ffi::c_int != 0
            {
                // A virtual line pinned to the left edge covers the columns.
            } else if (*f.statuscol).draw {
                let at = self.byte_col();
                wlv.draw_statuscol(
                    wp,
                    wlv.row - wlv.startrow - wlv.filler_lines,
                    f.col_rows,
                    f.statuscol,
                );
                if (*wp).w_redr_statuscol {
                    return Step::Done;
                }
                if self.draw_text {
                    // Evaluating 'statuscolumn' may have freed the line.
                    self.refetch_line(wp, wlv.lnum, at);
                }
            } else {
                wlv.draw_foldcolumn(wp);
                // `w_scwidth` is zero when 'signcolumn' is "number".
                for sign_idx in 0..(*wp).w_scwidth {
                    wlv.draw_sign(false, wp, sign_idx);
                }
                wlv.draw_lnum_col(wp);
            }

            self.text_start_col = wlv.off;

            if f.col_rows > 0 {
                return self.columns_only(wlv, wp, f);
            }

            if !(*wp).w_briopt_sbr {
                wlv.handle_breakindent(wp);
            }
            wlv.handle_showbreak_and_filler(wp);
            if (*wp).w_briopt_sbr {
                wlv.handle_breakindent(wp);
            }

            wlv.col = wlv.off;
            self.columns_todo = false;
            if wlv.filler_todo <= 0 {
                self.left_columns_width = wlv.off;
            }
            if self.has_decor && wlv.row == wlv.startrow + wlv.filler_lines {
                // Hide virtual text over text hidden by 'nowrap' or
                // 'smoothscroll'.
                decor_redraw_col(
                    wp,
                    self.byte_col() - 1,
                    wlv.off,
                    true,
                    DecorStateRef::current(),
                    self.decor_provider_end_col - 1,
                );
            }
            if wlv.col >= self.view_width {
                wlv.off = self.view_width;
                wlv.col = wlv.off;
                return Step::RowFull;
            }
            Step::Go
        }
    }

    /// Finish a row when only the info columns are being redrawn.
    ///
    /// # Safety
    /// `wp` must be live and `f` must hold the caller's frame.
    pub(super) unsafe fn columns_only(
        &mut self,
        wlv: &mut WinLineVars,
        wp: *mut win_T,
        f: &LineFrame,
    ) -> Step {
        // SAFETY: the caller's window and frame.
        unsafe {
            wlv_put_linebuf(
                wp,
                wlv,
                wlv.off.min(self.view_width),
                false,
                self.bg_attr,
                0,
            );
            // More rows are needed when 'statuscolumn' is drawn, when
            // LineNrAbove or LineNrBelow differ from LineNr, or while there is
            // still filler.
            let more_rows = wlv.row + 1 - wlv.startrow < f.col_rows
                && ((*f.statuscol).draw
                    || win_hl_attr(wp, HLF_LNA) != win_hl_attr(wp, HLF_N)
                    || win_hl_attr(wp, HLF_LNB) != win_hl_attr(wp, HLF_N));
            if !more_rows && wlv.filler_todo <= 0 {
                return Step::Done;
            }
            wlv.row += 1;
            if wlv.row == f.endrow {
                return Step::Done;
            }
            wlv.filler_todo -= 1;
            self.virt_line_index = -1;
            if wlv.filler_todo == 0 && ((*wp).w_botfill || !self.draw_text) {
                return Step::Done;
            }
            // Deliberately not `start_line`: the line buffer already holds
            // what the previous row drew and only the cursor is reset.
            wlv.col = 0;
            wlv.off = 0;
            Step::NextRow
        }
    }

    /// Is the screen row full, with more of the line still to come?
    ///
    /// # Safety
    /// `wp` must be a live window and the loop's pointers readable.
    pub(super) unsafe fn row_is_full(&self, wlv: &WinLineVars, wp: *mut win_T) -> bool {
        // SAFETY: the caller's window and the loop's own pointers.
        unsafe {
            wlv.col >= self.view_width
                && (!self.has_foldtext || wlv.filler_todo > 0)
                && (wlv.col <= self.left_columns_width
                    || *self.ptr as ::core::ffi::c_int != NUL
                    || wlv.filler_todo > 0
                    || ((*wp).w_onebuf_opt.wo_list != 0
                        && (*wp).w_p_lcs_chars.eol != NUL as schar_T
                        && self.lcs_eol_todo)
                    || (wlv.extra_todo != 0
                        && (wlv.extra_fill != NUL as schar_T
                            || *wlv.extra_text as ::core::ffi::c_int != NUL))
                    || (self.may_have_inline_virt
                        && wlv.has_more_inline_virt(self.ptr.offset_from(self.line))))
        }
    }

    /// Hand the finished screen row to the grid and set up the next one.
    ///
    /// # Safety
    /// `wp`, `buf`, `f` and `grid` must be live.
    pub(super) unsafe fn finish_screen_line(
        &mut self,
        wlv: &mut WinLineVars,
        wp: *mut win_T,
        buf: *mut buf_T,
        f: &LineFrame,
        grid: *mut GridView,
    ) -> Step {
        // SAFETY: the caller's window, buffer, frame and grid.
        unsafe {
            let grid_width = (*(*wp).w_grid.target).cols;
            let wrap = self.is_wrapped                        // wrapping, not a folded line
                && wlv.filler_todo <= 0                       // not drawing filler
                && self.lcs_eol_todo                          // the "eol" is still to come
                && wlv.row != f.endrow - 1                    // not the last row shown
                && self.view_width == grid_width              // the window spans its grid
                && (*wp).w_onebuf_opt.wo_rl == 0; // not right-to-left

            let mut draw_col = wlv.col - wlv.boguscols;
            for i in draw_col..self.view_width {
                *linebuf_vcol.get().add((wlv.off + (i - draw_col)) as usize) = wlv.vcol - 1;
            }

            // Fill the columns concealment pretended to use, so that
            // 'cursorline' still covers the whole row.
            if wlv.boguscols != 0 && (wlv.line_attr_lowprio != 0 || wlv.line_attr != 0) {
                let attr = hl_combine_attr(wlv.line_attr_lowprio, wlv.line_attr);
                while draw_col < self.view_width {
                    *linebuf_char.get().add(wlv.off as usize) = schar_from_ascii(b' ');
                    *linebuf_attr.get().add(wlv.off as usize) = attr as sattr_T;
                    // `linebuf_vcol` was filled by the loop above.
                    wlv.off += 1;
                    draw_col += 1;
                }
            }

            if self.virt_line_index >= 0 {
                draw_virt_text_item(
                    buf,
                    if self.virt_line_flags & kVLLeftcol as ::core::ffi::c_int != 0 {
                        0
                    } else {
                        self.text_start_col
                    },
                    (*self.virt_lines.items.offset(self.virt_line_index as isize)).line,
                    kHlModeReplace,
                    self.view_width,
                    0,
                    if self.virt_line_flags & kVLScroll as ::core::ffi::c_int != 0 {
                        (*wp).w_leftcol
                    } else {
                        0
                    },
                );
            } else if wlv.filler_todo <= 0 {
                draw_virt_text(wp, buf, self.text_start_col, &mut draw_col, wlv.row);
            }

            wlv_put_linebuf(
                wp,
                wlv,
                draw_col,
                true,
                self.bg_attr,
                if wrap {
                    SLF_WRAP as ::core::ffi::c_int
                } else {
                    0
                },
            );
            if wrap {
                let mut current_row = wlv.row;
                let mut dummy_col = 0;
                let current_grid = grid_adjust(grid, &raw mut current_row, &raw mut dummy_col);
                // Force a redraw of the first column of the next line.
                *(*current_grid)
                    .attrs
                    .add(*(*current_grid).line_offset.offset(current_row as isize + 1)) =
                    -1 as sattr_T;
            }

            wlv.boguscols = 0;
            wlv.vcol_off_co = 0;
            wlv.row += 1;

            // Without wrapping, and with the filler done, there is no next row.
            if !self.is_wrapped && wlv.filler_todo <= 0 {
                return Step::Done;
            }
            // The window is too narrow to draw anything in: fill it with "@".
            if wlv.col <= self.left_columns_width {
                win_draw_end(
                    wp,
                    schar_from_ascii(b'@'),
                    true,
                    wlv.row,
                    (*wp).w_view_height,
                    HLF_AT,
                );
                set_empty_rows(wp, wlv.row);
                wlv.row = f.endrow;
            }
            // The line got too long for the screen.
            if wlv.row == f.endrow {
                wlv.row += 1;
                return Step::Done;
            }

            wlv.start_line(wp);
            self.columns_todo = true;
            self.lcs_prec_todo = (*wp).w_p_lcs_chars.prec;
            if wlv.filler_todo <= 0 {
                wlv.need_showbreak = true;
            }
            if (*f.statuscol).draw
                && cpo_has(CpoFlag::NUMCOL)
                && wlv.row > wlv.startrow + wlv.filler_lines
            {
                // "n" in 'cpo': the status column is drawn on the first row
                // only.
                (*f.statuscol).draw = false;
            }
            wlv.filler_todo -= 1;
            self.virt_line_index = -1;
            self.virt_line_flags = 0;
            // The filler lines are below the last line of the file, or there
            // is no text to draw for this line.
            if wlv.filler_todo == 0 && ((*wp).w_botfill || !self.draw_text) {
                return Step::Done;
            }
            Step::Go
        }
    }

    /// The text has run out: fill the rest of the row for `'cursorcolumn'`,
    /// `'colorcolumn'`, a whole-line highlight or a terminal, draw the virtual
    /// texts and hand the row to the grid.
    ///
    /// # Safety
    /// `wp`, `buf` and `f` must be live.
    pub(super) unsafe fn finish_line(
        &mut self,
        wlv: &mut WinLineVars,
        wp: *mut win_T,
        buf: *mut buf_T,
        f: &LineFrame,
    ) {
        // SAFETY: the caller's window, buffer and frame.
        unsafe {
            // The line may end left of the left margin.
            wlv.vcol = wlv.vcol.max(self.start_vcol + wlv.col - win_col_off(wp));
            // Drop the bogus columns: 'cursorcolumn' wants drawing all the way
            // to the right edge.
            wlv.col -= wlv.boguscols;
            wlv.boguscols = 0;
            wlv.advance_color_col(wlv.hl_vcol());

            // Keep the alignment the same whether or not "eol" is in
            // 'listchars'.
            let eol_skip = ::core::ffi::c_int::from(self.lcs_eol_todo && self.eol_extra_cell == 0);
            if self.has_decor {
                decor_redraw_eol(
                    wp,
                    DecorStateRef::current(),
                    &raw mut wlv.line_attr,
                    wlv.col + eol_skip,
                );
            }

            // Increasing virtual columns past the end of the line, so a click
            // out there lands somewhere sensible.
            for i in wlv.col..self.view_width {
                *linebuf_vcol.get().add((wlv.off + (i - wlv.col)) as usize) =
                    wlv.vcol + (i - wlv.col);
            }

            if ((*wp).w_onebuf_opt.wo_cuc != 0
                && (*wp).w_virtcol >= wlv.hl_vcol() - self.eol_extra_cell
                && ((*wp).w_virtcol as ptrdiff_t)
                    < self.view_width as ptrdiff_t * (wlv.row - wlv.startrow + 1) as ptrdiff_t
                        + self.start_vcol as ptrdiff_t
                && wlv.lnum != (*wp).w_cursor.lnum)
                || !wlv.color_cols.is_null()
                || wlv.line_attr_lowprio != 0
                || wlv.line_attr != 0
                || wlv.diff_hlf != HLF_NONE
                || !(*(*wp).w_buffer).terminal.is_null()
            {
                self.fill_past_eol(wlv, wp, f);
            }

            if self.fold_vt.size > 0 {
                draw_virt_text_item(
                    buf,
                    self.text_start_col,
                    self.fold_vt,
                    kHlModeCombine,
                    self.view_width,
                    0,
                    0,
                );
            }
            draw_virt_text(wp, buf, self.text_start_col, &mut wlv.col, wlv.row);
            // SLF_INC_VCOL fills grid->vcols[] with increasing columns, so
            // that "curswant" (or "coladd" under 'virtualedit') is right when
            // the user clicks past the end of the line.
            wlv_put_linebuf(
                wp,
                wlv,
                wlv.col,
                true,
                self.bg_attr,
                SLF_INC_VCOL as ::core::ffi::c_int,
            );
            wlv.row += 1;

            // Record the cursor line's height while it is known, which saves a
            // `plines_win` later.
            if self.in_curline {
                (*curwin.get()).w_cline_row = wlv.startrow;
                (*curwin.get()).w_cline_height = wlv.row - wlv.startrow;
                (*curwin.get()).w_cline_folded = self.has_fold;
                (*curwin.get()).w_valid |= WinValid::CHEIGHT | WinValid::CROW;
            }
        }
    }

    /// Draw the blanks past the end of the line that still carry a highlight.
    ///
    /// # Safety
    /// `wp` and `f` must be live.
    pub(super) unsafe fn fill_past_eol(
        &mut self,
        wlv: &mut WinLineVars,
        wp: *mut win_T,
        f: &LineFrame,
    ) {
        // SAFETY: the caller's window and frame.
        unsafe {
            let mut rightmost_vcol = get_rightmost_vcol(wp, wlv.color_cols);
            let cuc_attr = win_hl_attr(wp, HLF_CUC);
            let mc_attr = win_hl_attr(wp, HLF_MC);

            if wlv.diff_hlf == HLF_TXD || wlv.diff_hlf == HLF_TXA {
                wlv.diff_hlf = HLF_CHD;
                wlv.set_line_attr_for_diff(wp);
            }
            let diff_attr = if wlv.diff_hlf != HLF_NONE {
                win_hl_attr(wp, wlv.diff_hlf)
            } else {
                0
            };
            let base_attr = hl_combine_attr(wlv.line_attr_lowprio, diff_attr);
            if base_attr != 0 || wlv.line_attr != 0 || !(*(*wp).w_buffer).terminal.is_null() {
                // Something applies to the whole row, so there is no column to
                // stop at.
                rightmost_vcol = ::core::ffi::c_int::MAX;
            }

            while wlv.col < self.view_width {
                *linebuf_char.get().add(wlv.off as usize) = schar_from_ascii(b' ');
                wlv.advance_color_col(wlv.hl_vcol());

                let mut col_attr = base_attr;
                if (*wp).w_onebuf_opt.wo_cuc != 0
                    && wlv.hl_vcol() == (*wp).w_virtcol
                    && wlv.lnum != (*wp).w_cursor.lnum
                {
                    col_attr = hl_combine_attr(col_attr, cuc_attr);
                } else if !wlv.color_cols.is_null() && wlv.hl_vcol() == *wlv.color_cols {
                    col_attr = hl_combine_attr(col_attr, mc_attr);
                }
                if !(*(*wp).w_buffer).terminal.is_null()
                    && wlv.vcol < TERM_ATTRS_MAX as ::core::ffi::c_int
                {
                    col_attr = hl_combine_attr(col_attr, *f.term_attrs.offset(wlv.vcol as isize));
                }
                col_attr = hl_combine_attr(col_attr, wlv.line_attr);

                *linebuf_attr.get().add(wlv.off as usize) = col_attr as sattr_T;
                // `linebuf_vcol` was filled by the loop in the caller.
                wlv.off += 1;
                wlv.col += 1;
                wlv.vcol += 1;

                if wlv.hl_vcol() > rightmost_vcol {
                    break;
                }
            }
        }
    }
}
