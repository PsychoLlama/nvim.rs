//! `win_update`'s second half: walking the window one line at a time.
//!
//! [`draw_window_lines`] takes what `winupdate.rs` decided and turns it into
//! calls to `win_line`, one per buffer line that has to be drawn, skipping the
//! ones whose `w_lines[]` entry says they are still correct. It also keeps that
//! array in step: every entry records which buffer lines a screen line run
//! covers and how many rows it took, and the next redraw trusts it.
//!
//! The tail is what fills the window when the buffer runs out: the `@` markers
//! for a last line that does not fit, and the `'fillchars'` "eob" area below the
//! last line.

#![deny(unsafe_op_in_unsafe_fn)]

#[allow(unused_imports)]
use super::*;
use crate::src::nvim::decoration::SIGN_WIDTH;

/// What the previous line of the walk was.
///
/// Used to decide whether a syntax-highlighted line after the changed range
/// still has to be drawn: after a folded line the state is unknown, after a
/// normal line it can be asked for.
#[derive(Copy, Clone, PartialEq, Eq)]
enum DidUpdate {
    /// Did not draw a line.
    None,
    /// Drew a normal line.
    Line,
    /// Drew a folded line.
    Fold,
}

/// Where the walk got to.
struct Walk {
    /// Next `w_lines[]` entry to fill.
    idx: c_int,
    /// Next window row to draw at.
    row: c_int,
    /// First row of the line being drawn.
    srow: c_int,
    /// Next buffer line to draw.
    lnum: linenr_T,
    /// Hit the end of the buffer.
    eof: bool,
    /// Finished the last line that fits.
    didline: bool,
    /// Last line whose syntax state was parsed, 0 for none.
    syntax_last_parsed: linenr_T,
    /// Whether the changed lines have already been scrolled for.
    scrolled_for_mod: bool,
    /// What the previous iteration did.
    did_update: DidUpdate,
}

/// Draw window `wp`, and answer the `w_botline` it had on entry.
///
/// The answer is what [`win_update`]'s tail compares against to decide whether
/// the approximation it had been carrying was wrong.
///
/// # Safety
/// `wp` must be a live window, `buf` its buffer, and the decoration and
/// search-highlight state must be set up for this redraw.
pub(crate) unsafe fn draw_window_lines(
    wp: *mut win_T,
    buf: *mut buf_T,
    rg: &mut Regions,
    cursorline_fi: foldinfo_T,
    spv: &mut spellvars_T,
) -> linenr_T {
    // SAFETY: the caller's window and buffer, during a redraw.
    unsafe {
        let mut w = Walk {
            idx: 0,
            row: 0,
            srow: 0,
            lnum: (*wp).w_topline,
            eof: false,
            didline: false,
            syntax_last_parsed: 0,
            scrolled_for_mod: false,
            did_update: DidUpdate::None,
        };

        // A 'statuscolumn' whose width changed (or errored) restarts the walk
        // from the top. Upstream spells that as a `goto` into the loop, from
        // two places -- the bottom of the loop and the filler-line draw in the
        // tail below.
        let old_botline = 'restart: loop {
            w.idx = 0;
            w.row = 0;
            w.lnum = (*wp).w_topline;
            w.eof = false;
            w.didline = false;

            loop {
                // Stop at the end of the window; the check for going *past* it
                // is at the bottom.
                if w.row == (*wp).w_view_height {
                    w.didline = true;
                    break;
                }
                if w.lnum > (*buf).b_ml.ml_line_count {
                    w.eof = true;
                    break;
                }

                // Remembered for the "the line did not fit" case below.
                w.srow = w.row;

                if line_needs_drawing(wp, buf, rg, &w) {
                    if !draw_one_line(wp, buf, rg, &mut w, cursorline_fi, spv) {
                        break;
                    }
                } else {
                    skip_one_line(wp, buf, rg, &mut w, cursorline_fi, spv);
                    if w.row > (*wp).w_view_height {
                        break;
                    }
                }

                if (*wp).w_redr_statuscol {
                    restart_for_statuscol(wp);
                    continue 'restart;
                }
                if w.lnum > (*buf).b_ml.ml_line_count {
                    w.eof = true;
                    break;
                }
            }

            // The window has been drawn with both the old and the new cursor
            // line, so the remembered one can move.
            (*wp).w_last_cursorline = (*wp).w_cursorline;
            (*wp).w_last_cursor_lnum_rnu = if (*wp).w_onebuf_opt.wo_rnu != 0 {
                (*wp).w_cursor.lnum
            } else {
                0
            };
            (*wp).w_lines_valid = (*wp).w_lines_valid.max(w.idx);
            (*wp).w_display_tick = display_tick.get();

            // Tell the syntax machinery where parsing stopped.
            if w.syntax_last_parsed != 0 && syntax_present(wp) {
                syntax_end_parsing(wp, w.syntax_last_parsed + 1);
            }

            let old_botline = (*wp).w_botline;
            (*wp).w_empty_rows = 0;
            (*wp).w_filler_rows = 0;

            if !w.eof && !w.didline {
                draw_unfinished_last_line(wp, &w);
            } else {
                if w.eof {
                    // Filler text below the last line. `win_line` recognises
                    // `ml_line_count + 1` and draws only the filler.
                    (*wp).w_botline = (*buf).b_ml.ml_line_count + 1;
                    let fill = win_get_fill(wp, (*wp).w_botline);
                    if fill > 0 && !(*wp).w_botfill && w.row < (*wp).w_view_height {
                        let mut zero_spv = spellvars_T::default();
                        w.row = win_line(
                            wp,
                            (*wp).w_botline,
                            w.row,
                            (*wp).w_view_height,
                            0,
                            false,
                            &raw mut zero_spv,
                            foldinfo_T::default(),
                        );
                        if (*wp).w_redr_statuscol {
                            w.eof = false;
                            restart_for_statuscol(wp);
                            continue 'restart;
                        }
                    }
                } else if dollar_vcol.get() == -1 || wp != curwin.get() {
                    (*wp).w_botline = w.lnum;
                }

                draw_end_of_buffer(wp, buf, rg, &w);
            }

            break old_botline;
        };

        old_botline
    }
}

/// Whether the line at `w.lnum` has to be drawn, or its `w_lines[]` entry can
/// be trusted.
///
/// Every disjunct is a reason the entry is stale: it is in one of the three
/// areas, the entry does not exist, the line straddles the first row a scroll
/// invalidated, it is inside the changed range, or it is the cursor line coming
/// or going.
///
/// # Safety
/// `wp` must be a live window and `buf` its buffer.
unsafe fn line_needs_drawing(wp: *mut win_T, buf: *mut buf_T, rg: &Regions, w: &Walk) -> bool {
    // SAFETY: the caller's window, buffer and `w_lines` array.
    unsafe {
        if w.row < rg.top_end
            || (w.row >= rg.mid_start && w.row < rg.mid_end)
            || rg.top_to_mod
            || w.idx >= (*wp).w_lines_valid
            || w.row + (*(*wp).w_lines.add(w.idx as usize)).wl_size as c_int > rg.bot_start
        {
            return true;
        }

        // The changed range. This is asked BEFORE the cursor-line tests below,
        // as upstream orders it, and the order is load-bearing:
        // `syntax_check_changed` finishes the previous line and stores the
        // syntax state for this one, so short-circuiting past it on the cursor
        // line would leave the state cache one line behind.
        if rg.mod_top != 0
            && (w.lnum == rg.mod_top
                || (w.lnum >= rg.mod_top
                    // Keep going past the change while the highlighting can
                    // still differ from what is on screen: after a folded line
                    // the syntax state is unknown, after a normal one the
                    // syntax machinery can be asked.
                    && (w.lnum < rg.mod_bot
                        || w.did_update == DidUpdate::Fold
                        || (w.did_update == DidUpdate::Line
                            && syntax_present(wp)
                            && ((foldmethodIsSyntax(wp) && hasAnyFolding(wp) != 0)
                                || syntax_check_changed(w.lnum)))
                        // A match at a fixed position may need redrawing when
                        // lines were inserted or deleted.
                        || (!(*wp).w_match_head.is_null()
                            && (*buf).b_mod_set
                            && (*buf).b_mod_xlines != 0))))
        {
            return true;
        }

        w.lnum == (*wp).w_cursorline || w.lnum == (*wp).w_last_cursorline
    }
}

/// Draw the line at `w.lnum`, and record what it took.
///
/// Answers false when the walk must stop -- the line ran past the end of the
/// window.
///
/// # Safety
/// `wp` must be a live window and `buf` its buffer.
unsafe fn draw_one_line(
    wp: *mut win_T,
    buf: *mut buf_T,
    rg: &mut Regions,
    w: &mut Walk,
    cursorline_fi: foldinfo_T,
    spv: &mut spellvars_T,
) -> bool {
    // SAFETY: the caller's window, buffer and `w_lines` array.
    unsafe {
        if w.lnum == rg.mod_top {
            rg.top_to_mod = false;
        }

        // Folded lines are drawn as one line for all of them. The cursor line's
        // fold info was computed once, before the walk.
        let foldinfo = if (*wp).w_onebuf_opt.wo_cul != 0 && w.lnum == (*wp).w_cursor.lnum {
            cursorline_fi
        } else {
            fold_info(wp, w.lnum)
        };

        // A concealed line with no filler lines takes no rows at all.
        let concealed = decor_conceal_line(wp, w.lnum - 1, false);
        if concealed && win_get_fill(wp, w.lnum) == 0 {
            let step = if foldinfo.fi_lines != 0 {
                foldinfo.fi_lines
            } else {
                1
            };
            if w.lnum == rg.mod_top && w.lnum < rg.mod_bot {
                rg.mod_top += step;
            }
            w.lnum += step;
            spv.spv_capcol_lnum = 0;
            return true;
        }

        scroll_for_changed_lines(wp, rg, w);

        let wl = (*wp).w_lines.add(w.idx as usize);
        if foldinfo.fi_lines == 0
            && w.idx < (*wp).w_lines_valid
            && (*wl).wl_valid
            && (*wl).wl_lnum == w.lnum
            && w.lnum > (*wp).w_topline
            && dy_flags.get() & (kOptDyFlagLastline | kOptDyFlagTruncate) == 0
            && w.srow + (*wl).wl_size as c_int > (*wp).w_view_height
            && win_get_fill(wp, w.lnum) == 0
        {
            // This line is not going to fit. Draw nothing here; the "@" lines
            // below take the rest.
            w.row = (*wp).w_view_height + 1;
        } else {
            prepare_search_hl(wp, screen_search_hl.ptr(), w.lnum);
            // Let the syntax machinery know lines were skipped.
            if w.syntax_last_parsed != 0 && w.syntax_last_parsed + 1 < w.lnum && syntax_present(wp)
            {
                syntax_end_parsing(wp, w.syntax_last_parsed + 1);
            }

            // Spell checking only applies to real buffer text: a concealed line
            // or a fold whose 'foldtext' replaces it has none.
            let display_buf_line =
                !concealed && (foldinfo.fi_lines == 0 || *(*wp).w_onebuf_opt.wo_fdt == 0);

            let mut zero_spv = spellvars_T::default();
            let spv_arg: *mut spellvars_T = if display_buf_line {
                &raw mut *spv
            } else {
                &raw mut zero_spv
            };
            w.row = win_line(
                wp,
                w.lnum,
                w.srow,
                (*wp).w_view_height,
                0,
                concealed,
                spv_arg,
                foldinfo,
            );

            if display_buf_line {
                w.syntax_last_parsed = w.lnum;
            } else {
                spv.spv_capcol_lnum = 0;
            }

            let lastlnum = w.lnum + foldinfo.fi_lines - linenr_T::from(foldinfo.fi_lines > 0);
            (*wl).wl_folded = foldinfo.fi_lines > 0;
            (*wl).wl_foldend = lastlnum;
            (*wl).wl_lastlnum = lastlnum;
            w.did_update = if foldinfo.fi_lines > 0 {
                DidUpdate::Fold
            } else {
                DidUpdate::Line
            };

            // Concealed lines below this one belong to the same screen run --
            // unless a virtual line is attached below it, which has to be drawn
            // and so ends the run. A virtual line below a *second* adjacent
            // concealed line is concealed with it.
            let mut virt_below = decor_virt_lines(
                wp,
                lastlnum,
                lastlnum + 1,
                ::core::ptr::null_mut(),
                ::core::ptr::null_mut(),
                true,
            ) > 0;
            while !virt_below
                && (*wl).wl_lastlnum < (*buf).b_ml.ml_line_count
                && decor_conceal_line(wp, (*wl).wl_lastlnum, false)
            {
                virt_below = false;
                (*wl).wl_lastlnum += 1;
                hasFolding(
                    wp,
                    (*wl).wl_lastlnum,
                    ::core::ptr::null_mut(),
                    &raw mut (*wl).wl_lastlnum,
                );
            }
        }

        (*wl).wl_lnum = w.lnum;
        (*wl).wl_valid = true;

        // `dollar_vcol >= 0` on the cursor line means it was not fully drawn
        // for a change command, so its recorded height must not move.
        let is_curline = wp == curwin.get() && w.lnum == (*wp).w_cursor.lnum;

        if w.row > (*wp).w_view_height {
            // Past the end of the grid. The height may still be needed later.
            if dollar_vcol.get() == -1 || !is_curline {
                (*wl).wl_size = plines_win(wp, w.lnum, true) as uint16_t;
            }
            w.idx += 1;
            return false;
        }
        if dollar_vcol.get() == -1 || !is_curline {
            (*wl).wl_size = (w.row - w.srow) as uint16_t;
        }
        w.lnum = (*wl).wl_lastlnum + 1;
        w.idx += 1;
        true
    }
}

/// Insert or delete screen rows so the lines below the change stay where they
/// are, instead of being redrawn.
///
/// Runs once per redraw, at the first line of the changed range.
///
/// # Safety
/// `wp` must be a live window.
unsafe fn scroll_for_changed_lines(wp: *mut win_T, rg: &mut Regions, w: &mut Walk) {
    // SAFETY: the caller's window and its `w_lines` array.
    unsafe {
        // Not when the change continues to the end, and not for changed lines
        // in a top area that was already scrolled for.
        let at_change = !w.scrolled_for_mod
            && rg.mod_bot != MAXLNUM as linenr_T
            && w.lnum >= rg.mod_top
            && w.lnum < rg.mod_bot.max(rg.mod_top + 1)
            && (!rg.scrolled_down || w.row >= rg.top_end);
        if !at_change {
            return;
        }
        w.scrolled_for_mod = true;

        // Count the rows the changed lines take *now*, from `w_lines[]`, which
        // still describes what is on the grid.
        let mut old_cline_height = 0;
        let mut old_rows = 0;
        let mut i = w.idx;
        while i < (*wp).w_lines_valid {
            let wl = (*wp).w_lines.add(i as usize);
            // Only valid entries have a meaningful `wl_lnum`; invalid ones are
            // part of the changed area.
            if (*wl).wl_valid && (*wl).wl_lnum == rg.mod_bot {
                break;
            }
            if (*wl).wl_lnum == (*wp).w_cursor.lnum {
                old_cline_height = (*wl).wl_size as c_int;
            }
            old_rows += (*wl).wl_size as c_int;
            if (*wl).wl_valid && (*wl).wl_lastlnum + 1 == rg.mod_bot {
                // The last valid entry above `mod_bot`; the invalid entries
                // after it are part of the change too.
                i += 1;
                while i < (*wp).w_lines_valid && !(*(*wp).w_lines.add(i as usize)).wl_valid {
                    old_rows += (*(*wp).w_lines.add(i as usize)).wl_size as c_int;
                    i += 1;
                }
                break;
            }
            i += 1;
        }

        if i >= (*wp).w_lines_valid {
            // No valid line below the change: redraw to the end of the window.
            // Inserting or deleting rows would buy nothing.
            rg.bot_start = 0;
            rg.bot_scroll_start = 0;
            return;
        }

        // Count the rows they will take, and scroll by the difference.
        let mut new_rows = 0;
        let mut j = w.idx;
        let mut l = w.lnum;
        while l < rg.mod_bot {
            if dollar_vcol.get() >= 0
                && wp == curwin.get()
                && old_cline_height > 0
                && l == (*wp).w_cursor.lnum
            {
                // The cursor line is not fully redrawn, so its height is
                // unchanged.
                new_rows += old_cline_height;
                j += 1;
            } else {
                let n = plines_correct_topline(wp, l, &raw mut l, true, ::core::ptr::null_mut());
                new_rows += n;
                j += c_int::from(n > 0); // do not count concealed lines
            }
            if new_rows > (*wp).w_view_height - w.row - 2 {
                new_rows = 9999; // too much: redraw the rest
                break;
            }
            l += 1;
        }

        let xtra_rows = new_rows - old_rows;
        if xtra_rows < 0 {
            // The text got shorter: scroll the rows below it up, and redraw
            // from where they used to end. Not worth it if there is barely any
            // text left, or if the scroll fails.
            if w.row - xtra_rows >= (*wp).w_view_height - 2 {
                rg.mod_bot = MAXLNUM as linenr_T;
            } else {
                win_scroll_lines(wp, w.row, xtra_rows);
                rg.bot_start = (*wp).w_view_height + xtra_rows;
                rg.bot_scroll_start = rg.bot_start;
            }
        } else if xtra_rows > 0 {
            // The text got taller: scroll the rows below it down. They keep
            // their contents, so only the end-of-buffer area is stale.
            if w.row + xtra_rows >= (*wp).w_view_height - 2 {
                rg.mod_bot = MAXLNUM as linenr_T;
            } else {
                win_scroll_lines(wp, w.row + old_rows, xtra_rows);
                rg.bot_scroll_start = 0;
                if rg.top_end > w.row + old_rows {
                    // The part of the top area that still needs updating was
                    // scrolled down with everything else.
                    rg.top_end += xtra_rows;
                }
            }
        }

        // Move the `w_lines[]` entries to match, unless the rest is being
        // redrawn anyway.
        if rg.mod_bot != MAXLNUM as linenr_T && i != j {
            move_line_entries(wp, rg, w, i, j, new_rows);
        }
    }
}

/// Shift the `w_lines[]` entries after a changed range gained or lost lines.
///
/// `i` is where the entries below the change start now, `j` where they belong.
///
/// # Safety
/// `wp` must be a live window.
unsafe fn move_line_entries(
    wp: *mut win_T,
    rg: &mut Regions,
    w: &Walk,
    mut i: c_int,
    mut j: c_int,
    new_rows: c_int,
) {
    // SAFETY: the caller's window and its `w_lines` array.
    unsafe {
        if j < i {
            // Upwards.
            let mut at_row = w.row + new_rows;
            loop {
                if i >= (*wp).w_lines_valid {
                    (*wp).w_lines_valid = j;
                    break;
                }
                *(*wp).w_lines.add(j as usize) = *(*wp).w_lines.add(i as usize);
                // Stop at a line that will not fit.
                if at_row + (*(*wp).w_lines.add(j as usize)).wl_size as c_int > (*wp).w_view_height
                {
                    (*wp).w_lines_valid = j + 1;
                    break;
                }
                at_row += (*(*wp).w_lines.add(j as usize)).wl_size as c_int;
                j += 1;
                i += 1;
            }
            rg.bot_start = rg.bot_start.min(at_row);
        } else {
            // Downwards.
            j -= i;
            (*wp).w_lines_valid = ((*wp).w_lines_valid + j).min((*wp).w_view_height);
            let mut at = (*wp).w_lines_valid;
            while at - j >= w.idx {
                *(*wp).w_lines.add(at as usize) = *(*wp).w_lines.add((at - j) as usize);
                at -= 1;
            }
            // The entries for the inserted lines are invalid now, but `wl_size`
            // is read above, so it is zeroed rather than left alone.
            while at >= w.idx {
                let wl = (*wp).w_lines.add(at as usize);
                (*wl).wl_size = 0;
                (*wl).wl_valid = false;
                at -= 1;
            }
        }
    }
}

/// Advance past a line whose `w_lines[]` entry is still correct.
///
/// The text does not need redrawing, but the number column might: `'number'`
/// below inserted or deleted lines, and `'relativenumber'` whenever the cursor
/// moved to another line.
///
/// # Safety
/// `wp` must be a live window and `buf` its buffer.
unsafe fn skip_one_line(
    wp: *mut win_T,
    buf: *mut buf_T,
    rg: &Regions,
    w: &mut Walk,
    cursorline_fi: foldinfo_T,
    spv: &mut spellvars_T,
) {
    // SAFETY: the caller's window, buffer and `w_lines` array.
    unsafe {
        let wl = (*wp).w_lines.add(w.idx as usize);
        let numbers_moved = ((*wp).w_onebuf_opt.wo_nu != 0
            && rg.mod_top != 0
            && w.lnum >= rg.mod_bot
            && (*buf).b_mod_set
            && (*buf).b_mod_xlines != 0)
            || ((*wp).w_onebuf_opt.wo_rnu != 0
                && (*wp).w_last_cursor_lnum_rnu != (*wp).w_cursor.lnum);
        if numbers_moved {
            let info = if (*wp).w_onebuf_opt.wo_cul != 0 && w.lnum == (*wp).w_cursor.lnum {
                cursorline_fi
            } else {
                fold_info(wp, w.lnum)
            };
            // A non-zero `col_rows` tells `win_line` to draw only the columns.
            win_line(
                wp,
                w.lnum,
                w.srow,
                (*wp).w_view_height,
                (*wl).wl_size as c_int,
                false,
                &raw mut *spv,
                info,
            );
        }

        w.row += (*wl).wl_size as c_int;
        w.idx += 1;
        if w.row > (*wp).w_view_height {
            return;
        }
        w.lnum = (*(*wp).w_lines.add((w.idx - 1) as usize)).wl_lastlnum + 1;
        w.did_update = DidUpdate::None;
        spv.spv_capcol_lnum = 0;
    }
}

/// Reset the walk state so the window is drawn again from its top line.
///
/// # Safety
/// `wp` must be a live window.
unsafe fn restart_for_statuscol(wp: *mut win_T) {
    // SAFETY: the caller's window and the redraw's decoration state.
    unsafe {
        (*wp).w_redr_statuscol = false;
        (*wp).w_lines_valid = 0;
        (*wp).w_valid &= !VALID_WCOL;
        decor_redraw_reset(wp, decor_state.ptr());
        decor_providers_invoke_win(wp);
    }
}

/// The last line did not fit in the window: say so, per `'display'`.
///
/// # Safety
/// `wp` must be a live window.
unsafe fn draw_unfinished_last_line(wp: *mut win_T, w: &Walk) {
    // SAFETY: the caller's window; the grid batch is opened and flushed here.
    unsafe {
        let at_attr = hl_combine_attr(win_bg_attr(wp), win_hl_attr(wp, HLF_AT));

        if w.lnum == (*wp).w_topline {
            // A single line that does not fit. Do not overwrite it -- it can
            // still be edited.
            (*wp).w_botline = w.lnum + 1;
            return;
        }

        if win_get_fill(wp, w.lnum) >= (*wp).w_view_height - w.srow {
            // The window ends in filler lines.
            (*wp).w_botline = w.lnum;
            (*wp).w_filler_rows = (*wp).w_view_height - w.srow;
            return;
        }

        if dy_flags.get() & kOptDyFlagTruncate != 0 {
            // "@@@" in the last screen line, and nothing else on it.
            grid_line_start(&raw mut (*wp).w_grid, (*wp).w_view_height - 1);
            grid_line_fill(
                0,
                (*wp).w_view_width.min(3),
                (*wp).w_p_fcs_chars.lastline,
                at_attr,
            );
            grid_line_fill(3, (*wp).w_view_width, schar_from_ascii(b' '), at_attr);
            grid_line_flush();
        } else if dy_flags.get() & kOptDyFlagLastline != 0 {
            // "@@@" at the end of the last screen line, over the text. Four
            // cells when three would split a double-width character in half.
            grid_line_start(&raw mut (*wp).w_grid, (*wp).w_view_height - 1);
            let width =
                if grid_line_getchar(((*wp).w_view_width - 3).max(0), ::core::ptr::null_mut()) == 0
                {
                    4
                } else {
                    3
                };
            grid_line_fill(
                ((*wp).w_view_width - width).max(0),
                (*wp).w_view_width,
                (*wp).w_p_fcs_chars.lastline,
                at_attr,
            );
            grid_line_flush();
        } else {
            // A column of "@" down the rows the line would have taken.
            win_draw_end(
                wp,
                (*wp).w_p_fcs_chars.lastline,
                true,
                w.srow,
                (*wp).w_view_height,
                HLF_AT,
            );
        }
        set_empty_rows(wp, w.srow);
        (*wp).w_botline = w.lnum;
    }
}

/// Fill the rows below the last buffer line with `'fillchars'` "eob".
///
/// Where that starts is the interesting part: the rows a scroll left stale have
/// to be covered even though no line was drawn over them, which is what
/// `bot_scroll_start` records.
///
/// # Safety
/// `wp` must be a live window and `buf` its buffer.
unsafe fn draw_end_of_buffer(wp: *mut win_T, buf: *mut buf_T, rg: &Regions, w: &Walk) {
    // SAFETY: the caller's window and buffer.
    unsafe {
        let mut lastline = rg.bot_scroll_start;
        if rg.mid_end >= w.row {
            lastline = lastline.min(rg.mid_start);
        }
        // The change reached past the end of the buffer, so nothing below it
        // can be trusted.
        if rg.mod_bot > (*buf).b_ml.ml_line_count {
            lastline = 0;
        }

        win_draw_end(
            wp,
            (*wp).w_p_fcs_chars.eob,
            false,
            lastline.max(w.row),
            (*wp).w_view_height,
            HLF_EOB,
        );
        set_empty_rows(wp, w.row);
    }
}

/// Scroll `line_count` screen rows at window row `row`.
///
/// Positive `line_count` scrolls down, making room at `row`; negative deletes
/// rows there. Nothing happens when the area to move would be off the window --
/// the caller redraws it instead.
pub unsafe fn win_scroll_lines(wp: *mut win_T, row: c_int, line_count: c_int) {
    // SAFETY: a live window; `grid_adjust` maps its rows onto the grid that
    // carries them.
    unsafe {
        if !redrawing() || line_count == 0 {
            return;
        }

        let mut col = 0;
        let mut row_off = 0;
        let grid = grid_adjust(&raw mut (*wp).w_grid, &raw mut row_off, &raw mut col);

        // The bounds are the grid's rather than the window's because
        // `curs_columns` reaches here from outside `update_screen`, when the
        // two may disagree.
        let checked_width = ((*grid).cols - col).min((*wp).w_view_width);
        let checked_height = ((*grid).rows - row_off).min((*wp).w_view_height);

        // Nothing would be moved; the caller draws over the whole area.
        if row + line_count.abs() >= checked_height {
            return;
        }

        if line_count < 0 {
            grid_del_lines(
                grid,
                row + row_off,
                -line_count,
                checked_height + row_off,
                col,
                checked_width,
            );
        } else {
            grid_ins_lines(
                grid,
                row + row_off,
                line_count,
                checked_height + row_off,
                col,
                checked_width,
            );
        }
    }
}

/// Clear window rows `startrow..endrow` and mark each with `c1`.
///
/// With `draw_margin` the fold, sign and number columns are drawn as blanks
/// first, so the marker starts where the text would.
pub unsafe fn win_draw_end(
    wp: *mut win_T,
    c1: schar_T,
    draw_margin: bool,
    startrow: c_int,
    endrow: c_int,
    hl: hlf_T,
) {
    debug_assert!(hl >= 0 && hl < HLF_COUNT, "hl >= 0 && hl < HLF_COUNT");
    // SAFETY: a live window; each grid batch is opened and flushed per row.
    unsafe {
        let view_width = (*wp).w_view_width;
        let fdc = compute_foldcolumn(wp, 0);
        let scwidth = (*wp).w_scwidth;

        // The `win_hl_attr` lookups deliberately stay inside the loop, in
        // upstream's order: it hands out attribute ids in call order, so
        // hoisting `hl` above the three margin groups would renumber them.
        for row in startrow..endrow {
            grid_line_start(&raw mut (*wp).w_grid, row);

            let mut n = 0;
            if draw_margin {
                if fdc > 0 {
                    n = grid_line_fill(
                        n,
                        view_width.min(n + fdc),
                        schar_from_ascii(b' '),
                        win_hl_attr(wp, HLF_FC),
                    );
                }
                if scwidth > 0 {
                    n = grid_line_fill(
                        n,
                        view_width.min(n + scwidth * SIGN_WIDTH as c_int),
                        schar_from_ascii(b' '),
                        win_hl_attr(wp, HLF_SC),
                    );
                }
                if ((*wp).w_onebuf_opt.wo_nu != 0 || (*wp).w_onebuf_opt.wo_rnu != 0)
                    && vim_strchr(p_cpo.get(), CPO_NUMCOL).is_null()
                {
                    let width = number_width(wp) + 1;
                    n = grid_line_fill(
                        n,
                        view_width.min(n + width),
                        schar_from_ascii(b' '),
                        win_hl_attr(wp, HLF_N),
                    );
                }
            }

            let attr = win_hl_attr(wp, hl);
            if n < view_width {
                grid_line_put_schar(n, c1, attr);
                n += 1;
            }
            grid_line_clear_end(n, view_width, win_bg_attr(wp), attr);

            if (*wp).w_onebuf_opt.wo_rl != 0 {
                grid_line_mirror(view_width);
            }
            grid_line_flush();
        }
    }
}
