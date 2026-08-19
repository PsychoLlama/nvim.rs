//! Where a position is on the screen -- `curs_columns()` and the
//! `screenpos()`/`virtcol2col()` builtins.
//!
//! [`curs_columns`] is the horizontal half of the viewport: it computes the
//! cursor's screen column from its virtual column, applies `'sidescroll'` and
//! `'sidescrolloff'` to `w_leftcol`, and picks `w_skipcol` when the line wraps.
//! `textpos2screenpos` answers the same question for an arbitrary position on
//! behalf of `screenpos()`, and `virtcol2col` inverts it.
//!
//! Original: `src/nvim/move.c`, Vim/Neovim, Vim license.

#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{CStr, c_char, c_int};

use super::*;
use crate::drawscreen::{UPD_NOT_VALID, UPD_SOME_VALID, win_scroll_lines};
use crate::eval::typval::{
    tv_check_for_number_arg, tv_dict_add_nr, tv_dict_alloc_ret, tv_get_number, tv_get_number_chk,
};
use crate::eval::window::find_win_by_nr_or_id;
use crate::main::{dollar_vcol, e_invalid_line_number_nr, p_ss};
use crate::mbyte::utf_head_off;
use crate::mouse::vcol2col;
use crate::os::cshim::gettext;
use crate::semsg_c;
use crate::types::{
    EvalFuncData, FAIL, colnr_T, dict_T, int64_t, linenr_T, pos_T, size_t, typval_T, varnumber_T,
    win_T,
};
use crate::winlayer::{Pos, Win};

impl Win {
    /// Scroll the window's own grid by `lines`, so that a `w_skipcol` change
    /// does not have to redraw everything.
    fn scroll_grid_lines(self, lines: c_int) {
        // SAFETY: a live window with a grid attached.
        unsafe { win_scroll_lines(self.raw(), 0, lines) };
    }
}

/// [`Win::curs_columns`], for the callers still holding a raw window.
///
/// # Safety
/// `wp` must be a valid window.
pub unsafe fn curs_columns(wp: *mut win_T, may_scroll: c_int) {
    // SAFETY: the caller's promise.
    unsafe { Win::new(wp) }.curs_columns(may_scroll != 0);
}

impl Win {
    /// Compute `w_wcol` and `w_virtcol`, and with them `w_wrow`,
    /// `w_cline_row` and `w_leftcol`. `may_scroll` allows a horizontal
    /// scroll.
    pub(super) fn curs_columns(self, may_scroll: bool) {
        curs_columns_win(self, may_scroll);
    }
}

fn curs_columns_win(mut win: Win, may_scroll: bool) {
    // First make sure `w_topline` is valid (the cursor may have moved).
    win.update_topline();
    // Then that `w_cline_row` is.
    if !win.w_valid.has(WinValid::CROW) {
        curs_rows(win);
    }

    let (startcol, mut endcol) = if win.w_cline_folded {
        // In a folded line the cursor is always in the first column.
        win.w_virtcol = win.w_leftcol;
        (win.w_leftcol, win.w_leftcol)
    } else {
        let (start, cursor, end) = win.virtual_vcol_triple(win.cursor());
        win.w_virtcol = cursor;
        (start, end)
    };

    // Remove the `$` of a change command once the cursor moves onto it.
    if startcol > dollar_vcol.get() {
        dollar_vcol.set(-1);
    }

    let extra = win.col_off();
    win.w_wcol = win.w_virtcol + extra;
    endcol += extra;

    // Now `w_wrow`, counting screen lines from `w_cline_row`.
    win.w_wrow = win.w_cline_row;

    let width1 = win.w_view_width - extra;
    let mut width2 = 0;
    let mut did_sub_skipcol = false;
    if width1 <= 0 {
        // No room for text: put the cursor in the last column of the window,
        // and without 'wrap' on the last non-empty line.
        win.w_wcol = win.w_view_width - 1;
        win.w_wrow = if win.w_onebuf_opt.wo_wrap != 0 {
            win.w_view_height - 1
        } else {
            win.w_view_height - 1 - win.w_empty_rows
        };
    } else if win.w_onebuf_opt.wo_wrap != 0 && win.w_view_width != 0 {
        width2 = width1 + win.col_off2();
        let (wcol, wrow, subbed) = arith::wrap_cursor_cell(
            win.w_wcol,
            win.w_wrow,
            win.w_skipcol,
            win.w_cursor.lnum == win.w_topline,
            width1,
            width2,
            win.w_view_width,
        );
        win.w_wcol = wcol;
        win.w_wrow = wrow;
        did_sub_skipcol = subbed;
    } else if may_scroll && !win.w_cline_folded {
        // No line wrapping: compute `w_leftcol` if scrolling is on and the
        // line is not folded. With scrolling off `w_leftcol` is assumed 0.
        let scrolled = arith::sidescroll_leftcol(
            startcol,
            endcol,
            win.w_leftcol,
            win.w_wcol,
            extra,
            win.w_view_width,
            width1,
            win.sidescrolloff(),
            p_ss.get(),
        );
        if let Some(new_leftcol) = scrolled
            && new_leftcol != win.w_leftcol
        {
            win.w_leftcol = new_leftcol;
            win.check_anchored_floats();
            // The screen has to be redrawn with the new `w_leftcol`.
            win.redraw_later(UPD_NOT_VALID);
        }
        let leftcol = win.w_leftcol;
        win.w_wcol -= leftcol;
    } else if win.w_wcol > win.w_leftcol {
        let leftcol = win.w_leftcol;
        win.w_wcol -= leftcol;
    } else {
        win.w_wcol = 0;
    }

    // Skip over filler lines. At the top `w_topfill` counts the ones drawn
    // above the window's first line.
    let fill = if win.w_cursor.lnum == win.w_topline {
        win.w_topfill
    } else {
        win.fill_above(win.w_cursor.lnum)
    };
    win.w_wrow += fill;

    let mut plines = 0;
    let so = win.scrolloff();
    let prev_skipcol = win.w_skipcol;
    // A single line that does not fit on the screen: find a `w_skipcol` that
    // shows the text around the cursor, without scrolling all the time.
    let too_tall = win.w_wrow >= win.w_view_height
        || (prev_skipcol > 0 || win.w_wrow as int64_t + so >= win.w_view_height as int64_t) && {
            plines = win.plines_nofill(win.w_cursor.lnum, false);
            // The C spells this `plines - 1 >= w_view_height`; `plines` is a
            // screen-line count, so the two never differ.
            plines > win.w_view_height
        };
    if too_tall
        && win.w_view_height != 0
        && win.w_cursor.lnum == win.w_topline
        && width2 > 0
        && win.w_view_width != 0
    {
        if plines == 0 {
            plines = win.plines(win.w_cursor.lnum, false);
        }
        win.w_skipcol = arith::skipcol_for_tall_line(
            win.w_skipcol,
            win.w_virtcol,
            so,
            width1,
            width2,
            win.w_view_height,
            win.w_wrow,
            plines,
        );
        let (skipcol, wrow, scrolled) = arith::fit_skipcol_to_window(
            win.w_skipcol,
            prev_skipcol,
            win.w_wrow,
            did_sub_skipcol,
            width2,
            win.w_view_height,
        );
        win.w_skipcol = skipcol;
        win.w_wrow = wrow;
        // TODO(bfredl): this is very suspicious when not called by
        // `win_update()`. We should not randomly alter screen state outside
        // of `update_screen()` :(
        if !win.w_grid.target.is_null() {
            win.scroll_grid_lines(scrolled);
        }
    } else if win.w_onebuf_opt.wo_sms == 0 {
        win.w_skipcol = 0;
    }
    if prev_skipcol != win.w_skipcol {
        win.redraw_later(UPD_SOME_VALID);
    }

    redraw_for_cursorcolumn(win);

    // `w_leftcol` and `w_skipcol` are valid now; keep `check_cursor_moved()`
    // from thinking otherwise.
    win.w_valid_leftcol = win.w_leftcol;
    win.w_valid_skipcol = win.w_skipcol;
    win.w_valid |= WinValid::WCOL | WinValid::WROW | WinValid::VIRTCOL;
}

/// The screen position of the character at `pos` in window `wp`. The answers
/// are one-based, and zero when the character is not visible.
///
/// # Safety
/// `wp` must be a valid window, `pos` a position in its buffer, and the four
/// out-params must be writable.
pub unsafe fn textpos2screenpos(
    wp: *mut win_T,
    pos: *mut pos_T,
    rowp: *mut c_int,
    scolp: *mut c_int,
    ccolp: *mut c_int,
    ecolp: *mut c_int,
    local: bool,
) {
    // SAFETY: the caller's promise.
    let (win, pos) = unsafe { (Win::new(wp), Pos::new(pos)) };
    let (mut scol, mut ccol, mut ecol): (colnr_T, colnr_T, colnr_T) = (0, 0, 0);
    let mut coloff: colnr_T = 0;
    let mut visible_row = false;
    let mut is_folded = false;

    let mut lnum = pos.lnum;
    let mut row = if lnum >= win.w_topline && lnum <= win.w_botline {
        let fold_start = win.fold_first(lnum);
        is_folded = fold_start.is_some();
        lnum = fold_start.unwrap_or(lnum);
        // The screen line line `lnum` begins on, which can be negative when
        // it is the top line and `w_skipcol` is set.
        let mut row = win.plines_range(win.w_topline, lnum - 1, c_int::MAX);
        row -= adjust_plines_for_skipcol(win);
        // Filler lines drawn above this buffer line.
        row += if lnum == win.w_topline {
            win.w_topfill
        } else {
            win.fill_above(lnum)
        };
        visible_row = true;
        row
    } else if !local || lnum < win.w_topline {
        0
    } else {
        win.w_view_height - 1
    };

    let existing_row = lnum > 0 && lnum <= win.buffer().line_count();
    if (local || visible_row) && existing_row {
        let off = win.col_off();
        if is_folded {
            row += if local {
                0
            } else {
                win.w_winrow + win.w_winrow_off
            } + 1;
            coloff = if local {
                0
            } else {
                win.w_wincol + win.w_wincol_off
            } + 1
                + off;
        } else {
            debug_assert!(lnum == pos.lnum, "lnum == pos->lnum");
            (scol, ccol, ecol) = win.vcol_triple(pos);

            // As in `validate_cursor_col()`.
            let mut col = scol + off;
            let width = win.w_view_width - off + win.col_off2();
            // Long line wrapping, adjusting the row.
            if win.w_onebuf_opt.wo_wrap != 0 && col >= win.w_view_width && width > 0 {
                let rowoff = if visible_row {
                    arith::wrap_rowoff(col, win.w_view_width, width)
                } else {
                    0
                };
                col -= rowoff * width;
                row += rowoff;
            }
            col -= win.w_leftcol;

            if col >= 0 && col < win.w_view_width && row >= 0 && row < win.w_view_height {
                coloff = col - scol
                    + if local {
                        0
                    } else {
                        win.w_wincol + win.w_wincol_off
                    }
                    + 1;
                row += if local {
                    0
                } else {
                    win.w_winrow + win.w_winrow_off
                } + 1;
            } else {
                // The character is left of, right of or below the window.
                scol = 0;
                ccol = 0;
                ecol = 0;
                if local {
                    coloff = if col < 0 { -1 } else { win.w_view_width + 1 };
                } else {
                    row = 0;
                }
            }
        }
    }
    // SAFETY: the caller's promise about the out-params.
    unsafe {
        *rowp = row;
        *scolp = scol + coloff;
        *ccolp = ccol + coloff;
        *ecolp = ecol + coloff;
    }
}

/// `screenpos({winid}, {lnum}, {col})`.
///
/// # Safety
/// The evaluator's calling convention: `argvars` and `rettv` must be valid.
pub unsafe fn f_screenpos(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the evaluator's calling convention.
    let (dict, wp) = unsafe { (alloc_dict_ret(rettv), find_win_by_nr_or_id(argvars)) };
    if wp.is_null() {
        return;
    }
    // SAFETY: the evaluator's calling convention: three arguments, checked
    // by the builtin table.
    let (lnum, col) = unsafe { (arg_number(argvars, 1), arg_number(argvars, 2)) };
    let mut pos = pos_T {
        lnum: lnum as linenr_T,
        col: (col as colnr_T - 1).max(0),
        coladd: 0,
    };
    // SAFETY: `find_win_by_nr_or_id` answered a live window.
    if pos.lnum > unsafe { Win::new(wp) }.buffer().line_count() {
        // SAFETY: a NUL-terminated format string and the one argument it
        // names. Not `semsg!`: this is vim's own `printf`.
        unsafe {
            semsg_c!(
                gettext(&raw const e_invalid_line_number_nr as *const c_char),
                pos.lnum,
            )
        };
        return;
    }
    let (mut row, mut scol, mut ccol, mut ecol) = (0, 0, 0, 0);
    let (r, s, c, e) = (&raw mut row, &raw mut scol, &raw mut ccol, &raw mut ecol);
    // SAFETY: a live window, and five out-params of this frame.
    unsafe { textpos2screenpos(wp, &raw mut pos, r, s, c, e, false) };
    for (name, value) in [
        (c"row", row),
        (c"col", scol),
        (c"curscol", ccol),
        (c"endcol", ecol),
    ] {
        // SAFETY: a live Dict and a NUL-terminated key.
        unsafe { dict_add_nr(dict, name, value) };
    }
}

/// `tv_dict_alloc_ret`, answering the Dict it installed.
///
/// # Safety
/// `rettv` must be a writable return value.
unsafe fn alloc_dict_ret(rettv: *mut typval_T) -> *mut dict_T {
    unsafe {
        tv_dict_alloc_ret(rettv);
        (*rettv).vval.v_dict
    }
}

/// The `n`th argument of a builtin, as a number.
///
/// # Safety
/// `argvars` must hold at least `n + 1` values.
unsafe fn arg_number(argvars: *mut typval_T, n: isize) -> varnumber_T {
    unsafe { tv_get_number(argvars.offset(n)) }
}

/// `tv_dict_add_nr` with the key spelled as a C string literal.
///
/// # Safety
/// `dict` must be a live Dict.
unsafe fn dict_add_nr(dict: *mut dict_T, key: &CStr, value: c_int) {
    let bytes = key.to_bytes();
    unsafe {
        tv_dict_add_nr(
            dict,
            bytes.as_ptr().cast::<c_char>(),
            bytes.len() as size_t,
            value as varnumber_T,
        )
    };
}

/// The character column that shows at virtual (screen) column `vcol`. The
/// first column is one; for a multibyte character the column of its first
/// byte is answered.
///
/// # Safety
/// `wp` must be a valid window and `lnum` a line of its buffer.
unsafe fn virtcol2col(win: Win, lnum: linenr_T, vcol: c_int) -> c_int {
    // SAFETY: a live window and a line of its buffer.
    let offset = unsafe { vcol2col(win.raw(), lnum, vcol - 1, ::core::ptr::null_mut()) };
    // SAFETY: a live window and a line of its buffer.
    let line = unsafe { win.buffer().line(lnum) };
    // SAFETY: `vcol2col` answers a byte index within the line.
    if unsafe { line.byte(offset) } != 0 {
        return offset + 1;
    }
    if offset == 0 {
        // An empty line.
        return 0;
    }
    // Move back onto the first byte of the last character.
    // SAFETY: `offset` is past the line's first byte and within it.
    let head = unsafe { utf_head_off(line.raw(), line.raw().offset((offset - 1) as isize)) };
    offset - head
}

/// `virtcol2col({winid}, {lnum}, {col})`.
///
/// # Safety
/// The evaluator's calling convention: `argvars` and `rettv` must be valid.
pub unsafe fn f_virtcol2col(argvars: *mut typval_T, rettv: *mut typval_T, _fptr: EvalFuncData) {
    // SAFETY: the evaluator's calling convention.
    unsafe { (*rettv).vval.v_number = -1 };
    // SAFETY: the evaluator's calling convention: three arguments.
    let typed = unsafe { (0..3).all(|n| tv_check_for_number_arg(argvars, n) != FAIL) };
    if !typed {
        return;
    }
    // SAFETY: the evaluator's calling convention.
    let wp = unsafe { find_win_by_nr_or_id(argvars) };
    if wp.is_null() {
        return;
    }
    let mut error = false;
    // SAFETY: the evaluator's calling convention, and `error` is of this frame.
    let lnum = unsafe { tv_get_number_chk(argvars.offset(1), &raw mut error) } as linenr_T;
    // SAFETY: `find_win_by_nr_or_id` answered a live window.
    let win = unsafe { Win::new(wp) };
    if error || lnum < 0 || lnum > win.buffer().line_count() {
        return;
    }
    // SAFETY: the evaluator's calling convention, and `error` is of this frame.
    let screencol = unsafe { tv_get_number_chk(argvars.offset(2), &raw mut error) } as c_int;
    if error || screencol < 0 {
        return;
    }
    // SAFETY: a live window and a line of its buffer.
    let col = unsafe { virtcol2col(win, lnum, screencol) };
    // SAFETY: the evaluator's calling convention.
    unsafe { (*rettv).vval.v_number = col as varnumber_T };
}
